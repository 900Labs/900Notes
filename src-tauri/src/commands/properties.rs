use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn get_page_properties(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PageProperty>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_page_properties(&page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_page_property(
    input: SetPropertyInput,
    state: State<'_, AppState>,
) -> Result<PageProperty, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_page_property(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_page_property(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_page_property(&id).map_err(|e| e.to_string())
}
