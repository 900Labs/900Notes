use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn get_backlinks(page_id: String, state: State<'_, AppState>) -> Result<Vec<Backlink>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_backlinks(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_outgoing_links(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Link>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_outgoing_links(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rebuild_links(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rebuild_all_links().map_err(|e| e.to_string())
}
