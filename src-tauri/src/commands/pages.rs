use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn create_page(input: CreatePageInput, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_page(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page(id: String, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_by_id(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_pages(state: State<'_, AppState>) -> Result<Vec<Page>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_pages().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page_tree(state: State<'_, AppState>) -> Result<Vec<PageTreeNode>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_tree().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page_tree_metadata(state: State<'_, AppState>) -> Result<Vec<PageTreeNodeMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_tree_metadata().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page_titles(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_titles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_pages_metadata(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<PageMetadata>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_pages_metadata(limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_page(input: UpdatePageInput, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_page(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_page(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_page(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_page(id: String, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.restore_page(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn duplicate_page(id: String, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.duplicate_page(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_page(input: MovePageInput, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.move_page(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_pages(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<Page>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_pages(limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_trash(state: State<'_, AppState>) -> Result<Vec<Page>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_trash().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn empty_trash(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.empty_trash().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_pages(
    query: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_pages(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_delete_page(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.secure_delete_page(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_empty_trash(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.secure_empty_trash().map_err(|e| e.to_string())
}
