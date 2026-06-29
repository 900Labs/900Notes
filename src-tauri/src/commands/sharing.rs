use tauri::State;

use crate::services::sharing;
use crate::AppState;

#[tauri::command]
pub fn export_share_bundle(
    page_ids: Vec<String>,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let bundle = sharing::export_share_bundle(&db, &page_ids).map_err(|e| e.to_string())?;
    sharing::encrypt_bundle(&bundle, &passphrase)
}

#[tauri::command]
pub fn import_share_bundle(
    encrypted_data: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let bundle = sharing::decrypt_bundle(&encrypted_data, &passphrase)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    sharing::import_share_bundle(&db, &bundle).map_err(|e| e.to_string())
}
