# Codebase Audit Report

## 1. Executive Summary

**Overall readiness rating: MVP-ready**

The codebase can support a controlled demo or limited internal use, but has known gaps that must be fixed before public users or sensitive data. The architecture is sound, the code is well-organized, and the build pipeline is functional. However, several security and reliability issues prevent production readiness.

**Is it safe to ship?** Not as-is for public release. Safe for developer testing and controlled demos.

**Biggest strengths:**
- Clean separation of concerns: Rust backend (commands → services → db), Svelte 5 frontend (stores → components)
- 134 Tauri commands with 134 matching TypeScript wrappers — zero mismatch
- All SQL queries use parameterized statements — no injection risk
- Comprehensive documentation (16 docs, 4 ADRs, threat model, privacy model)
- CI pipeline with fmt, clippy, type check, build, and test gates
- `npm audit`: 0 vulnerabilities

**Biggest risks:**
- Encryption is effectively bypassed after first unlock — plaintext DB persists on disk permanently
- No CSP set in Tauri config — plugin system uses `new Function()` with no sandboxing
- Zero tests in the entire codebase (0 Rust tests, 0 frontend tests)
- No DB transactions for multi-step operations — partial failures leave inconsistent state
- Sync protocol has no authentication, no encryption, and unbounded memory allocation

**Top 3 things to fix first:**
1. Fix encryption unlock to not leave plaintext DB on disk (or document as accepted risk)
2. Add CSP to Tauri config
3. Add DB transactions for multi-step operations (import, secure delete, etc.)

**What must be fixed before production use:**
- Encryption plaintext DB persistence
- CSP configuration
- Sync unbounded read (DoS vector)
- DB transaction wrapping for multi-step operations
- App.svelte missing encryption unlock gate (app crashes if encryption enabled and app restarts)

---

## 2. System Overview

| Aspect | Detail |
|--------|--------|
| Tech stack | Tauri v2 (Rust backend + Svelte 5 frontend) |
| Database | SQLite (local, single file at `{APP_DATA_DIR}/900notes.db`) |
| Editor | ProseMirror (block-based WYSIWYG) |
| Search | SQLite FTS5 |
| Styling | TailwindCSS 3 |
| Crypto | AES-256-GCM, SHA-256 (100k iterations), getrandom CSPRNG |
| Sync | mDNS discovery + TCP, Automerge CRDT |
| Build | Vite (frontend), cargo (backend), `cargo tauri build` |
| CI | GitHub Actions: npm check, npm build, cargo fmt, cargo clippy, cargo test |
| License | Apache 2.0 |

**Main modules:**
- `src-tauri/src/commands/` — 16 command modules (pages, tags, links, sync, encryption, plugins, automation, importers, etc.)
- `src-tauri/src/db/mod.rs` — 2727 lines, single-file DB layer with all schema + queries
- `src-tauri/src/services/` — encryption, sync, sharing, CRDT, workspace, export/import, markdown, importers
- `src/stores/app.svelte.ts` — 620 lines, all Svelte 5 Runes stores
- `src/lib/api.ts` — 493 lines, 134 Tauri IPC wrappers
- `src/lib/editor/` — ProseMirror schema, nodeviews, plugins

**Main user flows:**
1. Create/edit/delete pages → ProseMirror editor → debounced save → SQLite
2. Wiki links `[[title]]` → link engine extracts + matches → backlinks panel
3. Search → FTS5 query → command palette results
4. Encryption → passphrase → AES-256-GCM encrypt DB file → unlock on next launch
5. Sync → mDNS discover peers → TCP handshake → exchange page metadata
6. Import → Evernote ENEX / Notion MD / Obsidian vault / Roam JSON → ProseMirror JSON → SQLite

---

## 3. Commands Run

