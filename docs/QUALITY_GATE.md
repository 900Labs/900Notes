# Quality Gate

Run the local quality gate before opening a pull request:

```bash
./scripts/verify-local.sh
```

Run the public release gate before making the repository public or tagging a public release:

```bash
./scripts/verify-public-release.sh
```

## Checks

1. **Public release gate**: `./scripts/verify-public-release.sh`
2. **i18n coverage**: `npm run check:i18n`
3. **Frontend type check**: `npm run check` (svelte-check + tsc)
4. **Frontend build**: `npm run build` (vite production build)
5. **Rust format**: `cargo fmt --check` (no formatting changes)
6. **Rust lints**: `cargo clippy -- -D warnings` (no warnings)
7. **Rust tests**: `cargo test` (all tests pass)
8. **Rust build**: `cargo build` (compiles without errors)
9. **Cargo audit**: `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195`

The temporary RustSec ignores cover `quick-xml 0.39.4`, which is pulled transitively through `plist -> tauri`. `plist 1.9.0` currently pins `quick-xml ^0.39.2`, so `quick-xml >=0.41.0` is not selectable yet. Current Tauri desktop dependencies may also produce allowed unmaintained-crate warnings from the GTK/WebKit stack. Remove the ignores when the upstream dependency chain moves.

## Pre-merge Requirements

- All checks pass
- No new compiler warnings
- Documentation updated if behavior changes
- Translation keys added to all 10 languages if UI text was added
- No generated build output, local-only artifacts, fake binary assets, or high-confidence secrets are tracked
