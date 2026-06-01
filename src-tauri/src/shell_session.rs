use crate::error::{into_invoke_err, GnomadError};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

const START_MARKER: &str = "__GNOMAD_START__";
const EXIT_PREFIX: &str = "__GNOMAD_EXIT__";
const CWD_PREFIX: &str = "__GNOMAD_CWD__";

/// How long with no new PTY bytes while waiting for exit marker → stalled (e.g. password prompt).
const DEFAULT_STALL_MS: u64 = 45_000;
/// Poll interval while waiting for command completion.
const WAIT_POLL_MS: u64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellRunState {
    /// Exit marker seen, status 0.
    Completed,
    /// Exit marker seen, non-zero status.
    Failed,
    /// Exceeded max wait; partial output returned.
    Timeout,
    /// No output for stall window while command still running.
    Stalled,
    /// Wait ended without exit marker (shell hiccup / parse miss).
    Incomplete,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShellRunResult {
    pub state: ShellRunState,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub status_code: i32,
    pub cwd: String,
    pub duration_ms: u64,
    pub saw_output: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShellSessionStatus {
    pub active: bool,
    pub shell: String,
    pub cwd: String,
}

#[derive(Clone, Serialize)]
struct ShellOutputEvent {
    chunk: String,
    stream: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
struct ShellRunProgressEvent {
    command: String,
    phase: String,
    exit_code: Option<i32>,
    message: Option<String>,
}

struct RunWaiter {
    active: AtomicBool,
    buffer: Mutex<String>,
    done: (Mutex<bool>, Condvar),
    last_output_at: Arc<Mutex<Instant>>,
    saw_start: AtomicBool,
}

enum WaitOutcome {
    Finished(String),
    Timeout(String),
    Stalled(String),
}

impl RunWaiter {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            buffer: Mutex::new(String::new()),
            done: (Mutex::new(false), Condvar::new()),
            last_output_at: Arc::new(Mutex::new(Instant::now())),
            saw_start: AtomicBool::new(false),
        }
    }

    fn begin(&self) {
        self.active.store(true, Ordering::SeqCst);
        *self.buffer.lock().unwrap() = String::new();
        *self.done.0.lock().unwrap() = false;
        self.saw_start.store(false, Ordering::SeqCst);
        *self.last_output_at.lock().unwrap() = Instant::now();
    }

    fn push_chunk(&self, chunk: &str) -> bool {
        if !self.active.load(Ordering::SeqCst) {
            return false;
        }
        *self.last_output_at.lock().unwrap() = Instant::now();
        let mut buf = self.buffer.lock().unwrap();
        buf.push_str(chunk);
        if buf.contains(START_MARKER) {
            self.saw_start.store(true, Ordering::SeqCst);
        }
        if buf.contains(EXIT_PREFIX) {
            *self.done.0.lock().unwrap() = true;
            self.done.1.notify_all();
            return true;
        }
        false
    }

    fn snapshot(&self) -> String {
        self.buffer.lock().unwrap().clone()
    }

    fn finish(&self) -> String {
        self.active.store(false, Ordering::SeqCst);
        self.snapshot()
    }

    fn idle_for(&self) -> Duration {
        self.last_output_at
            .lock()
            .unwrap()
            .elapsed()
    }

    fn wait(
        &self,
        timeout: Duration,
        stall: Duration,
    ) -> Result<WaitOutcome, String> {
        let started = Instant::now();
        let (lock, cvar) = &self.done;
        let mut done = lock.lock().unwrap();

        loop {
            if *done {
                return Ok(WaitOutcome::Finished(self.finish()));
            }

            let elapsed = started.elapsed();
            if elapsed >= timeout {
                let partial = self.finish();
                return Ok(WaitOutcome::Timeout(partial));
            }

            if self.saw_start.load(Ordering::SeqCst) && self.idle_for() >= stall {
                let partial = self.finish();
                return Ok(WaitOutcome::Stalled(partial));
            }

            let remaining = timeout.saturating_sub(elapsed);
            let wait_for = Duration::from_millis(WAIT_POLL_MS).min(remaining);
            let (guard, wait_result) = cvar
                .wait_timeout(done, wait_for)
                .map_err(|e| e.to_string())?;
            done = guard;
            if *done {
                return Ok(WaitOutcome::Finished(self.finish()));
            }
            if wait_result.timed_out() {
                continue;
            }
        }
    }
}

struct ActiveSession {
    #[allow(dead_code)]
    child: Box<dyn portable_pty::Child + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    waiter: Arc<RunWaiter>,
    cwd: PathBuf,
    shell: String,
}

pub struct ShellSessionState {
    inner: Mutex<Option<ActiveSession>>,
}

impl Default for ShellSessionState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

fn emit_run_progress(
    app: &AppHandle,
    command: &str,
    phase: &str,
    exit_code: Option<i32>,
    message: Option<String>,
) {
    let _ = app.emit(
        "shell-run-progress",
        ShellRunProgressEvent {
            command: command.to_string(),
            phase: phase.to_string(),
            exit_code,
            message,
        },
    );
}

fn send_interrupt(writer: &Arc<Mutex<Box<dyn Write + Send>>>) {
    let Ok(mut w) = writer.lock() else {
        return;
    };
    let _ = w.write_all(&[0x03]); // Ctrl+C
    let _ = w.flush();
}

fn default_shell() -> (String, Vec<String>) {
    #[cfg(target_os = "windows")]
    {
        return ("cmd.exe".to_string(), vec!["/Q".to_string()]);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        (shell, vec!["-f".to_string()])
    }
}

fn default_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"))
    })
}

