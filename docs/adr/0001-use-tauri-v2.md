# ADR-0001: Use Tauri v2 for Desktop Shell

- **Status**: accepted
- **Date**: 2026-06-28
- **Deciders**: 900 Labs team
- **Context**: MVP

---

## Context

900Notes needs a desktop shell that works on Windows, macOS, and Linux with minimal resource usage. The target users are in developing economies, often running older hardware with 4GB RAM. Traditional Electron apps bundle Chromium (~150MB) and consume significant memory. We need a lighter alternative that still provides a modern web-based UI.

Constraints:
- Must work offline with no cloud dependencies
- Must run on low-resource hardware (4GB RAM, older CPUs)
- Must produce small binaries suitable for download on slow connections
- Must support Windows, macOS, and Linux
- Must be open source (Apache 2.0 compatible)

## Decision

Use **Tauri v2** as the desktop shell. Tauri uses the system's native webview (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux) instead of bundling Chromium.

## Consequences

### Positive
- Binary size: ~10-20MB vs ~150MB+ for Electron
- Memory usage: significantly lower (no bundled Chromium process)
- Startup time: faster (native webview is already loaded by the OS)
- Security: Tauri's IPC model is more restrictive than Electron's Node.js integration
- Consistent with the 900 Labs ecosystem (900Invoice, 900Word use Tauri v2)

### Negative
- Webview differences across platforms (WebKitGTK on Linux may lag in CSS support)
- Tauri v2 is newer with a smaller community than Electron
- Some web APIs may not be available in all webviews

### Neutral
- Rust backend is required (not a negative for us — we want Rust)
- Frontend must be a web framework (Svelte 5 in our case)

## Alternatives Considered

1. **Electron** — Rejected due to large binary size (~150MB), high memory usage, and Chromium dependency. Not suitable for low-resource hardware.

2. **Native app (Qt/GTK)** — Rejected due to development speed. Building a rich-text editor in native toolkits is significantly more effort than using ProseMirror in a webview.

3. **PWA (Progressive Web App)** — Rejected because it requires a browser and doesn't provide reliable local file system access. Also can't guarantee offline operation on all platforms.

4. **Flutter Desktop** — Rejected due to smaller desktop ecosystem and lack of Rust backend integration. Also requires bundling the Flutter engine (~20MB).

## References

- [Tauri v2 documentation](https://v2.tauri.app/)
- [Tauri vs Electron comparison](https://v2.tauri.app/conceptual/introduction/)
- 900Invoice and 900Word repos (same stack)
