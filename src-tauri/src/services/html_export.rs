use serde_json::Value;

use crate::db::Database;
use crate::models::*;

pub fn prosemirror_to_html(content: &str) -> String {
    let doc: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut output = String::new();
    if let Some(nodes) = doc.get("content").and_then(|c| c.as_array()) {
        for node in nodes {
            render_node_html(node, &mut output);
        }
    }
    output
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Returns the URL unchanged only when its scheme is safe to emit in an
/// exported `href`. `javascript:`, `data:`, `vbscript:`, and other dangerous
/// schemes are rejected so an exported or in-editor link cannot carry a script
/// payload when opened outside the app's strict CSP. Fragment links and
/// scheme-less relative URLs are allowed.
fn safe_href(url: &str) -> &str {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return "";
    }
    if let Some(colon) = trimmed.find(':') {
        if colon > 0 {
            let scheme = trimmed[..colon].to_ascii_lowercase();
            // A scheme starts with a letter and contains only letters, digits,
            // '+', '-', '.' (RFC 3986). Anything else (e.g. "/path/a:b") is a
            // relative URL, not a scheme.
            let looks_like_scheme = scheme.chars().enumerate().all(|(i, c)| {
                if i == 0 {
                    c.is_ascii_lowercase()
                } else {
                    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '+' || c == '-' || c == '.'
                }
            });
            if looks_like_scheme
                && !matches!(
                    scheme.as_str(),
                    "http" | "https" | "mailto" | "ftp" | "ftps" | "tel"
                )
            {
                return "";
            }
        }
    }
    trimmed
}

fn render_node_html(node: &Value, output: &mut String) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(1);
            output.push_str(&format!("<h{}>", level));
            render_inline_html(node, output);
            output.push_str(&format!("</h{}>\n", level));
        }
        "paragraph" => {
            output.push_str("<p>");
            render_inline_html(node, output);
            output.push_str("</p>\n");
        }
        "bullet_list" => {
            output.push_str("<ul>\n");
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for item in items {
                    output.push_str("<li>");
                    if let Some(item_content) = item.get("content").and_then(|c| c.as_array()) {
                        for child in item_content {
                            render_node_html(child, output);
                        }
                    }
                    output.push_str("</li>\n");
                }
            }
            output.push_str("</ul>\n");
        }
        "ordered_list" => {
            output.push_str("<ol>\n");
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for item in items {
                    output.push_str("<li>");
                    if let Some(item_content) = item.get("content").and_then(|c| c.as_array()) {
                        for child in item_content {
                            render_node_html(child, output);
                        }
                    }
                    output.push_str("</li>\n");
                }
            }
            output.push_str("</ol>\n");
        }
        "todo_item" => {
            let checked = node
                .get("attrs")
                .and_then(|a| a.get("checked"))
                .and_then(|c| c.as_bool())
                .unwrap_or(false);
            output.push_str("<div class=\"todo-item\">");
            output.push_str(if checked {
                "<input type=\"checkbox\" checked disabled> "
            } else {
                "<input type=\"checkbox\" disabled> "
            });
            render_inline_html(node, output);
            output.push_str("</div>\n");
        }
        "code_block" => {
            output.push_str("<pre><code>");
            if let Some(text) = node
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| {
                    arr.first()
                        .and_then(|n| n.get("text"))
                        .and_then(|t| t.as_str())
                })
            {
                output.push_str(&escape_html(text));
            }
            output.push_str("</code></pre>\n");
        }
        "blockquote" => {
            output.push_str("<blockquote>");
            if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                for child in content {
                    render_node_html(child, output);
                }
            }
            output.push_str("</blockquote>\n");
        }
        "divider" => {
            output.push_str("<hr>\n");
        }
        "audio_block" => {
            let title = node
                .get("attrs")
                .and_then(|a| a.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("Audio Note");
            output.push_str(&format!(
                "<div class=\"audio-block\"><strong>🎙 {}</strong></div>\n",
                escape_html(title)
            ));
        }
        "image" => {
            let src = node
                .get("attrs")
                .and_then(|a| a.get("src"))
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let alt = node
                .get("attrs")
                .and_then(|a| a.get("alt"))
                .and_then(|a| a.as_str())
                .unwrap_or("");
            output.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\">\n",
                escape_html(src),
                escape_html(alt)
            ));
        }
        _ => {
            render_inline_html(node, output);
        }
    }
}