fn shell_escape(s: &str) -> String {
    if cfg!(target_os = "windows") {
        s.replace('"', "\"\"")
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// Reject English instructions; allow real CLI (including `test`, `uname`, `.`, `source`, etc.).
fn is_probably_natural_language(command: &str) -> bool {
    let lower = command.to_lowercase();
    const PHRASES: &[&str] = &[
        "try to",
        "install it again",
        "please",
        "could you",
        "would you",
        "let me",
        "make sure",
        "check if",
        "is installed",
        "are installed",
        "not installed",
        "run the",
        "you can",
        "you should",
        "want to",
        "need to",
        "whether",
        "installed or not",
        "homebrew is",
    ];
    if PHRASES.iter().any(|p| lower.contains(p)) {
        return true;
    }
    if lower.contains("__gnomad") {
        return true;
    }

    let words: Vec<&str> = command.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }

    let first = words[0].to_ascii_lowercase();
    if matches!(
        first.as_str(),
        "try" | "please" | "could" | "would" | "check" | "verify" | "see" | "find" | "tell"
            | "help" | "ensure" | "make"
    ) {
        let second = words[1].to_ascii_lowercase();
        if matches!(
            second.as_str(),
            "to" | "if" | "the" | "that" | "whether" | "out" | "it" | "me" | "sure"
        ) {
            return true;
        }
    }

    // Long all-alpha sentence with no shell syntax → likely prose from the model.
    if words.len() >= 4
        && !command
            .chars()
            .any(|c| matches!(c, '|' | '&' | ';' | '$' | '`' | '<' | '>' | '=' | '\\'))
        && !command.contains('/')
        && !command.contains('-')
    {
        let alpha_words = words
            .iter()
            .filter(|w| w.chars().all(|c| c.is_ascii_alphabetic()))
            .count();
        if alpha_words >= 4 {
            return true;
        }
    }

    false
}

pub(crate) fn looks_like_shell_command(command: &str) -> bool {
    let t = command.trim();
    if t.is_empty() || t.len() > 2000 {
        return false;
    }
    if is_probably_natural_language(t) {
        return false;
    }
    let first = t.split_whitespace().next().unwrap_or("");
    !first.is_empty()
        && first.len() <= 128
        && first
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '\\' | '@' | '$' | '~'))
}

