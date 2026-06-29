# Development Guide

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.88+ | [rustup.rs](https://rustup.rs) |
| Node.js | 20.19+, 22.12+, or 24+ | [nodejs.org](https://nodejs.org) |
| Tauri CLI | v2 | `cargo install tauri-cli --version "^2"` |

### System Dependencies

**macOS**: No additional dependencies required.

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

**Windows**: Install [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

See the full [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Setup

```bash
git clone https://github.com/900Labs/900Notes.git
cd 900Notes
npm install
cargo tauri dev
```

The app opens at `http://localhost:1420` in the Tauri webview. Hot-reload is active for frontend changes. Rust changes trigger a rebuild.

## Project Structure

```
900Notes/
├── src/                          # Svelte 5 frontend
│   ├── components/               # UI components by feature
│   │   ├── editor/               # ProseMirror editor, toolbar, slash menu, tag input
│   │   ├── sidebar/              # Page tree, tag list, sidebar shell
│   │   ├── search/               # Search palette (Ctrl/Cmd+K)
│   │   ├── backlinks/            # Backlinks panel
│   │   ├── settings/             # Settings modal
│   │   └── common/               # Shared UI primitives (future)
│   ├── stores/                   # Svelte 5 Runes state stores
│   │   └── app.svelte.ts         # PageStore, TagStore, BacklinkStore, SettingsStore
│   ├── i18n/                     # Translation files
│   │   └── index.ts              # 6 languages, locale store, t() derived store
│   ├── lib/                      # Frontend libraries
│   │   ├── api.ts                # Tauri IPC wrapper (typed)
│   │   ├── types.ts              # TypeScript interfaces matching Rust models
│   │   └── editor/               # ProseMirror schema and editor setup
│   │       ├── schema.ts         # Node and mark definitions
│   │       └── index.ts          # Editor factory, input rules, commands
│   ├── utils/                    # Utility functions (future)
│   ├── App.svelte                # Root component, 3-pane layout
│   ├── main.ts                   # Entry point
│   └── app.css                   # Global styles, TailwindCSS, ProseMirror styles
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri app builder, state, plugin registration
│   │   ├── main.rs               # Binary entry point
│   │   ├── commands/             # Tauri IPC command handlers
│   │   │   ├── mod.rs            # Module declarations
│   │   │   ├── pages.rs          # 13 page commands
│   │   │   ├── tags.rs           # 6 tag commands
│   │   │   ├── links.rs          # 3 link commands
│   │   │   ├── settings.rs       # 3 settings commands
│   │   │   └── export_import.rs  # 4 export/import commands
│   │   ├── models/               # Data structures
│   │   │   ├── mod.rs
│   │   │   ├── page.rs           # Page, PageTreeNode, CreatePageInput, etc.
│   │   │   ├── tag.rs            # Tag, CreateTagInput, UpdateTagInput
│   │   │   └── link.rs           # Link, Backlink
│   │   ├── db/                   # SQLite layer
│   │   │   └── mod.rs            # Database struct, migrations, all queries, FTS5, link engine
│   │   └── services/             # Business logic
│   │       ├── mod.rs
│   │       ├── export_import.rs  # Workspace JSON export/import
│   │       └── markdown.rs       # ProseMirror ↔ Markdown conversion
│   ├── capabilities/             # Tauri permission capabilities
│   │   └── default.json
│   ├── Cargo.toml                # Rust dependencies
│   ├── build.rs                  # Tauri build script
│   └── tauri.conf.json           # Tauri configuration
├── docs/                         # Documentation
│   ├── README.md                 # Documentation index
│   ├── ARCHITECTURE.md           # System design and data flow
│   ├── API.md                    # Complete Tauri command reference
│   ├── DATABASE.md               # SQLite schema reference
│   ├── DEVELOPMENT.md            # This file
│   ├── EDITOR.md                 # ProseMirror schema guide
│   ├── I18N.md                   # Internationalization guide
│   ├── ROADMAP.md                # MVP and post-MVP phases
│   ├── SPRINT_PLAN.md            # 20-sprint post-MVP plan
│   ├── QUALITY_GATE.md           # Pre-merge validation
│   └── adr/                      # Architecture Decision Records
├── scripts/                      # Validation scripts
│   ├── verify-local.sh           # Local quality gate
│   └── verify-public-release.sh  # Public release privacy gate
├── .github/workflows/            # CI/CD
│   └── ci.yml                    # Quality gate workflow
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── svelte.config.js
├── tsconfig.json
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE
└── README.md
```

## Coding Standards

### Rust

- Follow `rustfmt` formatting. Run `cargo fmt` before committing.
- No clippy warnings. Run `cargo clippy -- -D warnings`.
- Use `thiserror` for error types. All command handlers return `Result<T, String>`.
- Use `serde` with `#[serde(rename_all = "camelCase")]` for all structs exposed to the frontend.
- UUID v4 for all IDs.
- ISO 8601 (RFC 3339) for all timestamps via `chrono::Utc::now().to_rfc3339()`.

### TypeScript / Svelte

- Use TypeScript for all new files. No plain `.js` files.
- Use Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`) — not Svelte 4 stores.
- Follow the existing pattern: stores in `src/stores/app.svelte.ts`, API calls in `src/lib/api.ts`.
- Use TailwindCSS classes for styling. No inline styles unless dynamic values are needed.
- All UI text must use translation keys from `src/i18n/index.ts`.

### Commits

- Use clear, descriptive messages: `Add graph view component` or `Fix link engine case sensitivity`.
- Squash-merge is the default for PRs.

## Adding a New Tauri Command

1. **Rust model** (if needed): Add struct in `src-tauri/src/models/` with `#[serde(rename_all = "camelCase")]`.
2. **Database method**: Add query method in `src-tauri/src/db/mod.rs`.
3. **Command handler**: Add `#[tauri::command]` function in `src-tauri/src/commands/`.
4. **Register command**: Add to `invoke_handler` in `src-tauri/src/lib.rs`.
5. **Frontend API**: Add typed wrapper in `src/lib/api.ts`.
6. **Frontend type**: Add TypeScript interface in `src/lib/types.ts` (if new type).
7. **Store method**: Add to the appropriate store in `src/stores/app.svelte.ts`.
8. **UI**: Use in Svelte component.
9. **Documentation**: Update `docs/API.md` with the new command.

## Adding a New Block Type to the Editor

1. **Schema**: Add node definition in `src/lib/editor/schema.ts`.
2. **Input rules**: Add markdown shortcut in `src/lib/editor/index.ts` `buildInputRules()`.
3. **Toolbar**: Add button in `src/components/editor/EditorToolbar.svelte`.
4. **Slash menu**: Add item in `src/components/editor/SlashMenu.svelte`.
5. **CSS**: Add styles in `src/app.css` under `.ProseMirror` section.
6. **Markdown conversion**: Add rendering in `src-tauri/src/services/markdown.rs` `render_node()` and `markdown_to_prosemirror()`.
7. **Translation keys**: Add labels for the block type in all 6 languages in `src/i18n/index.ts`.

## Adding a New Language

See [I18N Guide](I18N.md).

## Testing

### Rust Tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Frontend Type Check
```bash
npm run check
```

### Full Quality Gate
```bash
./scripts/verify-local.sh
```

### Manual Smoke Test
1. Launch with `cargo tauri dev`.
2. Create a new page — verify it appears in the sidebar.
3. Type in the editor — verify auto-save works (check no errors in console).
4. Type `[[` — verify wiki link autocomplete appears.
5. Create a second page, link to the first — verify backlinks panel updates.
6. Add a tag — verify it appears in the sidebar Tags tab.
7. Press Ctrl/Cmd+K — verify search palette opens.
8. Open Settings — verify theme toggle and language switch work.
9. Export workspace — verify JSON file downloads.
10. Delete a page — verify it appears in Trash. Restore it.

## Debugging

### Frontend
- Open DevTools in the Tauri webview (right-click → Inspect Element, or Ctrl/Cmd+Shift+I).
- Check the console for IPC errors.
- Use `$inspect()` in Svelte components for reactive debugging.

### Backend
- Add `eprintln!()` statements in Rust code. Output appears in the terminal running `cargo tauri dev`.
- Inspect the SQLite database directly:
  ```bash
  sqlite3 ~/Library/Application Support/com.ninelabs.notes/900notes.db
  ```

### Database Location
- **macOS**: `~/Library/Application Support/com.ninelabs.notes/900notes.db`
- **Linux**: `~/.local/share/com.ninelabs.notes/900notes.db`
- **Windows**: `%APPDATA%\com.ninelabs.notes\900notes.db`
