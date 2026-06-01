use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const STABLE_ENDPOINT: &str =
    "https://github.com/davidthegnomad/gnomad-desktop-assistant/releases/latest/download/latest.json";
const BETA_ENDPOINT: &str =
    "https://github.com/davidthegnomad/gnomad-desktop-assistant/releases/download/v0.1.0-alpha/latest.json";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub channel: String,
}

fn endpoint_for_channel(channel: &str) -> Result<Url, String> {
    let raw = if channel.eq_ignore_ascii_case("beta") {
        BETA_ENDPOINT
    } else {
        STABLE_ENDPOINT
    };
    Url::parse(raw).map_err(|e| format!("Invalid updater endpoint: {e}"))
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    channel: Option<String>,
) -> Result<UpdateCheckResult, String> {
    let channel = channel.unwrap_or_else(|| "stable".into());
    let current_version = app.package_info().version.to_string();
    let endpoint = endpoint_for_channel(&channel)?;

    let update = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Updater misconfigured: {e}"))?
        .build()
        .map_err(|e| format!("Updater build failed: {e}"))?
        .check()
        .await
        .map_err(|e| format!("Update check failed: {e}"))?;

    match update {
        Some(info) => Ok(UpdateCheckResult {
            available: true,
            current_version,
            version: Some(info.version.clone()),
            notes: info.body.clone(),
            date: info.date.map(|d| d.to_string()),
            channel,
        }),
        None => Ok(UpdateCheckResult {
            available: false,
            current_version,
            version: None,
            notes: None,
            date: None,
            channel,
        }),
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle, channel: Option<String>) -> Result<(), String> {
    let channel = channel.unwrap_or_else(|| "stable".into());
    let endpoint = endpoint_for_channel(&channel)?;
    let update = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|e| format!("Updater misconfigured: {e}"))?
        .build()
        .map_err(|e| format!("Updater build failed: {e}"))?
        .check()
        .await
        .map_err(|e| format!("Update check failed: {e}"))?
        .ok_or_else(|| "No update available.".to_string())?;

    update
        .download_and_install(
            |chunk, total| {
                let _ = (chunk, total);
            },
            || {},
        )
        .await
        .map_err(|e| format!("Update install failed: {e}"))?;

    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}
