use tauri::{AppHandle, Manager, State};

use crate::services::workspace::Workspace;
use crate::{AppState, WorkspaceState};

#[tauri::command]
pub fn list_workspaces(state: State<'_, WorkspaceState>) -> Result<Vec<Workspace>, String> {
    let registry = state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .load_registry()?;
    Ok(registry.workspaces)
}

#[tauri::command]
pub fn get_active_workspace(state: State<'_, WorkspaceState>) -> Result<Workspace, String> {
    let registry = state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .load_registry()?;
    registry
        .workspaces
        .into_iter()
        .find(|w| w.id == registry.active_id)
        .ok_or("No active workspace".to_string())
}

#[tauri::command]
pub fn create_workspace(
    name: String,
    state: State<'_, WorkspaceState>,
) -> Result<Workspace, String> {
    state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .create_workspace(&name)
}

#[tauri::command]
pub fn delete_workspace(id: String, state: State<'_, WorkspaceState>) -> Result<(), String> {
    state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .delete_workspace(&id)
}

#[tauri::command]
pub fn rename_workspace(
    id: String,
    name: String,
    state: State<'_, WorkspaceState>,
) -> Result<Workspace, String> {
    state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .rename_workspace(&id, &name)
}

#[tauri::command]
pub fn switch_workspace(
    id: String,
    app: AppHandle,
    state: State<'_, WorkspaceState>,
) -> Result<String, String> {
    let workspace = state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .switch_workspace(&id)?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join(&workspace.db_path);
    let database = crate::db::Database::open(&db_path).map_err(|e| format!("Open DB: {e}"))?;

    let app_state = app.state::<AppState>();
    *app_state.db.lock().map_err(|e| e.to_string())? = database;

    Ok(workspace.id)
}
