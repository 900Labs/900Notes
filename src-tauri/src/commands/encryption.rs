use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::{AppHandle, Manager, State};

use crate::services::encryption::EncryptionService;
use crate::AppState;

fn get_encryption_service(app: &AppHandle) -> Result<EncryptionService, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join("900notes.db");
    Ok(EncryptionService::new(&db_path))
}

#[tauri::command]
pub fn is_encryption_enabled(app: AppHandle) -> Result<bool, String> {
    let service = get_encryption_service(&app)?;
    Ok(service.is_encrypted())
}

#[tauri::command]
pub fn enable_encryption(
    passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let service = get_encryption_service(&app)?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join("900notes.db");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    // Swap to in-memory DB to release the file lock before reading the file
    let placeholder =
        crate::db::Database::open(std::path::Path::new(":memory:")).map_err(|e| e.to_string())?;
    *state.db.lock().map_err(|e| e.to_string())? = placeholder;

    service.enable_encryption(&passphrase, &db_path)?;

    // Decrypt back to disk so the current session can continue working.
    // The plaintext DB will be re-encrypted and deleted on app shutdown.
    service.decrypt_to_path(&passphrase, &db_path)?;
    let database = crate::db::Database::open(&db_path)
        .map_err(|e| format!("Open DB after encryption: {e}"))?;
    *state.db.lock().map_err(|e| e.to_string())? = database;

    *state.passphrase.lock().map_err(|e| e.to_string())? = Some(passphrase);

    Ok(())
}

#[tauri::command]
pub fn unlock_database(
    passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let service = get_encryption_service(&app)?;
    if !service.is_encrypted() {
        return Ok(false);
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join("900notes.db");

    service.decrypt_to_path(&passphrase, &db_path)?;

    let database =
        crate::db::Database::open(&db_path).map_err(|e| format!("Open decrypted DB: {e}"))?;

    *state.db.lock().map_err(|e| e.to_string())? = database;
    *state.passphrase.lock().map_err(|e| e.to_string())? = Some(passphrase);

    if let Err(error) = crate::start_web_clipper(&state) {
        eprintln!("Failed to start web clipper server after unlock: {error}");
    }

    Ok(true)
}

#[tauri::command]
pub fn disable_encryption(
    passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let service = get_encryption_service(&app)?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join("900notes.db");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    service.disable_encryption(&passphrase, &db_path)?;

    let database =
        crate::db::Database::open(&db_path).map_err(|e| format!("Open plain DB: {e}"))?;
    *state.db.lock().map_err(|e| e.to_string())? = database;
    *state.passphrase.lock().map_err(|e| e.to_string())? = None;

    if let Err(error) = crate::start_web_clipper(&state) {
        eprintln!("Failed to start web clipper server after disabling encryption: {error}");
    }

    Ok(())
}

#[tauri::command]
pub fn change_passphrase(
    old_passphrase: String,
    new_passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let service = get_encryption_service(&app)?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join("900notes.db");
    let temp_path = app_data_dir.join("900notes_temp.db");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    service.change_passphrase(&old_passphrase, &new_passphrase, &temp_path)?;

    let database = crate::db::Database::open(&db_path).map_err(|e| format!("Open DB: {e}"))?;
    *state.db.lock().map_err(|e| e.to_string())? = database;
    *state.passphrase.lock().map_err(|e| e.to_string())? = Some(new_passphrase);

    Ok(())
}

#[tauri::command]
pub fn verify_passphrase(passphrase: String, app: AppHandle) -> Result<bool, String> {
    let service = get_encryption_service(&app)?;
    service.verify_passphrase(&passphrase)
}

#[tauri::command]
pub fn export_encrypted_workspace(
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let export =
        crate::services::export_import::export_workspace(&db).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec(&export).map_err(|e| format!("Serialize: {e}"))?;
    let encrypted = crate::services::encryption::encrypt_data(&json, &passphrase)?;
    let encoded = BASE64.encode(&encrypted);
    Ok(encoded)
}

#[tauri::command]
pub fn import_encrypted_workspace(
    encrypted_data: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let decoded = BASE64
        .decode(&encrypted_data)
        .map_err(|e| format!("Decode: {e}"))?;
    let plaintext = crate::services::encryption::decrypt_data(&decoded, &passphrase)?;
    let export: crate::services::export_import::WorkspaceExport =
        serde_json::from_slice(&plaintext).map_err(|e| format!("Deserialize: {e}"))?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::services::export_import::import_workspace(&db, &export).map_err(|e| e.to_string())
}
