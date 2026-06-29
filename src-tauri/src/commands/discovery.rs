use tauri::State;

use crate::models::*;
use crate::AppState;

// ── Tag Groups ──

#[tauri::command]
pub fn get_all_tag_groups(state: State<'_, AppState>) -> Result<Vec<TagGroup>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_tag_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag_group(
    input: CreateTagGroupInput,
    state: State<'_, AppState>,
) -> Result<TagGroup, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_tag_group(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag_group(
    input: UpdateTagGroupInput,
    state: State<'_, AppState>,
) -> Result<TagGroup, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_tag_group(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag_group(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_tag_group(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag_to_group(
    group_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_tag_to_group(&group_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag_from_group(
    group_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_tag_from_group(&group_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tags_in_group(group_id: String, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tags_in_group(&group_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ungrouped_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_ungrouped_tags().map_err(|e| e.to_string())
}

// ── Related Pages ──

#[tauri::command]
pub fn get_related_pages(
    page_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<RelatedPage>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_related_pages(&page_id, limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}
