//! Persistent PTY shell session for agent commands and interactive terminal UI.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use crate::agent::read_settings;
use crate::agent::settings::should_sandbox_shell;
use crate::agent::tokens::hitl::{enforce_hitl, HitlScope, HitlTokenState};
use crate::agent::AgentSettingsState;
use crate::config::agent_secrets;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};
use crate::shell::exec::ShellOutputSink;
use crate::shell::exec::ShellRunResult;
use crate::shell::rules::looks_like_shell_command;
use crate::shell::sandbox::sandboxed_shell_command;

const START_MARKER: &str = "__GNOMAD_START__";
const EXIT_PREFIX: &str = "__GNOMAD_EXIT__";
const CWD_PREFIX: &str = "__GNOMAD_CWD__";
const DEFAULT_STALL_MS: u64 = 45_000;
const WAIT_POLL_MS: u64 = 200;

#[derive(Debug, Clone)]
pub struct ShellSessionStatus {
    pub active: bool,
    pub shell: String,
    pub cwd: String,
}

enum WaitOutcome {
    Finished(String),
    Timeout(String),
    Stalled(String),
}

struct RunWaiter {
    active: AtomicBool,
    buffer: Mutex<String>,
    done: (Mutex<bool>, Condvar),
    last_output_at: Arc<Mutex<Instant>>,
    saw_start: AtomicBool,
    run_sink: Mutex<Option<ShellOutputSink>>,
}

impl RunWaiter {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            buffer: Mutex::new(String::new()),
            done: (Mutex::new(false), Condvar::new()),
            last_output_at: Arc::new(Mutex::new(Instant::now())),
            saw_start: AtomicBool::new(false),
            run_sink: Mutex::new(None),
        }
    }

    fn begin(&self, run_sink: Option<ShellOutputSink>) {
        self.active.store(true, Ordering::SeqCst);
        *self.buffer.lock().unwrap() = String::new();
        *self.done.0.lock().unwrap() = false;
        self.saw_start.store(false, Ordering::SeqCst);
        *self.last_output_at.lock().unwrap() = Instant::now();
        *self.run_sink.lock().unwrap() = run_sink;
    }

    fn push_chunk(&self, chunk: &str, live_sink: Option<&ShellOutputSink>) -> bool {
        if !self.active.load(Ordering::SeqCst) {
            if let Some(sink) = live_sink {
                sink(chunk);
            }
            return false;
        }
        *self.last_output_at.lock().unwrap() = Instant::now();
        if let Some(sink) = live_sink {
            sink(chunk);
        }
        if let Some(sink) = self.run_sink.lock().unwrap().as_ref() {
            sink(chunk);
        }
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
        *self.run_sink.lock().unwrap() = None;
        self.snapshot()
    }

    fn idle_for(&self) -> Duration {
        self.last_output_at.lock().unwrap().elapsed()
    }

    fn wait(&self, timeout: Duration, stall: Duration) -> Result<WaitOutcome, String> {
        let started = Instant::now();
        let (lock, cvar) = &self.done;
        let mut done = lock.lock().unwrap();

        loop {
            if *done {
                return Ok(WaitOutcome::Finished(self.finish()));
            }

            let elapsed = started.elapsed();
            if elapsed >= timeout {
                return Ok(WaitOutcome::Timeout(self.finish()));
            }

            if self.saw_start.load(Ordering::SeqCst) && self.idle_for() >= stall {
                return Ok(WaitOutcome::Stalled(self.finish()));
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
    live_sink: Arc<Mutex<Option<ShellOutputSink>>>,
}

impl Default for ShellSessionState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            live_sink: Arc::new(Mutex::new(None)),
        }
    }
}

impl ShellSessionState {
    pub fn set_live_sink(&self, sink: Option<ShellOutputSink>) {
        *self.live_sink.lock().unwrap() = sink;
    }

