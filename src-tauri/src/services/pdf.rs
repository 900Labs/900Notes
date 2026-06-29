use serde_json::Value;

use crate::models::Page;

const PAGE_WIDTH_PT: f32 = 595.28;
const PAGE_HEIGHT_PT: f32 = 841.89;
const MARGIN_PT: f32 = 56.0;

#[derive(Clone, Copy)]
enum PdfTextStyle {
    Title,
    Heading(u64),
    Body,
    Code,
    Quote,
}

#[derive(Clone)]
struct PdfLine {
    text: String,
    style: PdfTextStyle,
}

impl PdfLine {
    fn new(text: impl Into<String>, style: PdfTextStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

fn font_size(style: PdfTextStyle) -> f32 {
    match style {
        PdfTextStyle::Title => 22.0,
        PdfTextStyle::Heading(1) => 18.0,
        PdfTextStyle::Heading(2) => 15.0,
        PdfTextStyle::Heading(_) => 13.0,
        PdfTextStyle::Body | PdfTextStyle::Quote => 11.5,
        PdfTextStyle::Code => 10.0,
    }
}

fn line_height(style: PdfTextStyle) -> f32 {
    font_size(style) * 1.45
}

fn font_name(style: PdfTextStyle) -> &'static str {
    match style {
        PdfTextStyle::Title | PdfTextStyle::Heading(_) => "F2",
        PdfTextStyle::Body | PdfTextStyle::Code | PdfTextStyle::Quote => "F1",
    }
}

fn x_pos(style: PdfTextStyle) -> f32 {
    match style {
        PdfTextStyle::Quote => MARGIN_PT + 14.0,
        PdfTextStyle::Code => MARGIN_PT + 10.0,
        _ => MARGIN_PT,
    }
}

fn max_chars(style: PdfTextStyle) -> usize {
    match style {
        PdfTextStyle::Title => 48,
        PdfTextStyle::Heading(1) => 58,
        PdfTextStyle::Heading(_) => 68,
        PdfTextStyle::Code => 88,
        PdfTextStyle::Quote => 76,
        PdfTextStyle::Body => 82,
    }
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.chars().count() + 1 + word.chars().count() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }

