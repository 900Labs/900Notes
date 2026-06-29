# Contributing to 900Notes

We welcome contributions from developers worldwide — especially those in the regions 900Notes serves. Every line of code from a developer in Lagos, Nairobi, Accra, or Mumbai makes this tool better for the people it's built for.

## Setup

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.88+ | [rustup.rs](https://rustup.rs) |
| Node.js | 20.19+, 22.12+, or 24+ | [nodejs.org](https://nodejs.org) |
| Tauri CLI | v2 | `cargo install tauri-cli --version "^2"` |

### System Dependencies

**macOS**: No additional dependencies required.

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

**Windows**: Install [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

See the full [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/).

### Clone and Run

```bash
git clone https://github.com/900Labs/900Notes.git
cd 900Notes
npm install
cargo tauri dev
```

The app opens in the Tauri webview with hot-reload active for frontend changes. Rust changes trigger a rebuild.

For the full development guide, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Development Workflow

1. Create a branch from `main`: `git checkout -b feature/your-feature`
2. Make your changes
3. Run the quality gate: `./scripts/verify-local.sh`
4. Open a pull request with a clear description

## Coding Standards

### Rust
- Follow `rustfmt` formatting. Run `cargo fmt` before committing.
- No clippy warnings. Run `cargo clippy -- -D warnings`.
- Use `thiserror` for error types. All command handlers return `Result<T, String>`.
- Use `serde` with `#[serde(rename_all = "camelCase")]` for all structs exposed to the frontend.
- UUID v4 for all IDs.
- ISO 8601 (RFC 3339) for all timestamps via `chrono::Utc::now().to_rfc3339()`.

### TypeScript / Svelte
- Use TypeScript for all new files. No plain `.js` files.
- Use Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`) — not Svelte 4 stores.
- Follow the existing pattern: stores in `src/stores/app.svelte.ts`, API calls in `src/lib/api.ts`.
- Use TailwindCSS classes for styling. No inline styles unless dynamic values are needed.
- All UI text must use translation keys from `src/i18n/index.ts`.

### Commits
- Use clear, descriptive messages: `Add graph view component` or `Fix link engine case sensitivity`.
- Squash-merge is the default for PRs.

### Documentation
- Update `docs/API.md` when adding or changing Tauri commands.
- Update `docs/DATABASE.md` when changing the SQLite schema.
- Update `docs/EDITOR.md` when changing the ProseMirror schema.
- Add an ADR in `docs/adr/` for significant architectural decisions.
- Update the README if user-facing behavior changes.

## Pull Request Process

1. Ensure the quality gate passes: `./scripts/verify-local.sh`
2. Fill out the PR template completely
3. Update documentation if behavior changes
4. Add translation keys to all 6 languages if adding UI text
5. Request review from a maintainer

## How to Contribute

### Add a Translation
1. See [docs/I18N.md](docs/I18N.md) for the full guide.
2. Add your language to `src/i18n/index.ts`.
3. Translate all keys from the English (`en`) section.
4. Test by switching to your language in Settings.

### Add a Block Type
1. See [docs/EDITOR.md](docs/EDITOR.md) for the full guide.
2. Add the node to `src/lib/editor/schema.ts`.
3. Add an input rule, toolbar button, slash menu item, and CSS styles.
4. Add Markdown conversion in `src-tauri/src/services/markdown.rs`.
5. Add translation keys for the block name.

### Add a Tauri Command
1. See the "Adding a New Tauri Command" section in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).
2. Add the database method, command handler, registration, and frontend API wrapper.
3. Document the command in `docs/API.md`.

### Report a Bug
Use the Bug Report template on GitHub. Include your OS, 900Notes version, and steps to reproduce. If you're on low-resource hardware, mention your specs — it helps us optimize.

### Suggest a Feature
Use the Feature Request template on GitHub. Explain the problem it solves and who benefits. If the feature is particularly relevant for your region, tell us why.

## Quick Contribution Ideas

- Add a translation for your language to `src/i18n/`
- Add a new block type to the ProseMirror schema
- Improve search relevance and snippet formatting
- Add keyboard shortcuts
- Report bugs in your operating environment
- Improve documentation clarity
- Add ADRs for decisions you make in your PRs

## Code of Conduct

Be respectful. Be inclusive. We are building tools for the majority of the world — contributions from the communities we serve are especially valued.

Harassment, discrimination, or dismissive behavior toward anyone based on their background, location, language proficiency, or hardware limitations will not be tolerated.