fn render_inline_html(node: &Value, output: &mut String) {
    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            let child_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match child_type {
                "text" => {
                    let text = child.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let marks = child.get("marks").and_then(|m| m.as_array());
                    let mut open_tags: Vec<String> = Vec::new();
                    if let Some(marks) = marks {
                        for mark in marks {
                            let mark_type = mark.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            match mark_type {
                                "bold" => open_tags.push("<strong>".to_string()),
                                "italic" => open_tags.push("<em>".to_string()),
                                "strike" => open_tags.push("<s>".to_string()),
                                "code" => open_tags.push("<code>".to_string()),
                                "link" => {
                                    let href = mark
                                        .get("attrs")
                                        .and_then(|a| a.get("href"))
                                        .and_then(|h| h.as_str())
                                        .unwrap_or("");
                                    open_tags.push(format!(
                                        "<a href=\"{}\" rel=\"noopener noreferrer\">",
                                        escape_html(safe_href(href))
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                    for tag in &open_tags {
                        output.push_str(tag);
                    }
                    output.push_str(&escape_html(text));
                    for tag in open_tags.iter().rev() {
                        if tag.starts_with("<a ") {
                            output.push_str("</a>");
                        } else if tag == "<strong>" {
                            output.push_str("</strong>");
                        } else if tag == "<em>" {
                            output.push_str("</em>");
                        } else if tag == "<s>" {
                            output.push_str("</s>");
                        } else if tag == "<code>" {
                            output.push_str("</code>");
                        }
                    }
                }
                "wiki_link" => {
                    let title = child
                        .get("attrs")
                        .and_then(|a| a.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    output.push_str(&format!(
                        "<a href=\"#{}\" class=\"wiki-link\">{}</a>",
                        escape_html(title),
                        escape_html(title)
                    ));
                }
                "hard_break" => {
                    output.push_str("<br>");
                }
                _ => {
                    if let Some(text) = child.get("text").and_then(|t| t.as_str()) {
                        output.push_str(&escape_html(text));
                    }
                }
            }
        }
    }
}

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 720px; margin: 2rem auto; padding: 0 1rem; color: #1a1a1a; line-height: 1.6; }
  h1, h2, h3 { margin-top: 1.5em; }
  blockquote { border-left: 3px solid #ccc; margin-left: 0; padding-left: 1rem; color: #555; }
  code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: 'SF Mono', monospace; }
  pre code { display: block; padding: 1rem; overflow-x: auto; }
  .todo-item { padding: 4px 0; }
  .audio-block { padding: 8px; background: #f0f0f0; border-radius: 6px; margin: 8px 0; }
  .wiki-link { color: #6366f1; text-decoration: none; border-bottom: 1px dotted; }
  img { max-width: 100%; border-radius: 8px; }
  .page-meta { color: #888; font-size: 0.85em; margin-bottom: 1rem; }
  .page-list { list-style: none; padding: 0; }
  .page-list li { padding: 6px 0; }
  .page-list a { color: #6366f1; text-decoration: none; }
  .page-list a:hover { text-decoration: underline; }
</style>
</head>
<body>
<h1>{title}</h1>
<div class="page-meta">Published on {date}</div>
{content}
</body>
</html>"#;

pub fn export_page_html(page: &Page) -> String {
    let content_html = prosemirror_to_html(&page.content);
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    HTML_TEMPLATE
        .replace("{title}", &escape_html(&page.title))
        .replace("{date}", &date)
        .replace("{content}", &content_html)
}

pub fn export_pages_html(db: &Database, page_ids: &[String]) -> Result<String, crate::db::DbError> {
    let mut pages_html = String::new();
    let mut nav_items = String::new();

    for page_id in page_ids {
        let page = db.get_page_by_id(page_id)?;
        let content_html = prosemirror_to_html(&page.content);
        nav_items.push_str(&format!(
            "<li><a href=\"#page-{}\">{}</a></li>\n",
            escape_html(&page.id),
            escape_html(&page.title)
        ));
        pages_html.push_str(&format!(
            "<section id=\"page-{}\">\n<h2>{}</h2>\n{}\n</section>\n",
            escape_html(&page.id),
            escape_html(&page.title),
            content_html
        ));
    }

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Published Pages</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 720px; margin: 2rem auto; padding: 0 1rem; color: #1a1a1a; line-height: 1.6; }}
  h1, h2, h3 {{ margin-top: 1.5em; }}
  blockquote {{ border-left: 3px solid #ccc; margin-left: 0; padding-left: 1rem; color: #555; }}
  code {{ background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: 'SF Mono', monospace; }}
  pre code {{ display: block; padding: 1rem; overflow-x: auto; }}
  .todo-item {{ padding: 4px 0; }}
  .audio-block {{ padding: 8px; background: #f0f0f0; border-radius: 6px; margin: 8px 0; }}
  .wiki-link {{ color: #6366f1; text-decoration: none; border-bottom: 1px dotted; }}
  img {{ max-width: 100%; border-radius: 8px; }}
  .page-meta {{ color: #888; font-size: 0.85em; margin-bottom: 1rem; }}
  nav {{ background: #f8f8f8; padding: 1rem; border-radius: 8px; margin-bottom: 2rem; }}
  nav ul {{ list-style: none; padding: 0; margin: 0; }}
  nav li {{ padding: 4px 0; }}
  nav a {{ color: #6366f1; text-decoration: none; }}
  nav a:hover {{ text-decoration: underline; }}
  section {{ border-top: 1px solid #eee; padding-top: 1rem; margin-top: 2rem; }}
</style>
</head>
<body>
<h1>Published Pages</h1>
<div class="page-meta">Published on {date}</div>
<nav>
<ul>
{nav}
</ul>
</nav>
{pages}
</body>
</html>"#,
        date = date,
        nav = nav_items,
        pages = pages_html
    );

    Ok(full_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_href_allows_http_https_mailto_and_fragments() {
        assert_eq!(
            safe_href("https://example.com/path"),
            "https://example.com/path"
        );
        assert_eq!(safe_href("http://example.com"), "http://example.com");
        assert_eq!(
            safe_href("mailto:user@example.com"),
            "mailto:user@example.com"
        );
        assert_eq!(safe_href("tel:+18005550100"), "tel:+18005550100");
        assert_eq!(safe_href("#section"), "#section");
        assert_eq!(safe_href("/relative/path"), "/relative/path");
        assert_eq!(safe_href(""), "");
    }

    #[test]
    fn safe_href_blocks_dangerous_schemes_regardless_of_case_or_whitespace() {
        assert_eq!(safe_href("javascript:alert(1)"), "");
        assert_eq!(safe_href("JaVaScRiPt:alert(1)"), "");
        assert_eq!(safe_href("  javascript:alert(1)"), "");
        assert_eq!(safe_href("data:text/html,<script>"), "");
        assert_eq!(safe_href("vbscript:msgbox"), "");
    }

    #[test]
    fn link_mark_strips_javascript_href_on_export() {
        let doc = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}"#;
        let html = prosemirror_to_html(doc);
        assert!(!html.contains("javascript:"));
        assert!(html.contains("href=\"\""));
    }
}
