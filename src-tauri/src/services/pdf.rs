use printpdf::*;
use serde_json::Value;

use crate::models::Page;

const REGULAR_FONT: &[u8] = include_bytes!("../../assets/fonts/Roboto-Regular.ttf");
const BOLD_FONT: &[u8] = include_bytes!("../../assets/fonts/Roboto-Bold.ttf");

const PAGE_WIDTH: Mm = Mm(210.0);
const PAGE_HEIGHT: Mm = Mm(297.0);
const MARGIN: Mm = Mm(20.0);

struct PdfRenderer {
    ops: Vec<Op>,
    font_regular: FontId,
    font_bold: FontId,
    y_pos: f32,
}

impl PdfRenderer {
    fn new(font_regular: FontId, font_bold: FontId) -> Self {
        Self {
            ops: Vec::new(),
            font_regular,
            font_bold,
            y_pos: PAGE_HEIGHT.0 - MARGIN.0,
        }
    }

    fn write_line(&mut self, text: &str, bold: bool, size: f32) {
        self.y_pos -= size * 1.4;
        let pos = Point {
            x: Pt(MARGIN.0 * 2.834_645_7),
            y: Pt(self.y_pos * 2.834_645_7),
        };
        let font_handle = PdfFontHandle::External(if bold {
            self.font_bold.clone()
        } else {
            self.font_regular.clone()
        });
        self.ops.push(Op::SetFont {
            font: font_handle,
            size: Pt(size),
        });
        self.ops.push(Op::SetTextCursor { pos });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        });
        self.ops.push(Op::AddLineBreak);
    }

    fn write_title(&mut self, title: &str) {
        self.write_line(title, true, 24.0);
        self.y_pos -= 5.0;
    }

    fn write_heading(&mut self, level: usize, text: &str) {
        self.y_pos -= 5.0;
        let size = match level {
            1 => 20.0,
            2 => 16.0,
            _ => 14.0,
        };
        self.write_line(text, true, size);
        self.y_pos -= 3.0;
    }

    fn write_paragraph(&mut self, text: &str) {
        for line in wrap_text(text, 80) {
            self.write_line(&line, false, 12.0);
        }
    }

    fn write_code_block(&mut self, code: &str) {
        self.y_pos -= 3.0;
        for line in code.lines() {
            self.write_line(line, false, 10.0);
        }
        self.y_pos -= 3.0;
    }

    fn write_divider(&mut self) {
        self.y_pos -= 5.0;
        self.write_line("────────────────────────────────────────", false, 12.0);
        self.y_pos -= 3.0;
    }

    fn write_list_item(&mut self, marker: &str, text: &str) {
        self.write_line(&format!("{} {}", marker, text), false, 12.0);
    }

    fn write_empty(&mut self) {
        self.y_pos -= 12.0 * 1.4;
    }
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current = String::new();
        for word in words {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn extract_text(node: &Value) -> String {
    let mut text = String::new();
    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            let child_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if child_type == "text" {
                if let Some(t) = child.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            } else if child_type == "wiki_link" {
                if let Some(title) = child
                    .get("attrs")
                    .and_then(|a| a.get("title"))
                    .and_then(|t| t.as_str())
                {
                    text.push_str(title);
                }
            } else if child_type == "hard_break" {
                text.push('\n');
            } else if child_type == "math_inline" {
                if let Some(latex) = child
                    .get("attrs")
                    .and_then(|a| a.get("latex"))
                    .and_then(|t| t.as_str())
                {
                    text.push_str(latex);
                }
            }
        }
    }
    text
}

