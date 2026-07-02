# Quality Gate

Run the local quality gate before opening a pull request:

```bash
./scripts/verify-local.sh
```

## Checks

1. **Frontend type check**: `npm run check` (svelte-check + tsc)
2. **Frontend build**: `npm run build` (vite production build)
3. **Rust format**: `cargo fmt --check` (no formatting changes)
4. **Rust lints**: `cargo clippy -- -D warnings` (no warnings)
5. **Rust tests**: `cargo test` (all tests pass)
6. **Rust build**: `cargo build` (compiles without errors)
7. **Cargo audit**: `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195`

The temporary RustSec ignores cover `quick-xml 0.39.4`, which is pulled transitively through `plist -> tauri`. `plist 1.9.0` currently pins `quick-xml ^0.39.2`, so `quick-xml >=0.41.0` is not selectable yet. Remove the ignores when the upstream dependency chain moves.

## Pre-merge Requirements

- All checks pass
- No new compiler warnings
- Documentation updated if behavior changes
- Translation keys added to all 10 languages if UI text was added
