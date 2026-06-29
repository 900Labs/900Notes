use crate::services::importers;
use crate::AppState;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultResponse {
    pub pages_created: usize,
    pub errors: Vec<String>,
}

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub fn import_evernote(
    enex_content: String,
    state: State<'_, AppState>,
) -> Result<ImportResultResponse, String> {
    let db = state.db.lock().map_err(map_err)?;
    let result = importers::import_evernote_enex(&db, &enex_content);
    Ok(ImportResultResponse {
        pages_created: result.pages_created,
        errors: result.errors,
    })
}

#[tauri::command]
pub fn import_notion(
    dir_path: String,
    state: State<'_, AppState>,
) -> Result<ImportResultResponse, String> {
    let db = state.db.lock().map_err(map_err)?;
    let result = importers::import_notion_export(&db, &dir_path);
    Ok(ImportResultResponse {
        pages_created: result.pages_created,
        errors: result.errors,
    })
}

#[tauri::command]
pub fn import_obsidian(
    dir_path: String,
    state: State<'_, AppState>,
) -> Result<ImportResultResponse, String> {
    let db = state.db.lock().map_err(map_err)?;
    let result = importers::import_obsidian_vault(&db, &dir_path);
    Ok(ImportResultResponse {
        pages_created: result.pages_created,
        errors: result.errors,
    })
}

#[tauri::command]
pub fn import_roam(
    json_content: String,
    state: State<'_, AppState>,
) -> Result<ImportResultResponse, String> {
    let db = state.db.lock().map_err(map_err)?;
    let result = importers::import_roam_json(&db, &json_content);
    Ok(ImportResultResponse {
        pages_created: result.pages_created,
        errors: result.errors,
    })
}
