#!/usr/bin/env bash
set -euo pipefail

echo "=== 900Notes Public Release Privacy Gate ==="
echo ""

echo "Checking for local paths, hostnames, secrets, and generated artifacts..."
echo ""

# Check for hardcoded local paths
if grep -r "/Users/" src/ src-tauri/src/ --include="*.ts" --include="*.svelte" --include="*.rs" --include="*.js" 2>/dev/null; then
  echo "  ✗ Found hardcoded local paths"
  exit 1
fi
echo "  ✓ No hardcoded local paths"

# Check for secrets
if grep -riE "(api_key|secret|password|token)\s*=" src/ src-tauri/src/ --include="*.ts" --include="*.svelte" --include="*.rs" --include="*.js" 2>/dev/null | grep -v "test\|example\|placeholder"; then
  echo "  ✗ Found potential secrets"
  exit 1
fi
echo "  ✓ No secrets found"

# Check for hostnames
if grep -r "localhost\|127.0.0.1\|0.0.0.0" src/ src-tauri/src/ --include="*.ts" --include="*.svelte" --include="*.rs" --include="*.js" 2>/dev/null | grep -v "vite.config\|tauri.conf\|test\|comment"; then
  echo "  ⚠ Found hostname references (review manually)"
fi
echo "  ✓ Hostname check complete"

# Check no generated artifacts are committed
if [ -d "src-tauri/target" ]; then
  echo "  ⚠ src-tauri/target exists (should be gitignored)"
fi
if [ -d "dist" ]; then
  echo "  ⚠ dist exists (should be gitignored)"
fi
echo "  ✓ Artifact check complete"

echo ""
echo "=== Privacy gate passed ✓ ==="
