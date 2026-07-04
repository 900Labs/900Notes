mod commands;
mod db;
mod models;
mod services;

use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Mutex<db::Database>>,
    pub sync: Mutex<Option<services::sync::SyncService>>,
    pub crdt: Mutex<services::crdt::CrdtService>,
    pub passphrase: Mutex<Option<String>>,
}

pub struct WorkspaceState {
    pub service: Mutex<services::workspace::WorkspaceService>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("900notes.db");

            // Check if encryption is enabled — if so, don't auto-open the plaintext DB.
            // The frontend will prompt for passphrase and call unlock_database.
            let enc_service = services::encryption::EncryptionService::new(&db_path);
            let encryption_enabled = enc_service.is_encrypted();

            let database = if encryption_enabled {
                // Open an in-memory empty DB so commands don't panic before unlock.
                // The real DB will be swapped in by unlock_database.
                db::Database::open(std::path::Path::new(":memory:"))
                    .expect("failed to open placeholder database")
            } else {
                db::Database::open(&db_path).expect("failed to open database")
            };

            let crdt = services::crdt::CrdtService::load_from_db(&database)
                .expect("failed to load CRDT doc");
            let workspace_service = services::workspace::WorkspaceService::new(&app_data_dir);
            let db = Arc::new(Mutex::new(database));
            if !encryption_enabled {
                if let Err(error) = services::web_capture::start_clipper_server(
                    db.clone(),
                    services::web_capture::DEFAULT_CLIPPER_PORT,
                ) {
                    eprintln!("Failed to start web clipper server: {error}");
                }
            }
            app.manage(AppState {
                db,
                sync: Mutex::new(None),
                crdt: Mutex::new(crdt),
                passphrase: Mutex::new(None),
            });
            app.manage(WorkspaceState {
                service: Mutex::new(workspace_service),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::pages::create_page,
            commands::pages::get_page,
            commands::pages::get_all_pages,
            commands::pages::get_page_tree,
            commands::pages::get_page_tree_metadata,
            commands::pages::get_page_titles,
            commands::pages::get_recent_pages_metadata,
            commands::pages::update_page,
            commands::pages::delete_page,
            commands::pages::restore_page,
            commands::pages::duplicate_page,
            commands::pages::move_page,
            commands::pages::get_recent_pages,
            commands::pages::get_trash,
            commands::pages::empty_trash,
            commands::pages::search_pages,
            commands::pages::secure_delete_page,
            commands::pages::secure_empty_trash,
            commands::tags::get_all_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::get_page_tags,
            commands::tags::set_page_tags,
            commands::links::get_backlinks,
            commands::links::get_outgoing_links,
            commands::links::rebuild_links,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::export_import::export_workspace,
            commands::export_import::import_workspace,
            commands::export_import::export_page_markdown,
            commands::export_import::import_markdown,
            commands::properties::get_page_properties,
            commands::properties::set_page_property,
            commands::properties::delete_page_property,
            commands::templates::get_all_templates,
            commands::templates::create_template,
            commands::templates::update_template,
            commands::templates::delete_template,
            commands::templates::create_page_from_template,
            commands::templates::get_or_create_daily_note,
            commands::graph::get_graph_data,
            commands::searches::get_all_saved_searches,
            commands::searches::create_saved_search,
            commands::searches::update_saved_search,
            commands::searches::delete_saved_search,
            commands::searches::execute_saved_search,
            commands::searches::get_all_smart_folders,
            commands::searches::create_smart_folder,
            commands::searches::update_smart_folder,
            commands::searches::delete_smart_folder,
            commands::searches::get_smart_folder_pages,
            commands::history::get_page_revisions,
            commands::history::get_revision,
            commands::history::restore_revision,
            commands::history::delete_revision,
            commands::history::get_favorites,
            commands::history::add_favorite,
            commands::history::remove_favorite,
            commands::history::is_favorite,
            commands::history::reorder_favorites,
            commands::discovery::get_all_tag_groups,
            commands::discovery::create_tag_group,
            commands::discovery::update_tag_group,
            commands::discovery::delete_tag_group,
            commands::discovery::add_tag_to_group,
            commands::discovery::remove_tag_from_group,
            commands::discovery::get_tags_in_group,
            commands::discovery::get_ungrouped_tags,
            commands::discovery::get_related_pages,
            commands::attachments::create_attachment,
            commands::attachments::get_attachment,
            commands::attachments::get_attachments_for_page,
            commands::attachments::delete_attachment,
            commands::audio::create_audio_note,
            commands::audio::get_audio_note,
            commands::audio::get_audio_notes_for_page,
            commands::audio::update_audio_note,
            commands::audio::delete_audio_note,
            commands::pdf_ocr::export_page_pdf,
            commands::pdf_ocr::export_workspace_pdf,
            commands::pdf_ocr::ocr_attachment,
            commands::sync::start_sync,
            commands::sync::stop_sync,
            commands::sync::get_sync_status,
            commands::sync::sync_with_peer,
            commands::sync::sync_page_to_crdt,
            commands::sync::get_pending_sync_count,
            commands::sync::apply_crdt_to_db,
            commands::sharing::export_share_bundle,
            commands::sharing::import_share_bundle,
            commands::html_export::export_page_html,
            commands::html_export::export_pages_html,
            commands::workspace::list_workspaces,
            commands::workspace::get_active_workspace,
            commands::workspace::create_workspace,
            commands::workspace::delete_workspace,
            commands::workspace::rename_workspace,
            commands::workspace::switch_workspace,
            commands::encryption::is_encryption_enabled,
            commands::encryption::enable_encryption,
            commands::encryption::unlock_database,
            commands::encryption::disable_encryption,
            commands::encryption::change_passphrase,
            commands::encryption::verify_passphrase,
            commands::encryption::export_encrypted_workspace,
            commands::encryption::import_encrypted_workspace,
            commands::plugins::get_all_plugins,
            commands::plugins::get_enabled_plugins,
            commands::plugins::install_plugin,
            commands::plugins::set_plugin_enabled,
            commands::plugins::uninstall_plugin,
            commands::plugins::scan_plugins_dir,
            commands::plugins::read_plugin_file,
            commands::automation::api_create_page,
            commands::automation::api_capture_web_page,
            commands::automation::api_get_page,
            commands::automation::api_update_page,
            commands::automation::api_delete_page,
            commands::automation::api_search_pages,
            commands::automation::api_get_all_pages,
            commands::automation::api_get_page_tree,
            commands::automation::api_get_recent_pages,
            commands::automation::api_create_tag,
            commands::automation::api_get_all_tags,
            commands::automation::api_set_page_tags,
            commands::automation::api_get_backlinks,
            commands::automation::api_get_setting,
            commands::automation::api_set_setting,
            commands::importers::import_evernote,
            commands::importers::import_notion,
            commands::importers::import_obsidian,
            commands::importers::import_roam,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let state = app_handle.state::<AppState>();
                let passphrase = {
                    let guard = state.passphrase.lock().unwrap_or_else(|e| e.into_inner());
                    guard.clone()
                };
                if let Some(ref passphrase) = passphrase {
                    let app_data_dir = app_handle
                        .path()
                        .app_data_dir()
                        .expect("failed to get app data dir");
                    let db_path = app_data_dir.join("900notes.db");

                    if db_path.exists() {
                        {
                            if let Ok(db) = state.db.lock() {
                                let _ = db.checkpoint();
                            }
                        }

                        let enc_service = services::encryption::EncryptionService::new(&db_path);
                        if enc_service.is_encrypted() {
                            match enc_service.re_encrypt_on_shutdown(passphrase, &db_path) {
                                Ok(()) => {}
                                Err(e) => eprintln!("Failed to re-encrypt on shutdown: {e}"),
                            }
                        }
                    }
                }
            }
        });
}