    pub fn status(&self) -> ShellSessionStatus {
        if let Ok(guard) = self.inner.lock() {
            if let Some(s) = guard.as_ref() {
                return ShellSessionStatus {
                    active: true,
                    shell: s.shell.clone(),
                    cwd: s.cwd.to_string_lossy().to_string(),
                };
            }
        }
        ShellSessionStatus {
            active: false,
            shell: default_shell().0,
            cwd: default_cwd().to_string_lossy().to_string(),
        }
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(mut old) = guard.take() {
                let _ = old.child.kill();
            }
        }
    }

    pub fn interrupt(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(session) = guard.as_ref() {
                send_interrupt(&session.writer);
            }
        }
    }

    pub fn write_line(&self, line: &str) -> Result<(), String> {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        let session = guard
            .as_ref()
            .ok_or_else(|| "Shell session is not active.".to_string())?;
        write_line_to_writer(&session.writer, line)
    }

    pub fn ensure_session(
        &self,
        paths: &DataPaths,
        settings_state: &AgentSettingsState,
        cwd: Option<String>,
    ) -> Result<(), String> {
        let desired = cwd
            .map(PathBuf::from)
            .filter(|p| p.is_absolute() || p.exists())
            .unwrap_or_else(default_cwd);

        let settings = read_settings(settings_state);
        let sandbox = should_sandbox_shell(&settings);
        let workspace = settings.workspace_root.clone();
        let secrets_enabled = settings.agent_secrets_enabled;

        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        let needs_spawn = match guard.as_ref() {
            None => true,
            Some(s) => s.cwd != desired,
        };

        if needs_spawn {
            if let Some(mut old) = guard.take() {
                let _ = old.child.kill();
            }
            let session = spawn_session(
                paths,
                desired,
                sandbox,
                workspace,
                secrets_enabled,
                Arc::clone(&self.live_sink),
            )?;
            *guard = Some(session);
        }
        Ok(())
    }
}

fn default_shell() -> (String, Vec<String>) {
    #[cfg(target_os = "windows")]
    {
        return ("cmd.exe".to_string(), vec!["/Q".to_string()]);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        (shell, vec!["-i".to_string()])
    }
}

fn default_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"))
    })
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
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