fn build_wrapped_command(command: &str, cwd: &PathBuf) -> String {
    #[cfg(target_os = "windows")]
    {
        let cwd_s = cwd.to_string_lossy();
        let cmd_escaped = command.replace('"', "\"\"");
        format!(
            "@echo {START_MARKER} & cd /d \"{cwd_s}\" 2>nul & {cmd_escaped} & echo {EXIT_PREFIX}%ERRORLEVEL%"
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        let cwd_s = shell_escape(&cwd.to_string_lossy());
        let cmd_q = shell_escape(command);
        format!(
            "printf '%s\\n' '{START_MARKER}'; cd {cwd_s} 2>/dev/null || true; eval {cmd_q}; __gnomad_ec=$?; printf '%s%s\\n' '{CWD_PREFIX}' \"$(pwd)\"; printf '%s%s\\n' '{EXIT_PREFIX}' \"$__gnomad_ec\""
        )
    }
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.next() == Some('[') {
                for ch in chars.by_ref() {
                    if ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn sanitize_terminal_output(text: &str) -> String {
    let cleaned = strip_ansi(text);
    let mut lines: Vec<&str> = Vec::new();
    for line in cleaned.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.contains(START_MARKER)
            || t.contains(EXIT_PREFIX)
            || t.contains(CWD_PREFIX)
            || t.contains("__gnomad_ec")
            || t.starts_with("printf '%s")
            || (t.contains("2>/dev/null") && t.contains("|| true"))
        {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_string()
}

fn build_shell_init() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        ""
    }
    #[cfg(not(target_os = "windows"))]
    {
        "unsetopt PROMPT_SP 2>/dev/null; export PS1=''; export RPROMPT=''; stty -echo 2>/dev/null; true"
    }
}

fn parse_run_output(raw: &str) -> (String, Option<i32>, Option<String>) {
    let mut status_code: Option<i32> = None;
    let mut session_cwd: Option<String> = None;
    let mut output = raw.to_string();

    if let Some(idx) = output.rfind(CWD_PREFIX) {
        let tail = &output[idx + CWD_PREFIX.len()..];
        let path: String = tail
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if !path.is_empty() && (path.starts_with('/') || path.contains(":\\")) {
            session_cwd = Some(path);
        }
    }

    if let Some(idx) = output.rfind(EXIT_PREFIX) {
        let tail = &output[idx + EXIT_PREFIX.len()..];
        let code_str: String = tail
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        if let Ok(code) = code_str.parse::<i32>() {
            status_code = Some(code);
        }
        output.truncate(idx);
    }

    if let Some(idx) = output.find(START_MARKER) {
        output = output[idx + START_MARKER.len()..].to_string();
    }

    let trimmed = sanitize_terminal_output(output.trim());
    (trimmed, status_code, session_cwd)
}

fn classify_result(wait: WaitOutcome, duration_ms: u64) -> ShellRunResult {
    match wait {
        WaitOutcome::Finished(raw) => {
            let (stdout, code, _) = parse_run_output(&raw);
            if let Some(code) = code {
                let saw_output = !stdout.is_empty();
                let success = code == 0;
                let state = if success {
                    ShellRunState::Completed
                } else {
                    ShellRunState::Failed
                };
                let msg = if success {
                    "Command completed successfully.".into()
                } else {
                    format!("Command failed with exit code {code}.")
                };
                return ShellRunResult {
                    state,
                    success,
                    stdout,
                    stderr: String::new(),
                    status_code: code,
                    cwd: String::new(),
                    duration_ms,
                    saw_output,
                    message: msg,
                };
            }
            let saw_output = !stdout.is_empty();
            ShellRunResult {
                state: ShellRunState::Incomplete,
                success: false,
                stdout,
                stderr: String::new(),
                status_code: -1,
                cwd: String::new(),
                duration_ms,
                saw_output,
                message: "Terminal output did not include an exit status marker.".into(),
            }
        }
        WaitOutcome::Timeout(partial) => {
            let (stdout, code, _) = parse_run_output(&partial);
            let saw_output = !stdout.is_empty();
            ShellRunResult {
                state: ShellRunState::Timeout,
                success: false,
                stdout,
                stderr: String::new(),
                status_code: code.unwrap_or(-1),
                cwd: String::new(),
                duration_ms,
                saw_output,
                message: format!(
                    "Command timed out after {}s. Partial terminal output was captured.",
                    duration_ms / 1000
                ),
            }
        }
        WaitOutcome::Stalled(partial) => {
            let (stdout, code, _) = parse_run_output(&partial);
            let saw_output = !stdout.is_empty();
            ShellRunResult {
                state: ShellRunState::Stalled,
                success: false,
                stdout,
                stderr: String::new(),
                status_code: code.unwrap_or(-1),
                cwd: String::new(),
                duration_ms,
                saw_output,
                message: "Command appears stalled — no new terminal output (may be waiting for password or input).".into(),
            }
        }
    }
}

fn spawn_session(
    app: &AppHandle,
    cwd: PathBuf,
    sandbox: bool,
    workspace: PathBuf,
) -> Result<ActiveSession, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Failed to open PTY: {e}"))?;

    let (shell, args) = default_shell();
    let (program, program_args) = if sandbox {
        crate::shell_sandbox::sandboxed_shell_command(app, &workspace, &shell, &args)?
    } else {
        (shell.clone(), args.clone())
    };

    let mut cmd = CommandBuilder::new(&program);
    for arg in program_args {
        cmd.arg(arg);
    }
    cmd.cwd(&cwd);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn shell in PTY: {e}"))?;

    #[cfg(not(target_os = "windows"))]
    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("PTY writer error: {e}"))?;
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("PTY reader error: {e}"))?;

    let writer: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(writer));
    let waiter = Arc::new(RunWaiter::new());
    let waiter_reader = Arc::clone(&waiter);
    let app_reader = app.clone();

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if !chunk.is_empty() {
                        let _ = app_reader.emit(
                            "shell-output",
                            ShellOutputEvent {
                                chunk: chunk.clone(),
                                stream: "stdout".into(),
                            },
                        );
                        waiter_reader.push_chunk(&chunk);
                    }
                }
                Err(_) => break,
            }
        }
    });

    std::thread::sleep(Duration::from_millis(300));
    let init = build_shell_init();
    if !init.is_empty() {
        write_line(&writer, init)?;
        std::thread::sleep(Duration::from_millis(150));
    }

    Ok(ActiveSession {
        child,
        writer,
        waiter,
        cwd,
        shell,
    })
}

