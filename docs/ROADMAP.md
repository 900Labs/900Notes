# Roadmap

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
- [x] i18n (6 languages: English, French, Spanish, Swahili, Hindi, Arabic RTL)
- [x] Settings (theme, language, font size, line spacing)
- [x] Light/dark theme
- [x] Trash with soft-delete and restore

## Post-MVP

### Phase 1: Enhanced Productivity
- Graph view (interactive force-directed graph of page connections)
- Note templates (meeting notes, daily journal, project pages)
- Command palette (extended Ctrl/Cmd+K for actions)
- Outline view (table of contents from headings)
- Page properties/metadata (custom key-value pairs)
- Daily notes with automatic linking

### Phase 2: Organization & Discovery
- Saved searches
- Smart folders (rule-based dynamic collections)
- Page history/versioning with diff view
- Favorites/bookmarks
- Related pages suggestions
- Tag groups (hierarchical)

### Phase 3: Content Richness
- Image embedding (stored as blobs in SQLite)
- File attachments
- Math/LaTeX rendering
- Mermaid diagram support
- PDF export
- OCR (offline, Tesseract-based)
- Audio notes

### Phase 4: Sync & Collaboration
- Local network sync (no cloud)
- CRDT-based sync engine for multi-device
- Workspace sharing (export subset as shareable bundle)
- Read-only published views (static HTML export)
- Team workspaces (multiple local workspaces)

### Phase 5: Security & Privacy
- End-to-end encryption (encrypt SQLite at rest)
- Encrypted export bundles
- Threat model documentation
- Privacy model documentation
- Secure delete (overwrite deleted content)

### Phase 6: Accessibility & Platform
- Screen reader support (ARIA, semantic HTML)
- High contrast theme (WCAG AA)
- Keyboard-only navigation
- Legacy hardware optimization (4GB RAM profiling)
- Mobile companion (read-only)
- Additional languages (Portuguese, Bengali, Urdu, Amharic)

### Phase 7: Extensibility
- Plugin system (local plugins for custom blocks, themes)
- Custom blocks (user-defined block types)
- Automation API (local IPC for scripting)
- Import from other tools (Evernote, Notion, Obsidian, Roam)
- Web clipper (browser extension)
