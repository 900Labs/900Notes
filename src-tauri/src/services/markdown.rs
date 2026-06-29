use serde_json::Value;

pub fn prosemirror_to_markdown(content: &str) -> String {
    let doc: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut output = String::new();
    if let Some(nodes) = doc.get("content").and_then(|c| c.as_array()) {
        for node in nodes {
            render_node(node, &mut output, 0);
        }
    }
    output.trim_end().to_string()
}

fn render_node(node: &Value, output: &mut String, _depth: usize) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match node_type {
        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(1);
            output.push_str(&"#".repeat(level as usize));
            output.push(' ');
            render_inline(node, output);
            output.push('\n');
        }
        "paragraph" => {
            render_inline(node, output);
            output.push('\n');
        }
        "bullet_list" => {
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for item in items {
                    output.push_str("- ");
                    if let Some(item_content) = item.get("content").and_then(|c| c.as_array()) {
                        for child in item_content {
                            render_node(child, output, _depth + 1);
                        }
                    }
                    output.push('\n');
                }
            }
        }
        "ordered_list" => {
            if let Some(items) = node.get("content").and_then(|c| c.as_array()) {
                for (i, item) in items.iter().enumerate() {
                    output.push_str(&format!("{}. ", i + 1));
                    if let Some(item_content) = item.get("content").and_then(|c| c.as_array()) {
                        for child in item_content {
                            render_node(child, output, _depth + 1);
                        }
                    }
                    output.push('\n');
                }
            }
        }
        "todo_item" => {
            let checked = node
                .get("attrs")
                .and_then(|a| a.get("checked"))
                .and_then(|c| c.as_bool())
                .unwrap_or(false);
            output.push_str(if checked { "- [x] " } else { "- [ ] " });
            render_inline(node, output);
            output.push('\n');
        }
        "code_block" => {
            output.push_str("```\n");
            if let Some(text) = node
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| {
                    arr.first()
                        .and_then(|n| n.get("text"))
                        .and_then(|t| t.as_str())
                })
            {
                output.push_str(text);
            }
            output.push_str("\n```\n");
        }
        "blockquote" => {
            output.push_str("> ");
            if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                for child in content {
                    render_node(child, output, _depth + 1);
                }
            }
        }
        "divider" => {
            output.push_str("---\n");
        }
        "audio_block" => {
            let title = node
                .get("attrs")
                .and_then(|a| a.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("Audio Note");
            let duration = node
                .get("attrs")
                .and_then(|a| a.get("duration"))
                .and_then(|d| d.as_f64())
                .unwrap_or(0.0);
            let transcription = node
                .get("attrs")
                .and_then(|a| a.get("transcription"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            output.push_str(&format!("🎙 **{}** ({:.0}s)\n", title, duration));
            if !transcription.is_empty() {
                output.push_str(&format!("> {}\n", transcription));
            }
        }
        _ => {
            render_inline(node, output);
        }
    }
}

fn render_inline(node: &Value, output: &mut String) {
    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        for child in content {
            let child_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match child_type {
                "text" => {
                    let text = child.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let marks = child.get("marks").and_then(|m| m.as_array());
                    let mut prefix = String::new();
                    let mut suffix = String::new();
                    if let Some(marks) = marks {
                        for mark in marks {
                            let mark_type = mark.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            match mark_type {
                                "bold" => {
                                    prefix.push_str("**");
                                    suffix.insert_str(0, "**");
                                }
                                "italic" => {
                                    prefix.push('*');
                                    suffix.insert(0, '*');
                                }
                                "strike" => {
                                    prefix.push_str("~~");
                                    suffix.insert_str(0, "~~");
                                }
                                "code" => {
                                    prefix.push('`');
                                    suffix.insert(0, '`');
                                }
                                "link" => {
                                    let href = mark
                                        .get("attrs")
                                        .and_then(|a| a.get("href"))
                                        .and_then(|h| h.as_str())
                                        .unwrap_or("");
                                    prefix.push('[');
                                    suffix.insert_str(0, &format!("]({})", href));
                                }
                                _ => {}
                            }
                        }
                    }
                    output.push_str(&prefix);
                    output.push_str(text);
                    output.push_str(&suffix);
                }
                "wiki_link" => {
                    let title = child
                        .get("attrs")
                        .and_then(|a| a.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    output.push_str(&format!("[[{}]]", title));
                }
                "hard_break" => {
                    output.push_str("  \n");
                }
                _ => {
                    if let Some(text) = child.get("text").and_then(|t| t.as_str()) {
                        output.push_str(text);
                    }
                }
            }
        }
    }
}

pub fn markdown_to_prosemirror(md: &str, _title: &str) -> String {
    let mut content_nodes: Vec<Value> = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            content_nodes.push(serde_json::json!({
                "type": "heading",
                "attrs": {"level": 3},
                "content": [{"type": "text", "text": rest}]
            }));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            content_nodes.push(serde_json::json!({
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": rest}]
            }));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            content_nodes.push(serde_json::json!({
                "type": "heading",
                "attrs": {"level": 1},
                "content": [{"type": "text", "text": rest}]
            }));
        } else if trimmed == "---" {
            content_nodes.push(serde_json::json!({"type": "divider"}));
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            content_nodes.push(serde_json::json!({
                "type": "todo_item",
                "attrs": {"checked": true},
                "content": [{"type": "text", "text": rest}]
            }));
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            content_nodes.push(serde_json::json!({
                "type": "todo_item",
                "attrs": {"checked": false},
                "content": [{"type": "text", "text": rest}]
            }));
        } else if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            content_nodes.push(serde_json::json!({
                "type": "bullet_list",
                "content": [{
                    "type": "list_item",
                    "content": [{"type": "paragraph", "content": [{"type": "text", "text": rest}]}]
                }]
            }));
        } else if let Some(rest) = trimmed.strip_prefix("> ") {
            content_nodes.push(serde_json::json!({
                "type": "blockquote",
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": rest}]}]
            }));
        } else {
            content_nodes.push(serde_json::json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": trimmed}]
            }));
        }
    }

    if content_nodes.is_empty() {
        content_nodes.push(serde_json::json!({"type": "paragraph"}));
    }

    serde_json::json!({
        "type": "doc",
        "content": content_nodes
    })
    .to_string()
}
