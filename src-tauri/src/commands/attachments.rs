use serde::{Deserialize, Serialize};
use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn create_attachment(
    input: CreateAttachmentInput,
    state: State<'_, AppState>,
) -> Result<Attachment, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_attachment(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_attachment(
    id: String,
    state: State<'_, AppState>,
) -> Result<GetAttachmentResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let (attachment, data) = db.get_attachment(&id).map_err(|e| e.to_string())?;
    Ok(GetAttachmentResponse { attachment, data })
}

#[tauri::command]
pub fn get_attachments_for_page(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Attachment>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_attachments_for_page(&page_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_attachment(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_attachment(&id).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAttachmentResponse {
    pub attachment: Attachment,
    pub data: Vec<u8>,
}
