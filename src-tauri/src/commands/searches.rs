use tauri::State;

use crate::models::*;
use crate::AppState;

// ── Saved Searches ──

#[tauri::command]
pub fn get_all_saved_searches(state: State<'_, AppState>) -> Result<Vec<SavedSearch>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_saved_searches().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_saved_search(
    input: CreateSavedSearchInput,
    state: State<'_, AppState>,
) -> Result<SavedSearch, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_saved_search(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_saved_search(
    input: UpdateSavedSearchInput,
    state: State<'_, AppState>,
) -> Result<SavedSearch, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_saved_search(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_saved_search(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_saved_search(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn execute_saved_search(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute_saved_search(&id).map_err(|e| e.to_string())
}

// ── Smart Folders ──

#[tauri::command]
pub fn get_all_smart_folders(state: State<'_, AppState>) -> Result<Vec<SmartFolder>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_smart_folders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_smart_folder(
    input: CreateSmartFolderInput,
    state: State<'_, AppState>,
) -> Result<SmartFolder, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_smart_folder(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_smart_folder(
    input: UpdateSmartFolderInput,
    state: State<'_, AppState>,
) -> Result<SmartFolder, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_smart_folder(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_smart_folder(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_smart_folder(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_smart_folder_pages(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PageTreeNode>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_smart_folder_pages(&id).map_err(|e| e.to_string())
}
