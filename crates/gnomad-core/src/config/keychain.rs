use keyring::Entry;

const SERVICE_NAME: &str = "com.gnomadstudio.gnomad";

pub fn store_credential(key: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to initialize keyring entry: {}", e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to write credential to system keychain: {}", e))?;

    Ok(())
}

pub fn get_credential_value(key: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to initialize keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => Ok(password),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(format!(
            "Failed to retrieve credential from system keychain: {}",
            e
        )),
    }
}

pub fn delete_credential(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to initialize keyring entry: {}", e))?;

    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!(
            "Failed to delete credential from system keychain: {}",
            e
        )),
    }
}

pub fn has_credential(key: &str) -> Result<bool, String> {
    let value = get_credential_value(key)?;
    Ok(!value.trim().is_empty())
}
