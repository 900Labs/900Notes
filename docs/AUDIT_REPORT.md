# 900Notes Full Audit Report

**Date**: 2026-06-29  
**Scope**: All 20 sprints — security, error handling, dead code, integration, smoke tests

---

## 1. Security Review

### Critical Issues Found & Fixed

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| S1 | **Weak key derivation** — `derive_key()` used a single SHA-256 pass, vulnerable to brute-force. Replaced with 100,000-iteration iterative hashing. | Critical | ✅ Fixed |
| S2 | **Non-cryptographic RNG** — `random_bytes()` used UUID v4 XOR for salt/nonce generation. Replaced with `getrandom` CSPRNG. | Critical | ✅ Fixed |
| S3 | **Sharing service same weaknesses** — `sharing.rs` had its own `derive_key()` with single SHA-256 pass and used UUID for salt/nonce. Fixed with same iterative hashing + CSPRNG. | Critical | ✅ Fixed |
| S4 | **XSS in Mermaid error rendering** — Error message injected into `innerHTML` without escaping. Fixed to use `textContent`. | High | ✅ Fixed |

### Security Notes (Acceptable)

- **KaTeX `innerHTML`**: KaTeX has built-in HTML escaping — safe by design.
- **Mermaid `innerHTML` with SVG**: Mermaid sanitizes its SVG output — acceptable for local-first app.
- **Plugin loader `new Function()`**: Intentional design for JS plugin loading. Plugins run in webview context with full access. Documented in `docs/PLUGINS.md` as a known limitation. Future: sandboxed execution.
- **SQL injection**: All DB queries use parameterized statements (`params![]`). No string-formatted SQL found.
- **Sync binds to 0.0.0.0**: Acceptable for local network sync. Documented behavior.

### Remaining Security Recommendations

1. **PBKDF2/Argon2**: The iterative SHA-256 is better than single-pass but not as strong as PBKDF2 or Argon2. Consider migrating to `argon2` crate in the future.
2. **Plugin sandboxing**: Consider Web Workers or iframe sandboxing for plugin execution.
3. **Sync authentication**: No authentication between sync peers. Anyone on the network can sync. Add a shared secret or key exchange.

---

## 2. Error Handling & Panic Audit

### Issues Found & Fixed

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| E1 | **5x `Mutex::unwrap()` in `sync.rs`** — Would panic on poisoned mutex (e.g., if a sync thread panics). Replaced with `unwrap_or_else(\|e\| e.into_inner())` for poison recovery. | High | ✅ Fixed |

### Acceptable `unwrap()`/`expect()` (5 in `lib.rs`)

- `app_data_dir().expect()` — App can't function without data dir, fail-fast is correct.
- `Database::open().expect()` — App can't function without DB, fail-fast is correct.
- `CrdtService::load_from_db().expect()` — Same reasoning.
- `create_dir_all().expect()` — Same reasoning.
- `generate_context!().expect()` — Tauri runtime, fail-fast is correct.

---

## 3. Dead Code & Unused Imports

### Issues Found & Fixed

| # | Issue | Status |
|---|-------|--------|
| D1 | Unused `Arc` and `Mutex` imports in `commands/plugins.rs` | ✅ Removed |
| D2 | Non-snake-case `pagesCreated` field in `ImportResultResponse` | ✅ Fixed with `#[serde(rename_all = "camelCase")]` |
| D3 | `pageTitles` in `EditorView.svelte` not using `$state()` | ✅ Fixed |
| D4 | `nodes` in `GraphView.svelte` not using `$state()` | ✅ Fixed |

### Remaining Warnings (21, all pre-existing)

- 10x a11y warnings (click handlers on divs, ARIA roles, autofocus) — cosmetic, not blocking
- 3x self-closing HTML tag warnings — cosmetic
- 2x "reference only captures initial value" — Svelte 5 reactivity nuance, not a bug
- 6x other minor warnings

---

## 4. Integration Check

### Command Registration

- **134 Tauri commands** registered in `lib.rs`
- **134 TypeScript API wrappers** in `api.ts`
- **All command names match** (verified via diff)
- **All DB methods referenced by commands exist** (verified via `cargo check`)

### Module Structure

- All `commands/mod.rs` entries have corresponding `.rs` files
- All `services/mod.rs` entries have corresponding `.rs` files
- All `models/mod.rs` entries have corresponding `.rs` files
- Frontend types in `types.ts` match Rust models (verified via `npm run check`)

### Cross-Feature Integration

