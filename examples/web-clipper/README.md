# 900Notes Web Clipper

A Chrome/Firefox extension that saves web content as notes in 900Notes.

## Features

- **Clip full pages**: Save the entire page content as a new note
- **Clip selections**: Right-click selected text to clip just that portion
- **Keyboard shortcut**: `Ctrl+Shift+S` (or `Cmd+Shift+S` on Mac) to clip the current page
- **Configurable port**: Set the 900Notes local server port in the popup

## Installation

### Chrome/Edge

1. Open `chrome://extensions/`
2. Enable "Developer mode" (top right)
3. Click "Load unpacked"
4. Select the `examples/web-clipper/` directory
5. The extension icon will appear in your toolbar

### Firefox

1. Open `about:debugging#/runtime/this-firefox`
2. Click "Load Temporary Add-on"
3. Select the `manifest.json` file in the `examples/web-clipper/` directory

## Usage

1. Navigate to any web page
2. Either:
   - Click the extension icon and click "Clip This Page"
   - Select text, right-click, and choose "Clip to 900Notes"
   - Press `Ctrl+Shift+S` (or `Cmd+Shift+S`)
3. The page content will be saved as a new note in 900Notes

## How It Works

1. The **content script** (`content.js`) extracts page text and builds a ProseMirror JSON document
2. The **background script** (`background.js`) sends the clipped content to the 900Notes local server
3. The 900Notes app receives the clip and creates a new page

## Architecture

```
Browser Page → content.js (extract content)
             → background.js (send to 900Notes)
             → 900Notes local server (create page)
```

## Files

| File | Description |
|------|-------------|
| `manifest.json` | Extension manifest (MV3) |
| `background.js` | Service worker — handles commands, context menu, and HTTP requests |
| `content.js` | Content script — extracts page content and builds ProseMirror doc |
| `popup.html` | Popup UI for settings and manual clipping |
| `popup.js` | Popup logic — save port settings, trigger clip |

## Notes

- The extension communicates with 900Notes via a local HTTP endpoint
- The default port is 1420 (matching the Vite dev server)
- Icons are placeholders — replace with actual PNG icons before publishing
