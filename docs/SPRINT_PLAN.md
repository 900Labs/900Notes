# Sprint Plan

20 sprints across 7 phases. Each sprint is 2 weeks. 2-3 features per sprint.

---

## Sprint 1 — Phase 1a: Navigation & Structure
**Goal**: Improve page navigation and add structured metadata.

- [ ] Command palette (extend Ctrl/Cmd+K beyond search — create page, toggle theme, switch workspace, insert block)
- [ ] Outline view (auto-generated table of contents from H1-H3 headings)
- [ ] Page properties/metadata (custom key-value pairs per page, displayed above content)

**Deliverables**: Extended quick switcher, outline sidebar toggle, properties schema in SQLite.

---

## Sprint 2 — Phase 1b: Templates & Daily Notes
**Goal**: Reduce friction for recurring note types.

- [ ] Note templates (meeting notes, daily journal, project page, blank — stored as special pages)
- [ ] Daily notes (auto-create a page per day with date title, auto-link to previous/next day)

**Deliverables**: Template picker on new page, daily note command, date-based page linking.

---

## Sprint 3 — Phase 1c: Graph View
**Goal**: Visualize the knowledge base structure.

- [ ] Interactive force-directed graph (nodes = pages, edges = links, colored by tags)
- [ ] Graph filtering (by tag, by date range, by connection count)
- [ ] Click node to navigate, hover to preview

**Deliverables**: New graph view tab, D3 or custom canvas renderer, graph data query.

---

## Sprint 4 — Phase 2a: Saved Searches & Smart Folders
**Goal**: Let users persist and organize their search workflows.

- [ ] Saved searches (store query + filters, pin to sidebar)
- [ ] Smart folders (rule-based dynamic collections: tag + date + property filters)

**Deliverables**: Saved searches table in SQLite, smart folder sidebar section, rule builder UI.

---

## Sprint 5 — Phase 2b: History & Favorites
**Goal**: Track changes and pin important pages.

- [ ] Page history/versioning (snapshot on each save, diff view, restore previous version)
- [ ] Favorites/bookmarks (pin specific pages outside the tree, quick-access bar)

**Deliverables**: Page revisions table, diff renderer, favorites sidebar section.

---

## Sprint 6 — Phase 2c: Discovery & Tag Organization
**Goal**: Help users find related content and organize tags.

- [ ] Related pages (suggest pages with shared tags, shared backlinks, or similar content)
- [ ] Tag groups (hierarchical tag organization, group headers in sidebar)

**Deliverables**: Related pages algorithm, tag_groups table, tag group UI in sidebar.

---

## Sprint 7 — Phase 3a: Images & Attachments
**Goal**: Support rich media in notes.

- [ ] Image embedding (paste/drag images, stored as BLOBs in SQLite, rendered inline)
- [ ] File attachments (link to local files, store metadata, open from note)

**Deliverables**: Image block in ProseMirror schema, attachments table, drag-and-drop handlers.

---

## Sprint 8 — Phase 3b: Math & Diagrams
**Goal**: Support technical and academic content.

- [ ] Math/LaTeX rendering (inline `$...$` and block `$$...$$` via KaTeX, offline bundled)
- [ ] Mermaid diagram support (render from code blocks, offline bundled)

**Deliverables**: Math mark/node in schema, KaTeX integration, Mermaid block type.

---

## Sprint 9 — Phase 3c: PDF Export & OCR
**Goal**: Export and ingest content from physical sources.

- [ ] PDF export (export single page or entire workspace to PDF, lightweight renderer)
- [ ] OCR (extract text from embedded images using Tesseract, offline)

**Deliverables**: PDF generation service in Rust, Tesseract integration, OCR command.

---

## Sprint 10 — Phase 3d: Audio Notes
**Goal**: Capture spoken knowledge.

- [ ] Audio recording (record from microphone, store as BLOB in SQLite)
- [ ] Audio playback (inline player in note, basic controls)
- [ ] Basic transcription (optional, offline Whisper-based, deferred if too heavy)

**Deliverables**: Audio block in schema, recording commands, audio player component.

---

## Sprint 11 — Phase 4a: Local Network Sync
**Goal**: Sync between devices without cloud.

- [ ] Local network discovery (mDNS, find other 900Notes instances on same network)
- [ ] Sync protocol (exchange changed pages over local TCP, conflict detection)

