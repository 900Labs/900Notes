use tauri::State;

use crate::models::*;
use crate::AppState;

// ── Page Revisions ──

#[tauri::command]
pub fn get_page_revisions(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PageRevision>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_revisions(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_revision(id: String, state: State<'_, AppState>) -> Result<PageRevision, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_revision(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_revision(revision_id: String, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.restore_revision(&revision_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_revision(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_revision(&id).map_err(|e| e.to_string())
}

// ── Favorites ──

#[tauri::command]
pub fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Favorite>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_favorites().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_favorite(page_id: String, state: State<'_, AppState>) -> Result<Favorite, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_favorite(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_favorite(page_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_favorite(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_favorite(page_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.is_favorite(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_favorites(
    ordered_page_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.reorder_favorites(&ordered_page_ids)
        .map_err(|e| e.to_string())
}
