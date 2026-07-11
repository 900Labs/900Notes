# 900Notes

900Notes is a free, open source notes editor that works offline. It stores your notes on your computer, opens without an account, and does not require a subscription.

## Why we built it

Useful software should not become unavailable because someone cannot justify another monthly bill. Many students, researchers, independent workers, and small teams use older laptops or unreliable internet connections. Most popular notes apps are designed around permanent connectivity and recurring payments.

900 Labs builds practical open source tools for people who get priced out of the software they need. 900Notes is part of that mission: a capable knowledge workspace that runs locally, works without the internet, and leaves the user in control of the data.

Free. Forever.

## What it does

- Rich text notes with headings, lists, tasks, code blocks, quotes, math, Mermaid diagrams, images, files, and audio
- Nested pages, trash and restore, favorites, page history, templates, and daily notes
- `[[page links]]`, backlinks, related pages, and knowledge graphs
- Tags, tag groups, tag filters, full text search, saved searches, and smart folders
- Exact JSON workspace backup and restore, Markdown and PDF export, and encrypted sharing bundles
- Imports from Markdown, Notion exports, Obsidian vaults, Evernote ENEX, and Roam JSON
- Translation dictionaries for ten languages, with right-to-left layout support for Arabic and Urdu
- Optional per-workspace encryption and optional local network sync
- No telemetry and no hosted 900Notes service

## Install

Download the public-beta package for your operating system from the [Releases page](https://github.com/900Labs/900Notes/releases). Version 1.6.0 provides macOS Apple Silicon, Windows x64, and Linux x64 packages.

The packages are not signed or notarized yet. Your operating system may show an untrusted-publisher warning. Check the SHA-256 values in the release notes before using an override. You can also build the app from source.

### Run from source

Requirements:

- Node.js 22 or newer
- Rust 1.88 or newer
- The [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system

```bash
git clone https://github.com/900Labs/900Notes.git
cd 900Notes
npm ci
npm run tauri:dev
```

Build a desktop package:

```bash
npm run tauri:build
```

## First steps

1. Select **New Page** and start writing. Changes save automatically.
2. Type `/` to insert a block or use Markdown-style shortcuts such as `# `, `- `, and `> `.
3. Type `[[` and choose another page to create a link. Open Backlinks or Graph View to explore connections.
4. Add tags below the editor. Select a tag in the sidebar to see matching pages.
5. Open **Settings > Data** to make a workspace backup. Restoring a backup exactly replaces the active workspace after confirmation.
6. Open **Settings > Security** if you want to encrypt the active workspace. Keep the passphrase safe because it cannot be recovered.

## Data and privacy

Each workspace uses a local SQLite database in the operating system's application data directory. Attachments are stored in the same database and are limited to 25 MB per file. A workspace backup includes notes, trash, tags, properties, attachments, audio metadata, custom templates, searches, smart folders, favorites, tag groups, revisions, and settings. It does not include passphrases, clipper tokens, sync state, or plugin code.

900Notes does not send telemetry. Network access occurs only when the user starts a local sync session or uses an explicit capture or sharing feature. See [SECURITY.md](SECURITY.md), [docs/PRIVACY_MODEL.md](docs/PRIVACY_MODEL.md), and [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) for the boundaries.

Encryption protects a workspace at rest after a clean shutdown. While the app is unlocked it uses a temporary plaintext database. If the app or computer stops unexpectedly, that recovery copy is retained so newer notes are not overwritten. It is removed after the next successful unlock and clean shutdown.

## Backups

The version 1.6 backup format uses exact replace semantics. Importing a valid backup replaces all user-created data in the active workspace inside one database transaction. Older partial-export files are rejected rather than imported ambiguously. Keep a second copy of important backups on storage you control.

## Development

```bash
npm ci
npm test
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
./scripts/verify-public-release.sh
```

The complete local gate is `./scripts/verify-local.sh`. Architecture and contributor details are in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Community

Bug reports and focused feature proposals are welcome. Please read [SUPPORT.md](SUPPORT.md), [CONTRIBUTING.md](CONTRIBUTING.md), [GOVERNANCE.md](GOVERNANCE.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before opening an issue or pull request.

Security problems should be reported privately using the process in [SECURITY.md](SECURITY.md), not in a public issue.

## License

Apache License 2.0. See [LICENSE](LICENSE).
