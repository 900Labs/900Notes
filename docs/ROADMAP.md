# Roadmap

Status updated: 2026-07-08. Checked items are implemented in the current app/API; unchecked items remain future hardening, certification, or distribution work.
Version 1.6.0 adds exact backups, page-bound autosave, active-workspace restoration, structured wiki links, and public repository documentation.

## MVP (Current)

- [x] Project scaffold (Tauri v2 + Rust + Svelte 5 + TailwindCSS)
- [x] SQLite schema with FTS5, migrations, triggers
- [x] Page CRUD (create, read, update, delete, duplicate, restore)
- [x] Nested page tree with collapsible sidebar
- [x] ProseMirror WYSIWYG editor with block types and inline formatting
- [x] Slash command menu for block insertion
- [x] Bidirectional linking (`[[page name]]` with autocomplete)
- [x] Backlinks panel
- [x] Tags with colors and sidebar filtering
- [x] Full-text search with snippet previews (Ctrl/Cmd+K)
- [x] Export/import (JSON workspace, Markdown export/import)
- [x] i18n (10 languages: English, French, Spanish, Swahili, Hindi, Arabic RTL, Portuguese, Bengali, Urdu RTL, Amharic)
- [x] Settings (theme, language, font size, line spacing)
- [x] Light/dark theme
- [x] Trash with soft-delete and restore

## Post-MVP

### Phase 1: Enhanced Productivity
- [x] Graph view (interactive force-directed graph of page connections)
- [x] Note templates (meeting notes, daily journal, project pages)
- [x] Command palette (extended Ctrl/Cmd+K for actions)
- [x] Outline view (table of contents from headings)
- [x] Page properties/metadata (custom key-value pairs)
- [x] Daily notes with automatic linking

### Phase 2: Organization & Discovery
- [x] Saved searches
- [x] Smart folders (rule-based dynamic collections)
- [x] Page history/versioning
- [x] Favorites/bookmarks
- [x] Related pages suggestions
- [x] Tag groups (hierarchical)

### Phase 3: Content Richness
- [x] Image embedding (stored as blobs in SQLite)
- [x] File attachments
- [x] Math/LaTeX rendering
- [x] Mermaid diagram support
- [x] PDF export
- [x] OCR (offline, Tesseract-based)
- [x] Audio notes

### Phase 4: Sync & Collaboration
- [x] Local network sync (no cloud)
- [x] CRDT-based sync engine for multi-device
- [x] Workspace sharing (export subset as shareable bundle)
- [ ] Read-only published views (static HTML export)
- [x] Multiple local workspaces
- [ ] Team workspaces

### Phase 5: Security & Privacy
- [x] Local database encryption at rest
- [x] Encrypted export bundles
- [x] Threat model documentation
- [x] Privacy model documentation
- [x] Secure delete (overwrite deleted content)
- [ ] Argon2id/PBKDF2 passphrase KDF with versioned metadata

### Phase 6: Accessibility & Platform
- [ ] Screen reader support audit and remediation
- [x] High contrast theme
- [x] Keyboard-only navigation for primary workflows
- [ ] Legacy hardware optimization (4GB RAM profiling)
- [x] Mobile companion (read-only)
- [x] Additional languages (Portuguese, Bengali, Urdu, Amharic)

### Phase 7: Extensibility
- [x] Plugin system (local plugins for custom blocks, themes)
- [x] Custom blocks (user-defined block types)
- [x] Automation API (local IPC for scripting)
- [x] Import from other tools (Evernote, Notion, Obsidian, Roam)
- [x] Web clipper (browser extension)
- [ ] Published/signed browser extension and system share targets

### Phase 8: Distribution
- [x] Linux, macOS, and Windows CI smoke checks
- [x] Versioned changelog and release checklist
- [ ] Signed Windows packages
- [ ] Signed and notarized macOS packages
- [ ] Install verification on supported Linux package formats
