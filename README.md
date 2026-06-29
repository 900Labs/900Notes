# 900Notes

Knowledge base and note-taking with bidirectional linking. Your second brain, fully owned.

Free. Offline. Yours.

Website • Features • Installation • Documentation • Contributing

## The Problem

A student in Lagos can't afford Notion's $10/month subscription. A researcher in Nairobi loses access to their notes when the internet drops. A small team in Accra has no way to build a shared knowledge base without sending their data to servers they don't control. Every existing note-taking tool either requires a subscription, a constant internet connection, or both.

Knowledge workers in developing economies are locked out of the tools that make modern knowledge work possible. Their ideas, research, and institutional memory are trapped in scattered documents and lost notebooks — not because they lack the skills, but because the tools are priced for San Francisco, not for where the majority of the world actually lives and works.

## The Solution

900Notes is a desktop knowledge base that works completely offline. It runs on the hardware you already own. No subscriptions. No cloud dependencies. No internet required after the first download.

Built by [900 Labs](https://www.900labs.com) — building enterprise-grade open source tools for the 900 million+ people in developing economies who are priced out of the software that modern businesses depend on.

## Features

### Note & Page Management
- Create, edit, delete, and duplicate notes
- Nested page hierarchy with drag-and-drop reordering
- Page titles, emoji icons, and cover colors
- Page pinning for quick access
- Trash with soft-delete and restore
- Auto-save with recovery drafts

### Rich-Text WYSIWYG Editor
- Block-based editor (ProseMirror) with Notion-like experience
- Block types: headings, paragraphs, lists, to-dos, code blocks, blockquotes, dividers, tables
- Inline formatting: bold, italic, underline, strikethrough, inline code
- Slash command menu (`/`) for block insertion
- Markdown shortcuts (type `#`, `-`, `>`, `[]`, ` ``` ` etc.)
- Keyboard shortcuts for common actions
- Paste sanitization

### Bidirectional Linking
- `[[page name]]` syntax auto-converted to links
- Autocomplete dropdown when typing `[[`
- Backlinks panel showing all pages linking to the current page
- Click links to navigate between pages
- Case-insensitive title matching

### Tags
- Add/remove colored tags on any page
- Tag autocomplete
- Filter pages by tag in sidebar
- Create, rename, and delete tags

### Full-Text Search
- Instant search powered by SQLite FTS5
- Results with snippet previews and highlighted matches
- Search by title and content
- Keyboard shortcut (Ctrl/Cmd+K) to open search

### Data Storage & Portability
- All data in a single SQLite file at `{APP_DATA_DIR}/900notes.db`
- UUID v4 IDs for offline-safe creation
- Export entire workspace to JSON
- Export individual pages to Markdown
- Import from JSON (round-trip restore)
- Import from Markdown files
- Attachment storage is capped at 25 MB per file to protect low-memory devices
- No telemetry, no network calls

### Internationalization
- 10 languages: English, French, Spanish, Swahili, Hindi, Arabic (RTL), Portuguese, Bengali, Urdu (RTL), Amharic
- Locale-aware formatting
- RTL layout support for Arabic and Urdu

### Settings
- Theme: light, dark, or system
- Language selection
- Editor preferences (font size, line spacing)
- Data location display
- Export/import controls

### Security & Encryption
- AES-256-GCM database encryption at rest
- Iterative SHA-256 key derivation (100,000 rounds) with CSPRNG salts
- Encryption unlock gate — app requires passphrase on startup when encrypted
- Plaintext database re-encrypted and deleted on app shutdown
- Encrypted share bundles and workspace export
- Content Security Policy (CSP) hardening
- Secure delete (overwrite before delete)

### LAN Sync
- Peer-to-peer sync over local network (mDNS + TCP)
- Opt-in — only activates when explicitly started
- Requires the same 12+ character pairing secret on each device
- Sync handshakes are encrypted with AES-256-GCM using the pairing secret
- Sync message size cap (100MB) to prevent DoS

### Accessibility
- Full ARIA roles and labels
- Keyboard navigation support
- Skip-to-content link
- RTL layout for Arabic

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Backend | Rust |
| Frontend | Svelte 5 (Runes) |
| Database | SQLite (local, single file) |
| Editor | ProseMirror (block-based WYSIWYG) |
| Full-text search | SQLite FTS5 |
| Styling | TailwindCSS |
| License | Apache 2.0 |

### Why Tauri v2?

Tauri uses the system's native webview instead of bundling Chromium. The result: smaller binaries, lower memory usage, and faster startup. On a 4-year-old laptop with 4 GB of RAM running three browser tabs, this difference is everything.

## Installation

### Releases
Tagged releases are published on the [releases page](https://github.com/900Labs/900Notes/releases/latest) when available. Until platform binaries are published, build from source.

### Build from Source
Prerequisites:
- Rust 1.88+ — install from [rustup.rs](https://rustup.rs)
- Node.js 20.19+, 22.12+, or 24+ — install from [nodejs.org](https://nodejs.org)
- Tauri CLI v2: `cargo install tauri-cli --version "^2"`
- Tauri v2 system dependencies — see [v2.tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
# Clone the repository
git clone https://github.com/900Labs/900Notes.git
cd 900Notes

# Install frontend dependencies
npm install

# Run in development mode (hot-reload)
cargo tauri dev

# Build for production
cargo tauri build
```

Production app bundles and installers are written under `src-tauri/target/release/bundle/`.

## Project Structure

```
900Notes/
├── src/                          # Svelte 5 frontend
│   ├── components/               # UI components by feature
│   │   ├── editor/               # ProseMirror editor, toolbar, slash menu
│   │   ├── sidebar/              # Page tree, tags, recent, trash
│   │   ├── search/               # Search palette
│   │   ├── backlinks/            # Backlinks panel
│   │   ├── settings/             # Settings modal
│   │   └── common/               # Shared UI primitives
│   ├── stores/                   # Svelte 5 Runes state stores
│   ├── i18n/                     # Translation files (10 languages)
│   ├── lib/                      # API wrapper, types, editor schema
│   └── utils/                    # Date, search utilities
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri IPC command handlers
│       ├── models/               # Data structures (Page, Tag, Link)
│       ├── db/                   # SQLite schema, migrations, queries, FTS5
│       └── services/             # Export/import, markdown conversion
├── docs/                         # Documentation
│   └── adr/                      # Architecture Decision Records
├── scripts/                      # Validation and release scripts
├── .github/                      # CI/CD workflows and templates
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE
└── README.md
```

## Data Storage

All data is stored locally in a single SQLite file. No cloud. No server.
- Location: `{APP_DATA_DIR}/900notes.db`
- IDs: UUID v4 for offline-safe creation
- Full-text search: SQLite FTS5 virtual table

Your data never leaves your machine unless you explicitly export it.

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md) — system design, data flow, and offline model
- [Threat Model](docs/THREAT_MODEL.md) — security boundaries, mitigations, and residual risks
- [Privacy Model](docs/PRIVACY_MODEL.md) — data inventory and data-flow rules
- [Plugin System](docs/PLUGINS.md) — local plugin format, security notes, and examples
- [Mobile Companion](docs/MOBILE.md) — read-only mobile build architecture and CSP
- [Roadmap](docs/ROADMAP.md) — MVP scope and post-MVP phases
- [Quality Gate](docs/QUALITY_GATE.md) — required pre-merge validation baseline
- [Maintainer Handoff](docs/MAINTAINER_HANDOFF.md) — post-audit remediation summary and review checklist

## Contributing

We welcome contributions from developers worldwide — especially those in the regions 900Notes serves. Every line of code from a developer in Lagos, Nairobi, Accra, or Mumbai makes this tool better for the people it's built for.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding standards, and the PR process.

Quick contribution ideas:
- Add a translation for your language to `src/i18n/`
- Add a new block type to the ProseMirror schema
- Improve search relevance and snippet formatting
- Report bugs in your operating environment

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.
You are free to use, modify, and distribute this software — including commercially. You do not owe us anything.

## Security

To report a vulnerability, email security@900labs.com. See [SECURITY.md](SECURITY.md) for the full process.

## Part of the 900 Labs Ecosystem

900Notes is part of the 900 Labs open-source portfolio:

- [900PDF](https://github.com/900-labs/900pdf)
- [900CRM](https://github.com/900-labs/900crm)
- [900Invoice](https://github.com/900Labs/900Invoice)
- [900Word](https://github.com/900Labs/900Word)

All tools are built on the same Tauri v2 + Rust + Svelte 5 stack. They share conventions, libraries, and the same commitment: free forever, offline-first, open source.

Learn more at [900labs.com/open-source](https://www.900labs.com/open-source).
