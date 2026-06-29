use crate::db::Database;
use crate::models::{CreatePageInput, Page};
use serde_json::{json, Value};
use std::path::Path;

pub struct ImportResult {
    pub pages_created: usize,
    pub errors: Vec<String>,
}

fn create_page_from_content(
    db: &Database,
    title: String,
    content: String,
    parent_id: Option<String>,
) -> Result<Page, String> {
    let input = CreatePageInput {
        title,
        content: Some(content),
        parent_id,
        icon: None,
    };
    db.create_page(&input).map_err(|e| e.to_string())
}

fn paragraph(text: &str) -> Value {
    json!({
        "type": "paragraph",
        "content": [{ "type": "text", "text": text }]
    })
}

fn heading(level: u64, text: &str) -> Value {
    json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [{ "type": "text", "text": text }]
    })
}

fn bullet_list(items: &[String]) -> Value {
    let list_items: Vec<Value> = items
        .iter()
        .map(|item| {
            json!({
                "type": "list_item",
                "content": [paragraph(item)]
            })
        })
        .collect();
    json!({
        "type": "bullet_list",
        "content": list_items
    })
}

fn build_doc(nodes: Vec<Value>) -> String {
    json!({
        "type": "doc",
        "content": nodes
    })
    .to_string()
}

pub fn import_evernote_enex(db: &Database, enex_content: &str) -> ImportResult {
    let mut pages_created = 0;
    let mut errors = Vec::new();

    let parsed = match roxmltree::Document::parse(enex_content) {
        Ok(doc) => doc,
        Err(e) => {
            errors.push(format!("XML parse error: {}", e));
            return ImportResult {
                pages_created,
                errors,
            };
        }
    };

    for note in parsed.descendants().filter(|n| n.has_tag_name("note")) {
        let title = note
            .descendants()
            .find(|n| n.has_tag_name("title"))
            .and_then(|n| n.text())
            .unwrap_or("Untitled")
            .to_string();

        let content_html = note
            .descendants()
            .find(|n| n.has_tag_name("content"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_string();

        let nodes = html_to_prosemirror(&content_html);
        let content = build_doc(nodes);

        match create_page_from_content(db, title, content, None) {
            Ok(_) => pages_created += 1,
            Err(e) => errors.push(format!("Failed to create page: {}", e)),
        }
    }

    ImportResult {
        pages_created,
        errors,
    }
}

pub fn import_notion_export(db: &Database, dir_path: &str) -> ImportResult {
    let mut pages_created = 0;
    let mut errors = Vec::new();
    let dir = Path::new(dir_path);

    if !dir.is_dir() {
        errors.push(format!("Not a directory: {}", dir_path));
        return ImportResult {
            pages_created,
            errors,
        };
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            errors.push(format!("Failed to read directory: {}", e));
            return ImportResult {
                pages_created,
                errors,
            };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content_str = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Failed to read {:?}: {}", path, e));
                    continue;
                }
            };

            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string();

            let nodes = markdown_text_to_prosemirror(&content_str);
            let pm_content = build_doc(nodes);

            match create_page_from_content(db, title, pm_content, None) {
                Ok(_) => pages_created += 1,
                Err(e) => errors.push(format!("Failed to create page: {}", e)),
            }
        }
    }

    ImportResult {
        pages_created,
        errors,
    }
}

pub fn import_obsidian_vault(db: &Database, dir_path: &str) -> ImportResult {
    let mut pages_created = 0;
    let mut errors = Vec::new();
    let dir = Path::new(dir_path);

    if !dir.is_dir() {
        errors.push(format!("Not a directory: {}", dir_path));
        return ImportResult {
            pages_created,
            errors,
        };
    }

    fn walk_dir(dir: &Path, db: &Database, pages_created: &mut usize, errors: &mut Vec<String>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Failed to read {:?}: {}", dir, e));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, db, pages_created, errors);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content_str = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("Failed to read {:?}: {}", path, e));
                        continue;
                    }
                };

                let title = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();

                let nodes = markdown_text_to_prosemirror(&content_str);
                let pm_content = build_doc(nodes);

                match create_page_from_content(db, title, pm_content, None) {
                    Ok(_) => *pages_created += 1,
                    Err(e) => errors.push(format!("Failed to create page: {}", e)),
                }
            }
        }
    }

    walk_dir(dir, db, &mut pages_created, &mut errors);

    ImportResult {
        pages_created,
        errors,
    }
}

pub fn import_roam_json(db: &Database, json_content: &str) -> ImportResult {
    let mut pages_created = 0;
    let mut errors = Vec::new();

    let roam_data: Vec<Value> = match serde_json::from_str(json_content) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("JSON parse error: {}", e));
            return ImportResult {
                pages_created,
                errors,
            };
        }
    };

    for page in roam_data {
        let title = page
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let children = page.get("children").and_then(|c| c.as_array());

        let mut nodes = vec![heading(1, &title)];

        if let Some(children) = children {
            for child in children {
                if let Some(text) = child.get("string").and_then(|s| s.as_str()) {
                    nodes.push(paragraph(text));
                }
            }
        }

        let content = build_doc(nodes);

        match create_page_from_content(db, title, content, None) {
            Ok(_) => pages_created += 1,
            Err(e) => errors.push(format!("Failed to create page: {}", e)),
        }
    }

    ImportResult {
        pages_created,
        errors,
    }
}

fn html_to_prosemirror(html: &str) -> Vec<Value> {
    let mut nodes = Vec::new();
    let mut current_text = String::new();

    let stripped = html
        .replace("<en-note>", "")
        .replace("</en-note>", "")
        .replace("<div>", "\n")
        .replace("</div>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n");

    let without_tags = strip_html_tags(&stripped);

    for line in without_tags.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current_text.is_empty() {
                nodes.push(paragraph(current_text.trim()));
                current_text.clear();
            }
        } else {
            if !current_text.is_empty() {
                current_text.push(' ');
            }
            current_text.push_str(trimmed);
        }
    }

    if !current_text.is_empty() {
        nodes.push(paragraph(current_text.trim()));
    }

    if nodes.is_empty() {
        nodes.push(paragraph(""));
    }

    nodes
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            output.push(ch);
        }
    }
    output
}

fn markdown_text_to_prosemirror(md: &str) -> Vec<Value> {
    let mut nodes = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("### ") {
            nodes.push(heading(3, rest));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            nodes.push(heading(2, rest));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            nodes.push(heading(1, rest));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let items: Vec<String> = md
                .lines()
                .filter(|l| l.trim_start().starts_with("- ") || l.trim_start().starts_with("* "))
                .map(|l| l.trim_start()[2..].to_string())
                .collect();
            nodes.push(bullet_list(&items));
            break;
        } else {
            nodes.push(paragraph(trimmed));
        }
    }

    if nodes.is_empty() {
        nodes.push(paragraph(""));
    }

    nodes
}
