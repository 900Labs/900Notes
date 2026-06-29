use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn start_sync(
    device_name: Option<String>,
    port: Option<u16>,
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let device_id = uuid::Uuid::new_v4().to_string();
    let name = device_name.unwrap_or_else(|| {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "900Notes Device".to_string())
    });
    let p = port.unwrap_or(9876);

    let mut sync_guard = state.sync.lock().map_err(|e| e.to_string())?;
    if sync_guard.is_some() {
        return Err("Sync already running".to_string());
    }

    let mut service = crate::services::sync::SyncService::new(&device_id, &name, p);
    service.start(state.db.clone()).map_err(|e| e.to_string())?;

    let status = SyncStatus {
        enabled: true,
        device_id: device_id.clone(),
        device_name: name,
        port: p,
        peers: service.get_peers(),
        last_sync: None,
    };

    *sync_guard = Some(service);
    Ok(status)
}

#[tauri::command]
pub fn stop_sync(state: State<'_, AppState>) -> Result<(), String> {
    let mut sync_guard = state.sync.lock().map_err(|e| e.to_string())?;
    if let Some(mut service) = sync_guard.take() {
        service.stop();
    }
    Ok(())
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let sync_guard = state.sync.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let last_sync = db.get_last_sync_time().map_err(|e| e.to_string())?;

    if let Some(service) = sync_guard.as_ref() {
        Ok(SyncStatus {
            enabled: service.is_running(),
            device_id: String::new(),
            device_name: String::new(),
            port: 0,
            peers: service.get_peers(),
            last_sync,
        })
    } else {
        Ok(SyncStatus {
            enabled: false,
            device_id: String::new(),
            device_name: String::new(),
            port: 0,
            peers: Vec::new(),
            last_sync,
        })
    }
}

#[tauri::command]
pub fn sync_with_peer(
    peer_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SyncConflict>, String> {
    let sync_guard = state.sync.lock().map_err(|e| e.to_string())?;
    let service = sync_guard.as_ref().ok_or("Sync not running")?;

    let peers = service.get_peers();
    let peer = peers
        .into_iter()
        .find(|p| p.id == peer_id)
        .ok_or("Peer not found")?;

    drop(sync_guard);

    let sync_guard = state.sync.lock().map_err(|e| e.to_string())?;
    let service = sync_guard.as_ref().ok_or("Sync not running")?;
    service
        .sync_with_peer(&peer, state.db.clone())
        .map_err(|e| e.to_string())
}

// ── CRDT commands ──

#[tauri::command]
pub fn sync_page_to_crdt(page_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let page = db.get_page_by_id(&page_id).map_err(|e| e.to_string())?;
    drop(db);

    let mut crdt = state.crdt.lock().map_err(|e| e.to_string())?;
    crdt.upsert_page_in_crdt(&page);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    crdt.save_to_db(&db).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_pending_sync_count(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_pending_sync_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_crdt_to_db(state: State<'_, AppState>) -> Result<i64, String> {
    let crdt = state.crdt.lock().map_err(|e| e.to_string())?;
    let pages = crdt.read_pages_from_crdt();
    drop(crdt);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0i64;
    for page in &pages {
        let meta = page.to_page_meta();
        db.upsert_page_from_sync(&meta).map_err(|e| e.to_string())?;
        count += 1;
    }

    let now = chrono::Utc::now().to_rfc3339();
    db.set_last_sync_time(&now).map_err(|e| e.to_string())?;
    Ok(count)
}
