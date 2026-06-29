# Plugin System

900Notes supports local plugins that extend the editor with custom blocks, commands, and hooks.

## Architecture

- **Backend (Rust)**: Scans a `plugins/` directory in the app data folder, reads `plugin.json` manifests, and stores plugin metadata in SQLite.
- **Frontend (TypeScript)**: Dynamically loads plugin JS files, provides a `PluginApi` for registering custom blocks, commands, and hooks.
- **Custom Blocks**: Plugin-defined ProseMirror node specs that can be merged into the editor schema.
- **Settings UI**: A "Plugins" tab in Settings allows enabling/disabling and removing plugins.

## Plugin Structure

Each plugin lives in its own directory under `<app_data_dir>/plugins/<plugin-id>/`:

```
plugins/
  com.example.callout/
    plugin.json    # Manifest
    index.js       # Entry point
  com.example.wordcount/
    plugin.json
    index.js
```

## Manifest (`plugin.json`)

```json
{
  "id": "com.example.callout",
  "name": "Callout Block",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Adds a callout block type.",
  "entryPoint": "index.js",
  "customBlocks": [
    {
      "nodeType": "callout",
      "group": "block",
      "content": "inline*",
      "attrs": { "variant": { "default": "info" } },
      "toDom": "(node) => ['div', { class: 'callout' }, 0]",
      "parseDom": "[{ tag: 'div.callout' }]",
      "icon": "💡",
      "label": "Callout"
    }
  ]
}
```

### Manifest Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique plugin identifier (reverse-DNS recommended) |
| `name` | string | Display name |
| `version` | string | Semver version |
| `author` | string | Author name |
| `description` | string | Short description |
| `entryPoint` | string | JS file name (relative to plugin dir) |
| `customBlocks` | array | Custom block definitions (see below) |

### Custom Block Definition

| Field | Type | Description |
|-------|------|-------------|
| `nodeType` | string | ProseMirror node type name |
| `group` | string | Node group (usually `block`) |
| `content` | string \| null | Content expression (e.g. `inline*`) |
| `attrs` | object | Node attributes with defaults |
| `toDom` | string | JS function body for `toDOM` (returns array) |
| `parseDom` | string \| null | JS array literal for `parseDOM` rules |
| `icon` | string \| null | Emoji or icon for slash menu |
| `label` | string | Display label for slash menu |

## Plugin API

The entry point JS file receives a `plugin` object with these methods:

```js
// Register a custom block type
plugin.registerBlock({
  nodeType: 'callout',
  group: 'block',
  content: 'inline*',
  attrs: { variant: { default: 'info' } },
  toDom: "(node) => ['div', { class: 'callout' }, 0]",
  parseDom: "[{ tag: 'div.callout' }]",
  icon: '💡',
  label: 'Callout',
})

// Register a command (appears in command palette)
plugin.registerCommand('myCommand', 'My Command', () => {
  console.log('Command executed')
})

// Register a hook
plugin.registerHook('pageSave', (pageId) => {
  console.log('Page saved:', pageId)
})
```

### Available Hooks

| Event | Args | Description |
|-------|------|-------------|
| `pageSave` | `(pageId)` | Fired after a page is saved |
| `pageCreate` | `(pageId)` | Fired after a new page is created |
| `pageDelete` | `(pageId)` | Fired before a page is deleted |
| `editorInit` | `(editor)` | Fired after the editor is initialized |

## Installation

1. Place plugin directories in `<app_data_dir>/plugins/`
2. Open Settings → Plugins
3. Click "Scan for Plugins"
4. Toggle plugins on/off as needed

## Example Plugins

See `examples/plugins/`:

- **callout** — Adds a callout block type with variants (info, warning, tip, danger)
- **wordcount** — Adds a word count command and logs word count on save

## Security

- Plugins run in the webview context with access to the DOM and Tauri APIs
- Plugins are loaded via `new Function()` (not ES modules) for simplicity
- Only enable plugins from trusted sources
- Future: sandboxed plugin execution, permission system
