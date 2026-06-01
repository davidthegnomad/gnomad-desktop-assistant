use std::process::Command;

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "PowerShell command failed".into()
        } else {
            stderr
        })
    }
}

#[tauri::command]
pub fn get_active_window() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    {
        // Spawns a single unified AppleScript process to grab both app name and window title
        let script = "tell application \"System Events\"
            try
                set frontmostProcess to first process whose frontmost is true
                set processName to name of frontmostProcess
                try
                    set windowName to name of window 1 of frontmostProcess
                on error
                    set windowName to \"Unknown Window\"
                end try
                return processName & \"|||\" & windowName
            on error
                return \"Unknown|||Unknown Window\"
            end try
        end tell";

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        let (app_name, window_title) = match output {
            Ok(out) if out.status.success() => {
                let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let parts: Vec<&str> = res.split("|||").collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (res, "Unknown Window".to_string())
                }
            }
            _ => ("Unknown".to_string(), "Unknown Window".to_string()),
        };

        Ok(serde_json::json!({
            "os": "macos",
            "app_name": app_name,
            "window_title": window_title
        }))
    }

    #[cfg(target_os = "linux")]
    {
        // Try wayland/grim/compositor query or standard xdotool if available
        let app_name = if let Ok(output) = Command::new("xdotool").args(["getactivewindow", "getwindowclassname"]).output() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "Active Linux App".to_string()
        };

        let window_title = if let Ok(output) = Command::new("xdotool").args(["getactivewindow", "getwindowname"]).output() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            "Active Linux Window".to_string()
        };

        Ok(serde_json::json!({
            "os": "linux",
            "app_name": app_name,
            "window_title": window_title
        }))
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class GnomadWin32 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
}
"@;
$h = [GnomadWin32]::GetForegroundWindow();
$sb = New-Object System.Text.StringBuilder 512;
[void][GnomadWin32]::GetWindowText($h, $sb, 512);
$pid = 0u;
[void][GnomadWin32]::GetWindowThreadProcessId($h, [ref]$pid);
$proc = if ($pid -gt 0) { (Get-Process -Id $pid -ErrorAction SilentlyContinue).ProcessName } else { 'Unknown' };
Write-Output ($proc + '|||' + $sb.ToString())
"#;

        let (app_name, window_title) = match run_powershell(script) {
            Ok(res) => {
                let parts: Vec<&str> = res.split("|||").collect();
                if parts.len() >= 2 {
                    (
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                    )
                } else {
                    ("Unknown".into(), res)
                }
            }
            _ => ("Unknown".into(), "Unknown Window".into()),
        };

        Ok(serde_json::json!({
            "os": "windows",
            "app_name": app_name,
            "window_title": window_title
        }))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(serde_json::json!({
            "os": "other",
            "app_name": "Unknown",
            "window_title": "Unknown"
        }))
    }
}

#[tauri::command]
pub fn get_clipboard_text() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| format!("Failed to run pbpaste: {}", e))?;
        
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(text)
    }

    #[cfg(target_os = "linux")]
    {
        // Try wl-paste first (Wayland), then fallback to xclip (X11)
        if let Ok(output) = Command::new("wl-paste").output() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.is_empty() {
                return Ok(text);
            }
        }
        
        let output = Command::new("xclip")
            .args(["-o", "-selection", "clipboard"])
            .output();

        match output {
            Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
            Err(_) => Err("No clipboard utility found (wl-paste or xclip)".to_string()),
        }
    }

    #[cfg(target_os = "windows")]
    {
        match run_powershell(
            "Get-Clipboard -Raw -ErrorAction SilentlyContinue | Out-String -Width 4096",
        ) {
            Ok(text) => Ok(text),
            Err(e) => Err(format!("Clipboard read failed: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported operating system for clipboard access".to_string())
    }
}
