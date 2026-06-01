use crate::error::{into_invoke_err, GnomadError};
use serde::Serialize;
use std::path::{Path, PathBuf};
#[cfg(feature = "embedded-llm")]
use std::num::NonZeroU32;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedLlmStatus {
    pub available: bool,
    pub loaded: bool,
    pub model_path: Option<String>,
    pub message: String,
}

pub struct EmbeddedLlmState {
    inner: Mutex<EmbeddedLlmInner>,
}

struct EmbeddedLlmInner {
    loaded_path: Option<PathBuf>,
}

impl Default for EmbeddedLlmState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(EmbeddedLlmInner {
                loaded_path: None,
            }),
        }
    }
}

enum WorkerMsg {
    Complete {
        path: PathBuf,
        prompt: String,
        max_tokens: u32,
        reply: std::sync::mpsc::Sender<Result<String, String>>,
    },
    Unload {
        reply: std::sync::mpsc::Sender<()>,
    },
}

struct InferenceWorker {
    tx: std::sync::mpsc::Sender<WorkerMsg>,
}

static INFERENCE_WORKER: OnceLock<InferenceWorker> = OnceLock::new();

fn worker_handle() -> &'static InferenceWorker {
    INFERENCE_WORKER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("embedded-llm".into())
            .spawn(move || inference_worker_loop(rx))
            .expect("spawn embedded-llm worker");
        InferenceWorker { tx }
    })
}

fn inference_worker_loop(rx: std::sync::mpsc::Receiver<WorkerMsg>) {
    for msg in rx {
        match msg {
            WorkerMsg::Complete {
                path,
                prompt,
                max_tokens,
                reply,
            } => {
                let result = imp::complete_gguf_on_worker(&path, &prompt, max_tokens);
                let _ = reply.send(result);
            }
            WorkerMsg::Unload { reply } => {
                imp::unload_on_worker();
                let _ = reply.send(());
            }
        }
    }
}

#[cfg(feature = "embedded-llm")]
mod imp {
    use super::*;
    use llama_cpp_2::context::params::LlamaContextParams;
    use llama_cpp_2::llama_backend::LlamaBackend;
    use llama_cpp_2::llama_batch::LlamaBatch;
    use llama_cpp_2::model::params::LlamaModelParams;
    use llama_cpp_2::model::{AddBos, LlamaModel};
    use llama_cpp_2::sampling::LlamaSampler;

    struct WorkerEngine {
        backend: LlamaBackend,
        model: LlamaModel,
        path: PathBuf,
    }

    thread_local! {
        static ENGINE: std::cell::RefCell<Option<WorkerEngine>> = const { std::cell::RefCell::new(None) };
    }

    pub fn unload_on_worker() {
        ENGINE.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }

    fn ensure_engine(path: &Path) -> Result<(), String> {
        if !path.is_file() {
            return Err(into_invoke_err(GnomadError::Llm {
                message: "GGUF model file not found. Choose a .gguf in Settings → Agent access."
                    .into(),
                detail: Some(path.display().to_string()),
            }));
        }

        ENGINE.with(|cell| {
            let mut guard = cell.borrow_mut();
            if guard
                .as_ref()
                .is_some_and(|e| e.path == path)
            {
                return Ok(());
            }

            let backend = LlamaBackend::init().map_err(|e| {
                into_invoke_err(GnomadError::Internal {
                    message: "Failed to initialize llama.cpp backend.".into(),
                    detail: Some(e.to_string()),
                })
            })?;
            let model = LlamaModel::load_from_file(
                &backend,
                path,
                &LlamaModelParams::default(),
            )
            .map_err(|e| {
                into_invoke_err(GnomadError::Llm {
                    message: "Failed to load GGUF model.".into(),
                    detail: Some(e.to_string()),
                })
            })?;

            *guard = Some(WorkerEngine {
                backend,
                model,
                path: path.to_path_buf(),
            });
            Ok(())
        })
    }

