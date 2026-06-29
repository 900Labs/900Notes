# Architecture Overview

## System Design

900Notes is a desktop application built on Tauri v2, using a Rust backend with a Svelte 5 frontend. All data is stored locally in a single SQLite database file.

```
┌─────────────────────────────────────────┐
│              Tauri v2 Shell              │
│  ┌───────────────┐  ┌─────────────────┐ │
│  │   Svelte 5    │  │     Rust        │ │
│  │   Frontend    │←→│    Backend      │ │
│  │               │  │                 │ │
│  │  ProseMirror  │  │  SQLite + FTS5  │ │
│  │  TailwindCSS  │  │  Link Engine    │ │
│  │ i18n (10 lang)│  │  Export/Import  │ │
│  └───────────────┘  └─────────────────┘ │
│         IPC via Tauri Commands           │
└─────────────────────────────────────────┘
```

## Data Flow

1. **User interaction** → Svelte component → Tauri IPC command
2. **Tauri command** → Rust function → SQLite query
3. **SQLite response** → Rust model → serde JSON → Svelte store
4. **Svelte store** → Reactive UI update

## Data Model

### SQLite Schema

- **pages**: id (UUID), parent_id (self-referential FK), title, content (JSON ProseMirror doc), icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
- **tags**: id (UUID), name (unique), color, created_at
- **page_tags**: junction table (page_id, tag_id)
- **links**: id (UUID), source_page_id, target_page_id, link_text, created_at
- **pages_fts**: FTS5 virtual table over page titles and content
- **settings**: key-value store for app preferences
- **attachments**: BLOB storage for images, PDFs, and audio with a 25 MB per-file cap
- **sync_state / sync_queue**: CRDT state and pending sync operations
- **plugins**: Local plugin manifests discovered under the app data plugin root

### Triggers

- `pages_fts_insert/update/delete`: Keep the FTS5 index in sync with the pages table automatically.
- Startup migration code repairs older contentless FTS tables and repopulates the index so snippets can be generated reliably.

## Link Engine

When a page's content is updated, the Rust backend:

1. Deletes all existing outgoing links from that page
2. Scans the content JSON for `[[wiki link]]` syntax
3. Matches link text against page titles (case-insensitive)
4. Inserts new link records into the `links` table

Backlinks are queried by selecting from `links` where `target_page_id` matches the current page.

## Editor Architecture

Following the 900Word pattern:

1. **ProseMirror** is the editing surface in the frontend
2. The document model is a JSON object stored as text in SQLite
3. **Rust** owns the persistence and link extraction — content is sanitized before storage
4. The schema supports: paragraphs, headings (H1-H3), lists, to-dos, code blocks, blockquotes, dividers, tables, wiki links, and inline marks (bold, italic, underline, strike, code, link)

## Offline Model

- No network calls are made by default
- No telemetry is collected
- All data stays in `{APP_DATA_DIR}/900notes.db`
- Export/import and explicitly enabled LAN sync are the only ways data leaves the machine
- The app works fully offline after the first download

## i18n Architecture

- Translation files are in `src/i18n/index.ts`
- 10 languages: English, French, Spanish, Swahili, Hindi, Arabic, Portuguese, Bengali, Urdu, Amharic
- Arabic and Urdu use RTL layout (document direction set dynamically)
- The `t` store is derived from the `locale` store, providing reactive translations

## Encryption

- **AES-256-GCM** encryption for the database at rest
- Key derivation: iterative SHA-256 with 100,000 iterations and per-encryption salt
- Salt and nonce generated with CSPRNG (`getrandom`)
- On startup with encryption enabled, an in-memory placeholder DB is used until the user unlocks with their passphrase
- On app shutdown (`ExitRequested`), the plaintext DB is re-encrypted and deleted
- The passphrase is stored in `AppState` as `Mutex<Option<String>>` for the duration of the session
- Encrypted share bundles and workspace export/import also use AES-256-GCM

## Sync

- **mDNS** discovery for peers on the local network
- **TCP** server for peer-to-peer sync
- Sync is opt-in and only activates when the user explicitly starts it
- Starting sync requires a 12+ character pairing secret
- Sync handshakes are encrypted with AES-256-GCM using the pairing secret
- Sync handshake messages are capped at 100MB to prevent OOM DoS
- **Residual risk**: mDNS still advertises that 900Notes sync is running on the LAN. Pairing-secret strength is user controlled.

## Content Security Policy

- Desktop CSP is configured in `tauri.conf.json`:
  - `default-src 'self'`
  - `script-src 'self' 'unsafe-eval' 'unsafe-inline'` (needed for plugin loading via `new Function()` and Svelte's injected styles)
  - `style-src 'self' 'unsafe-inline'`
  - `img-src 'self' data: blob:`
  - `font-src 'self' data:`
  - `connect-src 'self' ipc: http://ipc.localhost`
- Mobile CSP is configured in `tauri.mobile.conf.json` without `unsafe-eval` because the mobile companion does not load plugins.

## Database Integrity

- Multi-step DB operations (secure delete, empty trash, workspace import) are wrapped in BEGIN/COMMIT/ROLLBACK transactions for atomicity
- Page revisions are pruned to the last 50 per page on each update
- WAL checkpoint is performed before encryption operations to flush all data
- Page moves reject self-parent and descendant-parent cycles before updating `pages.parent_id`
- Search snippets are escaped before the `<mark>` highlight tags are restored
- Attachment writes are rejected if the BLOB is larger than 25 MB

## Testing

- 17 Rust unit tests covering DB CRUD operations, search snippets, page move cycles, attachment limits, plugin path validation, encrypted sync framing, and PDF export smoke coverage
- Tests use in-memory SQLite (`Connection::open_in_memory()`)
- CI includes `cargo audit --file src-tauri/Cargo.lock` for Rust dependency vulnerability scanning
