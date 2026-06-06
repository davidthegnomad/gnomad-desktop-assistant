import { invoke } from "@tauri-apps/api/core";

export type ProviderMode = "cloud" | "local";
export type ThemeMode = "light" | "dark" | "system";

const LS_THEME = "omni_theme";
const LS_ONBOARDING = "omni_onboarding_complete";
const LS_PROVIDER = "omni_provider";
const LS_MODEL = "omni_model";
const LS_LOCAL_MODEL = "omni_local_model";
const LS_UPDATE_CHANNEL = "omni_update_channel";
const LS_AUTO_CHECK_UPDATES = "omni_auto_check_updates";
const LS_ERROR_LOG = "omni_error_log";
const LS_VOICE_INPUT = "omni_voice_input";

export type UpdateChannel = "stable" | "beta";

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
  return localStorage.getItem(LS_LOCAL_MODEL) || "";
}

export function setStoredLocalModel(model: string) {
  localStorage.setItem(LS_LOCAL_MODEL, model);
}

export function getStoredUpdateChannel(): UpdateChannel {
  return localStorage.getItem(LS_UPDATE_CHANNEL) === "beta" ? "beta" : "stable";
}

export function setStoredUpdateChannel(channel: UpdateChannel) {
  localStorage.setItem(LS_UPDATE_CHANNEL, channel);
}

export function getStoredAutoCheckUpdates(): boolean {
  return localStorage.getItem(LS_AUTO_CHECK_UPDATES) === "true";
}

export function setStoredAutoCheckUpdates(enabled: boolean) {
  localStorage.setItem(LS_AUTO_CHECK_UPDATES, enabled ? "true" : "false");
}

export function getErrorLogEnabled(): boolean {
  return localStorage.getItem(LS_ERROR_LOG) === "true";
}

export function setErrorLogEnabled(enabled: boolean) {
  localStorage.setItem(LS_ERROR_LOG, enabled ? "true" : "false");
}

export function getVoiceInputEnabled(): boolean {
  return localStorage.getItem(LS_VOICE_INPUT) === "true";
}

export function setVoiceInputEnabled(enabled: boolean) {
  localStorage.setItem(LS_VOICE_INPUT, enabled ? "true" : "false");
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

export async function deleteCredential(key: string): Promise<void> {
  await invoke("delete_credential", { key });
}

export async function hasCredential(key: string): Promise<boolean> {
  try {
    return await invoke<boolean>("has_credential", { key });
  } catch {
    return false;
  }
}

export async function hasProviderConfigured(): Promise<boolean> {
  try {
    return await invoke<boolean>("has_llm_configured");
  } catch {
    return false;
  }
}
