use tauri::State;

use crate::models::{Plugin, PluginManifest};
use crate::AppState;

fn map_db_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub fn get_all_plugins(state: State<'_, AppState>) -> Result<Vec<Plugin>, String> {
    let db = state.db.lock().map_err(map_db_err)?;
    db.get_all_plugins().map_err(map_db_err)
}

#[tauri::command]
pub fn get_enabled_plugins(state: State<'_, AppState>) -> Result<Vec<Plugin>, String> {
    let db = state.db.lock().map_err(map_db_err)?;
    db.get_enabled_plugins().map_err(map_db_err)
}

#[tauri::command]
pub fn install_plugin(
    manifest: PluginManifest,
    state: State<'_, AppState>,
) -> Result<Plugin, String> {
    let db = state.db.lock().map_err(map_db_err)?;
    db.install_plugin(&manifest).map_err(map_db_err)
}

#[tauri::command]
pub fn set_plugin_enabled(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_db_err)?;
    db.set_plugin_enabled(&id, enabled).map_err(map_db_err)
}

#[tauri::command]
pub fn uninstall_plugin(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(map_db_err)?;
    db.uninstall_plugin(&id).map_err(map_db_err)
}

#[tauri::command]
pub fn scan_plugins_dir(
    app_data_dir: String,
    state: State<'_, AppState>,
) -> Result<Vec<Plugin>, String> {
    let plugins_dir = std::path::Path::new(&app_data_dir).join("plugins");
    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir).map_err(map_db_err)?;
        return Ok(Vec::new());
    }

    let mut installed = Vec::new();
    let db = state.db.lock().map_err(map_db_err)?;

    for entry in std::fs::read_dir(&plugins_dir).map_err(map_db_err)? {
        let entry = entry.map_err(map_db_err)?;
        let plugin_dir = entry.path();
        if !plugin_dir.is_dir() {
            continue;
        }

        let manifest_path = plugin_dir.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        let manifest_content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let manifest: PluginManifest = match serde_json::from_str(&manifest_content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match db.install_plugin(&manifest) {
            Ok(plugin) => installed.push(plugin),
            Err(_) => continue,
        }
    }

    Ok(installed)
}

#[tauri::command]
pub fn read_plugin_file(
    app_data_dir: String,
    plugin_id: String,
    file_path: String,
) -> Result<String, String> {
    let full_path = std::path::Path::new(&app_data_dir)
        .join("plugins")
        .join(&plugin_id)
        .join(&file_path);

    std::fs::read_to_string(&full_path).map_err(map_db_err)
}
