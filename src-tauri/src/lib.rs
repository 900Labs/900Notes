mod commands;
mod db;
mod models;
mod services;

use std::sync::{Arc, Mutex};
#[cfg(desktop)]
use tauri::Emitter;
use tauri::Manager;

#[cfg(desktop)]
const NATIVE_MENU_EVENT: &str = "900notes-menu-action";

pub struct AppState {
    pub db: Arc<Mutex<db::Database>>,
    pub sync: Mutex<Option<services::sync::SyncService>>,
    pub crdt: Mutex<services::crdt::CrdtService>,
    pub passphrase: Mutex<Option<String>>,
    pub clipper_started: Mutex<bool>,
    pub clipper_token: String,
}

pub struct WorkspaceState {
    pub service: Mutex<services::workspace::WorkspaceService>,
}

pub fn start_web_clipper(state: &AppState) -> Result<(), String> {
    let mut started = state.clipper_started.lock().map_err(|e| e.to_string())?;
    if *started {
        return Ok(());
    }

    services::web_capture::start_clipper_server(
        state.db.clone(),
        services::web_capture::DEFAULT_CLIPPER_PORT,
        state.clipper_token.clone(),
    )?;
    *started = true;
    Ok(())
}

#[cfg(desktop)]
fn native_menu_item<R: tauri::Runtime, M: tauri::Manager<R>>(
    manager: &M,
    id: &str,
    text: &str,
    accelerator: Option<&str>,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
    let mut builder = tauri::menu::MenuItemBuilder::with_id(id, text);
    if let Some(accelerator) = accelerator {
        builder = builder.accelerator(accelerator);
    }
    builder.build(manager)
}

#[cfg(desktop)]
fn build_native_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::menu::Menu<R>> {
    use tauri::menu::{Menu, SubmenuBuilder};

    let new_page = native_menu_item(app, "newPage", "New Page", Some("CmdOrCtrl+KeyN"))?;
    let quick_capture = native_menu_item(
        app,
        "openQuickCapture",
        "Quick Capture",
        Some("CmdOrCtrl+Shift+KeyC"),
    )?;
    let web_capture = native_menu_item(
        app,
        "openWebCapture",
        "Web Capture",
        Some("CmdOrCtrl+Shift+KeyL"),
    )?;
    let daily_note = native_menu_item(app, "dailyNote", "Today", None)?;
    let from_template = native_menu_item(app, "newFromTemplate", "From Template", None)?;
    let create = SubmenuBuilder::new(app, "Create")
        .item(&new_page)
        .separator()
        .item(&quick_capture)
        .item(&web_capture)
        .separator()
        .item(&daily_note)
        .item(&from_template)
        .build()?;

    let command_palette = native_menu_item(
        app,
        "openCommandPalette",
        "Command Palette",
        Some("CmdOrCtrl+KeyK"),
    )?;
    let recent_pages = native_menu_item(app, "openRecent", "Recent Pages", None)?;
    let favorites = native_menu_item(app, "toggleFavorites", "Favorites", None)?;
    let smart_folders = native_menu_item(app, "toggleSmartFolders", "Smart Folders", None)?;
    let navigate = SubmenuBuilder::new(app, "Navigate")
        .item(&command_palette)
        .separator()
        .item(&recent_pages)
        .item(&favorites)
        .item(&smart_folders)
        .build()?;

    let knowledge_graph = native_menu_item(app, "toggleGraph", "Knowledge Graph", None)?;
    let local_graph = native_menu_item(app, "openLocalGraph", "Local Graph", None)?;
    let outline = native_menu_item(app, "toggleOutline", "Outline", None)?;
    let backlinks = native_menu_item(app, "toggleBacklinks", "Backlinks", None)?;
    let related_pages = native_menu_item(app, "toggleRelated", "Related Pages", None)?;
    let page_history = native_menu_item(app, "toggleHistory", "Page History", None)?;
    let view = SubmenuBuilder::new(app, "View")
        .item(&knowledge_graph)
        .item(&local_graph)
        .separator()
        .item(&outline)
        .item(&backlinks)
        .item(&related_pages)
        .item(&page_history)
        .build()?;

    let markdown = native_menu_item(app, "exportMarkdown", "Current Page as Markdown", None)?;
    let page_pdf = native_menu_item(app, "exportPagePdf", "Current Page as PDF", None)?;
    let workspace_pdf = native_menu_item(app, "exportWorkspacePdf", "Workspace as PDF", None)?;
    let import_backup = native_menu_item(app, "openDataSettings", "Import and Backup", None)?;
    let export = SubmenuBuilder::new(app, "Export")
        .item(&markdown)
        .item(&page_pdf)
        .separator()
        .item(&workspace_pdf)
        .separator()
        .item(&import_backup)
        .build()?;

    let local_sync = native_menu_item(app, "openSyncSettings", "Local Sync", None)?;
    let sharing = native_menu_item(app, "openSharingSettings", "Sharing", None)?;
    let workspaces = native_menu_item(app, "openWorkspacesSettings", "Workspaces", None)?;
    let security = native_menu_item(app, "openSecuritySettings", "Security", None)?;
    let plugins = native_menu_item(app, "openPluginsSettings", "Plugins", None)?;
    let settings = native_menu_item(app, "openSettings", "Settings", Some("CmdOrCtrl+Comma"))?;
    let tools = SubmenuBuilder::new(app, "Tools")
        .item(&local_sync)
        .item(&sharing)
        .item(&workspaces)
        .separator()
        .item(&security)
        .item(&plugins)
        .separator()
        .item(&settings)
        .build()?;

    Menu::with_items(app, &[&create, &navigate, &view, &export, &tools])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(desktop)]
    let builder = builder.menu(build_native_menu).on_menu_event(|app, event| {
        let _ = app.emit(NATIVE_MENU_EVENT, event.id().as_ref().to_string());
    });

    builder
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
            let clipper_token = services::web_capture::load_or_create_clipper_token(&app_data_dir)
                .expect("failed to load web clipper token");
            let app_state = AppState {
                db,
                sync: Mutex::new(None),
                crdt: Mutex::new(crdt),
                passphrase: Mutex::new(None),
                clipper_started: Mutex::new(false),
                clipper_token,
            };
            if !encryption_enabled {
                if let Err(error) = start_web_clipper(&app_state) {
                    eprintln!("Failed to start web clipper server: {error}");
                }
            }
            app.manage(app_state);
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
