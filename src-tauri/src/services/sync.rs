use mdns_sd::ServiceDaemon;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::db::Database;
use crate::models::*;

const SERVICE_TYPE: &str = "_900notes._tcp.local.";

pub struct SyncService {
    daemon: Option<ServiceDaemon>,
    server_thread: Option<thread::JoinHandle<()>>,
    peers: Arc<Mutex<Vec<SyncDeviceInfo>>>,
    device_id: String,
    device_name: String,
    port: u16,
    running: Arc<Mutex<bool>>,
}

impl SyncService {
    pub fn new(device_id: &str, device_name: &str, port: u16) -> Self {
        SyncService {
            daemon: None,
            server_thread: None,
            peers: Arc::new(Mutex::new(Vec::new())),
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            port,
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
        browse_thread.thread().unpark();

        // Start TCP server
        *self.running.lock().map_err(|e| e.to_string())? = true;
        let running = self.running.clone();
        let port = self.port;
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
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
                        thread::spawn(move || {
                            handle_sync_connection(stream, db, did, dname);
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
        if let Some(thread) = self.server_thread.take() {
            thread.join().ok();
        }
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    pub fn get_peers(&self) -> Vec<SyncDeviceInfo> {
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).clone()
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

        let json = serde_json::to_string(&handshake).map_err(|e| e.to_string())?;
        let json_bytes = json.as_bytes();
        let len = json_bytes.len() as u32;
        stream
            .write_all(&len.to_be_bytes())
            .map_err(|e| e.to_string())?;
        stream.write_all(json_bytes).map_err(|e| e.to_string())?;

        // Read response
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        let mut resp_buf = vec![0u8; resp_len];
        stream
            .read_exact(&mut resp_buf)
            .map_err(|e| e.to_string())?;

        let remote_handshake: SyncHandshake =
            serde_json::from_slice(&resp_buf).map_err(|e| e.to_string())?;

        // Merge remote pages into local DB
        let db_guard = db.lock().map_err(|e| e.to_string())?;
        let local_pages: HashMap<String, String> = db_guard
            .get_all_pages_for_sync()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|p| (p.id.clone(), p.updated_at.clone()))
            .collect();

        let conflicts = Vec::new();
        for remote_meta in &remote_handshake.page_metas {
            if let Some(local_updated) = local_pages.get(&remote_meta.id) {
                if local_updated == &remote_meta.updated_at {
                    continue;
                }
            }
            db_guard
                .upsert_page_from_sync(remote_meta)
                .map_err(|e| e.to_string())?;
        }

        drop(db_guard);
        Ok(conflicts)
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn handle_sync_connection(
    mut stream: TcpStream,
    db: Arc<Mutex<Database>>,
    device_id: String,
    device_name: String,
) {
    // Read handshake
    let mut len_buf = [0u8; 4];
    if stream.read_exact(&mut len_buf).is_err() {
        return;
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 100 * 1024 * 1024 {
        return;
    }
    let mut buf = vec![0u8; len];
    if stream.read_exact(&mut buf).is_err() {
        return;
    }

    let remote_handshake: SyncHandshake = match serde_json::from_slice(&buf) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Merge remote pages
    if let Ok(db_guard) = db.lock() {
        for remote_meta in &remote_handshake.page_metas {
            let _ = db_guard.upsert_page_from_sync(remote_meta);
        }

        // Send our pages back
        let pages = match db_guard.get_all_pages_for_sync() {
            Ok(p) => p,
            Err(_) => return,
        };
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
        let handshake = SyncHandshake {
            device_id,
            device_name,
            page_metas,
        };
        drop(db_guard);

        let json = match serde_json::to_string(&handshake) {
            Ok(j) => j,
            Err(_) => return,
        };
        let json_bytes = json.as_bytes();
        let resp_len = json_bytes.len() as u32;
        if stream.write_all(&resp_len.to_be_bytes()).is_err() {
            return;
        }
        if stream.write_all(json_bytes).is_err() {}
    }
}
