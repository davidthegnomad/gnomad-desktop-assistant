pub use gnomad_core::config::keychain::{
    delete_credential as delete_credential_inner, get_credential_value, has_credential as has_credential_inner,
    store_credential as store_credential_inner,
};

#[tauri::command]
pub fn store_credential(key: &str, value: &str) -> Result<(), String> {
    store_credential_inner(key, value)
}

#[tauri::command]
pub fn get_credential(key: &str) -> Result<String, String> {
    get_credential_value(key)
}

#[tauri::command]
pub fn has_credential(key: &str) -> Result<bool, String> {
    has_credential_inner(key)
}

#[tauri::command]
pub fn delete_credential(key: &str) -> Result<(), String> {
    delete_credential_inner(key)
}