| Command | Result | Notes |
|---------|--------|-------|
| `cargo clippy -- -D warnings` | ✅ Pass (after fixing 4 clippy errors) | Fixed: needless borrows in `encryption.rs`, `sharing.rs`, `importers.rs` |
| `cargo fmt --check` | ✅ Pass (after `cargo fmt`) | Fixed: indentation in `importers.rs` |
| `cargo test` | ✅ Pass | 0 tests exist — pass is vacuous |
| `cargo check` | ✅ Pass | 0 warnings |
| `npm run check` | ✅ Pass | 0 errors, 21 warnings (a11y, reactivity) |
| `npm run build` | ✅ Pass | Vite build succeeds |
| `npm audit` | ✅ 0 vulnerabilities | |
| `cargo audit` | ❌ Not installed | `cargo-audit` not installed in environment — could not check Rust dependency vulnerabilities |

---

## 4. Critical and High Findings

### Finding: Encryption bypassed after first unlock — plaintext DB persists on disk

* **Severity:** Critical
* **Confidence:** Verified
* **Category:** Security
* **File(s):** `src-tauri/src/commands/encryption.rs:62`, `src-tauri/src/services/encryption.rs:106`
* **Evidence:**
  ```rust
  // encryption.rs:62 — unlock_database decrypts to the plaintext DB path
  service.decrypt_to_path(&passphrase, &db_path)?;
  ```
  The plaintext DB is written to `900notes.db` and never deleted. On next app launch, `lib.rs:31` opens `900notes.db` directly:
  ```rust
  let database = db::Database::open(&db_path).expect("failed to open database");
  ```
  No check for encryption state before opening. The app will open the plaintext DB without requiring a passphrase.
* **What is wrong:** `unlock_database` decrypts the DB to the same path used for normal operation. The plaintext file persists across restarts. Encryption is only effective until the first unlock.
* **Why it matters:** Users who enable encryption expect their data to be encrypted at rest. After the first unlock, their data is permanently stored in plaintext. The threat model (`docs/THREAT_MODEL.md:98`) claims "Database is re-encrypted when the app closes" — this is false. No such code exists.
* **How to fix:** Either (a) re-encrypt and delete plaintext DB on app shutdown via Tauri's `on_window_event` / `RunEvent`, or (b) use an in-memory DB after decrypt and never write plaintext to disk, or (c) document as accepted risk and remove the false claim from the threat model.
* **Suggested tests:** Enable encryption → unlock → restart app → verify passphrase is required.
* **Priority:** P0

### Finding: No Content Security Policy in Tauri config

* **Severity:** High
* **Confidence:** Verified
* **Category:** Security
* **File(s):** `src-tauri/tauri.conf.json:25`
* **Evidence:**
  ```json
  "security": {
    "csp": null
  }
  ```
  The plugin loader (`src/lib/plugins/loader.ts:52`) uses `new Function('plugin', jsCode)` to execute arbitrary JS from the filesystem. With no CSP, there is no defense-in-depth against malicious plugins or compromised plugin files.
* **What is wrong:** CSP is explicitly set to `null`, disabling all content security policies. The app loads and executes arbitrary JavaScript from the filesystem via `new Function()`.
* **Why it matters:** A compromised or malicious plugin has full access to the Tauri IPC bridge with no restrictions. CSP would limit the blast radius of injection attacks.
* **How to fix:** Set a restrictive CSP in `tauri.conf.json`:
  ```json
  "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
  ```
  Note: `'unsafe-inline'` is needed for Svelte's injected styles. `'unsafe-eval'` would be needed for `new Function()` in plugins — consider sandboxing plugins in a Web Worker instead.
* **Suggested tests:** Load app with CSP → verify editor, KaTeX, Mermaid render correctly.
* **Priority:** P1

### Finding: App.svelte has no encryption unlock gate

* **Severity:** High
* **Confidence:** Verified
* **Category:** Reliability / Security
* **File(s):** `src/App.svelte:28-33`
* **Evidence:**
  ```typescript
  onMount(async () => {
    await settingsStore.loadSettings()
    await pageStore.loadPageTree()  // ← crashes if DB is encrypted
    await pageStore.loadRecentPages()
    await tagStore.loadTags()
  })
  ```
  `EncryptionStore` exists in `src/stores/app.svelte.ts:541` but is never imported or used in `App.svelte`. If encryption is enabled and the app restarts, `loadPageTree()` will call Tauri commands that try to access an encrypted DB, producing unhandled promise rejections.
