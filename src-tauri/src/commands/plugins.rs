use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Manager, State};

use crate::models::{Plugin, PluginManifest};
use crate::AppState;

fn map_db_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn plugins_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    Ok(app_data_dir.join("plugins"))
}

fn is_safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn is_safe_plugin_id(value: &str) -> bool {
    is_safe_relative_path(value) && Path::new(value).components().count() == 1
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
pub fn scan_plugins_dir(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Plugin>, String> {
    let plugins_dir = plugins_root(&app)?;
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
    app: AppHandle,
    plugin_id: String,
    file_path: String,
) -> Result<String, String> {
    if !is_safe_plugin_id(&plugin_id) {
        return Err("Invalid plugin id".to_string());
    }
    if !is_safe_relative_path(&file_path) {
        return Err("Invalid plugin file path".to_string());
    }

    let plugins_dir = plugins_root(&app)?;
    let plugin_dir = plugins_dir.join(&plugin_id);
    let canonical_plugin_dir = plugin_dir.canonicalize().map_err(map_db_err)?;
    let full_path = plugin_dir.join(&file_path);
    let canonical_full_path = full_path.canonicalize().map_err(map_db_err)?;
    if !canonical_full_path.starts_with(&canonical_plugin_dir) {
        return Err("Plugin file path escapes plugin directory".to_string());
    }

    std::fs::read_to_string(&canonical_full_path).map_err(map_db_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_plugin_ids_as_single_safe_directory_names() {
        assert!(is_safe_plugin_id("wordcount"));
        assert!(!is_safe_plugin_id(""));
        assert!(!is_safe_plugin_id("../wordcount"));
        assert!(!is_safe_plugin_id("/tmp/wordcount"));
        assert!(!is_safe_plugin_id("plugins/wordcount"));
    }

    #[test]
    fn validates_plugin_file_paths_as_relative_descendants() {
        assert!(is_safe_relative_path("index.js"));
        assert!(is_safe_relative_path("dist/index.js"));
        assert!(!is_safe_relative_path(""));
        assert!(!is_safe_relative_path("../index.js"));
        assert!(!is_safe_relative_path("/tmp/index.js"));
        assert!(!is_safe_relative_path("./index.js"));
    }
}
