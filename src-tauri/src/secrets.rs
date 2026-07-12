//! Passwords live in the OS keychain (Windows Credential Manager, macOS
//! Keychain, Linux kernel keyring) — never in the config file on disk.

const SERVICE: &str = "com.chincherpa.db-cloner";

fn entry(connection_id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, connection_id)
        .map_err(|e| format!("Schlüsselbund nicht verfügbar: {e}"))
}

pub fn set_password(connection_id: &str, password: &str) -> Result<(), String> {
    entry(connection_id)?
        .set_password(password)
        .map_err(|e| format!("Passwort konnte nicht im Schlüsselbund gespeichert werden: {e}"))
}

pub fn get_password(connection_id: &str) -> Result<String, String> {
    entry(connection_id)?.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => {
            "Kein Passwort für diese Verbindung gespeichert. Bitte in den Einstellungen neu eingeben.".to_string()
        }
        other => format!("Passwort konnte nicht gelesen werden: {other}"),
    })
}

pub fn delete_password(connection_id: &str) {
    if let Ok(e) = entry(connection_id) {
        let _ = e.delete_credential();
    }
}
