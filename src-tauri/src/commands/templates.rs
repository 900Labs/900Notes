use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn get_all_templates(state: State<'_, AppState>) -> Result<Vec<Template>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_templates().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_template(
    input: CreateTemplateInput,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_template(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_template(
    input: UpdateTemplateInput,
    state: State<'_, AppState>,
) -> Result<Template, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_template(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_template(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_template(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_page_from_template(
    template_id: String,
    title: String,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_page_from_template(&template_id, &title, parent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_or_create_daily_note(date: String, state: State<'_, AppState>) -> Result<Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_or_create_daily_note(&date)
        .map_err(|e| e.to_string())
}
