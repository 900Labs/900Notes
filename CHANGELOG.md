# Changelog

This project follows [Semantic Versioning](https://semver.org/).

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
