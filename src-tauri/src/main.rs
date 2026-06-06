// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    omni_taskbar_ai_lib::linux_webview::init_stability_env();
    omni_taskbar_ai_lib::run()
}
