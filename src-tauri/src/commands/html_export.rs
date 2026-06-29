use tauri::State;

use crate::services::html_export;
use crate::AppState;

#[tauri::command]
pub fn export_page_html(page_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let page = db.get_page_by_id(&page_id).map_err(|e| e.to_string())?;
    Ok(html_export::export_page_html(&page))
}

#[tauri::command]
pub fn export_pages_html(
    page_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    html_export::export_pages_html(&db, &page_ids).map_err(|e| e.to_string())
}
