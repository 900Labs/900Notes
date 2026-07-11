# Public Readiness

**Status**: Version 1.6.0 public beta released on 2026-07-11.
**Release target**: 1.6.0

This document tracks the release hygiene checks that sit after the 20 planned feature sprints.

## Completed Cleanup

- Sprint docs reconciled with `docs/SPRINT_REVIEW_LOG.md`; all 20 planned feature sprints are recorded as passed.
- Tracked `src/mobile/dist-mobile` build artifacts removed and ignored.
- Tracked `src-tauri/assets/fonts/Roboto-*.ttf` files removed; they were HTML pages from failed downloads, not font binaries.
- Public release gate rewritten to scan tracked files for local paths, high-confidence secrets, generated artifacts, fake binary assets, and i18n coverage.
- Recent web capture, smart view, and graph inspector copy added to all 10 locale dictionaries.
- CI now runs the public release gate and i18n coverage before the normal frontend/Rust checks.
- Stale PDF documentation updated to match the current built-in text PDF writer.
- Web clipper localhost writes now require a per-install token, including non-browser local requests.
- Web clipper startup is unlock-aware for encrypted workspaces.
- Outline panel heading clicks navigate the live editor.
- `package-lock.json` version metadata is synced with `package.json`.
- CI now runs `npm audit` and `npm run build:mobile`; a manual Release Gate workflow packages the macOS app and uploads the DMG/checksum artifact.

## Previous Verification Snapshot

Validated on 2026-07-08:

- `./scripts/verify-public-release.sh` passed. It still prints expected hostname warnings for local-only clipper, sync, and development surfaces.
- `./scripts/verify-local.sh` passed through i18n coverage, frontend type check, frontend build, Rust formatting, Rust clippy, Rust tests, and Rust build.
- `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195` passed with 17 allowed GTK/WebKit/Tauri stack warnings.
- `npm run build:mobile` passed.
- `npm audit` passed with 0 vulnerabilities.
- `npm run tauri:build` produced a macOS app bundle and DMG.
- `git diff --check` passed.

## Version 1.6.0 Release Snapshot

Validated on 2026-07-11:

- The Builder/Reviewer cycle closed all confirmed code findings, including save retry, close coordination, workspace isolation, exact restore, encryption cleanup, and inbound sync shutdown races.
- 14 frontend tests and 43 Rust tests passed.
- Svelte, TypeScript, Rust formatting, Clippy, desktop build, mobile build, npm audit, privacy checks, version checks, i18n checks, and Tauri capability checks passed.
- RustSec reported no blocking advisory with the two documented ignores; 17 upstream desktop-stack warnings remain allowed.
- GitHub-hosted Linux, macOS, and Windows quality gates passed on the merged release commit.
- Unsigned AppImage, DEB, RPM, DMG, MSI, and NSIS packages built successfully and were published with SHA-256 digests.
- The independent Reviewer approved the code and final packaging changes with no remaining findings.
- Public Git history and tags use organization identities. The pre-public repository is kept in a separate private archive.

## Ongoing Release Checks

Native window close must still be tested on each target platform. The automated capability check confirms that the close coordinator can call the Tauri window destroy API, but it cannot prove native close-request behavior on macOS, Windows, or Linux.

```bash
./scripts/verify-public-release.sh
./scripts/verify-local.sh
npm run build:mobile
npm audit
npm run tauri:build
```

Expected caveat: `cargo audit` may still report allowed warnings from the GTK/WebKit/Tauri desktop stack, as documented in `docs/MAINTAINER_HANDOFF.md`. Those warnings are not treated as the removed `lopdf`/`printpdf` blocker.

## Public Positioning

Version 1.6.0 is a public beta. The source and checksummed packages are public, but packages remain unsigned. Remaining work includes plugin sandboxing, stronger passphrase KDF metadata, signed artifacts, and broader platform-specific install verification.