* **What is wrong:** The encryption store is implemented but never wired into the app's startup flow. The app has no unlock screen.
* **Why it matters:** Encryption feature is broken at runtime — the app will crash on restart if encryption is enabled.
* **How to fix:** Import `encryptionStore` in `App.svelte`, call `encryptionStore.checkStatus()` in `onMount`, and render an unlock prompt when `encryptionStore.enabled && !encryptionStore.unlocked`.
* **Suggested tests:** Enable encryption → restart app → verify unlock prompt appears.
* **Priority:** P0

### Finding: No DB transactions for multi-step operations

* **Severity:** High
* **Confidence:** Verified
* **Category:** Reliability
* **File(s):** `src-tauri/src/db/mod.rs:839-867` (secure_delete_page), `src-tauri/src/services/export_import.rs:40-72` (import_workspace), `src-tauri/src/services/sharing.rs:100-141` (import_share_bundle)
* **Evidence:**
  ```rust
  // secure_delete_page: 7 separate statements, no transaction
  self.conn.execute("UPDATE pages SET content = ?1, title = ?1, ...", params![...])?;
  self.conn.execute("DELETE FROM page_revisions WHERE page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM links WHERE source_page_id = ?1 OR target_page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM page_tags WHERE page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM page_properties WHERE page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM attachments WHERE page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM audio_notes WHERE page_id = ?1", params![id])?;
  self.conn.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
  self.conn.execute_batch("VACUUM;")?;
  ```
  Zero transactions in the entire codebase (`grep -n "transaction\|begin\|commit\|rollback" src/db/mod.rs` returns nothing).
