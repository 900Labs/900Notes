use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::db::Database;
use crate::models::{
    CreatePageInput, CreateTagInput, Page, PageMetadata, SetPropertyInput, Tag, WebCaptureInput,
};

pub const DEFAULT_CLIPPER_PORT: u16 = 17690;
pub const CLIPPER_TOKEN_FILE: &str = "web-clipper-token";

const MAX_CLIPPER_BODY_BYTES: usize = 2 * 1024 * 1024;
const READ_CHUNK_BYTES: usize = 8 * 1024;
const CLIPPER_AUTH_HEADER: &str = "X-900Notes-Clipper-Token";
const CLIPPER_TOKEN_BYTES: usize = 32;

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipperPayload {
    title: Option<String>,
    source_url: Option<String>,
    url: Option<String>,
    body: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    use_inbox: Option<bool>,
    captured_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipSuccess {
    ok: bool,
    page_id: String,
    title: String,
}

#[derive(Serialize)]
struct JsonError {
    ok: bool,
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
    port: u16,
}

pub fn load_or_create_clipper_token(app_data_dir: &Path) -> Result<String, String> {
    let token_path = app_data_dir.join(CLIPPER_TOKEN_FILE);
    match std::fs::read_to_string(&token_path) {
        Ok(token) => {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("read web clipper token: {error}")),
    }

    std::fs::create_dir_all(app_data_dir).map_err(|e| format!("create app data dir: {e}"))?;
    let token = generate_clipper_token()?;
    write_new_token_file(&token_path, &token)?;
    let token = std::fs::read_to_string(&token_path)
        .map_err(|e| format!("read web clipper token: {e}"))?
        .trim()
        .to_string();
    if token.is_empty() {
        Err("web clipper token file is empty".to_string())
    } else {
        Ok(token)
    }
}

pub fn start_clipper_server(
    db: Arc<Mutex<Database>>,
    port: u16,
    auth_token: String,
    workspace_locked: Arc<AtomicBool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("failed to bind web clipper on 127.0.0.1:{port}: {e}"))?;
    let auth_token = Arc::new(auth_token);

    thread::Builder::new()
        .name("900notes-web-clipper".to_string())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let db = db.clone();
                        let auth_token = auth_token.clone();
                        let workspace_locked = workspace_locked.clone();
                        thread::spawn(move || {
                            if let Err(error) = handle_clipper_connection(
                                stream,
                                db,
                                port,
                                auth_token,
                                workspace_locked,
                            ) {
                                eprintln!("Web clipper request failed: {error}");
                            }
                        });
                    }
                    Err(error) => {
                        eprintln!("Web clipper listener stopped: {error}");
                        break;
                    }
                }
            }
        })
        .map_err(|e| format!("failed to start web clipper thread: {e}"))?;

    Ok(())
}

pub fn capture_web_page(db: &Database, input: WebCaptureInput) -> Result<Page, String> {
    let source_url = input.source_url.trim().to_string();
    if !(source_url.starts_with("https://") || source_url.starts_with("http://")) {
        return Err("sourceUrl must start with http:// or https://".to_string());
    }

    let captured_at =
        trim_optional(input.captured_at).unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let title = trim_optional(input.title).unwrap_or_else(|| title_from_url(&source_url));
    let body = trim_optional(input.body);

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

fn handle_clipper_connection(
    mut stream: TcpStream,
    db: Arc<Mutex<Database>>,
    port: u16,
    auth_token: Arc<String>,
    workspace_locked: Arc<AtomicBool>,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(map_err)?;
    let request = read_request(&mut stream)?;
    let cors_origin = match request_cors_origin(&request) {
        Ok(origin) => origin,
        Err(error) => {
            write_json_error(&mut stream, 403, &error, None)?;
            return Ok(());
        }
    };

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health") => {
            let body = serde_json::to_string(&HealthResponse {
                ok: true,
                service: "900Notes Web Clipper",
                port,
            })
            .map_err(map_err)?;
            write_response(&mut stream, 200, Some(&body), cors_origin.as_deref())
        }
        ("OPTIONS", "/api/clip") => write_response(&mut stream, 204, None, cors_origin.as_deref()),
        ("POST", "/api/clip") => handle_clip_post(
            &mut stream,
            request,
            db,
            cors_origin.as_deref(),
            auth_token.as_str(),
            workspace_locked.as_ref(),
        ),
        _ => write_json_error(&mut stream, 404, "not found", cors_origin.as_deref()),
    }
}