fn ensure_session(
    app: &AppHandle,
    state: &ShellSessionState,
    settings_state: &crate::agent_settings::AgentSettingsState,
    cwd: Option<String>,
) -> Result<(), String> {
    let desired = cwd
        .map(PathBuf::from)
        .filter(|p| p.is_absolute() || p.exists())
        .unwrap_or_else(default_cwd);

    let (sandbox, workspace) = {
        let guard = settings_state.inner.lock().map_err(|e| e.to_string())?;
        (
            crate::agent_settings::should_sandbox_shell(&guard),
            guard.workspace_root.clone(),
        )
    };

    let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
    let needs_spawn = match guard.as_ref() {
        None => true,
        Some(s) => s.cwd != desired,
    };

    if needs_spawn {
        if let Some(mut old) = guard.take() {
            let _ = old.child.kill();
        }
        let session = spawn_session(app, desired, sandbox, workspace)?;
        *guard = Some(session);
    }
    Ok(())
}

fn write_line(writer: &Arc<Mutex<Box<dyn Write + Send>>>, line: &str) -> Result<(), String> {
    let mut w = writer.lock().map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    let payload = format!("{line}\r\n");
    #[cfg(not(target_os = "windows"))]
    let payload = format!("{line}\n");
    w.write_all(payload.as_bytes())
        .map_err(|e| format!("Failed to write to shell: {e}"))?;
    w.flush().map_err(|e| format!("Failed to flush shell: {e}"))?;
    Ok(())
}

pub fn session_cwd_snapshot(state: &ShellSessionState) -> (String, bool) {
    let guard = match state.inner.lock() {
        Ok(g) => g,
        Err(_) => return (String::new(), false),
    };
    match guard.as_ref() {
        Some(s) => (s.cwd.to_string_lossy().to_string(), true),
        None => (String::new(), false),
    }
}

#[tauri::command]
pub fn shell_session_status(state: State<'_, ShellSessionState>) -> Result<ShellSessionStatus, String> {
    let guard = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(match guard.as_ref() {
        Some(s) => ShellSessionStatus {
            active: true,
            shell: s.shell.clone(),
            cwd: s.cwd.to_string_lossy().to_string(),
        },
        None => ShellSessionStatus {
            active: false,
            shell: default_shell().0,
            cwd: default_cwd().to_string_lossy().to_string(),
        },
    })
}

#[tauri::command]
pub fn shell_session_reset(state: State<'_, ShellSessionState>) -> Result<(), String> {
    let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
    if let Some(mut old) = guard.take() {
        let _ = old.child.kill();
    }
    Ok(())
}

#[tauri::command]
pub fn shell_session_interrupt(state: State<'_, ShellSessionState>) -> Result<(), String> {
    let guard = state.inner.lock().map_err(|e| e.to_string())?;
    if let Some(session) = guard.as_ref() {
        send_interrupt(&session.writer);
    }
    Ok(())
}

#[tauri::command]
pub fn validate_shell_command(command: String) -> bool {
    looks_like_shell_command(command.trim())
}

