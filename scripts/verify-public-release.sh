#!/usr/bin/env bash
set -euo pipefail

echo "=== 900Notes Public Release Privacy Gate ==="
echo ""

echo "Checking tracked files for local paths, secrets, generated artifacts, and release metadata..."
echo ""

fail_with_matches() {
  local message="$1"
  local matches="$2"
  echo "$matches"
  echo "  ✗ $message"
  exit 1
}

local_paths="$(git grep -n -I -E '/(Users|home)/[^[:space:]\"'\'']+' -- . ':!scripts/verify-public-release.sh' || true)"
if [ -n "$local_paths" ]; then
  fail_with_matches "Found hardcoded local paths" "$local_paths"
fi
echo "  ✓ No hardcoded local paths in tracked files"

secret_tokens="$(git grep -n -I -E 'BEGIN (RSA |EC |OPENSSH |DSA |PRIVATE )?PRIVATE KEY|ghp_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-[A-Za-z0-9]{20,}|xox[abprs]-[A-Za-z0-9-]{20,}|AKIA[0-9A-Z]{16}' -- . ':!scripts/verify-public-release.sh' || true)"
if [ -n "$secret_tokens" ]; then
  fail_with_matches "Found high-confidence secret material" "$secret_tokens"
fi

secret_assignments="$(git grep -n -I -E '(api[_-]?key|secret|password|token)[A-Za-z0-9_ -]*[:=][[:space:]]*['\''\"][^'\''\"]{16,}['\''\"]' -- . ':!scripts/verify-public-release.sh' || true)"
if [ -n "$secret_assignments" ]; then
  fail_with_matches "Found suspicious secret assignments" "$secret_assignments"
fi
echo "  ✓ No high-confidence secrets found"

typographic_dashes="$(git grep -n -I -E '—|–' -- README.md CONTRIBUTING.md SECURITY.md SUPPORT.md GOVERNANCE.md CODE_OF_CONDUCT.md CHANGELOG.md index.html src public docs examples .github || true)"
if [ -n "$typographic_dashes" ]; then
  fail_with_matches "Found prohibited em or en dashes in public-facing source" "$typographic_dashes"
fi
echo "  ✓ No em or en dashes in public-facing source"

hostnames="$(git grep -n -I -E 'localhost|127\.0\.0\.1|0\.0\.0\.0' -- src src-tauri ':!src-tauri/target' ':!src/mobile/dist-mobile' || true)"
if [ -n "$hostnames" ]; then
  echo "$hostnames"
  echo "  ⚠ Hostname references found; expected for local-only clipper/sync/dev surfaces"
fi
echo "  ✓ Hostname check reviewed"

tracked_existing_files="$(git ls-files | while IFS= read -r path; do [ ! -e "$path" ] || printf '%s\n' "$path"; done)"

tracked_artifacts="$(printf '%s\n' "$tracked_existing_files" | grep -E '(^|/)(node_modules|dist|dist-ssr|target)/|^src/mobile/dist-mobile/|^src-tauri/assets/fonts/|\.DS_Store$' || true)"
if [ -n "$tracked_artifacts" ]; then
  fail_with_matches "Found generated or local-only artifacts tracked by git" "$tracked_artifacts"
fi
echo "  ✓ No generated artifacts tracked"

bad_binary_assets=""
while IFS= read -r asset; do
  if file "$asset" | grep -E 'HTML document' >/dev/null; then
    bad_binary_assets="${bad_binary_assets}${asset}\n"
  fi
done <<EOF
$(printf '%s\n' "$tracked_existing_files" | grep -E '\.(ttf|otf|woff|woff2|png|jpe?g|gif|webp|pdf)$' || true)
EOF

if [ -n "$bad_binary_assets" ]; then
  fail_with_matches "Found binary assets that are actually HTML documents" "$(printf "%b" "$bad_binary_assets")"
fi
echo "  ✓ Binary asset MIME check passed"

npm run check:i18n
echo "  ✓ i18n coverage complete"
npm run check:version
echo "  ✓ version consistency complete"
npm run check:capabilities
echo "  ✓ Tauri capability coverage complete"

echo ""
echo "=== Privacy gate passed ✓ ==="
