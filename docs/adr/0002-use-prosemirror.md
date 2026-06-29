# ADR-0002: Use ProseMirror for Rich-Text Editor

- **Status**: accepted
- **Date**: 2026-06-28
- **Deciders**: 900 Labs team
- **Context**: MVP

---

## Context

900Notes needs a block-based WYSIWYG editor with support for headings, lists, to-dos, code blocks, blockquotes, tables, wiki links, and inline formatting. The editor must support markdown shortcuts (type `#` for heading, `-` for bullet list) and a slash command menu. The document model must be serializable to JSON for storage in SQLite.

Constraints:
- Must work offline (no CDN dependencies)
- Must be lightweight (no heavy runtime)
- Must support custom node types (wiki links)
- Must produce JSON output for storage
- Must support markdown shortcuts and slash commands

## Decision

Use **ProseMirror** as the editor framework. ProseMirror is a toolkit for building rich-text editors with a schema-based document model.

## Consequences

### Positive
- Schema-based: we define exactly which nodes and marks are allowed
- JSON document model: `doc.toJSON()` / `Node.fromJSON()` for easy storage
- Modular: plugins for keymaps, input rules, history, drop cursor
- Custom nodes: wiki links are a first-class custom node type
- No runtime dependency: ProseMirror is a library, not a framework
- Well-maintained with extensive documentation
- Used by major products (New York Times, Atlassian, GitLab)

### Negative
- Steep learning curve (ProseMirror's API is complex)
- No built-in UI (we build the toolbar and menus ourselves)
- Schema changes require migration logic for existing documents
- Bundle size: ~100KB gzipped for the core + plugins

### Neutral
- The document model is a tree of nodes, not HTML — this is a different mental model from contenteditable-based editors

## Alternatives Considered

1. **TipTap (built on ProseMirror)** — Rejected to avoid an extra abstraction layer. We need fine-grained control over the schema and plugins, which ProseMirror provides directly.

2. **CodeMirror + Markdown** — Rejected because we want WYSIWYG, not a split-pane markdown editor. The target users (students, researchers in developing economies) benefit from a visual editor.

3. **Quill** — Rejected because its document model (Delta) is less flexible than ProseMirror's schema. Custom node types like wiki links are harder to implement.

4. **Slate** — Rejected because it's React-specific. We use Svelte 5.

5. **Lexical** — Rejected because it's React-specific (Meta's framework). We use Svelte 5.

## References

- [ProseMirror documentation](https://prosemirror.net/docs/)
- [ProseMirror schema guide](https://prosemirror.net/docs/guide/#schema)
- 900Word uses a similar approach for its editor