pub fn run_shell_command(
    app: &AppHandle,
    state: &ShellSessionState,
    settings_state: &crate::agent_settings::AgentSettingsState,
    hitl_state: &crate::hitl_token::HitlTokenState,
    command: String,
    cwd: Option<String>,
    timeout_ms: Option<u64>,
    stall_ms: Option<u64>,
    hitl_approved: Option<bool>,
    approval_token: Option<String>,
) -> Result<ShellRunResult, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: "Command cannot be empty.".into(),
            detail: None,
        }));
    }

    if !looks_like_shell_command(trimmed) {
        let preview: String = trimmed.chars().take(120).collect();
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: format!("Not a valid shell command: \"{preview}\""),
            detail: Some(
                "Use a real CLI line, not English prose (e.g. command -v brew).".into(),
            ),
        }));
    }

    crate::hitl_token::enforce_hitl(
        hitl_state,
        trimmed,
        crate::hitl_token::HitlScope::ShellRun,
        approval_token.as_deref(),
        hitl_approved,
    )?;

    ensure_session(&app, &state, settings_state, cwd)?;
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(120_000).clamp(5_000, 600_000));
    let stall = Duration::from_millis(stall_ms.unwrap_or(DEFAULT_STALL_MS).clamp(5_000, 120_000));

    let (writer, waiter, cwd_path) = {
        let guard = state.inner.lock().map_err(|e| e.to_string())?;
        let session = guard
            .as_ref()
            .ok_or_else(|| "Shell session failed to start.".to_string())?;
        (
            Arc::clone(&session.writer),
            Arc::clone(&session.waiter),
            session.cwd.clone(),
        )
    };

    emit_run_progress(&app, trimmed, "started", None, None);

    let wrapped = build_wrapped_command(trimmed, &cwd_path);
    let run_started = Instant::now();
    waiter.begin();
    write_line(&writer, &wrapped)?;
    emit_run_progress(&app, trimmed, "running", None, None);

    let wait_result = waiter.wait(timeout, stall);

    let mut parsed_session_cwd: Option<String> = None;
    let mut result = match wait_result {
        Ok(outcome) => {
            if let WaitOutcome::Finished(ref raw) = outcome {
                let (_, _, cwd) = parse_run_output(raw);
                parsed_session_cwd = cwd;
            }
            if matches!(outcome, WaitOutcome::Timeout(_) | WaitOutcome::Stalled(_)) {
                send_interrupt(&writer);
                std::thread::sleep(Duration::from_millis(200));
            }
            classify_result(outcome, run_started.elapsed().as_millis() as u64)
        }
        Err(e) => {
            send_interrupt(&writer);
            return Err(e);
        }
    };

    if let Some(new_cwd) = parsed_session_cwd {
        let path = PathBuf::from(&new_cwd);
        if path.is_absolute() {
            if let Ok(mut guard) = state.inner.lock() {
                if let Some(session) = guard.as_mut() {
                    session.cwd = path;
                }
            }
            result.cwd = new_cwd;
        }
    }

    if result.cwd.is_empty() {
        result.cwd = state
            .inner
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|s| s.cwd.to_string_lossy().to_string())
            .unwrap_or_else(|| cwd_path.to_string_lossy().to_string());
    }

    let phase = match result.state {
        ShellRunState::Completed => "completed",
        ShellRunState::Failed => "failed",
        ShellRunState::Timeout => "timeout",
        ShellRunState::Stalled => "stalled",
        ShellRunState::Incomplete => "incomplete",
    };
    emit_run_progress(
        &app,
        trimmed,
        phase,
        Some(result.status_code),
        Some(result.message.clone()),
    );

    Ok(result)
}

#[tauri::command]
pub fn shell_session_run(
    app: AppHandle,
    state: State<'_, ShellSessionState>,
    settings_state: State<'_, crate::agent_settings::AgentSettingsState>,
    hitl_state: State<'_, crate::hitl_token::HitlTokenState>,
    command: String,
    cwd: Option<String>,
    timeout_ms: Option<u64>,
    stall_ms: Option<u64>,
    hitl_approved: Option<bool>,
    approval_token: Option<String>,
) -> Result<ShellRunResult, String> {
    run_shell_command(
        &app,
        state.inner(),
        settings_state.inner(),
        hitl_state.inner(),
        command,
        cwd,
        timeout_ms,
        stall_ms,
        hitl_approved,
        approval_token,
    )
}

#[cfg(test)]
mod tests {
    use super::{looks_like_shell_command, parse_run_output, EXIT_PREFIX, START_MARKER};

    #[test]
    fn parses_markers() {
        let raw = format!("noise\n{}\nhello\n{}13", START_MARKER, EXIT_PREFIX);
        let (out, code, _) = parse_run_output(&raw);
        assert_eq!(code, Some(13));
        assert!(out.contains("hello"));
    }

    #[test]
    fn shell_command_validation() {
        assert!(looks_like_shell_command("command -v brew"));
        assert!(looks_like_shell_command("brew install wget"));
        assert!(looks_like_shell_command("test -f /opt/homebrew/bin/brew"));
        assert!(looks_like_shell_command("uname -a"));
        assert!(looks_like_shell_command(". ./script.sh"));

        assert!(!looks_like_shell_command("try to install it again"));
        assert!(!looks_like_shell_command("check if brew is installed"));
        assert!(!looks_like_shell_command(""));
    }
}
