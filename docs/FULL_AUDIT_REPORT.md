# Codebase Audit Report

**Date**: 2026-06-29
**Status**: Post-remediation audit status
**Primary handoff**: See [MAINTAINER_HANDOFF.md](MAINTAINER_HANDOFF.md)

## Executive Summary

The 2026-06-29 production-readiness audit found blockers in LAN sync authentication/encryption, mobile stored-content rendering, plugin file access, dependency vulnerability posture, attachment bounds, page hierarchy validation, FTS search snippets, and quality-gate configuration.

Those verified P0/P1 blockers have been remediated in this workspace. The project is no longer in the pre-remediation state described by the original audit notes.

Public-readiness follow-up on 2026-07-05 removed tracked generated mobile artifacts, removed failed Roboto downloads that were HTML pages rather than fonts, reconciled sprint documentation, and added tracked-file release checks plus i18n coverage enforcement.

## Fixed Findings

| Finding | Severity | Status | Primary Files |
|---------|----------|--------|---------------|
| LAN sync accepted unauthenticated LAN clients and exchanged page metadata without encryption | High | Fixed | `src-tauri/src/services/sync.rs`, `src-tauri/src/commands/sync.rs`, `src/components/settings/SettingsModal.svelte` |
| Mobile reader rendered stored note content through raw HTML and mobile CSP was disabled | High | Fixed | `src/mobile/MobileApp.svelte`, `src-tauri/tauri.mobile.conf.json` |
| Plugin file helper accepted caller-controlled app data directory and unsafe paths | High | Fixed | `src-tauri/src/commands/plugins.rs`, `src/lib/api.ts`, `src/lib/plugins/loader.ts` |
| Rust dependency audit found vulnerable `lopdf` via direct `printpdf` dependency | High | Fixed | `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/services/pdf.rs` |
| Local/CI cargo-audit command used an invalid manifest flag | Medium | Fixed | `scripts/verify-local.sh`, `.github/workflows/ci.yml` |
| Attachments were stored as unbounded SQLite BLOBs | Medium | Fixed | `src-tauri/src/db/mod.rs`, `src/lib/editor/index.ts` |
| Page moves could create self/descendant parent cycles | Medium | Fixed | `src-tauri/src/db/mod.rs` |
| Search used an FTS shape that could produce empty/broken snippets and inserted raw `<mark>` tags before escaping | Medium | Fixed | `src-tauri/src/db/mod.rs` |

## Verification Baseline

At minimum, maintainers should run:

```bash
npm run check
npm run check:i18n
npm run build
npm run build:mobile
./scripts/verify-public-release.sh
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195
npm audit
```

`./scripts/verify-local.sh` runs the core local gate except `npm audit` and the mobile build.

## Residual Risks

- Plugin JavaScript still runs in the desktop webview through `new Function()`. This is now documented and path-confined, but not sandboxed.
- Sync still relies on a user-entered text pairing secret. Use a strong unique secret; future work should use QR-code pairing or a PAKE/Noise-based flow.
- Database encryption still exposes plaintext while unlocked because the app needs a working SQLite database during the session.
- Key derivation is iterative SHA-256, not Argon2/PBKDF2.

## Current Readiness

After the above fixes and passing verification, the expected readiness target is **Beta-ready** for public open-source release. It is not yet Enterprise-ready because plugin sandboxing, stronger password KDFs, richer observability, automated frontend/E2E tests, and operational release runbooks remain future work.