* **What is wrong:** Multi-step DB operations are not atomic. If `secure_delete_page` fails after deleting revisions but before deleting the page, the page is left with no history. If `import_workspace` fails at page 50 of 100, half the data is imported.
* **Why it matters:** Data corruption on partial failure. Users may lose data or end up with inconsistent state.
* **How to fix:** Wrap multi-step operations in `self.conn.execute_batch("BEGIN; ... COMMIT;")` or use `self.conn.transaction()` (rusqlite's transaction API).
* **Suggested tests:** Test import with intentionally failing data at midpoint → verify no partial data in DB.
* **Priority:** P1

### Finding: Sync protocol unbounded memory allocation

* **Severity:** High
* **Confidence:** Verified
* **Category:** Security / Reliability
* **File(s):** `src-tauri/src/services/sync.rs:266-267`
* **Evidence:**
  ```rust
  let len = u32::from_be_bytes(len_buf) as usize;
  let mut buf = vec![0u8; len];  // ← up to 4GB allocation from untrusted network input
  ```
  No size cap on the length field. A malicious peer can send `0xFFFFFFFF` (4GB) as the length, causing an out-of-memory panic.
* **What is wrong:** The sync TCP protocol trusts a u32 length field from the network without any maximum size check.
* **Why it matters:** Any device on the same LAN can crash the app by sending a malformed sync handshake.
* **How to fix:** Add a maximum size check: `if len > 100 * 1024 * 1024 { return; }` (100MB cap).
* **Suggested tests:** Send sync handshake with length=0xFFFFFFFF → verify connection is rejected, not crashed.
* **Priority:** P1

### Finding: Sync has no authentication or encryption

* **Severity:** Medium
* **Confidence:** Verified
* **Category:** Security
* **File(s):** `src-tauri/src/services/sync.rs:119` (binds to 0.0.0.0), `src-tauri/src/services/sync.rs:255-322` (handle_sync_connection)
* **Evidence:**
  ```rust
  let listener = match TcpListener::bind(("0.0.0.0", port)) { ... };
  // ...
  // No authentication check in handle_sync_connection
  // No TLS, no shared secret, no key exchange
  ```
  The threat model (`docs/THREAT_MODEL.md:87`) documents this: "Sync traffic is not encrypted. Sprint 16+ should add TLS or Noise Protocol for sync."
* **What is wrong:** Any device on the LAN can connect to the sync server and receive all page content. No authentication, no encryption.
* **Why it matters:** On a shared network (coffee shop, office), anyone can read all notes via sync.
* **How to fix:** Add a shared secret exchanged out-of-band (QR code), or use TLS with self-signed certificates. Document as accepted risk for MVP if deferred.
* **Suggested tests:** Verify sync rejects connections without shared secret.
* **Priority:** P2 (documented in threat model, accepted for MVP)

### Finding: Zero tests in the entire codebase

* **Severity:** High
* **Confidence:** Verified
* **Category:** Testing
* **File(s):** Entire codebase
* **Evidence:**
  ```
  $ find . -name "*.test.*" -o -name "*.spec.*" -o -name "test_*" -o -name "*_test.*" | grep -v node_modules | grep -v target
  (no output)
  
  $ cargo test
  running 0 tests
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```
  CI runs `cargo test` which passes vacuously. No frontend test framework is configured.
* **What is wrong:** No unit tests, integration tests, or end-to-end tests exist. CI's test step is a no-op.
* **Why it matters:** Regressions cannot be detected automatically. Any code change could break critical flows (encryption, sync, import) without anyone knowing.
* **How to fix:** Add Rust unit tests for DB operations (`#[cfg(test)] mod tests` in `db/mod.rs`), frontend tests (vitest or playwright), and at minimum a smoke test that verifies the app launches.
* **Suggested tests:** This finding is about the absence of tests.
* **Priority:** P1

---

## 5. Area-by-Area Review

### Architecture

**What looks good:**
- Clean module separation: commands → services → db, with explicit module boundaries
- 134 Tauri commands all registered in one place (`lib.rs`), all have matching TS wrappers
- ADRs document key decisions (Tauri v2, ProseMirror, SQLite FTS5, JSON storage)
- Single-file DB layer is easy to navigate

**Issues found:**
- `db/mod.rs` is 2727 lines — a single god file. Not broken, but will become hard to maintain as features grow.
- `conn_execute` exposes raw SQL execution to external modules, breaking DB encapsulation (`src-tauri/src/db/mod.rs:1207`)
- `stores/app.svelte.ts` is 620 lines with all stores in one file — same concern as DB layer

**Recommended improvements:**
- Split `db/mod.rs` into `db/pages.rs`, `db/tags.rs`, `db/links.rs`, etc.
- Split `stores/app.svelte.ts` into per-feature store files
- Remove `conn_execute` and add proper DB methods for `import_workspace`

### Code Quality

**What looks good:**
- Consistent error handling: all commands return `Result<T, String>`, all `map_err` conversions
- Consistent naming: `camelCase` serde for frontend, `snake_case` for Rust
- Clippy passes with zero warnings (after fixes in this audit)
- `cargo fmt` passes

**Issues found:**
- `update_page` (`db/mod.rs:707,713`) calls `get_page_by_id` twice when content changes — redundant query
- Revision creation failure silently ignored: `let _ = self.create_revision(...)` (`db/mod.rs:714`)
- 21 Svelte warnings (a11y: div click handlers, ARIA roles, autofocus)

**Recommended improvements:**
- Combine the two `get_page_by_id` calls in `update_page` into one
- Log revision creation failures instead of silently ignoring
- Replace div click handlers with buttons for a11y

### Security

**What looks good:**
- All SQL queries use parameterized statements — no injection risk
- AES-256-GCM with CSPRNG (`getrandom`) for salt/nonce — properly implemented (fixed in prior audit)
- 100k-iteration key derivation — adequate for MVP
- KaTeX has built-in HTML escaping — safe
- Mermaid error rendering fixed to use `textContent` (fixed in prior audit)
- `npm audit`: 0 vulnerabilities

**Issues found:**
- Encryption bypassed after first unlock (Critical — see Finding 1)
- No CSP in Tauri config (High — see Finding 2)
- Sync has no auth/encryption (Medium — documented in threat model)
- Plugin system uses `new Function()` with no sandboxing
- `enable_encryption` reads DB file while SQLite connection is still open — will fail on Windows (file lock)

**Recommended improvements:**
- Close DB connection before reading the file in `enable_encryption`
- Add plugin sandboxing (Web Worker or iframe with sandbox attribute)
- Migrate to Argon2 for key derivation (stronger than iterative SHA-256)

### Reliability

**What looks good:**
- WAL mode enabled for crash resistance
- Foreign key constraints enabled (`PRAGMA foreign_keys=ON`)
- Soft-delete with trash and restore
- Revision history for content changes
- Mutex poison recovery in sync service (fixed in prior audit)

**Issues found:**
- No DB transactions for multi-step operations (High — see Finding 4)
- App crashes on restart if encryption is enabled (High — see Finding 3)
- Revisions grow unbounded — no pruning. Only deleted when page is deleted. DB will grow indefinitely.
- `enable_encryption` doesn't close the SQLite connection before reading the DB file — cross-platform issue

**Recommended improvements:**
- Add revision pruning (keep last N per page, or older than X days)
- Add app shutdown handler to re-encrypt DB if encryption is enabled
- Wrap `import_workspace`, `secure_delete_page`, `import_share_bundle` in transactions

### Testing

**What looks good:**
- CI pipeline exists with fmt, clippy, type check, build
- `verify-local.sh` script for local pre-commit checks

**Issues found:**
- Zero tests (High — see Finding 6)
- CI test step is vacuous — passes with 0 tests
- No frontend test framework configured (no vitest, no playwright)
- No smoke tests or integration tests

**Recommended improvements:**
- Add Rust `#[cfg(test)]` tests for DB CRUD operations
- Add vitest for frontend store logic
- Add a smoke test script that launches the app and verifies basic flows
- Make CI fail if test count is 0 (optional: `cargo test 2>&1 | grep "0 passed"` → fail)

### Performance

**What looks good:**
- SQLite with WAL mode, 64MB cache, 256MB mmap — well-tuned for local use
- FTS5 with proper triggers for incremental updates
- `build_tree` uses O(n) HashMap-based grouping (optimized in prior session)
- Page tree metadata endpoint avoids loading full content
- Debounced saves (not on every keystroke)

**Issues found:**
- `rebuild_links_for_page` loads ALL page titles on every content save (`db/mod.rs:1136-1144`) — O(n) per save, acceptable for personal use but won't scale
- `get_all_pages_for_sync` loads all pages with full content into memory — acceptable for personal sync
- Revisions grow unbounded — DB bloat over time

**Recommended improvements:**
- Cache page titles in a HashMap that's refreshed on page create/rename/delete
- Add revision pruning on save (keep last 50 per page)

### Database / Data Model

**What looks good:**
- Proper schema with foreign keys, cascade deletes, indexes
- FTS5 virtual table with insert/update/delete triggers
- UUID v4 for all IDs (offline-safe)
- ISO 8601 timestamps
- Soft-delete with `deleted_at` column
- Settings as key-value store (flexible)

**Issues found:**
- No transactions (High — see Finding 4)
- No revision pruning
- `page_revisions` table has no `updated_at` — can't track when revision was created vs. page update
- Schema is all in one `execute_batch` — no migration versioning. Future schema changes will need proper migration support.

**Recommended improvements:**
- Add a `schema_version` table and migration system
- Add revision pruning

### API / Integrations

**What looks good:**
- 134 commands covering all features
- Consistent `Result<T, String>` return type
- Automation API exposes CRUD for scripting
- Importers handle Evernote, Notion, Obsidian, Roam
- Web clipper extension documented (not yet functional — needs local HTTP server)

**Issues found:**
- No rate limiting on automation API (acceptable for local IPC)
- Web clipper expects HTTP server on port 1420 — not implemented
- `strip_html_tags` in importers is naive (doesn't handle `<script>` content, HTML entities) — but output is safe since it goes into ProseMirror text nodes

**Recommended improvements:**
- Document that web clipper is not yet functional
- Consider using an HTML parser crate instead of naive tag stripping

### Frontend / UI

**What looks good:**
- Svelte 5 Runes throughout (`$state`, `$derived`, `$effect`, `$props`)
- 6 languages with RTL support for Arabic
- Theme system (light/dark/system)
- Command palette with keyboard shortcut
- ProseMirror editor with slash menu, wiki links, math (KaTeX), diagrams (Mermaid)

**Issues found:**
- `App.svelte` missing encryption unlock gate (High — see Finding 3)
- 21 a11y warnings (div click handlers, missing ARIA roles, autofocus)
- `titleValue` and `iconValue` in `EditorView.svelte` capture initial value of `page` prop — compensated by `$effect` but fragile pattern
- No error boundaries — unhandled promise rejections crash the UI
- No loading skeletons or error states for failed API calls in most stores

**Recommended improvements:**
- Add global error handler for unhandled promise rejections
- Add error states to stores (some have `error` field, most don't)
- Fix a11y warnings: replace div click handlers with buttons

### DevOps / Deployment

**What looks good:**
- CI pipeline with 6 checks (fmt, clippy, type check, build, test, build)
- `verify-local.sh` for local pre-commit
- `verify-public-release.sh` exists
- PR template with checklist
- Tauri config for cross-platform builds

**Issues found:**
- CI test step is vacuous (0 tests)
- No `cargo audit` in CI (Rust dependency vulnerabilities not checked)
- No release workflow (no automated builds, no GitHub Releases automation)
- No Dockerfile (acceptable for Tauri — not containerized)

**Recommended improvements:**
- Add `cargo audit` to CI
- Add release workflow with cross-platform builds
- Add a check that fails CI if test count is 0

### Dependencies / Supply Chain

**What looks good:**
- `npm audit`: 0 vulnerabilities
- Dependencies are reasonable and well-maintained (Tauri 2, ProseMirror, Svelte 5, rusqlite)
- Lockfile present (`package-lock.json`)
- `Cargo.lock` present

**Issues found:**
- `cargo audit` not installed — Rust dependency vulnerabilities not checked
- `getrandom` 0.2 is current but 0.3 is available — minor
- No dependency review process

**Recommended improvements:**
- Install and run `cargo audit` in CI
- Consider Dependabot or Renovate for automated dependency updates

### Documentation / Developer Experience

**What looks good:**
- 16 documentation files covering architecture, API, database, editor, i18n, mobile, plugins, automation, privacy, threat model
- 4 ADRs for key decisions
- CONTRIBUTING.md with setup, coding standards, PR process
- README with clear setup instructions
- SPRINT_PLAN.md and SPRINT_REVIEW_LOG.md track progress

**Issues found:**
- Threat model claims "Database is re-encrypted when the app closes" — this is false (see Finding 1)
- Web clipper documented as functional but the HTTP server it expects is not implemented
- No troubleshooting guide
- `docs/ARCHITECTURE.md` doesn't mention sync, encryption, plugins, or automation (out of date)

**Recommended improvements:**
- Correct the false claim in threat model
- Mark web clipper as not-yet-functional
- Update architecture doc to cover all 20 sprints of features
- Add troubleshooting guide

---

## 6. Testing Gaps

| Flow / Module | Missing Test | Risk Covered | Priority |
|---------------|-------------|-------------|----------|
| DB CRUD (pages, tags, links) | Unit tests for create/read/update/delete | Data loss, corruption | P0 |
| Encryption enable/unlock/disable | Integration test: encrypt → restart → unlock → verify data | Encryption bypass, data lockout | P0 |
| DB transactions (import, secure delete) | Test partial failure → verify atomicity | Inconsistent state | P1 |
| Sync handshake | Test with malformed length field | DoS via OOM | P1 |
| Importers (Evernote, Notion, Obsidian, Roam) | Test with sample files | Import failure, data loss | P1 |
| Link engine | Test wiki link extraction and matching | Broken navigation | P1 |
| Frontend stores | Unit tests for PageStore, TagStore, etc. | State management bugs | P2 |
| ProseMirror editor | E2E test: type content → save → reload → verify | Editor data loss | P2 |
| i18n | Test all 6 languages render without missing keys | UX degradation | P3 |

---

## 7. Prioritized Action Plan

| Priority | Severity | Task | Area | Estimated Effort | Why It Matters |
|----------|----------|------|------|-----------------|----------------|
| P0 | Critical | Fix encryption unlock: re-encrypt on shutdown or use in-memory DB | Security | 4h | Encryption is bypassed after first unlock |
| P0 | High | Wire encryption unlock gate into App.svelte | Reliability | 2h | App crashes on restart with encryption enabled |
| P0 | High | Add DB unit tests for CRUD operations | Testing | 8h | No tests exist; regressions undetectable |
| P1 | High | Set CSP in tauri.conf.json | Security | 1h | No defense-in-depth against plugin injection |
| P1 | High | Wrap multi-step DB operations in transactions | Reliability | 4h | Partial failures leave inconsistent state |
| P1 | High | Add sync message size cap (100MB max) | Security | 0.5h | DoS via unbounded memory allocation |
| P1 | High | Correct false claim in threat model ("re-encrypted on close") | Documentation | 0.5h | False sense of security |
| P1 | High | Add `cargo audit` to CI | DevOps | 1h | Rust dependency vulnerabilities unchecked |
| P2 | Medium | Add revision pruning (keep last 50 per page) | Performance | 2h | DB grows unbounded |
| P2 | Medium | Fix redundant `get_page_by_id` in `update_page` | Performance | 0.5h | Extra query on every content save |
| P2 | Medium | Update ARCHITECTURE.md to cover all features | Documentation | 2h | Architecture doc is out of date |
| P2 | Medium | Mark web clipper as not-yet-functional | Documentation | 0.5h | Users will expect it to work |
| P3 | Low | Fix 21 a11y warnings | Frontend | 4h | Accessibility compliance |
| P3 | Low | Split `db/mod.rs` into per-feature modules | Architecture | 4h | Maintainability |
| P3 | Low | Add frontend test framework (vitest) | Testing | 4h | Frontend regression detection |

---

## 8. Suggested Builder Prompt

```
Implement the following verified P0 and P1 fixes for the 900Notes Tauri app. Do not break existing functionality.

## P0: Fix encryption unlock persistence

File: src-tauri/src/commands/encryption.rs
- In `unlock_database`, after decrypting to `900notes.db`, register a Tauri shutdown handler that re-encrypts the DB and deletes the plaintext file on app close.
- Use `app.on_window_event` in `src-tauri/src/lib.rs` to detect `WindowEvent::Destroyed` and call a new `re_encrypt_on_shutdown` function.
- The function should: read the plaintext DB, encrypt it with the stored passphrase (you'll need to store the passphrase in `AppState` as `Option<String>`), write the encrypted file, delete the plaintext file.
- If no passphrase is stored (encryption was never enabled), do nothing.
- Add a test: enable encryption → unlock → simulate shutdown → verify plaintext DB is deleted and encrypted DB exists.

## P0: Wire encryption unlock gate into App.svelte

File: src/App.svelte
- Import `encryptionStore` from `./stores/app.svelte`
- In `onMount`, call `await encryptionStore.checkStatus()` before loading pages
- If `encryptionStore.enabled && !encryptionStore.unlocked`, render an unlock prompt (passphrase input + unlock button) instead of the main UI
- Only load `pageStore.loadPageTree()` etc. after `encryptionStore.unlocked` is true
- Do not break the non-encrypted flow (when `!encryptionStore.enabled`, proceed as before)

## P0: Add DB unit tests

File: src-tauri/src/db/mod.rs
- Add `#[cfg(test)] mod tests` at the bottom
- Test: create page → get page → verify fields match
- Test: create page → update page → verify update
- Test: create page → soft delete → verify deleted_at is set → restore → verify deleted_at is null
- Test: create page → create tag → set page tags → get page tags → verify
- Test: create page with wiki link content → rebuild links → verify link exists
- Use `:memory:` SQLite for tests (`Database::open(Path::new(":memory:"))`)
- Run: `cargo test --manifest-path src-tauri/Cargo.toml`

## P1: Set CSP in tauri.conf.json

File: src-tauri/tauri.conf.json
- Change `"csp": null` to `"csp": "default-src 'self'; script-src 'self' 'unsafe-eval' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:"`
- `'unsafe-eval'` is needed for plugin loading via `new Function()`
- `'unsafe-inline'` is needed for Svelte's injected styles
- Run: `cargo tauri dev` → verify app loads, editor works, KaTeX renders, Mermaid renders

## P1: Wrap multi-step DB operations in transactions

File: src-tauri/src/db/mod.rs
- In `secure_delete_page`: wrap all DELETE/UPDATE statements in `self.conn.execute_batch("BEGIN; ... COMMIT;")` or use `let tx = self.conn.transaction()?;`
- In `import_workspace` (src-tauri/src/services/export_import.rs): wrap all INSERT statements in a transaction
- In `import_share_bundle` (src-tauri/src/services/sharing.rs): wrap all INSERT statements in a transaction
- Run: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

## P1: Add sync message size cap

File: src-tauri/src/services/sync.rs
- In `handle_sync_connection`, after reading `len` from the network, add: `if len > 100 * 1024 * 1024 { return; }`
- Run: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

## P1: Correct threat model false claim

File: docs/THREAT_MODEL.md
- Line 98: Change "Database is re-encrypted when the app closes" to "Database is re-encrypted when the app closes (P0 fix pending — not yet implemented)"
- Or if the P0 fix is complete, update to describe the new shutdown re-encryption behavior

## P1: Add cargo audit to CI

File: .github/workflows/ci.yml
- Add step after Rust setup: `cargo install cargo-audit && cargo audit --manifest-path src-tauri/Cargo.toml`

## Acceptance criteria

- `cargo test` runs at least 6 tests, all pass
- `cargo clippy -- -D warnings` passes with 0 warnings
- `cargo fmt --check` passes
- `npm run check` passes with 0 errors
- `npm run build` succeeds
- App launches with encryption disabled → normal flow works
- App launches with encryption enabled → unlock prompt appears → unlock succeeds → normal flow works
- App closes with encryption enabled → plaintext DB is deleted → encrypted DB exists
- Sync handshake with length > 100MB is rejected
```

---

## 9. Final Verdict

**Is this codebase ready to ship?**
No. It is MVP-ready — suitable for controlled demos and developer testing, but not for public release or sensitive data. Three P0 issues must be fixed first.

**Top 3 fixes:**
1. Fix encryption unlock persistence (plaintext DB stays on disk permanently)
2. Wire encryption unlock gate into App.svelte (app crashes on restart with encryption)
3. Add DB unit tests (zero tests means regressions are invisible)

**What should be done first?**
The encryption fixes (P0 items 1 and 2) — they are the most impactful security and reliability issues. The encryption feature is currently broken at runtime.

**What should be deferred?**
- Sync authentication/encryption (documented in threat model, accepted for MVP)
- Plugin sandboxing (complex, not blocking)
- a11y warning fixes (cosmetic)
- DB module splitting (maintainability, not blocking)
- Argon2 migration (current KDF is adequate for MVP)

**What remains unverified?**
- `cargo audit` could not be run (not installed) — Rust dependency vulnerabilities are unknown
- Runtime behavior of encryption (app not launched in this audit — static analysis only)
- Cross-platform behavior of `enable_encryption` (file lock issue on Windows is suspected but unverified)
- Mobile build (requires Android SDK, not tested)
- Web clipper functionality (HTTP server not implemented, documented as future)
