# Public Release Audit

Release target: 1.6.0

Status: Version 1.6.0 public beta released on 2026-07-11. Local and GitHub-hosted gates, the independent Builder/Reviewer cycle, sanitized public history, and unsigned package builds are green.

This audit focuses on whether people can install, understand, and trust 900Notes as an open source notes editor. It gives priority to note integrity, workspace isolation, backup portability, accurate documentation, and repeatable project checks.

## Release blockers addressed

- Editor saves keep the page ID that produced the edit. A delayed save cannot be redirected into a newly opened page.
- Save failures are visible and pending edits flush during navigation and component shutdown.
- Encrypted workspaces retain the newest recovery copy after an interrupted session and refresh the encrypted snapshot on a clean shutdown.
- Passphrase changes preserve the only usable database and update the active in-memory passphrase.
- Startup opens the workspace selected in the registry. Workspace changes clear and reload workspace-scoped frontend, sync, and CRDT state.
- Wiki links use canonical ProseMirror nodes. Backlink indexing also understands legacy `[[title]]` text.
- Workspace backups include user-created notes and trash, tags, properties, attachments, audio, templates, searches, smart folders, favorites, tag groups, revisions, and settings.
- Backup restore validates the format and version, warns the user, and performs exact replacement in a transaction.
- Tag filters show matching pages.
- External importers report partial failures instead of presenting partial work as a complete success.
- Workspace deletion requires confirmation.

## Automated evidence

The release gate covers:

- Frontend unit tests
- Svelte and TypeScript checks
- Desktop and mobile production builds
- Rust formatting, Clippy, unit tests, and build
- npm and Rust dependency audits
- Translation-key parity across ten locales
- Scans for local computer paths, high-confidence secrets, generated artifacts, and invalid binary assets
- CI smoke tests on Linux, macOS, and Windows

Run `./scripts/verify-local.sh` and `./scripts/verify-public-release.sh` before creating a release tag.

The 2026-07-11 release run passed 14 frontend tests, 43 Rust tests, Svelte and TypeScript checks, Rust formatting and Clippy, desktop and mobile builds, npm audit, RustSec audit with the documented upstream warnings, privacy checks, and Linux, macOS, and Windows package builds. The independent Reviewer reported no remaining code findings. The public release contains checksummed unsigned AppImage, DEB, RPM, DMG, MSI, and NSIS packages. The macOS DMG SHA-256 is `6ceb817291dd96e5ae87fc3bbe5fc750bd7cb066c832601618d5b49d44e3369c`.

## Ongoing manual checks

1. Create and edit two pages quickly, switch between them, restart, and verify both contain the correct text.
2. Create a workspace, switch to it, restart, and verify it remains active.
3. Back up a disposable workspace containing a trashed page, property, favorite, tag, and attachment. Restore it and compare the result.
4. Enable encryption, edit a note, force-stop the app, unlock it, and verify the latest edit remains.
5. Change the passphrase and verify the old passphrase fails after a clean restart.
6. Create a wiki link from autocomplete and verify backlinks and graph edges.
7. Select a tag and verify only matching pages appear.
8. Import a deliberately mixed valid and invalid external export and verify errors are visible.
9. Install each unsigned release candidate on its target platform and document the operating system warning.

## Known boundaries

- Release packages require maintainer signing and notarization credentials before they can be presented as trusted signed downloads.
- Encryption uses iterative SHA-256 rather than Argon2id. The limitation is documented and a versioned KDF migration remains future work.
- An interrupted unlocked encrypted session leaves a plaintext recovery copy until the next successful unlock and clean shutdown. This favors note recovery after a crash.
- Plugins execute with application access and should only be installed from trusted source code.
- Local network sync is opt-in and should only be used on trusted networks with a strong shared pairing secret.
