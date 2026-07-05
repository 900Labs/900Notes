# 900Notes

A note-taking app that actually works offline. Your notes stay on your computer. No subscriptions, no cloud, no account required.

Free. Forever.

## Why This Exists

Notion costs $10/month. Evernote keeps raising prices. Obsidian is great but sync costs money. If you're a student in Lagos, a researcher in Nairobi, or a small team in Accra, these tools are priced for San Francisco salaries, not for the rest of the world.

900Notes is our answer. It runs on whatever laptop you already have. It doesn't need the internet. Your data lives in a single file on your machine, and you can move it or back it up whenever you want.

Built by [900 Labs](https://www.900labs.com). We make open source tools for people who get priced out of the software they need to do their work.

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

Tauri uses your system's built-in webview instead of shipping a full copy of Chromium. That means the app is smaller, uses less memory, and starts faster. If you're running this on a 4-year-old laptop with 4GB RAM, you'll feel the difference.

## Installation

### Releases
Tagged releases are on the [releases page](https://github.com/900Labs/900Notes/releases/latest) when available. Until we publish pre-built binaries, you can build from source.

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

Everything lives in one SQLite file on your computer. No cloud, no server, no middleman.
- Location: `{APP_DATA_DIR}/900notes.db`
- IDs: UUID v4 so you can create notes offline without conflicts
- Full-text search built on SQLite FTS5

Your data doesn't leave your machine unless you choose to export it.

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md) - system design, data flow, and offline model
- [Threat Model](docs/THREAT_MODEL.md) - security boundaries, mitigations, and residual risks
- [Privacy Model](docs/PRIVACY_MODEL.md) - data inventory and data-flow rules
- [Plugin System](docs/PLUGINS.md) - local plugin format, security notes, and examples
- [Mobile Companion](docs/MOBILE.md) - read-only mobile build architecture and CSP
- [Roadmap](docs/ROADMAP.md) - MVP scope and post-MVP phases
- [Quality Gate](docs/QUALITY_GATE.md) - required pre-merge validation baseline
- [Public Readiness](docs/PUBLIC_READINESS.md) - public beta release hygiene checklist
- [Maintainer Handoff](docs/MAINTAINER_HANDOFF.md) - post-audit remediation summary and review checklist

## Contributing

We welcome contributions from anyone, especially developers in the places this tool is built for. If you're coding from Lagos, Nairobi, Accra, or Mumbai, your contributions make this better for everyone who uses it.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding standards, and the PR process.

Quick contribution ideas:
- Add a translation for your language to `src/i18n/`
- Add a new block type to the ProseMirror schema
- Improve search relevance and snippet formatting
- Report bugs in your operating environment

## License

Apache License 2.0. See [LICENSE](LICENSE) for the full text.
You can use, modify, and distribute this software, including for commercial purposes. You don't owe us anything.

## Security

To report a vulnerability, email security@900labs.com. See [SECURITY.md](SECURITY.md) for the full process.

## Part of the 900 Labs Ecosystem

900Notes is part of the 900 Labs open-source portfolio:

- [900PDF](https://github.com/900-labs/900pdf)
- [900CRM](https://github.com/900-labs/900crm)
- [900Invoice](https://github.com/900Labs/900Invoice)
- [900Word](https://github.com/900Labs/900Word)

All of our tools run on the same Tauri v2 + Rust + Svelte 5 stack. Same conventions, same libraries, same promise: free forever, works offline, open source.

Learn more at [900labs.com/open-source](https://www.900labs.com/open-source).
