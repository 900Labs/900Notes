# Sprint Plan

**Status**: Complete. The 20 planned post-MVP feature sprints were rechecked against `docs/SPRINT_REVIEW_LOG.md` on 2026-07-05, and every sprint is recorded there as passed.

20 feature sprints across 7 phases. Each sprint is 2 weeks. 2-3 features per sprint. A separate public-readiness sprint now tracks release hygiene after the feature roadmap.

---

## Sprint 1 - Phase 1a: Navigation & Structure
**Goal**: Improve page navigation and add structured metadata.

- [x] Command palette (extend Ctrl/Cmd+K beyond search - create page, toggle theme, switch workspace, insert block)
- [x] Outline view (auto-generated table of contents from H1-H3 headings)
- [x] Page properties/metadata (custom key-value pairs per page, displayed above content)

**Deliverables**: Extended quick switcher, outline sidebar toggle, properties schema in SQLite.

---

## Sprint 2 - Phase 1b: Templates & Daily Notes
**Goal**: Reduce friction for recurring note types.

- [x] Note templates (meeting notes, daily journal, project page, blank - stored as special pages)
- [x] Daily notes (auto-create a page per day with date title, auto-link to previous/next day)

**Deliverables**: Template picker on new page, daily note command, date-based page linking.

---

## Sprint 3 - Phase 1c: Graph View
**Goal**: Visualize the knowledge base structure.

- [x] Interactive force-directed graph (nodes = pages, edges = links, colored by tags)
- [x] Graph filtering (search, connection count, local depth, orphan visibility, and tag/recent color modes; date-range filtering remains deferred)
- [x] Click node to navigate, hover to preview

**Deliverables**: New graph view tab, D3 or custom canvas renderer, graph data query.

---

## Sprint 4 - Phase 2a: Saved Searches & Smart Folders
**Goal**: Let users persist and organize their search workflows.

- [x] Saved searches (store query + filters, pin to sidebar)
- [x] Smart folders (rule-based dynamic collections: tag + date + property filters)

**Deliverables**: Saved searches table in SQLite, smart folder sidebar section, rule builder UI.

---

## Sprint 5 - Phase 2b: History & Favorites
**Goal**: Track changes and pin important pages.

- [x] Page history/versioning (snapshot on each save, diff view, restore previous version)
- [x] Favorites/bookmarks (pin specific pages outside the tree, quick-access bar)

**Deliverables**: Page revisions table, diff renderer, favorites sidebar section.

---

## Sprint 6 - Phase 2c: Discovery & Tag Organization
**Goal**: Help users find related content and organize tags.

- [x] Related pages (suggest pages with shared tags, shared backlinks, or similar content)
- [x] Tag groups (hierarchical tag organization, group headers in sidebar)

**Deliverables**: Related pages algorithm, tag_groups table, tag group UI in sidebar.

---

## Sprint 7 - Phase 3a: Images & Attachments
**Goal**: Support rich media in notes.

- [x] Image embedding (paste/drag images, stored as BLOBs in SQLite, rendered inline)
- [x] File attachments (link to local files, store metadata, open from note)

**Deliverables**: Image block in ProseMirror schema, attachments table, drag-and-drop handlers.

---

## Sprint 8 - Phase 3b: Math & Diagrams
**Goal**: Support technical and academic content.

- [x] Math/LaTeX rendering (inline `$...$` and block `$$...$$` via KaTeX, offline bundled)
- [x] Mermaid diagram support (render from code blocks, offline bundled)

**Deliverables**: Math mark/node in schema, KaTeX integration, Mermaid block type.

---

## Sprint 9 - Phase 3c: PDF Export & OCR
**Goal**: Export and ingest content from physical sources.

- [x] PDF export (export single page or entire workspace to PDF, lightweight renderer)
- [x] OCR (extract text from embedded images using Tesseract, offline)

**Deliverables**: PDF generation service in Rust, Tesseract integration, OCR command.

---

## Sprint 10 - Phase 3d: Audio Notes
**Goal**: Capture spoken knowledge.

- [x] Audio recording (record from microphone, store as BLOB in SQLite)
- [x] Audio playback (inline player in note, basic controls)
- [x] Audio transcription field and export support; offline Whisper transcription remains deferred as intentionally too heavy

**Deliverables**: Audio block in schema, recording commands, audio player component.

---

## Sprint 11 - Phase 4a: Local Network Sync
**Goal**: Sync between devices without cloud.

- [x] Local network discovery (mDNS, find other 900Notes instances on same network)
- [x] Sync protocol (exchange changed pages over local TCP, conflict detection)

**Deliverables**: Sync service in Rust, mDNS discovery, sync settings UI.

---

## Sprint 12 - Phase 4b: CRDT Sync Engine
**Goal**: Conflict-free multi-device sync.