fn render_content(content: &str, font_regular: FontId, font_bold: FontId) -> Vec<Op> {
    let doc: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut r = PdfRenderer::new(font_regular, font_bold);

    if let Some(nodes) = doc.get("content").and_then(|c| c.as_array()) {
        for node in nodes {
            let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match node_type {
                "heading" => {
                    let level = node
                        .get("attrs")
                        .and_then(|a| a.get("level"))
                        .and_then(|l| l.as_u64())
                        .unwrap_or(1) as usize;
                    let text = extract_text(node);
                    r.write_heading(level, &text);
                }
                "paragraph" => {
                    let text = extract_text(node);
                    if text.is_empty() {
                        r.write_empty();
                    } else {
                        r.write_paragraph(&text);
                    }
                }
                "bullet_list" => {
                    if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                        for item in items {
                            let text = extract_text(item);
                            r.write_list_item("•", &text);
                        }
                    }
                }
                "ordered_list" => {
                    if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                        for (i, item) in items.iter().enumerate() {
                            let text = extract_text(item);
                            r.write_list_item(&format!("{}.", i + 1), &text);
                        }
                    }
                }
                "todo_item" => {
                    let checked = node
                        .get("attrs")
                        .and_then(|a| a.get("checked"))
                        .and_then(|c| c.as_bool())
                        .unwrap_or(false);
                    let text = extract_text(node);
                    r.write_list_item(if checked { "[x]" } else { "[ ]" }, &text);
                }
                "code_block" => {
                    let code = extract_text(node);
                    r.write_code_block(&code);
                }
                "blockquote" => {
                    let text = extract_text(node);
                    for line in wrap_text(&text, 76) {
                        r.write_line(&format!("  | {}", line), false, 12.0);
                    }
                }
                "divider" => {
                    r.write_divider();
                }
                "math_block" => {
                    let latex = node
                        .get("attrs")
                        .and_then(|a| a.get("latex"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    r.write_code_block(latex);
                }
                "mermaid_block" => {
                    let code = extract_text(node);
                    r.write_code_block(&format!("[Mermaid Diagram]\n{}", code));
                }
                "image" => {
                    let alt = node
                        .get("attrs")
                        .and_then(|a| a.get("alt"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    r.write_paragraph(&format!("[Image: {}]", alt));
                }
                _ => {
                    let text = extract_text(node);
                    if !text.is_empty() {
                        r.write_paragraph(&text);
                    }
                }
            }
        }
    }

    r.ops
}

pub fn export_page_pdf(page: &Page) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new(&format!("900Notes - {}", page.title));

    let mut font_warnings = Vec::new();
    let regular_font = ParsedFont::from_bytes(REGULAR_FONT, 0, &mut font_warnings)
        .ok_or_else(|| "failed to parse regular font".to_string())?;
    let bold_font = ParsedFont::from_bytes(BOLD_FONT, 0, &mut font_warnings)
        .ok_or_else(|| "failed to parse bold font".to_string())?;

    let font_regular = doc.add_font(&regular_font);
    let font_bold = doc.add_font(&bold_font);

    let mut title_r = PdfRenderer::new(font_regular.clone(), font_bold.clone());
    title_r.write_title(&page.title);

    let content_ops = render_content(&page.content, font_regular, font_bold);
    let mut all_ops = title_r.ops;
    all_ops.extend(content_ops);

    let page = PdfPage::new(PAGE_WIDTH, PAGE_HEIGHT, all_ops);
    let save_options = PdfSaveOptions::default();

    let mut save_warnings = Vec::new();
    let pdf_bytes = doc
        .with_pages(vec![page])
        .save(&save_options, &mut save_warnings);

    Ok(pdf_bytes)
}

pub fn export_pages_pdf(pages: &[Page]) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("900Notes - Workspace Export");

    let mut font_warnings = Vec::new();
    let regular_font = ParsedFont::from_bytes(REGULAR_FONT, 0, &mut font_warnings)
        .ok_or_else(|| "failed to parse regular font".to_string())?;
    let bold_font = ParsedFont::from_bytes(BOLD_FONT, 0, &mut font_warnings)
        .ok_or_else(|| "failed to parse bold font".to_string())?;

    let font_regular = doc.add_font(&regular_font);
    let font_bold = doc.add_font(&bold_font);

    let mut pdf_pages = Vec::new();

    for page in pages {
        if page.deleted_at.is_some() {
            continue;
        }

        let mut title_r = PdfRenderer::new(font_regular.clone(), font_bold.clone());
        title_r.write_title(&page.title);

        let content_ops = render_content(&page.content, font_regular.clone(), font_bold.clone());
        let mut all_ops = title_r.ops;
        all_ops.extend(content_ops);

        pdf_pages.push(PdfPage::new(PAGE_WIDTH, PAGE_HEIGHT, all_ops));
    }

    if pdf_pages.is_empty() {
        pdf_pages.push(PdfPage::new(PAGE_WIDTH, PAGE_HEIGHT, vec![]));
    }

    let save_options = PdfSaveOptions::default();

    let mut save_warnings = Vec::new();
    let pdf_bytes = doc
        .with_pages(pdf_pages)
        .save(&save_options, &mut save_warnings);

    Ok(pdf_bytes)
}
