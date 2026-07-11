use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(input: CreateTagInput, state: State<'_, AppState>) -> Result<Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_tag(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(input: UpdateTagInput, state: State<'_, AppState>) -> Result<Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_tag(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page_tags(page_id: String, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_tags(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pages_for_tag(
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PageMetadata>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_pages_for_tag(&tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_page_tags(
    page_id: String,
    tag_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_page_tags(&page_id, &tag_ids)
        .map_err(|e| e.to_string())
}
