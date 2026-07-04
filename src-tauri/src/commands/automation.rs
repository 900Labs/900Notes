use crate::models::{CreatePageInput, CreateTagInput, Page, UpdatePageInput, WebCaptureInput};
use crate::AppState;
use tauri::State;

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub fn api_create_page(
    title: String,
    content: Option<String>,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::models::Page, String> {
    let db = state.db.lock().map_err(map_err)?;
    let input = CreatePageInput {
        title,
        content: Some(content.unwrap_or_default()),
        parent_id,
        icon: None,
    };
    db.create_page(&input).map_err(map_err)
}

#[tauri::command]
pub fn api_capture_web_page(
    input: WebCaptureInput,
    state: State<'_, AppState>,
) -> Result<Page, String> {
    let db = state.db.lock().map_err(map_err)?;
    crate::services::web_capture::capture_web_page(&db, input)
}

#[tauri::command]
pub fn api_get_page(id: String, state: State<'_, AppState>) -> Result<crate::models::Page, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_page_by_id(&id).map_err(map_err)
}

#[tauri::command]
pub fn api_update_page(
    id: String,
    title: Option<String>,
    content: Option<String>,
    icon: Option<String>,
    cover_color: Option<String>,
    pinned: Option<bool>,
    state: State<'_, AppState>,
) -> Result<crate::models::Page, String> {
    let db = state.db.lock().map_err(map_err)?;
    let existing = db.get_page_by_id(&id).map_err(map_err)?;
    let input = UpdatePageInput {
        id: id.clone(),
        title: Some(title.unwrap_or(existing.title)),
        content: Some(content.unwrap_or(existing.content)),
        icon,
        cover_color,
        pinned,
    };
    db.update_page(&input).map_err(map_err)
}

#[tauri::command]
pub fn api_delete_page(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    db.delete_page(&id).map_err(map_err)
}

#[tauri::command]
pub fn api_search_pages(
    query: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::SearchResult>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.search_pages(&query, limit.unwrap_or(50))
        .map_err(map_err)
}

#[tauri::command]
pub fn api_get_all_pages(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::PageMetadata>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_all_pages_metadata().map_err(map_err)
}

#[tauri::command]
pub fn api_get_page_tree(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::PageTreeNodeMeta>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_page_tree_metadata().map_err(map_err)
}

#[tauri::command]
pub fn api_get_recent_pages(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::PageMetadata>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_recent_pages_metadata(limit.unwrap_or(20))
        .map_err(map_err)
}

#[tauri::command]
pub fn api_create_tag(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::models::Tag, String> {
    let db = state.db.lock().map_err(map_err)?;
    let input = CreateTagInput { name, color };
    db.create_tag(&input).map_err(map_err)
}

#[tauri::command]
pub fn api_get_all_tags(state: State<'_, AppState>) -> Result<Vec<crate::models::Tag>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_all_tags().map_err(map_err)
}

#[tauri::command]
pub fn api_set_page_tags(
    page_id: String,
    tag_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    db.set_page_tags(&page_id, &tag_ids).map_err(map_err)
}

#[tauri::command]
pub fn api_get_backlinks(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::Backlink>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_backlinks(&page_id).map_err(map_err)
}

#[tauri::command]
pub fn api_get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(map_err)?;
    db.get_setting(&key).map_err(map_err)
}

#[tauri::command]
pub fn api_set_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(map_err)?;
    db.set_setting(&key, &value).map_err(map_err)
}