fn handle_clip_post(
    stream: &mut TcpStream,
    request: HttpRequest,
    db: Arc<Mutex<Database>>,
    cors_origin: Option<&str>,
    auth_token: &str,
    workspace_locked: &AtomicBool,
) -> Result<(), String> {
    if workspace_locked.load(Ordering::Acquire) {
        return write_json_error(stream, 423, "workspace is locked", cors_origin);
    }
    if let Some(error) = clipper_auth_error(&request, auth_token) {
        return write_json_error(stream, 403, &error, cors_origin);
    }

    let content_type = request.header("content-type").unwrap_or_default();
    if !content_type
        .to_ascii_lowercase()
        .starts_with("application/json")
    {
        return write_json_error(
            stream,
            415,
            "content type must be application/json",
            cors_origin,
        );
    }

    let payload: ClipperPayload = match serde_json::from_slice(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return write_json_error(
                stream,
                400,
                &format!("invalid JSON payload: {error}"),
                cors_origin,
            );
        }
    };

    let input = match WebCaptureInput::try_from(payload) {
        Ok(input) => input,
        Err(error) => return write_json_error(stream, 400, &error, cors_origin),
    };

    let page = {
        let db = db.lock().map_err(map_err)?;
        capture_web_page(&db, input)
    };

    match page {
        Ok(page) => {
            let body = serde_json::to_string(&ClipSuccess {
                ok: true,
                page_id: page.id,
                title: page.title,
            })
            .map_err(map_err)?;
            write_response(stream, 200, Some(&body), cors_origin)
        }
        Err(error) => write_json_error(stream, 400, &error, cors_origin),
    }
}

fn clipper_auth_error(request: &HttpRequest, auth_token: &str) -> Option<String> {
    request
        .header("x-900notes-clipper-token")
        .is_none_or(|value| value != auth_token)
        .then(|| format!("missing or invalid {CLIPPER_AUTH_HEADER} header"))
}

impl HttpRequest {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

impl TryFrom<ClipperPayload> for WebCaptureInput {
    type Error = String;

    fn try_from(payload: ClipperPayload) -> Result<Self, Self::Error> {
        let source_url = payload
            .source_url
            .or(payload.url)
            .map(|url| url.trim().to_string())
            .filter(|url| !url.is_empty())
            .ok_or_else(|| "sourceUrl is required".to_string())?;

        Ok(WebCaptureInput {
            title: payload.title,
            source_url,
            body: payload.body.or(payload.content),
            tags: payload.tags,
            use_inbox: payload.use_inbox,
            captured_at: payload.captured_at,
        })
    }
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    let mut header_end = None;

