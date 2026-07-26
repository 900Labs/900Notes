use mdns_sd::ServiceDaemon;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::db::Database;
use crate::models::*;
use crate::services::encryption::{decrypt_data, encrypt_data};

const SERVICE_TYPE: &str = "_900notes._tcp.local.";
const MAX_SYNC_MESSAGE_BYTES: usize = 100 * 1024 * 1024;

pub struct SyncService {
    daemon: Option<ServiceDaemon>,
    browse_thread: Option<thread::JoinHandle<()>>,
    server_thread: Option<thread::JoinHandle<()>>,
    peers: Arc<Mutex<Vec<SyncDeviceInfo>>>,
    device_id: String,
    device_name: String,
    port: u16,
    pairing_secret: String,
    running: Arc<Mutex<bool>>,
}

impl SyncService {
    pub fn new(device_id: &str, device_name: &str, port: u16, pairing_secret: &str) -> Self {
        SyncService {
            daemon: None,
            browse_thread: None,
            server_thread: None,
            peers: Arc::new(Mutex::new(Vec::new())),
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            port,
            pairing_secret: pairing_secret.to_string(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self, db: Arc<Mutex<Database>>) -> Result<(), String> {
        // Start mDNS discovery
        let daemon = ServiceDaemon::new().map_err(|e| format!("mDNS init failed: {e}"))?;

        // Register our service
        let host_name = format!(
            "900notes-{}.local.",
            &self.device_id[..8.min(self.device_id.len())]
        );
        let mut props = std::collections::HashMap::new();
        props.insert("deviceName".to_string(), self.device_name.clone());
        let service_info = mdns_sd::ServiceInfo::new(
            SERVICE_TYPE,
            &self.device_id,
            &host_name,
            "",
            self.port,
            Some(props),
        )
        .map_err(|e| format!("mDNS service info failed: {e}"))?
        .enable_addr_auto();
        daemon
            .register(service_info)
            .map_err(|e| format!("mDNS register failed: {e}"))?;

        // Start browsing for peers
        let receiver = daemon
            .browse(SERVICE_TYPE)
            .map_err(|e| format!("mDNS browse failed: {e}"))?;
        let peers = self.peers.clone();
        let self_device_id = self.device_id.clone();
        let browse_thread = thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        let id = info.get_fullname().to_string();
                        if id.contains(&self_device_id) {
                            continue;
                        }
                        let name = info
                            .get_property_val_str("deviceName")
                            .unwrap_or("Unknown")
                            .to_string();
                        let host = info
                            .get_addresses()
                            .iter()
                            .next()
                            .map(|a| a.to_string())
                            .unwrap_or_default();
                        let port = info.get_port();
                        let device = SyncDeviceInfo {
                            id,
                            name,
                            host,
                            port,
                        };
                        if let Ok(mut p) = peers.lock() {
                            // Replace if exists, else add
                            if let Some(existing) = p.iter().position(|d| d.id == device.id) {
                                p[existing] = device;
                            } else {
                                p.push(device);
                            }
                        }
                    }
                    mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                        if let Ok(mut p) = peers.lock() {
                            p.retain(|d| d.id != fullname);
                        }
                    }
                    _ => {}
                }
            }
        });
        self.browse_thread = Some(browse_thread);

        // Start TCP server
        *self.running.lock().map_err(|e| e.to_string())? = true;
        let running = self.running.clone();
        let port = self.port;
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
        let pairing_secret = self.pairing_secret.clone();
        self.server_thread = Some(thread::spawn(move || {
            let listener = match TcpListener::bind(("0.0.0.0", port)) {
                Ok(l) => l,
                Err(_) => return,
            };
            listener.set_nonblocking(true).ok();
            while *running.lock().unwrap_or_else(|e| e.into_inner()) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let db = db.clone();
                        let did = device_id.clone();
                        let dname = device_name.clone();
                        let secret = pairing_secret.clone();
                        let worker_running = running.clone();
                        thread::spawn(move || {
                            handle_sync_connection(stream, db, worker_running, did, dname, secret);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }));

        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn stop(&mut self) {
        *self.running.lock().unwrap_or_else(|e| e.into_inner()) = false;
        if let Some(daemon) = self.daemon.take() {
            daemon.shutdown().ok();
        }
        if let Some(thread) = self.browse_thread.take() {
            thread.join().ok();
        }
        if let Some(thread) = self.server_thread.take() {
            thread.join().ok();
        }
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    pub fn get_peers(&self) -> Vec<SyncDeviceInfo> {
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn sync_with_peer(
        &self,
        peer: &SyncDeviceInfo,
        db: Arc<Mutex<Database>>,
    ) -> Result<Vec<SyncConflict>, String> {
        let db_guard = db.lock().map_err(|e| e.to_string())?;
        let pages = db_guard
            .get_all_pages_for_sync()
            .map_err(|e| e.to_string())?;
        let page_metas: Vec<PageSyncMeta> = pages
            .iter()
            .map(|p| PageSyncMeta {
                id: p.id.clone(),
                title: p.title.clone(),
                content: p.content.clone(),
                parent_id: p.parent_id.clone(),
                icon: p.icon.clone(),
                cover_color: p.cover_color.clone(),
                created_at: p.created_at.clone(),
                updated_at: p.updated_at.clone(),
                deleted_at: p.deleted_at.clone(),
                pinned: p.pinned,
                sort_order: p.sort_order,
            })
            .collect();
        drop(db_guard);

        let handshake = SyncHandshake {
            device_id: self.device_id.clone(),
            device_name: self.device_name.clone(),
            page_metas,
        };

        let addr = format!("{}:{}", peer.host, peer.port);
        let mut stream = TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e: std::net::AddrParseError| e.to_string())?,
            std::time::Duration::from_secs(5),
        )
        .map_err(|e| format!("connect failed: {e}"))?;

        write_encrypted_handshake(&mut stream, &handshake, &self.pairing_secret)?;
        let remote_handshake = read_encrypted_handshake(&mut stream, &self.pairing_secret)?;

        // Merge remote pages into local DB. A page that exists on both sides
        // with different timestamps is a real conflict: we do NOT silently
        // overwrite the local copy. Instead we record a SyncConflict so the UI
        // can surface it, and keep the local version. Only genuinely new pages
        // (absent locally) or identical ones are applied.
        let db_guard = db.lock().map_err(|e| e.to_string())?;
        let local_pages: HashMap<String, Page> = db_guard
            .get_all_pages_for_sync()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

        let mut conflicts = Vec::new();
        for remote_meta in &remote_handshake.page_metas {
            match local_pages.get(&remote_meta.id) {
                Some(local) if local.updated_at == remote_meta.updated_at => {
                    continue;
                }
                Some(local) => {
                    // Both sides have the page but disagree on content/time.
                    // Do not clobber local; report the conflict instead.
                    conflicts.push(SyncConflict {
                        page_id: remote_meta.id.clone(),
                        local_updated: local.updated_at.clone(),
                        remote_updated: remote_meta.updated_at.clone(),
                        local_title: local.title.clone(),
                        remote_title: remote_meta.title.clone(),
                    });
                }
                None => {
                    db_guard
                        .upsert_page_from_sync(remote_meta)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        drop(db_guard);
        Ok(conflicts)
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for SyncService {
    fn drop(&mut self) {
        self.stop();
    }
}

fn handle_sync_connection(
    mut stream: TcpStream,
    db: Arc<Mutex<Database>>,
    running: Arc<Mutex<bool>>,
    device_id: String,
    device_name: String,
    pairing_secret: String,
) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));

    let remote_handshake = match read_encrypted_handshake(&mut stream, &pairing_secret) {
        Ok(h) => h,
        Err(_) => return,
    };

    if let Some(handshake) =
        merge_inbound_handshake(&db, &running, &remote_handshake, device_id, device_name)
    {
        let _ = write_encrypted_handshake(&mut stream, &handshake, &pairing_secret);
    }
}

fn merge_inbound_handshake(
    db: &Arc<Mutex<Database>>,
    running: &Arc<Mutex<bool>>,
    remote_handshake: &SyncHandshake,
    device_id: String,
    device_name: String,
) -> Option<SyncHandshake> {
    // The database lock is the workspace swap/restore barrier. Checking the
    // running flag only after acquiring it guarantees that a stopped worker
    // cannot write to a replacement database. A worker that already passed
    // this check keeps the old database locked until its merge is complete.
    let db_guard = db.lock().ok()?;
    if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
        return None;
    }

    for remote_meta in &remote_handshake.page_metas {
        let _ = db_guard.upsert_page_from_sync(remote_meta);
    }

    let pages = db_guard.get_all_pages_for_sync().ok()?;
    let page_metas = pages
        .iter()
        .map(|p| PageSyncMeta {
            id: p.id.clone(),
            title: p.title.clone(),
            content: p.content.clone(),
            parent_id: p.parent_id.clone(),
            icon: p.icon.clone(),
            cover_color: p.cover_color.clone(),
            created_at: p.created_at.clone(),
            updated_at: p.updated_at.clone(),
            deleted_at: p.deleted_at.clone(),
            pinned: p.pinned,
            sort_order: p.sort_order,
        })
        .collect();

    Some(SyncHandshake {
        device_id,
        device_name,
        page_metas,
    })
}

fn write_encrypted_handshake<W: Write>(
    stream: &mut W,
    handshake: &SyncHandshake,
    pairing_secret: &str,
) -> Result<(), String> {
    let json = serde_json::to_vec(handshake).map_err(|e| e.to_string())?;
    if json.len() > MAX_SYNC_MESSAGE_BYTES {
        return Err("sync message too large".to_string());
    }
    let encrypted = encrypt_data(&json, pairing_secret)?;
    if encrypted.len() > MAX_SYNC_MESSAGE_BYTES {
        return Err("encrypted sync message too large".to_string());
    }
    let len: u32 = encrypted
        .len()
        .try_into()
        .map_err(|_| "sync message too large".to_string())?;
    stream
        .write_all(&len.to_be_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(&encrypted).map_err(|e| e.to_string())
}

fn read_encrypted_handshake<R: Read>(
    stream: &mut R,
    pairing_secret: &str,
) -> Result<SyncHandshake, String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_SYNC_MESSAGE_BYTES {
        return Err("sync message too large".to_string());
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let plaintext = decrypt_data(&buf, pairing_secret)?;
    if plaintext.len() > MAX_SYNC_MESSAGE_BYTES {
        return Err("sync message too large".to_string());
    }
    serde_json::from_slice(&plaintext).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::mpsc;

    fn test_handshake() -> SyncHandshake {
        SyncHandshake {
            device_id: "device-a".to_string(),
            device_name: "Device A".to_string(),
            page_metas: Vec::new(),
        }
    }

    #[test]
    fn encrypted_handshake_round_trips() {
        let mut frame = Vec::new();
        write_encrypted_handshake(&mut frame, &test_handshake(), "shared secret").unwrap();

        let mut cursor = Cursor::new(frame);
        let handshake = read_encrypted_handshake(&mut cursor, "shared secret").unwrap();
        assert_eq!(handshake.device_id, "device-a");
    }

    #[test]
    fn encrypted_handshake_rejects_wrong_secret() {
        let mut frame = Vec::new();
        write_encrypted_handshake(&mut frame, &test_handshake(), "shared secret").unwrap();

        let mut cursor = Cursor::new(frame);
        let result = read_encrypted_handshake(&mut cursor, "wrong secret");
        assert!(result.is_err());
    }

    #[test]
    fn dropping_service_signals_shutdown_without_double_join() {
        let service = SyncService::new("device", "Device", 0, "shared secret");
        let running = service.running.clone();
        *running.lock().unwrap() = true;
        drop(service);
        assert!(!*running.lock().unwrap());
    }

    #[test]
    fn stopped_inbound_worker_cannot_write_after_database_lock_delay() {
        let root = std::env::temp_dir().join(format!(
            "900notes-stopped-inbound-sync-{}",
            uuid::Uuid::new_v4()
        ));
        let db = Arc::new(Mutex::new(Database::open(&root).unwrap()));
        let running = Arc::new(Mutex::new(true));
        let remote_handshake = SyncHandshake {
            device_id: "remote-device".to_string(),
            device_name: "Remote Device".to_string(),
            page_metas: vec![PageSyncMeta {
                id: "must-not-be-written".to_string(),
                title: "Blocked page".to_string(),
                content: r#"{"type":"doc","content":[]}"#.to_string(),
                parent_id: None,
                icon: None,
                cover_color: None,
                created_at: "2026-07-11T00:00:00Z".to_string(),
                updated_at: "2026-07-11T00:00:00Z".to_string(),
                deleted_at: None,
                pinned: false,
                sort_order: 0,
            }],
        };

        let db_guard = db.lock().unwrap();
        let worker_db = db.clone();
        let worker_running = running.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            started_tx.send(()).unwrap();
            merge_inbound_handshake(
                &worker_db,
                &worker_running,
                &remote_handshake,
                "local-device".to_string(),
                "Local Device".to_string(),
            )
        });

        started_rx.recv().unwrap();
        *running.lock().unwrap() = false;
        drop(db_guard);

        assert!(worker.join().unwrap().is_none());
        assert!(db
            .lock()
            .unwrap()
            .get_all_pages_for_sync()
            .unwrap()
            .is_empty());
        drop(db);
        std::fs::remove_file(root).unwrap();
    }
}