**Deliverables**: Sync service in Rust, mDNS discovery, sync settings UI.

---

## Sprint 12 — Phase 4b: CRDT Sync Engine
**Goal**: Conflict-free multi-device sync.

- [ ] CRDT-based document model (automerge or similar for ProseMirror docs)
- [ ] Multi-device sync (merge changes from multiple devices without conflicts)
- [ ] Sync status UI (show last sync, pending changes, device list)

**Deliverables**: CRDT integration, sync queue, sync status panel.

---

## Sprint 13 — Phase 4c: Sharing & Team Workspaces
**Goal**: Share knowledge and support teams.

- [ ] Workspace sharing (export subset of pages as a shareable encrypted bundle)
- [ ] Read-only published views (export pages as static HTML, viewable in any browser)
- [ ] Team workspaces (multiple local workspaces with switching, separate databases)

**Deliverables**: Share bundle format, HTML export, workspace switcher.

---

## Sprint 14 — Phase 5a: Encryption
**Goal**: Protect data at rest.

- [x] End-to-end encryption (encrypt SQLite database at rest with user passphrase)
- [x] Encrypted export bundles (password-protected JSON exports)

**Deliverables**: Encryption layer in Rust (AES-256), passphrase setup flow, encrypted export.

---

## Sprint 15 — Phase 5b: Security Documentation & Secure Delete
**Goal**: Formalize security posture and ensure deleted data is gone.

- [x] Threat model documentation (formal threat model like 900Word)
- [x] Privacy model documentation (data flow, what stays local, what doesn't)
- [x] Secure delete (overwrite deleted content on disk, not just flag)

**Deliverables**: docs/THREAT_MODEL.md, docs/PRIVACY_MODEL.md, secure delete implementation.

---

## Sprint 16 — Phase 6a: Accessibility
**Goal**: Make 900Notes usable by everyone.

- [x] Screen reader support (ARIA labels, semantic HTML, live regions for dynamic content)
- [x] High contrast theme (WCAG AA compliant, tested with contrast checkers)
- [x] Keyboard-only navigation (full app usable without mouse, focus indicators, tab order)

**Deliverables**: ARIA audit and fixes, high-contrast theme, keyboard navigation map.

---

## Sprint 17 — Phase 6b: Performance & Mobile
**Goal**: Optimize for low-resource hardware and extend to mobile.

- [x] Legacy hardware optimization (profile and optimize for 4GB RAM, older CPUs, slow disks)
- [x] Mobile companion app (read-only, Tauri mobile or separate lightweight app)

**Deliverables**: Performance benchmarks, optimization changes, mobile app scaffold.

---

## Sprint 18 — Phase 6c: Language Expansion
**Goal**: Reach more users in their native language.

- [x] Portuguese (Brazilian + European)
- [x] Bengali
- [x] Urdu
- [x] Amharic
- [x] Right-to-left audit for any new RTL languages

**Deliverables**: 4 new translation files, RTL layout verification, locale-aware formatting.

---

## Sprint 19 — Phase 7a: Plugin System
**Goal**: Let users extend 900Notes.

- [x] Plugin system architecture (local plugins loaded from a directory, JS/WASM-based)
- [x] Custom blocks (user-defined block types with templates and rendering hooks)

**Deliverables**: Plugin loader, plugin API, custom block registration, example plugins.

---

## Sprint 20 — Phase 7b: Automation & Importers
**Goal**: Connect 900Notes to the rest of the user's workflow.

- [x] Automation API (local IPC API for scripting against the knowledge base)
- [x] Import from other tools (Evernote ENEX, Notion export, Obsidian vault, Roam JSON)
- [x] Web clipper (browser extension that saves web content as notes)

**Deliverables**: API documentation, 4 importers, browser extension, API examples.

---

## Summary

| Phase | Sprints | Features |
|-------|---------|----------|
| 1 — Enhanced Productivity | 3 | 6 |
| 2 — Organization & Discovery | 3 | 6 |
| 3 — Content Richness | 4 | 7 |
| 4 — Sync & Collaboration | 3 | 5 |
| 5 — Security & Privacy | 2 | 5 |
| 6 — Accessibility & Platform | 3 | 6 |
| 7 — Extensibility | 2 | 5 |
| **Total** | **20** | **40** |

At 2 weeks per sprint, the full post-MVP roadmap is approximately **40 weeks** (10 months) of development effort.