    while header_end.is_none() {
        let bytes_read = stream.read(&mut chunk).map_err(map_err)?;
        if bytes_read == 0 {
            return Err("connection closed before headers were read".to_string());
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        if buffer.len() > MAX_CLIPPER_BODY_BYTES {
            return Err("request is too large".to_string());
        }
        header_end = find_header_end(&buffer);
    }

    let header_end = header_end.expect("header end already checked");
    let body_start = header_end + 4;
    let headers_text = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = headers_text.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "missing request method".to_string())?
        .to_string();
    let path = request_parts
        .next()
        .ok_or_else(|| "missing request path".to_string())?
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let content_length = headers
        .get("content-length")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid Content-Length".to_string())
        })
        .transpose()?
        .unwrap_or(0);
    if content_length > MAX_CLIPPER_BODY_BYTES {
        return Err("request body is too large".to_string());
    }

    while buffer.len() < body_start + content_length {
        let bytes_read = stream.read(&mut chunk).map_err(map_err)?;
        if bytes_read == 0 {
            return Err("connection closed before body was read".to_string());
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        if buffer.len() > body_start + MAX_CLIPPER_BODY_BYTES {
            return Err("request body is too large".to_string());
        }
    }

    Ok(HttpRequest {
        method,
        path,
        headers,
        body: buffer[body_start..body_start + content_length].to_vec(),
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn request_cors_origin(request: &HttpRequest) -> Result<Option<String>, String> {
    match request.header("origin") {
        Some(origin) if is_allowed_extension_origin(origin) => Ok(Some(origin.to_string())),
        Some(_) => Err("origin is not allowed".to_string()),
        None => Ok(None),
    }
}

fn is_allowed_extension_origin(origin: &str) -> bool {
    origin.starts_with("chrome-extension://")
        || origin.starts_with("moz-extension://")
        || origin.starts_with("safari-web-extension://")
}

fn write_json_error(
    stream: &mut TcpStream,
    status: u16,
    error: &str,
    cors_origin: Option<&str>,
) -> Result<(), String> {
    let body = serde_json::to_string(&JsonError {
        ok: false,
        error: error.to_string(),
    })
    .map_err(map_err)?;
    write_response(stream, status, Some(&body), cors_origin)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    body: Option<&str>,
    cors_origin: Option<&str>,
) -> Result<(), String> {
    let body = body.unwrap_or_default();
    let mut headers = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status_reason(status),
        body.len()
    );
    if !body.is_empty() {
        headers.push_str("Content-Type: application/json\r\n");
    }
    if let Some(origin) = cors_origin {
        headers.push_str("Vary: Origin\r\n");
        headers.push_str(&format!("Access-Control-Allow-Origin: {origin}\r\n"));
        headers.push_str("Access-Control-Allow-Methods: POST, OPTIONS\r\n");
        headers
            .push_str("Access-Control-Allow-Headers: content-type, x-900notes-clipper-token\r\n");
        headers.push_str("Access-Control-Max-Age: 600\r\n");
    }
    headers.push_str("\r\n");

    stream.write_all(headers.as_bytes()).map_err(map_err)?;
    stream.write_all(body.as_bytes()).map_err(map_err)?;
    stream.flush().map_err(map_err)
}

fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "200 OK",
        204 => "204 No Content",
        400 => "400 Bad Request",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        415 => "415 Unsupported Media Type",
        _ => "500 Internal Server Error",
    }
}

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn generate_clipper_token() -> Result<String, String> {
    let mut bytes = [0u8; CLIPPER_TOKEN_BYTES];
    getrandom::getrandom(&mut bytes).map_err(|e| format!("CSPRNG: {e}"))?;
    Ok(BASE64_URL.encode(bytes))
}

