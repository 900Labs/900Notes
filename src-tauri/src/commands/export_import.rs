use tauri::State;

use crate::services::export_import::WorkspaceExport;
use crate::services::markdown;
use crate::AppState;

#[tauri::command]
pub fn export_workspace(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let export =
        crate::services::export_import::export_workspace(&db).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_workspace(data: String, state: State<'_, AppState>) -> Result<usize, String> {
    let workspace: WorkspaceExport = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    restore_workspace_state(&workspace, &state)
}

pub(crate) fn restore_workspace_state(
    workspace: &WorkspaceExport,
    state: &AppState,
) -> Result<usize, String> {
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::services::export_import::validate_workspace(&db, workspace)
            .map_err(|e| e.to_string())?;
    }

    let mut previous_sync = state.sync.lock().map_err(|e| e.to_string())?.take();
    if let Some(service) = previous_sync.as_mut() {
        service.stop();
    }

    let result = (|| {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let count = crate::services::export_import::import_workspace(&db, workspace)
            .map_err(|e| e.to_string())?;
        let crdt = crate::services::crdt::CrdtService::load_from_db(&db);
        drop(db);
        match crdt {
            Ok(crdt) => *state.crdt.lock().map_err(|e| e.to_string())? = crdt,
            Err(error) => {
                *state.crdt.lock().map_err(|e| e.to_string())? =
                    crate::services::crdt::CrdtService::new();
                return Err(format!(
                    "Workspace restored, but the sync index could not be rebuilt: {error}"
                ));
            }
        }
        Ok(count)
    })();

    match result {
        Ok(count) => Ok(count),
        Err(error) => {
            if let Some(mut service) = previous_sync {
                if let Err(restart_error) = service.start(state.db.clone()) {
                    return Err(format!(
                        "{error}. The previous sync service could not be restarted: {restart_error}"
                    ));
                }
                *state.sync.lock().map_err(|e| e.to_string())? = Some(service);
            }
            Err(error)
        }
    }
}

#[tauri::command]
pub fn export_page_markdown(page_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let page = db.get_page_by_id(&page_id).map_err(|e| e.to_string())?;
    let md = markdown::prosemirror_to_markdown(&page.content);
    Ok(format!("# {}\n\n{}", page.title, md))
}

#[tauri::command]
pub fn import_markdown(
    title: String,
    content: String,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::models::Page, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let pm_content = markdown::markdown_to_prosemirror(&content, &title);
    let input = crate::models::CreatePageInput {
        parent_id,
        title,
        content: Some(pm_content),
        icon: None,
    };
    db.create_page(&input).map_err(|e| e.to_string())
}