fn parse_run_output(raw: &str) -> (String, Option<i32>, Option<String>) {
    let mut status_code: Option<i32> = None;
    let mut session_cwd: Option<String> = None;
    let mut output = raw.to_string();

    if let Some(idx) = output.rfind(CWD_PREFIX) {
        let tail = &output[idx + CWD_PREFIX.len()..];
        let path: String = tail.lines().next().unwrap_or("").trim().to_string();
        if !path.is_empty() && path.starts_with('/') {
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

    (sanitize_terminal_output(output.trim()), status_code, session_cwd)
}

fn classify_result(wait: WaitOutcome, duration_ms: u64) -> ShellRunResult {
    match wait {
        WaitOutcome::Finished(raw) => {
            let (stdout, code, _) = parse_run_output(&raw);
            if let Some(code) = code {
                let success = code == 0;
                return ShellRunResult {
                    success,
                    stdout,
                    stderr: String::new(),
                    status_code: Some(code),
                    cwd: String::new(),
                    state: if success {
                        "completed".into()
                    } else {
                        "failed".into()
                    },
                    message: if success {
                        "Command completed successfully.".into()
                    } else {
                        format!("Command failed with exit code {code}.")
                    },
                    duration_ms: Some(duration_ms),
                };
            }
            ShellRunResult {
                success: false,
                stdout,
                stderr: String::new(),
                status_code: Some(-1),
                cwd: String::new(),
                state: "incomplete".into(),
                message: "Terminal output did not include an exit status marker.".into(),
                duration_ms: Some(duration_ms),
            }
        }
        WaitOutcome::Timeout(partial) => {
            let (stdout, code, _) = parse_run_output(&partial);
            ShellRunResult {
                success: false,
                stdout,
                stderr: String::new(),
                status_code: code.or(Some(-1)),
                cwd: String::new(),
                state: "timeout".into(),
                message: format!(
                    "Command timed out after {}s. Partial terminal output was captured.",
                    duration_ms / 1000
                ),
                duration_ms: Some(duration_ms),
            }
        }
        WaitOutcome::Stalled(partial) => {
            let (stdout, code, _) = parse_run_output(&partial);
            ShellRunResult {
                success: false,
                stdout,
                stderr: String::new(),
                status_code: code.or(Some(-1)),
                cwd: String::new(),
                state: "stalled".into(),
                message: "Command appears stalled — no new terminal output (may be waiting for password or input).".into(),
                duration_ms: Some(duration_ms),
            }
        }
    }
}

fn send_interrupt(writer: &Arc<Mutex<Box<dyn Write + Send>>>) {
    if let Ok(mut w) = writer.lock() {
        let _ = w.write_all(&[0x03]);
        let _ = w.flush();
    }
}

fn write_line_to_writer(
    writer: &Arc<Mutex<Box<dyn Write + Send>>>,
    line: &str,
) -> Result<(), String> {
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

fn spawn_session(
    paths: &DataPaths,
    cwd: PathBuf,
    sandbox: bool,
    workspace: PathBuf,
    secrets_enabled: bool,
    live_sink_holder: Arc<Mutex<Option<ShellOutputSink>>>,
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
        sandboxed_shell_command(paths, &workspace, &shell, &args)?
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
    for (key, value) in agent_secrets::shell_env_map(paths, secrets_enabled) {
        cmd.env(key, value);
    }

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

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if !chunk.is_empty() {
                        let live_sink = live_sink_holder
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(Arc::clone));
                        waiter_reader.push_chunk(
                            &chunk,
                            live_sink.as_ref(),
                        );
                    }
                }
                Err(_) => break,
            }
        }
    });

    std::thread::sleep(Duration::from_millis(300));
    let init = build_shell_init();
    if !init.is_empty() {
        write_line_to_writer(&writer, init)?;
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

pub fn run_shell_command_in_session(
    paths: &DataPaths,
    settings_state: &AgentSettingsState,
    hitl_state: &HitlTokenState,
    session_state: &ShellSessionState,
    command: String,
    cwd: Option<String>,
    hitl_approved: Option<bool>,
    approval_token: Option<String>,
    on_output: Option<ShellOutputSink>,
    timeout_ms: Option<u64>,
    stall_ms: Option<u64>,
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

    enforce_hitl(
        hitl_state,
        trimmed,
        HitlScope::ShellRun,
        approval_token.as_deref(),
        hitl_approved,
    )?;

    session_state.ensure_session(paths, settings_state, cwd)?;

    let timeout = Duration::from_millis(timeout_ms.unwrap_or(120_000).clamp(5_000, 600_000));
    let stall = Duration::from_millis(stall_ms.unwrap_or(DEFAULT_STALL_MS).clamp(5_000, 120_000));

    let (writer, waiter, cwd_path) = {
        let guard = session_state.inner.lock().map_err(|e| e.to_string())?;
        let session = guard
            .as_ref()
            .ok_or_else(|| "Shell session failed to start.".to_string())?;
        (
            Arc::clone(&session.writer),
            Arc::clone(&session.waiter),
            session.cwd.clone(),
        )
    };

    let wrapped = build_wrapped_command(trimmed, &cwd_path);
    let run_started = Instant::now();
    waiter.begin(on_output);
    write_line_to_writer(&writer, &wrapped)?;

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
            if let Ok(mut guard) = session_state.inner.lock() {
                if let Some(session) = guard.as_mut() {
                    session.cwd = path;
                }
            }
            result.cwd = new_cwd;
        }
    }

    if result.cwd.is_empty() {
        result.cwd = session_state
            .inner
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|s| s.cwd.to_string_lossy().to_string()))
            .unwrap_or_else(|| cwd_path.to_string_lossy().to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{parse_run_output, EXIT_PREFIX, START_MARKER};
    use crate::shell::rules::looks_like_shell_command;

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
        assert!(!looks_like_shell_command("please install wget"));
    }
}
