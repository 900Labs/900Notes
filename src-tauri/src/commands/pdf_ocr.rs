use tauri::State;

use crate::services::{ocr, pdf};
use crate::AppState;

#[tauri::command]
pub fn export_page_pdf(page_id: String, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let page = db.get_page_by_id(&page_id).map_err(|e| e.to_string())?;
    pdf::export_page_pdf(&page)
}

#[tauri::command]
pub fn export_workspace_pdf(state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let pages = db.get_all_pages().map_err(|e| e.to_string())?;
    pdf::export_pages_pdf(&pages)
}

#[tauri::command]
pub fn ocr_attachment(attachment_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let (attachment, data) = db
        .get_attachment(&attachment_id)
        .map_err(|e| e.to_string())?;
    if !attachment.is_image {
        return Err("Attachment is not an image".to_string());
    }
    ocr::ocr_image_bytes(&data, &attachment.mime_type)
}
