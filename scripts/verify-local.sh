#!/usr/bin/env bash
set -euo pipefail

echo "=== 900Notes Local Quality Gate ==="
echo ""

echo "[1/7] Frontend type check..."
npm run check
echo "  ✓ Type check passed"
echo ""

echo "[2/7] Frontend build..."
npm run build
echo "  ✓ Frontend build passed"
echo ""

echo "[3/7] Rust format check..."
cargo fmt --check --manifest-path src-tauri/Cargo.toml
echo "  ✓ Format check passed"
echo ""

echo "[4/7] Rust clippy..."
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
echo "  ✓ Clippy passed"
echo ""

echo "[5/7] Rust tests..."
cargo test --manifest-path src-tauri/Cargo.toml
echo "  ✓ Tests passed"
echo ""

echo "[6/7] Rust build..."
cargo build --manifest-path src-tauri/Cargo.toml
echo "  ✓ Build passed"
echo ""

echo "[7/7] Cargo audit..."
cargo audit --manifest-path src-tauri/Cargo.toml
echo "  ✓ Audit passed"
echo ""

echo "=== All checks passed ✓ ==="
