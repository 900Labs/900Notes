use crate::models::{
    CreatePageInput, CreateTagInput, Page, PageMetadata, SetPropertyInput, Tag, UpdatePageInput,
    WebCaptureInput,
};
use crate::AppState;
use tauri::State;

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn paragraph(text: &str) -> serde_json::Value {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        serde_json::json!({ "type": "paragraph" })
    } else {
        serde_json::json!({
            "type": "paragraph",
            "content": [{ "type": "text", "text": trimmed.replace('\n', " ") }],
        })
    }
}

fn link_paragraph(label: &str, url: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "paragraph",
        "content": [
            { "type": "text", "text": label },
            {
                "type": "text",
                "text": url,
                "marks": [{ "type": "link", "attrs": { "href": url } }],
            },
        ],
    })
}

fn prosemirror_from_web_capture(source_url: &str, body: Option<&str>, captured_at: &str) -> String {
    let mut content = vec![
        link_paragraph("Source: ", source_url),
        paragraph(&format!("Captured: {captured_at}")),
    ];

    if let Some(body) = body {
        for block in body.split("\n\n").map(str::trim).filter(|b| !b.is_empty()) {
            content.push(paragraph(block));
        }
    }

    serde_json::json!({
        "type": "doc",
        "content": content,
    })
    .to_string()
}

fn title_from_url(source_url: &str) -> String {
    let without_scheme = source_url
        .strip_prefix("https://")
        .or_else(|| source_url.strip_prefix("http://"))
        .unwrap_or(source_url);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme)
        .trim();
    if host.is_empty() {
        "Web capture".to_string()
    } else {
        host.to_string()
    }
}

fn find_inbox_id(pages: &[PageMetadata]) -> Option<String> {
    pages
        .iter()
        .find(|page| page.title.eq_ignore_ascii_case("Inbox"))
        .map(|page| page.id.clone())
}

fn tag_by_name(tags: &[Tag], name: &str) -> Option<Tag> {
    tags.iter()
        .find(|tag| tag.name.eq_ignore_ascii_case(name))
        .cloned()
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
    let source_url = input.source_url.trim().to_string();
    if !(source_url.starts_with("https://") || source_url.starts_with("http://")) {
        return Err("sourceUrl must start with http:// or https://".to_string());
    }

    let captured_at =
        trim_optional(input.captured_at).unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let title = trim_optional(input.title).unwrap_or_else(|| title_from_url(&source_url));
    let body = trim_optional(input.body);

    let db = state.db.lock().map_err(map_err)?;
    let parent_id = if input.use_inbox.unwrap_or(true) {
        match find_inbox_id(&db.get_all_pages_metadata().map_err(map_err)?) {
            Some(id) => Some(id),
            None => {
                let inbox = db
                    .create_page(&CreatePageInput {
                        parent_id: None,
                        title: "Inbox".to_string(),
                        content: Some(
                            serde_json::json!({
                                "type": "doc",
                                "content": [paragraph("Captured notes and incoming material.")],
                            })
                            .to_string(),
                        ),
                        icon: Some("IN".to_string()),
                    })
                    .map_err(map_err)?;
                Some(inbox.id)
            }
        }
    } else {
        None
    };

    let page = db
        .create_page(&CreatePageInput {
            parent_id,
            title: title.clone(),
            content: Some(prosemirror_from_web_capture(
                &source_url,
                body.as_deref(),
                &captured_at,
            )),
            icon: Some("CL".to_string()),
        })
        .map_err(map_err)?;

    if let Some(tag_names) = input.tags {
        let mut existing_tags = db.get_all_tags().map_err(map_err)?;
        let mut tag_ids = Vec::new();
        for name in tag_names
            .into_iter()
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
        {
            let tag = match tag_by_name(&existing_tags, &name) {
                Some(tag) => tag,
                None => {
                    let tag = db
                        .create_tag(&CreateTagInput { name, color: None })
                        .map_err(map_err)?;
                    existing_tags.push(tag.clone());
                    tag
                }
            };
            if !tag_ids.contains(&tag.id) {
                tag_ids.push(tag.id);
            }
        }
        if !tag_ids.is_empty() {
            db.set_page_tags(&page.id, &tag_ids).map_err(map_err)?;
        }
    }

    for (key, value) in [
        ("capture.type", "web".to_string()),
        ("capture.source_url", source_url),
        ("capture.source_title", title),
        ("capture.captured_at", captured_at),
    ] {
        db.set_page_property(&SetPropertyInput {
            page_id: page.id.clone(),
            key: key.to_string(),
            value,
        })
        .map_err(map_err)?;
    }

    Ok(page)
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
