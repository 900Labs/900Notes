# Maintainer Handoff for 1.6.0

## Release intent

Version 1.6.0 is the first public-readiness release. It concentrates on keeping notes attached to the correct page, preserving data through crashes and workspace changes, providing a complete backup, and making repository claims match the visible product.

## Review priorities

Review these paths with particular care:

- `src/components/editor/EditorView.svelte` and `src/lib/debounced-save.ts` for page-bound save ordering
- `src-tauri/src/services/encryption.rs` and `src-tauri/src/commands/encryption.rs` for recovery-copy and replacement behavior
- `src-tauri/src/commands/workspace.rs` and `src/stores/app.svelte.ts` for workspace switching and state reset
- `src-tauri/src/services/export_import.rs` and generic backup row handling in `src-tauri/src/db/mod.rs`
- Structured wiki-link insertion in `src/lib/editor/index.ts` and extraction in `src-tauri/src/db/mod.rs`
- Restore and deletion confirmations in `src/components/settings/SettingsModal.svelte`
- Public feature and privacy claims in `README.md`

## Required verification

```bash
npm ci
npm audit --audit-level=moderate
npm run check:i18n
npm test
npm run check
npm run build
npm run build:mobile
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
./scripts/verify-public-release.sh
```

The broader local gate also runs the Rust dependency audit:

```bash
./scripts/verify-local.sh
```

## Release procedure

1. Complete the manual checks in `docs/AUDIT_REPORT.md` on a disposable workspace.
2. Run the release gate on macOS, Windows, and Linux.
3. Configure platform signing and macOS notarization in repository secrets. Do not place credentials in the repository. See [docs/CODE_SIGNING.md](CODE_SIGNING.md) for the required secrets and the signed release workflow.
4. Build release artifacts from the exact 1.6.0 commit.
5. Record SHA-256 checksums and installation results.
6. Create the annotated `v1.6.0` tag only after the reviewed commit and checks are final.
7. Publish the changelog with the artifacts and checksums.

Do not reuse or move an older release tag.
