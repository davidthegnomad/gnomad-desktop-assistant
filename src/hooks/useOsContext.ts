import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PlatformInfo } from "../lib/platform";

export function useOsContext(platformInfo: PlatformInfo | null) {
  const [activeApp, setActiveApp] = useState("Loading...");
  const [activeTitle, setActiveTitle] = useState("Loading...");
  const [clipboardSnippet, setClipboardSnippet] = useState("Empty");
  const [accessibilityGranted, setAccessibilityGranted] = useState(true);

  const requestPermissions = async () => {
    try {
      await invoke("request_accessibility_permissions");
    } catch (err) {
      console.error(err);
    }
  };

  const testElevation = async () => {
    try {
      const res = await invoke<string>("execute_elevated_command", {
        command: "echo Success",
      });
      alert(`Elevation Successful: ${res}`);
    } catch (err) {
      alert(`Elevation Failed: ${err}`);
    }
  };

  useEffect(() => {
    const scrapeContext = async () => {
      if (platformInfo?.supportsActiveWindowContext) {
        try {
          const windowContext = await invoke<{
            app_name?: string;
            window_title?: string;
          }>("get_active_window");
          if (windowContext) {
            setActiveApp(windowContext.app_name || "Desktop");
            setActiveTitle(windowContext.window_title || "Screen");
          }
        } catch (err) {
          console.error("Window context scraping error:", err);
        }
      }

      if (platformInfo?.supportsClipboardContext) {
        try {
          const clip = await invoke<string>("get_clipboard_text");
          if (clip) {
            const trimmed = clip.trim();
            setClipboardSnippet(
              trimmed.length > 30 ? trimmed.substring(0, 30) + "..." : trimmed
            );
          } else {
            setClipboardSnippet("Empty");
          }
        } catch {
          setClipboardSnippet("Unreadable");
        }
      }
    };

    const checkPermissions = async () => {
      if (!platformInfo?.supportsAccessibilitySettings) {
        setAccessibilityGranted(true);
        return;
      }
      try {
        const granted = await invoke<boolean>("check_accessibility_permissions");
        setAccessibilityGranted(granted);
      } catch (err) {
        console.error(err);
      }
    };

    if (!platformInfo) return;

    scrapeContext();
    checkPermissions();
    const interval = setInterval(() => {
      scrapeContext();
      checkPermissions();
    }, 2500);
    return () => clearInterval(interval);
  }, [platformInfo]);

  return {
    activeApp,
    activeTitle,
    clipboardSnippet,
    accessibilityGranted,
    requestPermissions,
    testElevation,
  };
}
