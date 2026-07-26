use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::{AppHandle, Manager, State};

use crate::services::encryption::EncryptionService;
use crate::AppState;
use std::sync::atomic::Ordering;

fn get_encryption_service(app: &AppHandle) -> Result<EncryptionService, String> {
    let state = app.state::<AppState>();
    let db_path = state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
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
    crate::services::encryption::validate_passphrase(&passphrase)?;
    let service = get_encryption_service(&app)?;
    let db_path = state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    service.enable_encryption(&passphrase, &db_path)?;
    // The live plaintext now reflects a state we authored; bind it to the
    // snapshot so a later unlock can tell it apart from a swapped file.
    let _ = service.write_integrity_tag(&passphrase, &db_path);

    *state.passphrase.lock().map_err(|e| e.to_string())? = Some(passphrase);
    state.workspace_locked.store(false, Ordering::Release);

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

    let db_path = state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    // A plaintext database left by an interrupted session may be newer than the
    // encrypted snapshot. Validate the passphrase against the snapshot first,
    // then authenticate the leftover plaintext against an HMAC sidecar bound to
    // that snapshot before trusting it. A file swapped into the app data
    // directory by a local attacker fails this check and is discarded.
    if db_path.exists() {
        if !service.verify_passphrase(&passphrase)? {
            return Ok(false);
        }
        if !service.verify_integrity_tag(&passphrase, &db_path)? {
            // The recovery file is missing, stale, or tampered. Fall back to the
            // authoritative snapshot rather than opening an untrusted file.
            eprintln!(
                "Encryption integrity check failed for recovery DB; re-deriving from snapshot."
            );
            service.decrypt_to_path(&passphrase, &db_path)?;
        }
    } else {
        service.decrypt_to_path(&passphrase, &db_path)?;
    }

    let database =
        crate::db::Database::open(&db_path).map_err(|e| format!("Open decrypted DB: {e}"))?;

    *state.db.lock().map_err(|e| e.to_string())? = database;
    *state.passphrase.lock().map_err(|e| e.to_string())? = Some(passphrase.clone());
    state.workspace_locked.store(false, Ordering::Release);

    // Refresh the integrity sidecar so the file we just opened is trusted for
    // the remainder of this session.
    let _ = service.write_integrity_tag(&passphrase, &db_path);

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
    let db_path = state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    service.disable_encryption(&passphrase, &db_path)?;

    let database =
        crate::db::Database::open(&db_path).map_err(|e| format!("Open plain DB: {e}"))?;
    *state.db.lock().map_err(|e| e.to_string())? = database;
    *state.passphrase.lock().map_err(|e| e.to_string())? = None;
    state.workspace_locked.store(false, Ordering::Release);

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
    let db_path = state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    crate::services::encryption::validate_passphrase(&new_passphrase)?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.checkpoint().map_err(|e| e.to_string())?;
    drop(db);

    service.change_passphrase(&old_passphrase, &new_passphrase, &db_path)?;
    // Re-bind the integrity sidecar to the freshly written snapshot.
    let _ = service.write_integrity_tag(&new_passphrase, &db_path);
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
    crate::commands::export_import::restore_workspace_state(&export, &state)
}