- Plugin system uses existing DB infrastructure (no separate DB)
- Automation API wraps existing DB methods (no new queries)
- Importers use existing `create_page` DB method
- Web clipper expects local HTTP server (not yet implemented — documented limitation)
- Encryption/sharing use same `derive_key` pattern (now hardened)

---

## 5. Smoke Test Plan

### Pre-flight

- [ ] `cargo check` passes with 0 warnings
- [ ] `npm run check` passes with 0 errors
- [ ] `npm run build` succeeds

### Core Features

- [ ] **App launches** — `npm run tauri dev` starts without errors
- [ ] **Create page** — New page appears in sidebar tree
- [ ] **Edit page** — Content saves (debounced) and persists after reload
- [ ] **Delete page** — Page moves to trash, can be restored
- [ ] **Search** — Command palette (`Ctrl+K`) finds pages by title and content
- [ ] **Wiki links** — `[[Page Title]]` creates clickable links
- [ ] **Tags** — Create tags, assign to pages, filter by tag

### Rich Content

- [ ] **Slash menu** — Type `/` to see block options (heading, todo, code, etc.)
- [ ] **Code blocks** — Syntax highlighting renders
- [ ] **Math blocks** — LaTeX renders via KaTeX
- [ ] **Mermaid diagrams** — Diagram renders from text
- [ ] **Attachments** — Drag-and-drop image, verify it displays
- [ ] **Audio notes** — Record audio, verify playback

### Organization

- [ ] **Page tree** — Drag pages to reorder/nest
- [ ] **Templates** — Create template, create page from template
- [ ] **Daily note** — Click daily note button, verify today's date
- [ ] **Smart folders** — Create saved search, verify smart folder shows results
- [ ] **Tag groups** — Create tag group, add tags, verify grouping
- [ ] **Graph view** — Open graph, verify nodes and edges render

### Sync & Sharing

- [ ] **Export workspace** — Export to JSON, verify file contents
- [ ] **Import workspace** — Import JSON, verify pages created
- [ ] **Share bundle** — Export pages with passphrase, import on fresh instance
- [ ] **Sync** — Start sync on two instances, verify pages sync (manual test)

### Security

- [ ] **Enable encryption** — Set passphrase, verify DB is encrypted
- [ ] **Unlock database** — Restart app, enter passphrase, verify access
- [ ] **Change passphrase** — Change, verify old passphrase fails
- [ ] **Secure delete** — Secure delete a page, verify content is overwritten

### Accessibility

- [ ] **Language switch** — Change to Arabic, verify RTL layout
- [ ] **Language switch** — Change to Urdu, verify RTL layout
- [ ] **Language switch** — Change to Bengali, verify LTR renders correctly
- [ ] **Theme** — Switch between light/dark/system
- [ ] **Keyboard nav** — Tab through sidebar, editor, settings

### Plugins

- [ ] **Scan plugins** — Place example plugin in `plugins/` dir, scan, verify it appears
- [ ] **Enable/disable** — Toggle plugin, verify state persists
- [ ] **Remove plugin** — Uninstall, verify it disappears

### Importers

- [ ] **Evernote** — Import sample `.enex` file, verify pages created
- [ ] **Notion** — Point to Notion export dir, verify `.md` files imported
- [ ] **Obsidian** — Point to vault dir, verify recursive import
- [ ] **Roam** — Import Roam JSON, verify pages created

### Mobile

- [ ] **Mobile build** — `npm run build:mobile` succeeds
- [ ] **Mobile dev** — `npm run tauri:android dev` starts (requires Android SDK)

---

## 6. Summary

### Fixes Applied

| Category | Issues Fixed | Critical | High | Medium |
|----------|-------------|----------|------|--------|
| Security | 4 | 3 | 1 | 0 |
| Error Handling | 1 | 0 | 1 | 0 |
| Code Quality | 4 | 0 | 0 | 4 |
| **Total** | **9** | **3** | **2** | **4** |

### Verification Results

| Check | Before | After |
|-------|--------|-------|
| `cargo check` warnings | 3 | 0 |
| `npm run check` errors | 0 | 0 |
| `npm run check` warnings | 23 | 21 |
| `npm run build` | ✅ | ✅ |
| Command registration match | 134/134 | 134/134 |

### Remaining Recommendations (Non-blocking)

1. Migrate to Argon2 for key derivation (stronger than iterative SHA-256)
2. Add sync authentication (shared secret between peers)
3. Sandbox plugin execution (Web Workers or iframe)
4. Fix remaining 21 a11y warnings (div click handlers → buttons)
5. Implement local HTTP server for web clipper
6. Add automated tests (unit tests for DB, integration tests for commands)
