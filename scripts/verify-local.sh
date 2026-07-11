#!/usr/bin/env bash
set -euo pipefail

echo "=== 900Notes Local Quality Gate ==="
echo ""

echo "[1/9] i18n coverage..."
npm run check:i18n
echo "  ✓ i18n coverage passed"
npm run check:version
echo "  ✓ version consistency passed"
npm run check:capabilities
echo "  ✓ Tauri capability coverage passed"
echo ""

echo "[2/9] Frontend unit tests..."
npm test
echo "  ✓ Frontend unit tests passed"
echo ""

echo "[3/9] Frontend type check..."
npm run check
echo "  ✓ Type check passed"
echo ""

echo "[4/9] Frontend build..."
npm run build
echo "  ✓ Frontend build passed"
echo ""

echo "[5/9] Rust format check..."
cargo fmt --check --manifest-path src-tauri/Cargo.toml
echo "  ✓ Format check passed"
echo ""

echo "[6/9] Rust clippy..."
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
echo "  ✓ Clippy passed"
echo ""

echo "[7/9] Rust tests..."
cargo test --manifest-path src-tauri/Cargo.toml
echo "  ✓ Tests passed"
echo ""

echo "[8/9] Rust build..."
cargo build --manifest-path src-tauri/Cargo.toml
echo "  ✓ Build passed"
echo ""

echo "[9/9] Cargo audit..."
# quick-xml is pulled transitively through plist -> tauri. plist 1.9.0
# currently pins quick-xml ^0.39.2, so quick-xml >=0.41.0 is not selectable yet.
# Remove these ignores once the upstream chain moves.
cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195
echo "  ✓ Audit passed"
echo ""

echo "=== All checks passed ✓ ==="
