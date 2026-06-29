# Maintainer Handoff

**Date**: 2026-06-29  
**Audience**: Repo maintainer and original builder  
**Scope**: Production-readiness audit remediation for 900Notes

## Summary

This handoff covers the verified audit findings fixed after the 2026-06-29 codebase audit. The changes are intentionally scoped: they harden existing features without redesigning the app.

Primary outcome: the project no longer has the verified audit blockers around unauthenticated LAN sync, mobile stored-content XSS, plugin path traversal, vulnerable PDF dependency chain, broken cargo-audit command, unbounded attachment BLOBs, page hierarchy cycles, or unsafe search snippets.

## Behavior Changes

| Area | Change | User Impact |
|------|--------|-------------|
| LAN sync | Starting sync now requires a 12+ character pairing secret. Peers must use the same secret. Sync handshakes are encrypted with AES-256-GCM. | Users need to enter the same secret on each trusted device. |
| Mobile reader | Stored note content is parsed into typed reader blocks instead of rendered as raw HTML. | Malicious note text renders as text, not executable markup. |
| Mobile CSP | Mobile Tauri config now has a restrictive CSP and no `unsafe-eval`. | Mobile companion remains read-only and plugin-free. |
| Plugins | Rust derives the plugin root from `app_data_dir`, validates plugin IDs and entry paths, canonicalizes reads, and rejects escapes. | Plugins must live under `<app_data_dir>/plugins/<plugin-id>/`; `./index.js`, absolute paths, and `..` are rejected. |
| PDF export | Removed `printpdf`; export now uses a small built-in text PDF writer. | PDF export remains available for text content; advanced typography/images are intentionally not handled. |
| Attachments | Attachment BLOBs larger than 25 MB are rejected in Rust and prechecked in image/audio insertion paths. | Very large files fail fast instead of consuming memory/disk. |
| Page tree | Moving a page under itself, a descendant, or a missing/deleted parent is rejected. | Invalid drag/drop moves now fail instead of corrupting the page tree. |
| Search | FTS schema is repaired on startup if an older contentless table exists; snippets are escaped before `<mark>` tags are restored. | Search results should contain snippets and avoid rendering stored HTML. |
| Quality gate | `cargo audit` now uses `--file src-tauri/Cargo.lock`. | Local/CI audit checks the actual Tauri lockfile. |

## Files To Review

### Security and Runtime Fixes

- `src-tauri/src/services/sync.rs`
- `src-tauri/src/commands/sync.rs`
- `src/components/settings/SettingsModal.svelte`
- `src/stores/app.svelte.ts`
- `src/lib/api.ts`
- `src-tauri/src/commands/plugins.rs`
- `src/lib/plugins/loader.ts`
- `src/stores/plugins.svelte.ts`
- `src/mobile/MobileApp.svelte`
- `src-tauri/tauri.mobile.conf.json`

### Reliability and Supply Chain Fixes

- `src-tauri/src/db/mod.rs`
- `src/components/search/SearchPalette.svelte`
- `src/components/search/CommandPalette.svelte`
- `src-tauri/src/services/pdf.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src/lib/editor/index.ts`
- `.github/workflows/ci.yml`
- `scripts/verify-local.sh`

### Documentation

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/THREAT_MODEL.md`
- `docs/PRIVACY_MODEL.md`
- `docs/PLUGINS.md`
- `docs/DATABASE.md`
- `docs/API.md`
- `docs/MOBILE.md`
- `docs/QUALITY_GATE.md`
- `docs/FULL_AUDIT_REPORT.md`
- `docs/MAINTAINER_HANDOFF.md`

## Verification Commands

Run these before merging:

```bash
npm run check
npm run build
npm run build:mobile
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
cargo audit --file src-tauri/Cargo.lock
npm audit
./scripts/verify-local.sh
```

Expected caveats from the current toolchain:

- `npm run build` and `npm run build:mobile` pass, but Vite reports the existing `@import` order warning for `katex/dist/katex.min.css`.
- `npm run build` passes, but Vite reports existing large chunk warnings.
- `cargo audit --file src-tauri/Cargo.lock` exits successfully and no longer reports the removed `lopdf`/`printpdf` vulnerability. It still reports 17 allowed warnings from GTK3/unmaintained transitive crates and `glib` via the desktop webview stack.

## Regression Tests Added

| Module | Coverage |
|--------|----------|
| `src-tauri/src/services/sync.rs` | Encrypted handshake round trip and wrong-secret rejection |
| `src-tauri/src/commands/plugins.rs` | Plugin ID and entry-path validation |
| `src-tauri/src/db/mod.rs` | Attachment size rejection, page move cycle rejection, escaped highlighted search snippets |
| `src-tauri/src/services/pdf.rs` | PDF export smoke test for valid header/catalog/font |

## Manual QA Checklist

- Start sync on two local instances with different pairing secrets; verify sync fails.
- Start sync on two local instances with the same pairing secret; verify pages sync.
- Paste a note containing `<img src=x onerror=alert(1)>`, open it in the mobile companion, and verify it displays as text.
- Place an example plugin under `<app_data_dir>/plugins/wordcount/index.js`; scan and load it.
- Try a plugin manifest with `entryPoint: "../outside.js"` and verify it is rejected.
- Drag a parent page under its child and verify the operation fails without losing the page.
- Search for content containing `<img ...>` and verify snippets escape markup while preserving `<mark>` highlights.
- Attach an image under 25 MB and verify it renders.
- Try attaching a file over 25 MB and verify it is rejected.
- Export a page to PDF and verify the file opens.

## Remaining Follow-Ups

These are not part of the verified P0/P1 remediation, but they are the next useful hardening steps:

1. Replace text sync secrets with QR-code pairing or a PAKE/Noise-based pairing flow.
2. Sandbox plugins in a worker or iframe and add a permission model before treating third-party plugins as safe.
3. Migrate passphrase key derivation from iterative SHA-256 to Argon2id or PBKDF2 with versioned metadata.
4. Add frontend unit tests and Playwright smoke coverage for startup, editor save, search, settings, and mobile reader.
5. Add release runbook coverage for artifact signing, checksums, and platform-specific install verification.

## Acceptance Criteria For This Remediation

- All verification commands above pass.
- `cargo audit --file src-tauri/Cargo.lock` reports no vulnerable `lopdf`/`printpdf` path.
- Sync without the shared pairing secret cannot deserialize a handshake.
- Mobile stored content is never rendered through raw `{@html}`.
- Plugin file reads cannot escape `<app_data_dir>/plugins/<plugin-id>/`.
- Search snippets never emit stored user markup as active HTML.
- Active reference docs no longer describe LAN sync as plaintext or list `printpdf` as a dependency.
