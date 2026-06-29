# Editor Schema Guide

900Notes uses [ProseMirror](https://prosemirror.net/) as its rich-text editor. The document is stored as JSON in SQLite and rendered in the Svelte frontend.

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  Svelte Component                 │
│              (EditorView.svelte)                  │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │            ProseMirror Editor                │ │
│  │  ┌──────────────┐  ┌──────────────────────┐ │ │
│  │  │   Schema     │  │   Input Rules        │ │ │
│  │  │  (nodes +    │  │  (markdown shortcuts)│ │ │
│  │  │   marks)     │  │                      │ │ │
│  │  └──────────────┘  └──────────────────────┘ │ │
│  │  ┌──────────────┐  ┌──────────────────────┐ │ │
│  │  │   Keymap     │  │   Wiki Link Plugin   │ │ │
│  │  │  (shortcuts) │  │  ([[ autocomplete)   │ │ │
│  │  └──────────────┘  └──────────────────────┘ │ │
│  └─────────────────────────────────────────────┘ │
│                        ↕                          │
│              JSON (doc.toJSON())                  │
│                        ↕                          │
│              Tauri IPC (invoke)                   │
│                        ↕                          │
│              SQLite (content TEXT)                │
└──────────────────────────────────────────────────┘
```

## Schema Definition

The schema is defined in `src/lib/editor/schema.ts`.

### Nodes (Block Types)

| Node | Group | Content | Attributes | Markdown Shortcut |
|------|-------|---------|------------|-------------------|
| `doc` | — | `block+` | — | — |
| `paragraph` | block | `inline*` | — | (default) |
| `heading` | block | `inline*` | `level: 1\|2\|3` | `# `, `## `, `### ` |
| `bullet_list` | block | `list_item+` | — | `- ` or `* ` |
| `ordered_list` | block | `list_item+` | — | `1. ` |
| `list_item` | — | `paragraph block*` | — | (inside list) |
| `todo_item` | block | `inline*` | `checked: boolean` | `[] ` |
| `code_block` | block | `text*` | — | ` ``` ` |
| `blockquote` | block | `block+` | — | `> ` |
| `divider` | block | — | — | `---` |
| `image` | block | — | `src`, `alt`, `title`, `attachmentId` | Paste/drag image |
| `table` | block | `table_row+` | — | (future) |
| `table_row` | — | `table_cell+` | — | — |
| `table_cell` | — | `block+` | — | — |
| `text` | inline | — | — | — |
| `wiki_link` | inline | — | `title: string`, `pageId: string\|null` | `[[title]]` |
| `hard_break` | inline | — | — | `Shift+Enter` |
| `math_inline` | inline | — | `latex: string` | `$...$` |
| `math_block` | block | — | `latex: string` | `$$` |
| `mermaid_block` | block | `text*` | `rendered: boolean` | `~~~` |
| `audio_block` | block | — (atom) | `attachmentId, audioNoteId, duration, title, transcription` | SlashMenu → Audio Note |

### Audio Block

The `audio_block` node is an atom block node that renders an inline audio player. Audio data is stored as a BLOB in the `attachments` table, with metadata in the `audio_notes` table. Recording uses the browser `MediaRecorder` API via the editor action bar or SlashMenu.

### Marks (Inline Formatting)

| Mark | Shortcut | HTML Tag |
|------|----------|----------|
| `bold` | Ctrl/Cmd+B | `<strong>` |
| `italic` | Ctrl/Cmd+I | `<em>` |
| `underline` | Ctrl/Cmd+U | `<u>` |
| `strike` | Ctrl/Cmd+Shift+X | `<s>` |
| `code` | Ctrl/Cmd+`` ` `` | `<code>` |
| `link` | — | `<a href>` |

## Input Rules

Input rules are markdown-style shortcuts that transform text as you type. Defined in `buildInputRules()` in `src/lib/editor/index.ts`.

| Pattern | Action |
|---------|--------|
| `# ` | Convert to H1 |
| `## ` | Convert to H2 |
| `### ` | Convert to H3 |
| `- ` or `* ` | Convert to bullet list |
| `1. ` | Convert to ordered list |
| `[] ` | Convert to todo item |
| ` ``` ` | Convert to code block |
| `> ` | Convert to blockquote |
| `---` | Insert divider |

## Wiki Links

Wiki links use `[[page title]]` syntax. When the user types `[[`, a custom event is dispatched that the `WikiLinkAutocomplete` component listens for. The autocomplete shows matching page titles.

### How wiki links work

1. User types `[[` in the editor
2. The `wikiLink` plugin detects the `[[` pattern via `handleTextInput`
3. A `wiki-link-start` custom event is dispatched on `window`
4. The `WikiLinkAutocomplete` Svelte component shows a dropdown of matching titles
5. User selects a title (or presses Enter for the first match)
6. `insertWikiLink()` replaces the `[[` with a `wiki_link` node containing the title and page ID
7. The link renders as a clickable purple span
8. Clicking the link navigates to the target page

### Backlink resolution

When page content is saved, the Rust backend:
1. Extracts all `[[...]]` patterns from the raw JSON content
2. Matches each link text against page titles (case-insensitive)
3. Inserts rows into the `links` table
4. The backlinks panel queries `links` where `target_page_id` matches the current page

## Editor Commands

All commands are exported from `src/lib/editor/index.ts` and operate on an `EditorView` instance.

| Function | Description |
|----------|-------------|
| `createEditor(el, content, callbacks)` | Create a ProseMirror editor instance |
| `destroyEditor(view)` | Clean up editor instance |
| `insertWikiLink(view, title, pageId)` | Insert a wiki link node at cursor |
| `setHeading(view, level)` | Set current block to heading |
| `setParagraph(view)` | Set current block to paragraph |
| `toggleBulletList(view)` | Wrap selection in bullet list |
| `toggleOrderedList(view)` | Wrap selection in ordered list |
| `toggleBlockquote(view)` | Wrap selection in blockquote |
| `setCodeBlock(view)` | Set current block to code block |
| `insertDivider(view)` | Insert a horizontal rule |
| `insertTodoItem(view)` | Set current block to todo item |
| `toggleBold(view)` | Toggle bold mark |
| `toggleItalic(view)` | Toggle italic mark |
| `toggleUnderline(view)` | Toggle underline mark |
| `toggleStrike(view)` | Toggle strikethrough mark |
| `toggleCode(view)` | Toggle inline code mark |

## CSS Styling

ProseMirror editor styles are in `src/app.css` under the `.ProseMirror` selector. Key styles:

- Headings: H1 (1.875rem, bold), H2 (1.5rem, semibold), H3 (1.25rem, semibold)
- Code blocks: dark background (#1e1e2e), monospace font
- Blockquotes: left border accent, gray text
- Todo items: checkbox pseudo-elements, line-through when checked
- Wiki links: purple (#7c3aed), underlined, hover background
- Scrollbars: custom thin scrollbars with dark mode support
- RTL: mirrored padding and borders for Arabic

## Adding a New Block Type

1. Add the node definition to `schema` in `src/lib/editor/schema.ts`
2. Add an input rule in `buildInputRules()` in `src/lib/editor/index.ts`
3. Add a toolbar button in `src/components/editor/EditorToolbar.svelte`
4. Add a slash menu item in `src/components/editor/SlashMenu.svelte`
5. Add CSS styles in `src/app.css`
6. Add Markdown conversion in `src-tauri/src/services/markdown.rs`
7. Add translation keys in all 10 languages in `src/i18n/index.ts`