fn write_new_token_file(path: &Path, token: &str) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    match options.open(path) {
        Ok(mut file) => {
            file.write_all(token.as_bytes())
                .map_err(|e| format!("write web clipper token: {e}"))?;
            file.write_all(b"\n")
                .map_err(|e| format!("write web clipper token: {e}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = std::fs::read_to_string(path)
                .map_err(|e| format!("read web clipper token: {e}"))?;
            let existing = existing.trim();
            if existing.is_empty() {
                Err("web clipper token file is empty".to_string())
            } else {
                Ok(())
            }
        }
        Err(error) => Err(format!("create web clipper token: {error}")),
    }
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const TEST_CLIPPER_TOKEN: &str = "test-clipper-token";

    #[test]
    fn capture_web_page_creates_inbox_tags_and_metadata() {
        let db = Database::open(Path::new(":memory:")).unwrap();

        let page = capture_web_page(
            &db,
            WebCaptureInput {
                title: Some("Example Title".to_string()),
                source_url: "https://example.com/article".to_string(),
                body: Some("First paragraph.\n\nSecond paragraph.".to_string()),
                tags: Some(vec![
                    "web".to_string(),
                    "Research".to_string(),
                    "web".to_string(),
                    " ".to_string(),
                ]),
                use_inbox: Some(true),
                captured_at: Some("2026-07-04T12:00:00Z".to_string()),
            },
        )
        .unwrap();

        assert_eq!(page.title, "Example Title");
        assert_eq!(page.icon.as_deref(), Some("CL"));
        assert!(page.content.contains("https://example.com/article"));
        assert!(page.content.contains("First paragraph."));

        let pages = db.get_all_pages_metadata().unwrap();
        let inbox = pages
            .iter()
            .find(|page| page.title == "Inbox")
            .expect("Inbox page should be created");
        assert_eq!(page.parent_id.as_deref(), Some(inbox.id.as_str()));

        let tags = db.get_page_tags(&page.id).unwrap();
        let tag_names = tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
        assert_eq!(tag_names.len(), 2);
        assert!(tag_names.contains(&"web"));
        assert!(tag_names.contains(&"Research"));

        let properties = db.get_page_properties(&page.id).unwrap();
        let property_value = |key: &str| {
            properties
                .iter()
                .find(|property| property.key == key)
                .map(|property| property.value.as_str())
        };
        assert_eq!(property_value("capture.type"), Some("web"));
        assert_eq!(
            property_value("capture.source_url"),
            Some("https://example.com/article")
        );
        assert_eq!(
            property_value("capture.source_title"),
            Some("Example Title")
        );
        assert_eq!(
            property_value("capture.captured_at"),
            Some("2026-07-04T12:00:00Z")
        );
    }

    #[test]
    fn capture_web_page_rejects_non_http_sources() {
        let db = Database::open(Path::new(":memory:")).unwrap();

        let error = capture_web_page(
            &db,
            WebCaptureInput {
                title: None,
                source_url: "file:///tmp/example.html".to_string(),
                body: None,
                tags: None,
                use_inbox: None,
                captured_at: None,
            },
        )
        .unwrap_err();

        assert_eq!(error, "sourceUrl must start with http:// or https://");
    }

    #[test]
    fn load_or_create_clipper_token_persists_random_token() {
        let dir =
            std::env::temp_dir().join(format!("900notes-clipper-token-{}", uuid::Uuid::new_v4()));
        let token_path = dir.join(CLIPPER_TOKEN_FILE);

        let token = load_or_create_clipper_token(&dir).unwrap();

        assert!(token.len() >= 32);
        assert_eq!(std::fs::read_to_string(&token_path).unwrap().trim(), token);
        assert_eq!(load_or_create_clipper_token(&dir).unwrap(), token);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn clipper_auth_rejects_missing_token() {
        let request = request_with_token(None);

        assert_eq!(
            clipper_auth_error(&request, TEST_CLIPPER_TOKEN),
            Some("missing or invalid X-900Notes-Clipper-Token header".to_string())
        );
    }

    #[test]
    fn clipper_auth_rejects_wrong_token() {
        let request = request_with_token(Some("wrong-token"));

        assert_eq!(
            clipper_auth_error(&request, TEST_CLIPPER_TOKEN),
            Some("missing or invalid X-900Notes-Clipper-Token header".to_string())
        );
    }

    #[test]
    fn clipper_auth_accepts_valid_token() {
        let request = request_with_token(Some(TEST_CLIPPER_TOKEN));

        assert_eq!(clipper_auth_error(&request, TEST_CLIPPER_TOKEN), None);
    }

    fn request_with_token(token: Option<&str>) -> HttpRequest {
        let mut headers = HashMap::new();
        if let Some(token) = token {
            headers.insert("x-900notes-clipper-token".to_string(), token.to_string());
        }
        HttpRequest {
            method: "POST".to_string(),
            path: "/api/clip".to_string(),
            headers,
            body: Vec::new(),
        }
    }
}