- [x] CRDT-based document model (automerge or similar for ProseMirror docs)
- [x] CRDT document model and manual apply flow; full CRDT-over-TCP automation remains future hardening
- [x] Sync status UI (show last sync, pending changes, device list)

**Deliverables**: CRDT integration, sync queue, sync status panel.

---

## Sprint 13 - Phase 4c: Sharing & Team Workspaces
**Goal**: Share knowledge and support teams.

- [x] Workspace sharing (export subset of pages as a shareable encrypted bundle)
- [x] Read-only published views (export pages as static HTML, viewable in any browser)
- [x] Team workspaces (multiple local workspaces with switching, separate databases)

**Deliverables**: Share bundle format, HTML export, workspace switcher.

---

## Sprint 14 - Phase 5a: Encryption
**Goal**: Protect data at rest.

- [x] Local database encryption at rest with user passphrase
- [x] Encrypted export bundles (password-protected JSON exports)

**Deliverables**: Encryption layer in Rust (AES-256), passphrase setup flow, encrypted export.

---

## Sprint 15 - Phase 5b: Security Documentation & Secure Delete
**Goal**: Formalize security posture and ensure deleted data is gone.

- [x] Threat model documentation (formal threat model like 900Word)
- [x] Privacy model documentation (data flow, what stays local, what doesn't)
- [x] Secure delete (overwrite deleted content on disk, not just flag)

**Deliverables**: docs/THREAT_MODEL.md, docs/PRIVACY_MODEL.md, secure delete implementation.

---

## Sprint 16 - Phase 6a: Accessibility
**Goal**: Make 900Notes usable by everyone.

- [x] Screen reader support (ARIA labels, semantic HTML, live regions for dynamic content)
- [x] High contrast theme (WCAG AA compliant, tested with contrast checkers)
- [x] Keyboard-only navigation (full app usable without mouse, focus indicators, tab order)

**Deliverables**: ARIA audit and fixes, high-contrast theme, keyboard navigation map.

---

## Sprint 17 - Phase 6b: Performance & Mobile
**Goal**: Optimize for low-resource hardware and extend to mobile.

- [x] Legacy hardware optimization (profile and optimize for 4GB RAM, older CPUs, slow disks)
- [x] Mobile companion app (read-only, Tauri mobile or separate lightweight app)

**Deliverables**: Performance benchmarks, optimization changes, mobile app scaffold.

---

## Sprint 18 - Phase 6c: Language Expansion
**Goal**: Reach more users in their native language.

- [x] Portuguese (Brazilian + European)
- [x] Bengali
- [x] Urdu
- [x] Amharic
- [x] Right-to-left audit for any new RTL languages

**Deliverables**: 4 new translation files, RTL layout verification, locale-aware formatting.

---

## Sprint 19 - Phase 7a: Plugin System
**Goal**: Let users extend 900Notes.

- [x] Plugin system architecture (local plugins loaded from a directory, JS/WASM-based)
- [x] Custom blocks (user-defined block types with templates and rendering hooks)

**Deliverables**: Plugin loader, plugin API, custom block registration, example plugins.

---

## Sprint 20 - Phase 7b: Automation & Importers
**Goal**: Connect 900Notes to the rest of the user's workflow.

- [x] Automation API (local IPC API for scripting against the knowledge base)
- [x] Import from other tools (Evernote ENEX, Notion export, Obsidian vault, Roam JSON)
- [x] Web clipper (browser extension that saves web content as notes)

**Deliverables**: API documentation, 4 importers, browser extension, API examples.

---

## Summary

| Phase | Sprints | Features |
|-------|---------|----------|
| 1 - Enhanced Productivity | 3 | 6 |
| 2 - Organization & Discovery | 3 | 6 |
| 3 - Content Richness | 4 | 7 |
| 4 - Sync & Collaboration | 3 | 5 |
| 5 - Security & Privacy | 2 | 5 |
| 6 - Accessibility & Platform | 3 | 6 |
| 7 - Extensibility | 2 | 5 |
| **Total** | **20** | **40** |

At 2 weeks per sprint, the full post-MVP roadmap is approximately **40 weeks** (10 months) of development effort.

---

## Public Readiness Sprint - Release Hygiene
**Status**: Complete as of 2026-07-05.

**Goal**: Make the repository safe and coherent to open publicly after the 20 feature sprints.

- [x] Reconcile sprint documentation with `docs/SPRINT_REVIEW_LOG.md`
- [x] Remove tracked generated mobile build output
- [x] Remove tracked fake font assets that were HTML pages, not fonts
- [x] Add a tracked-file public release gate for local paths, secrets, generated artifacts, and masquerading binary assets
- [x] Add automated i18n key coverage across all 10 locales
- [x] Backfill recent web capture, smart view, and graph inspector translation keys
- [x] Update stale PDF/font documentation to match the current built-in PDF writer

**Deliverables**: Clean public release gate, complete locale coverage, reconciled sprint docs, and no tracked generated/fake binary artifacts.
