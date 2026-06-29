# 900Notes Mobile Companion

A read-only mobile companion app for 900Notes, built with Tauri and Svelte.

## Overview

The mobile companion provides a lightweight, read-only viewer for your 900Notes workspace. It shares the same Tauri backend and SQLite database as the desktop app, but renders a simplified mobile-optimized UI.

## Features

- **Page list**: Browse recent pages and all pages
- **Search**: Filter pages by title
- **Reader view**: Rendered view of page content (headings, paragraphs, lists, code blocks, blockquotes, images, todo items)
- **No editing**: The mobile app is intentionally read-only to keep the bundle small and the UI simple

## Development

```bash
# Run mobile dev server (frontend only)
npm run dev:mobile

# Run mobile Tauri app (desktop preview in mobile viewport)
npm run tauri:mobile:dev

# Build mobile app
npm run tauri:mobile:build
```

## Architecture

- **Entry point**: `src/mobile/index.html` → `src/mobile/main.ts` → `src/mobile/MobileApp.svelte`
- **Vite config**: `vite.mobile.config.ts` (separate from desktop config)
- **Tauri config**: `src-tauri/tauri.mobile.conf.json` (mobile viewport, separate identifier)
- **Shared backend**: Same Tauri commands and SQLite database as desktop
- **Shared stores**: Uses the same Svelte stores (`pageStore`, `settingsStore`) and i18n

## Future Work

- Tauri Mobile (iOS/Android) when stable
- Offline sync with the desktop app
- Edit support (with conflict resolution)
- Dark mode support
- Touch-optimized gestures
