# Public Readiness

**Status**: Public beta ready after this cleanup is committed.
**Last readiness sprint**: 2026-07-05

This document tracks the release hygiene checks that sit after the 20 planned feature sprints.

## Completed Cleanup

- Sprint docs reconciled with `docs/SPRINT_REVIEW_LOG.md`; all 20 planned feature sprints are recorded as passed.
- Tracked `src/mobile/dist-mobile` build artifacts removed and ignored.
- Tracked `src-tauri/assets/fonts/Roboto-*.ttf` files removed; they were HTML pages from failed downloads, not font binaries.
- Public release gate rewritten to scan tracked files for local paths, high-confidence secrets, generated artifacts, fake binary assets, and i18n coverage.
- Recent web capture, smart view, and graph inspector copy added to all 10 locale dictionaries.
- CI now runs the public release gate and i18n coverage before the normal frontend/Rust checks.
- Stale PDF documentation updated to match the current built-in text PDF writer.

## Latest Verification

Validated on 2026-07-05:

- `./scripts/verify-public-release.sh` passed. It still prints expected hostname warnings for local-only clipper, sync, and development surfaces.
- `./scripts/verify-local.sh` passed through i18n coverage, frontend type check, frontend build, Rust formatting, Rust clippy, Rust tests, and Rust build.
- `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195` passed with 17 allowed GTK/WebKit/Tauri stack warnings.
- `npm run build:mobile` passed.
- `npm audit` passed with 0 vulnerabilities.
- `git diff --check` passed.

## Required Before Making Public

```bash
./scripts/verify-public-release.sh
./scripts/verify-local.sh
npm run build:mobile
npm audit
```

Expected caveat: `cargo audit` may still report allowed warnings from the GTK/WebKit/Tauri desktop stack, as documented in `docs/MAINTAINER_HANDOFF.md`. Those warnings are not treated as the removed `lopdf`/`printpdf` blocker.

## Public Positioning

This repository is suitable for a public beta, not an enterprise-ready release. The remaining hardening work is plugin sandboxing, stronger passphrase KDF metadata, automated frontend/E2E smoke coverage, artifact signing, checksums, and platform-specific install verification.
