use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, State};

use crate::services::workspace::Workspace;
use crate::{AppState, WorkspaceState};

fn prepare_workspace_database(
    db_path: &std::path::Path,
    encrypted: bool,
) -> Result<(crate::db::Database, crate::services::crdt::CrdtService), String> {
    let database = if encrypted {
        crate::db::Database::open(std::path::Path::new(":memory:"))
            .map_err(|e| format!("Open locked workspace placeholder: {e}"))?
    } else {
        crate::db::Database::open(db_path).map_err(|e| format!("Open DB: {e}"))?
    };
    let crdt = crate::services::crdt::CrdtService::load_from_db(&database)
        .map_err(|e| format!("Load workspace sync state: {e}"))?;
    Ok((database, crdt))
}

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
    let app_state = app.state::<AppState>();
    let (previous_active, workspace) = {
        let service = state.service.lock().map_err(|e| e.to_string())?;
        let registry = service.load_registry()?;
        let workspace = registry
            .workspaces
            .iter()
            .find(|workspace| workspace.id == id)
            .cloned()
            .ok_or_else(|| "Workspace not found".to_string())?;
        (registry.active_id, workspace)
    };
    if previous_active == id {
        return Ok(id);
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Get app data dir: {e}"))?;
    let db_path = app_data_dir.join(&workspace.db_path);
    let encryption = crate::services::encryption::EncryptionService::new(&db_path);
    let target_encrypted = encryption.is_encrypted();
    let (database, crdt) = prepare_workspace_database(&db_path, target_encrypted)?;

    // Check every fallible runtime lock before changing durable state.
    drop(app_state.db.lock().map_err(|e| e.to_string())?);
    drop(app_state.crdt.lock().map_err(|e| e.to_string())?);
    drop(app_state.sync.lock().map_err(|e| e.to_string())?);
    drop(app_state.active_db_path.lock().map_err(|e| e.to_string())?);
    drop(app_state.passphrase.lock().map_err(|e| e.to_string())?);

    let current_path = app_state
        .active_db_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let current_passphrase = app_state
        .passphrase
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous_locked = app_state.workspace_locked.load(Ordering::Acquire);

    let mut previous_sync = app_state.sync.lock().map_err(|e| e.to_string())?.take();
    if let Some(service) = previous_sync.as_mut() {
        service.stop();
    }
    app_state.workspace_locked.store(true, Ordering::Release);
    if let Some(passphrase) = current_passphrase.as_deref() {
        if let Err(error) = crate::lock_encrypted_database(&app_state.db, &current_path, passphrase)
        {
            if let Some(mut service) = previous_sync {
                let _ = service.start(app_state.db.clone());
                *app_state.sync.lock().map_err(|e| e.to_string())? = Some(service);
            }
            app_state
                .workspace_locked
                .store(previous_locked, Ordering::Release);
            return Err(format!(
                "Could not lock the current workspace before switching: {error}"
            ));
        }
    }

    if let Err(error) = state
        .service
        .lock()
        .map_err(|e| e.to_string())?
        .switch_workspace(&id)
    {
        let restore_result = (|| -> Result<(), String> {
            if let Some(passphrase) = current_passphrase.as_deref() {
                let current_encryption =
                    crate::services::encryption::EncryptionService::new(&current_path);
                current_encryption.decrypt_to_path(passphrase, &current_path)?;
                let current_database = crate::db::Database::open(&current_path)
                    .map_err(|open_error| format!("Restore current workspace: {open_error}"))?;
                *app_state.db.lock().map_err(|e| e.to_string())? = current_database;
            }
            Ok(())
        })();
        if let Some(mut service) = previous_sync {
            let _ = service.start(app_state.db.clone());
            *app_state.sync.lock().map_err(|e| e.to_string())? = Some(service);
        }
        app_state
            .workspace_locked
            .store(previous_locked, Ordering::Release);
        if let Err(restore_error) = restore_result {
            return Err(format!(
                "Could not update active workspace registry: {error}. Runtime rollback failed: {restore_error}"
            ));
        }
        return Err(format!(
            "Could not update active workspace registry: {error}"
        ));
    }
    *app_state.db.lock().map_err(|e| e.to_string())? = database;
    *app_state.crdt.lock().map_err(|e| e.to_string())? = crdt;
    *app_state.active_db_path.lock().map_err(|e| e.to_string())? = db_path;
    *app_state.passphrase.lock().map_err(|e| e.to_string())? = None;
    app_state
        .workspace_locked
        .store(target_encrypted, Ordering::Release);

    Ok(workspace.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_target_is_rejected_before_runtime_transition() {
        let root = std::env::temp_dir().join(format!("900notes-corrupt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("corrupt.db");
        std::fs::write(&path, b"not a sqlite database").unwrap();
        assert!(prepare_workspace_database(&path, false).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
