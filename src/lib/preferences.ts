import { invoke } from "@tauri-apps/api/core";

export type ProviderMode = "cloud" | "local";
export type ThemeMode = "light" | "dark" | "system";

const LS_THEME = "omni_theme";
const LS_ONBOARDING = "omni_onboarding_complete";
const LS_PROVIDER = "omni_provider";
const LS_MODEL = "omni_model";
const LS_LOCAL_MODEL = "omni_local_model";

export function getStoredTheme(): ThemeMode {
  const v = localStorage.getItem(LS_THEME);
  if (v === "light" || v === "dark" || v === "system") return v;
  return "system";
}

export function setStoredTheme(theme: ThemeMode) {
  localStorage.setItem(LS_THEME, theme);
}

export function getOnboardingComplete(): boolean {
  return localStorage.getItem(LS_ONBOARDING) === "true";
}

export function setOnboardingComplete() {
  localStorage.setItem(LS_ONBOARDING, "true");
}

export function getStoredProvider(): ProviderMode {
  return localStorage.getItem(LS_PROVIDER) === "local" ? "local" : "cloud";
}

export function setStoredProvider(mode: ProviderMode) {
  localStorage.setItem(LS_PROVIDER, mode);
}

export function getStoredModel(): string {
  return localStorage.getItem(LS_MODEL) || "deepseek-chat";
}

export function setStoredModel(model: string) {
  localStorage.setItem(LS_MODEL, model);
}

export function getStoredLocalModel(): string {
  return localStorage.getItem(LS_LOCAL_MODEL) || "llama3.2";
}

export function setStoredLocalModel(model: string) {
  localStorage.setItem(LS_LOCAL_MODEL, model);
}

export function resolveTheme(mode: ThemeMode): "light" | "dark" {
  if (mode === "light" || mode === "dark") return mode;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function applyThemeToDocument(resolved: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", resolved);
}

export async function loadCredential(key: string): Promise<string> {
  try {
    return await invoke<string>("get_credential", { key });
  } catch {
    return "";
  }
}

export async function saveCredential(key: string, value: string): Promise<void> {
  await invoke("store_credential", { key, value });
}

export async function hasProviderConfigured(): Promise<boolean> {
  try {
    const configured = await invoke<boolean>("has_llm_configured");
    if (configured) return true;
  } catch {
    /* fallback below */
  }
  const provider = getStoredProvider();
  if (provider === "cloud") {
    const key = await loadCredential("llm_api_key");
    return key.trim().length > 0;
  }
  const url = await loadCredential("ollama_url");
  return url.trim().length > 0;
}