            while current.chars().count() > max_chars {
                let split_at = current
                    .char_indices()
                    .nth(max_chars)
                    .map(|(idx, _)| idx)
                    .unwrap_or(current.len());
                let rest = current.split_off(split_at);
                lines.push(current);
                current = rest;
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

fn push_wrapped(lines: &mut Vec<PdfLine>, text: &str, style: PdfTextStyle) {
    for line in wrap_text(text, max_chars(style)) {
        lines.push(PdfLine::new(line, style));
    }
}

fn extract_text(node: &Value) -> String {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "text" => node
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        "wiki_link" => node
            .get("attrs")
            .and_then(|a| a.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        "hard_break" => "\n".to_string(),
        "math_inline" => node
            .get("attrs")
            .and_then(|a| a.get("latex"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        _ => {
            let mut text = String::new();
            if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                for child in content {
                    text.push_str(&extract_text(child));
                }
            }
            text
        }
    }
}

fn push_node_lines(node: &Value, lines: &mut Vec<PdfLine>) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(1);
            push_wrapped(lines, &extract_text(node), PdfTextStyle::Heading(level));
        }
        "paragraph" => {
            let text = extract_text(node);
            push_wrapped(lines, &text, PdfTextStyle::Body);
        }
        "bullet_list" => {
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for item in items {
                    push_wrapped(
                        lines,
                        &format!("- {}", extract_text(item)),
                        PdfTextStyle::Body,
                    );
                }
            }
        }
        "ordered_list" => {
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for (index, item) in items.iter().enumerate() {
                    push_wrapped(
                        lines,
                        &format!("{}. {}", index + 1, extract_text(item)),
                        PdfTextStyle::Body,
                    );
                }
            }
        }
        "todo_item" => {
            let checked = node
                .get("attrs")
                .and_then(|a| a.get("checked"))
                .and_then(|c| c.as_bool())
                .unwrap_or(false);
            let marker = if checked { "[x]" } else { "[ ]" };
            push_wrapped(
                lines,
                &format!("{marker} {}", extract_text(node)),
                PdfTextStyle::Body,
            );
        }
        "code_block" => {
            for line in extract_text(node).lines() {
                push_wrapped(lines, line, PdfTextStyle::Code);
            }
        }
        "blockquote" => {
            push_wrapped(lines, &extract_text(node), PdfTextStyle::Quote);
        }
        "divider" => {
            lines.push(PdfLine::new(
                "----------------------------------------",
                PdfTextStyle::Body,
            ));
        }
        "math_block" => {
            let latex = node
                .get("attrs")
                .and_then(|a| a.get("latex"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            push_wrapped(lines, latex, PdfTextStyle::Code);
        }
        "mermaid_block" => {
            push_wrapped(
                lines,
                &format!("[Mermaid diagram]\n{}", extract_text(node)),
                PdfTextStyle::Code,
            );
        }
        "image" => {
            let alt = node
                .get("attrs")
                .and_then(|a| a.get("alt"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            push_wrapped(lines, &format!("[Image: {alt}]"), PdfTextStyle::Body);
        }
        _ => {
            let text = extract_text(node);
            if !text.is_empty() {
                push_wrapped(lines, &text, PdfTextStyle::Body);
            }
        }
    }
}

fn render_page_lines(page: &Page) -> Vec<PdfLine> {
    let mut lines = Vec::new();
    push_wrapped(&mut lines, &page.title, PdfTextStyle::Title);
    lines.push(PdfLine::new(String::new(), PdfTextStyle::Body));

    let doc: Value = match serde_json::from_str(&page.content) {
        Ok(value) => value,
        Err(_) => {
            push_wrapped(&mut lines, &page.content, PdfTextStyle::Body);
            return lines;
        }
    };

    if let Some(nodes) = doc.get("content").and_then(|c| c.as_array()) {
        for node in nodes {
            push_node_lines(node, &mut lines);
        }
    }

    lines
}

fn paginate_section(lines: &[PdfLine]) -> Vec<Vec<PdfLine>> {
    let mut pages = Vec::new();
    let mut current = Vec::new();
    let mut y = PAGE_HEIGHT_PT - MARGIN_PT;

    for line in lines {
        let needed = line_height(line.style);
        if y - needed < MARGIN_PT && !current.is_empty() {
            pages.push(current);
            current = Vec::new();
            y = PAGE_HEIGHT_PT - MARGIN_PT;
        }
        y -= needed;
        current.push(line.clone());
    }

    if !current.is_empty() {
        pages.push(current);
    }

    pages
}

fn pdf_text_hex(text: &str) -> String {
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        out.push_str(&format!("{unit:04X}"));
    }
    out.push('>');
    out
}

fn content_stream(lines: &[PdfLine]) -> Vec<u8> {
    let mut stream = String::from("BT\n");
    let mut y = PAGE_HEIGHT_PT - MARGIN_PT;

    for line in lines {
        let size = font_size(line.style);
        y -= line_height(line.style);
        stream.push_str(&format!(
            "/{font} {size:.2} Tf\n",
            font = font_name(line.style)
        ));
        stream.push_str(&format!(
            "1 0 0 1 {x:.2} {y:.2} Tm\n",
            x = x_pos(line.style)
        ));
        stream.push_str(&format!("{} Tj\n", pdf_text_hex(&line.text)));
    }

    stream.push_str("ET\n");
    stream.into_bytes()
}

fn build_pdf(sections: Vec<Vec<PdfLine>>) -> Vec<u8> {
    let mut page_lines = Vec::new();
    for section in sections {
        page_lines.extend(paginate_section(&section));
    }
    if page_lines.is_empty() {
        page_lines.push(Vec::new());
    }

    let page_count = page_lines.len();
    let page_object_ids: Vec<usize> = (0..page_count).map(|i| 5 + i * 2).collect();
    let content_object_ids: Vec<usize> = (0..page_count).map(|i| 6 + i * 2).collect();
    let object_count = 4 + page_count * 2;

    let mut objects: Vec<(usize, Vec<u8>)> = Vec::new();
    objects.push((1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()));
    objects.push((
        2,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            page_object_ids
                .iter()
                .map(|id| format!("{id} 0 R"))
                .collect::<Vec<_>>()
                .join(" "),
            page_count
        )
        .into_bytes(),
    ));
    objects.push((
        3,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ));
    objects.push((
        4,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_vec(),
    ));

    for (index, lines) in page_lines.iter().enumerate() {
        let page_object_id = page_object_ids[index];
        let content_object_id = content_object_ids[index];
        objects.push((
            page_object_id,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width:.2} {height:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {content_object_id} 0 R >>",
                width = PAGE_WIDTH_PT,
                height = PAGE_HEIGHT_PT,
            )
            .into_bytes(),
        ));

        let stream = content_stream(lines);
        let mut object = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
        object.extend_from_slice(&stream);
        object.extend_from_slice(b"endstream");
        objects.push((content_object_id, object));
    }

    objects.sort_by_key(|(id, _)| *id);

    let mut output = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize; object_count + 1];
    for (id, body) in objects {
        offsets[id] = output.len();
        output.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        output.extend_from_slice(&body);
        output.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = output.len();
    output.extend_from_slice(format!("xref\n0 {}\n", object_count + 1).as_bytes());
    output.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    output.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            object_count + 1,
            xref_offset
        )
        .as_bytes(),
    );

    output
}

pub fn export_page_pdf(page: &Page) -> Result<Vec<u8>, String> {
    Ok(build_pdf(vec![render_page_lines(page)]))
}

pub fn export_pages_pdf(pages: &[Page]) -> Result<Vec<u8>, String> {
    let sections: Vec<Vec<PdfLine>> = pages
        .iter()
        .filter(|page| page.deleted_at.is_none())
        .map(render_page_lines)
        .collect();
    Ok(build_pdf(sections))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_page(title: &str, content: &str) -> Page {
        Page {
            id: "page-1".to_string(),
            parent_id: None,
            title: title.to_string(),
            content: content.to_string(),
            icon: None,
            cover_color: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
            pinned: false,
            sort_order: 0,
        }
    }

    #[test]
    fn exports_valid_pdf_header_and_catalog() {
        let page = test_page(
            "PDF Test",
            r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello PDF"}]}]}"#,
        );

        let pdf = export_page_pdf(&page).unwrap();

        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("/Type /Catalog"));
        assert!(String::from_utf8_lossy(&pdf).contains("/Helvetica"));
    }
}