    fn run_completion_with_model(
        backend: &LlamaBackend,
        model: &LlamaModel,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(512);
        let mut ctx = model.new_context(backend, ctx_params).map_err(|e| {
            into_invoke_err(GnomadError::Llm {
                message: "Failed to create inference context.".into(),
                detail: Some(e.to_string()),
            })
        })?;

        let mut tokens = model.str_to_token(prompt, AddBos::Always).map_err(|e| {
            into_invoke_err(GnomadError::Llm {
                message: "Failed to tokenize prompt.".into(),
                detail: Some(e.to_string()),
            })
        })?;

        let n_ctx = ctx.n_ctx() as usize;
        let mut batch = LlamaBatch::new(n_ctx.min(4096), 1);

        if tokens.is_empty() {
            return Err(into_invoke_err(GnomadError::Llm {
                message: "Prompt produced no tokens.".into(),
                detail: None,
            }));
        }

        let mut pos: i32 = 0;
        let prompt_len = tokens.len();
        for (i, token) in tokens.iter().enumerate() {
            batch.clear();
            let is_last = i + 1 == prompt_len;
            batch
                .add(*token, pos, &[0], is_last)
                .map_err(|e| internal_batch_err(&e.to_string()))?;
            ctx.decode(&mut batch)
                .map_err(|e| internal_decode_err(&e.to_string()))?;
            if !is_last {
                pos += 1;
            }
        }

        let eos = model.token_eos();
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.7),
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::dist(0),
        ]);
        let mut out = String::new();

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, -1);
            if token == eos {
                break;
            }
            sampler.accept(token);
            let bytes = model.token_to_piece_bytes(token, 32, false, None).map_err(|e| {
                into_invoke_err(GnomadError::Llm {
                    message: "Failed to decode token.".into(),
                    detail: Some(e.to_string()),
                })
            })?;
            out.push_str(&String::from_utf8_lossy(&bytes));

            batch.clear();
            pos += 1;
            batch
                .add(token, pos, &[0], true)
                .map_err(|e| internal_batch_err(&e.to_string()))?;
            ctx.decode(&mut batch)
                .map_err(|e| internal_decode_err(&e.to_string()))?;
            tokens.push(token);
        }

        Ok(out)
    }

    fn internal_batch_err(detail: &str) -> String {
        into_invoke_err(GnomadError::Internal {
            message: "Batch error during embedded inference.".into(),
            detail: Some(detail.to_string()),
        })
    }

    fn internal_decode_err(detail: &str) -> String {
        into_invoke_err(GnomadError::Internal {
            message: "Decode error during embedded inference.".into(),
            detail: Some(detail.to_string()),
        })
    }

    pub fn complete_gguf_on_worker(
        path: &Path,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        ensure_engine(path)?;
        ENGINE.with(|cell| {
            let guard = cell.borrow();
            let engine = guard.as_ref().ok_or_else(|| {
                into_invoke_err(GnomadError::Internal {
                    message: "Embedded model failed to load.".into(),
                    detail: None,
                })
            })?;
            run_completion_with_model(&engine.backend, &engine.model, prompt, max_tokens)
        })
    }
}

#[cfg(not(feature = "embedded-llm"))]
mod imp {
    use super::*;
    use std::path::Path;

    pub fn unload_on_worker() {}

    pub fn complete_gguf_on_worker(
        _path: &Path,
        _prompt: &str,
        _max_tokens: u32,
    ) -> Result<String, String> {
        Err(into_invoke_err(GnomadError::Llm {
            message: "Embedded GGUF inference is not enabled in this build.".into(),
            detail: Some(
                "Rebuild with: cd src-tauri && cargo build --features embedded-llm. Or use Ollama."
                    .into(),
            ),
        }))
    }
}

pub fn embedded_llm_available() -> bool {
    cfg!(feature = "embedded-llm")
}

pub fn complete_with_gguf(path: &Path, prompt: &str, max_tokens: u32) -> Result<String, String> {
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();
    worker_handle()
        .tx
        .send(WorkerMsg::Complete {
            path: path.to_path_buf(),
            prompt: prompt.to_string(),
            max_tokens,
            reply: reply_tx,
        })
        .map_err(|_| {
            into_invoke_err(GnomadError::Internal {
                message: "Embedded inference worker stopped.".into(),
                detail: None,
            })
        })?;
    reply_rx.recv().map_err(|_| {
        into_invoke_err(GnomadError::Internal {
            message: "Embedded inference worker did not respond.".into(),
            detail: None,
        })
    })?
}

pub fn unload_gguf_worker() {
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();
    if worker_handle()
        .tx
        .send(WorkerMsg::Unload { reply: reply_tx })
        .is_ok()
    {
        let _ = reply_rx.recv();
    }
}

pub fn record_loaded(state: &EmbeddedLlmState, path: PathBuf) {
    if let Ok(mut guard) = state.inner.lock() {
        guard.loaded_path = Some(path);
    }
}

pub fn clear_loaded(state: &EmbeddedLlmState) {
    if let Ok(mut guard) = state.inner.lock() {
        guard.loaded_path = None;
    }
}

pub fn status(state: &EmbeddedLlmState) -> EmbeddedLlmStatus {
    let loaded_path = state
        .inner
        .lock()
        .ok()
        .and_then(|g| g.loaded_path.clone());

    if !embedded_llm_available() {
        return EmbeddedLlmStatus {
            available: false,
            loaded: false,
            model_path: loaded_path.map(|p| p.display().to_string()),
            message: "Build without embedded-llm feature. Use Ollama or rebuild with --features embedded-llm."
                .into(),
        };
    }

    EmbeddedLlmStatus {
        available: true,
        loaded: loaded_path.is_some(),
        model_path: loaded_path.map(|p| p.display().to_string()),
        message: "In-process GGUF inference ready (model cached on worker thread).".into(),
    }
}

#[tauri::command]
pub fn embedded_llm_status(state: tauri::State<'_, EmbeddedLlmState>) -> EmbeddedLlmStatus {
    status(state.inner())
}

#[tauri::command]
pub fn embedded_llm_complete(
    state: tauri::State<'_, EmbeddedLlmState>,
    gguf_path: String,
    prompt: String,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    let path = PathBuf::from(gguf_path.trim());
    let max = max_tokens.unwrap_or(128).clamp(16, 512);
    let out = complete_with_gguf(&path, &prompt, max)?;
    record_loaded(state.inner(), path);
    Ok(out)
}

#[tauri::command]
pub fn embedded_llm_unload(state: tauri::State<'_, EmbeddedLlmState>) {
    clear_loaded(state.inner());
    unload_gguf_worker();
}
