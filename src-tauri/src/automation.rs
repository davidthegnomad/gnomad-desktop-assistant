use std::process::Command;
use enigo::{Enigo, Settings, Keyboard, Mouse, Coordinate};
use tauri::Manager;

#[tauri::command]
pub fn capture_screen(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Generate a temporary screenshot path inside the app's cache directory
    let cache_dir = app_handle.path().app_cache_dir()
        .map_err(|e| format!("Failed to get app cache directory: {}", e))?;
    
    // Ensure directory exists
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        
    let screenshot_path = cache_dir.join("screenshot.png");
    let path_str = screenshot_path.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        // -x flag silences the shutter sound
        let output = Command::new("screencapture")
            .args(["-x", &path_str])
            .output()
            .map_err(|e| format!("Failed to execute screencapture: {}", e))?;

        if output.status.success() {
            Ok(path_str)
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try grim first (Wayland standard screenshot tool)
        if let Ok(output) = Command::new("grim").arg(&path_str).output() {
            if output.status.success() {
                return Ok(path_str);
            }
        }

        // Try gnome-screenshot
        if let Ok(output) = Command::new("gnome-screenshot").args(["-f", &path_str]).output() {
            if output.status.success() {
                return Ok(path_str);
            }
        }

        // Try import (ImageMagick)
        if let Ok(output) = Command::new("import").args(["-window", "root", &path_str]).output() {
            if output.status.success() {
                return Ok(path_str);
            }
        }

        Err("No screenshot utility found (grim, gnome-screenshot, or import)".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported operating system for screen capture".to_string())
    }
}

#[tauri::command]
pub fn simulate_click(x: i32, y: i32) -> Result<(), String> {
    // Try Enigo first
    let settings = Settings::default();
    if let Ok(mut enigo) = Enigo::new(&settings) {
        if enigo.move_mouse(x, y, Coordinate::Abs).is_ok() && 
           enigo.button(enigo::Button::Left, enigo::Direction::Click).is_ok() {
            return Ok(());
        }
    }

    // Fallback: OS-level commands
    #[cfg(target_os = "macos")]
    {
        let apple_script = format!(
            "tell application \"System Events\" to click at {{{}, {}}}",
            x, y
        );
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&apple_script)
            .output();
            
        match output {
            Ok(out) if out.status.success() => Ok(()),
            _ => Err("Enigo click failed, and AppleScript click fallback failed (likely requires Accessibility permission)".to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    {
        if crate::platform::linux_session_type() == "x11" {
            let output = Command::new("xdotool")
                .args(["mousemove", &x.to_string(), &y.to_string(), "click", "1"])
                .output();

            match output {
                Ok(out) if out.status.success() => return Ok(()),
                _ => {}
            }
        }
        Err("Enigo click failed; xdotool is unavailable on Wayland".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported operating system for input simulation".to_string())
    }
}

#[tauri::command]
pub fn simulate_typing(text: &str) -> Result<(), String> {
    // Try Enigo first
    let settings = Settings::default();
    if let Ok(mut enigo) = Enigo::new(&settings) {
        if enigo.text(text).is_ok() {
            return Ok(());
        }
    }

    // Fallback: OS-level commands
    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to type keystrokes
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let apple_script = format!(
            "tell application \"System Events\" to keystroke \"{}\"",
            escaped_text
        );
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&apple_script)
            .output();
            
        match output {
            Ok(out) if out.status.success() => Ok(()),
            _ => Err("Enigo typing failed, and AppleScript keystroke fallback failed".to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Fallback to xdotool (X11) or wtype (Wayland)
        if let Ok(output) = Command::new("wtype").arg(text).output() {
            if output.status.success() {
                return Ok(());
            }
        }

        if crate::platform::linux_session_type() == "x11" {
            let output = Command::new("xdotool").args(["type", text]).output();
            match output {
                Ok(out) if out.status.success() => return Ok(()),
                _ => {}
            }
        }

        Err("Enigo typing failed; install wtype on Wayland or use xdotool on X11".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported operating system for input simulation".to_string())
    }
}
