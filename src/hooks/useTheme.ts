import { useEffect, useState, useCallback } from "react";
import {
  applyThemeToDocument,
  getStoredTheme,
  resolveTheme,
  setStoredTheme,
  type ThemeMode,
} from "../lib/preferences";

export function useTheme() {
  const [themeMode, setThemeMode] = useState<ThemeMode>(getStoredTheme);
  const [resolved, setResolved] = useState<"light" | "dark">(() =>
    resolveTheme(getStoredTheme())
  );

  const apply = useCallback((mode: ThemeMode) => {
    const next = resolveTheme(mode);
    setResolved(next);
    applyThemeToDocument(next);
  }, []);

  useEffect(() => {
    apply(themeMode);
    setStoredTheme(themeMode);

    if (themeMode !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => apply("system");
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [themeMode, apply]);

  const cycleTheme = () => {
    setThemeMode((prev) => {
      if (prev === "light") return "dark";
      if (prev === "dark") return "system";
      return "light";
    });
  };

  const setTheme = (mode: ThemeMode) => setThemeMode(mode);

  return { themeMode, resolved, cycleTheme, setTheme };
}
