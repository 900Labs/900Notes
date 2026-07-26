# Changelog

This project follows [Semantic Versioning](https://semver.org/).

## Unreleased

### Security

- Replaced the iterative SHA-256 key derivation with the memory-hard Argon2id KDF for workspace encryption, encrypted share bundles, and the LAN sync transport. Legacy data remains readable and is upgraded on the next passphrase change or re-export (ENC-2).
- Hardened encrypted-workspace unlock against swapped recovery files: the leftover plaintext is now authenticated with an HMAC sidecar bound to the passphrase and snapshot; a file that fails the check is discarded and re-derived from the snapshot instead of being trusted (ENC-1).
- Enforced a 12-character minimum for workspace passphrases on enable and change, matching the sync pairing policy (ENC-3).
- Share bundle imports no longer overwrite local pages. Imported page ids that collide with existing local pages are remapped to fresh UUIDs, with parent links, tags, and properties rewritten, so a crafted bundle cannot replace local content via timestamps (SHARE-1).
- Blocked `javascript:`, `data:`, and other dangerous schemes in editor link hrefs and HTML export via a scheme allowlist (XSS-1).
- Tightened the desktop Content-Security-Policy to drop `unsafe-eval` and `unsafe-inline` from `script-src`, matching mobile (CSP-1).
- Disabled the unfinished plugin runtime (it relied on `eval`) and surfaced an experimental notice in Settings; plugin management remains available (PLUGIN-1).

### Changed

- LAN sync now persists a stable device identity across restarts instead of regenerating it on every start, and `get_sync_status` reports the real device id/name/port (SYNC-2).
- `sync_with_peer` now detects real page conflicts (same id, different updated timestamps) instead of silently applying last-write-wins, and returns them for the UI without overwriting the local copy (SYNC-1).
- Externalized the Plugins and Data settings strings for all ten locales (I18N-1).
- Clarified the README install section with an explicit SHA-256 verification command (REL-1).

### Fixed

- Web clipper now returns the correct `423 Locked` reason phrase instead of falling through to `500` (CLIP-1).
- Resolved high-severity PostCSS and DOMPurify npm advisories via `npm audit fix` (DEP-1).

### Tests

- Added Rust coverage for Argon2id derivation, the encrypted-DB integrity tag, passphrase validation, share-import ID remapping, and the export href allowlist.
- Added a frontend test for the link href sanitizer.

## 1.6.0

### Added

- Exact, versioned workspace backup and transactional restore for user-created data
- Frontend unit tests and cross-platform CI smoke checks
- Public support, governance, conduct, and release documentation

### Changed

- Workspace startup and switching now respect the active workspace and clear workspace-scoped state
- Wiki links use the editor's structured node format while retaining legacy `[[text]]` compatibility
- Tag selection now shows matching pages
- External importers report partial failures
- Encryption and passphrase handling preserve the newest usable database across interrupted sessions

### Fixed

- Prevented a delayed save from writing one page's content into another page
- Fixed passphrase rotation cleanup and in-memory passphrase state
- Added confirmation before workspace deletion and exact backup restore
- Aligned version and feature descriptions across the app and documentation

## 1.5.0

- Previous public beta feature set.
