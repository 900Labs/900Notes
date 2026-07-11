# ADR-0004: Store Editor Content as ProseMirror JSON

- **Status**: accepted
- **Date**: 2026-06-28
- **Deciders**: 900 Labs team
- **Context**: MVP

---

## Context

The editor produces a document that must be stored, searched, and converted to/from Markdown. We need to choose a storage format for page content.

Constraints:
- Must be searchable (full-text search needs plain text)
- Must be convertible to Markdown for export
- Must preserve all formatting, block types, and wiki links
- Must be storable in SQLite (TEXT column)
- Must survive schema evolution (adding new block types)

## Decision

Store editor content as **ProseMirror JSON** (the output of `doc.toJSON()`). The JSON is stored as a TEXT column in the `pages` table. For full-text search, a plain-text extraction is stored in the FTS5 index via triggers.

## Consequences

### Positive
- Lossless: all formatting, marks, and custom nodes (wiki links) are preserved
- Schema-aware: the JSON structure matches the ProseMirror schema, making round-trip editing trivial
- Extensible: adding new block types doesn't break existing documents (unknown nodes can be handled gracefully)
- Debuggable: the JSON is human-readable
- Link extraction: wiki links can be extracted by scanning the raw JSON string for `[[` and `]]` patterns

### Negative
- Larger than Markdown (JSON overhead - ~2x the size of equivalent Markdown)
- Not directly human-editable outside the app (Markdown export is provided for this)
- FTS5 index requires a separate plain-text extraction step (handled by triggers)

### Neutral
- The content column stores the full JSON string - SQLite handles TEXT columns efficiently up to several MB

## Alternatives Considered

1. **Markdown** - Rejected as the primary format because Markdown cannot represent all ProseMirror features (wiki links, todo items with checked state, custom attributes). Would require a lossy conversion on every save/load cycle.

2. **HTML** - Rejected because HTML is messier to parse and doesn't map cleanly to ProseMirror's schema. Also harder to extract wiki links from HTML.

3. **Custom binary format** - Rejected for complexity and lack of debuggability.

## References

- [ProseMirror JSON serialization](https://prosemirror.net/docs/ref/#model.Node.toJSON)
- `src-tauri/src/services/markdown.rs` - Markdown conversion implementation
