# API Reference

Complete Tauri command reference for 900Notes. All commands are invoked from the frontend via `@tauri-apps/api/core` `invoke()`.

## Pages

### `create_page`
Create a new page.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.parentId` | `string \| null` | No | Parent page ID for nesting |
| `input.title` | `string` | Yes | Page title |
| `input.content` | `string` | No | JSON ProseMirror document (defaults to empty paragraph) |
| `input.icon` | `string \| null` | No | Emoji icon |

**Returns**: `Page`

### `get_page`
Get a single page by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Page ID |

**Returns**: `Page`

### `get_all_pages`
Get all non-deleted pages, ordered by `sort_order`.

**Returns**: `Page[]`

### `get_page_tree`
Get all pages as a nested tree structure.

**Returns**: `PageTreeNode[]`

### `update_page`
Update one or more fields on a page. Only provided fields are updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Page ID |
| `input.title` | `string` | No | New title |
| `input.content` | `string` | No | New JSON ProseMirror document |
| `input.icon` | `string \| null` | No | New emoji icon |
| `input.coverColor` | `string \| null` | No | Cover color |
| `input.pinned` | `boolean` | No | Pin state |

**Returns**: `Page`

> When `content` is updated, the link engine automatically rebuilds outgoing links for that page.

### `delete_page`
Soft-delete a page (moves to trash).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Page ID |

**Returns**: `void`

### `restore_page`
Restore a soft-deleted page from trash.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Page ID |

**Returns**: `Page`

### `duplicate_page`
Create a copy of a page with all its tags.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Page ID |

**Returns**: `Page` (the new copy, titled `"{original title} (copy)"`)

### `move_page`
Move a page to a new parent and/or change its sort order.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Page ID |
| `input.parentId` | `string \| null` | Yes | New parent ID (null for root) |
| `input.sortOrder` | `number` | Yes | New sort position |

**Returns**: `Page`

### `get_recent_pages`
Get recently edited pages.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | `number` | No | Max results (default: 10) |

**Returns**: `Page[]`

### `get_trash`
Get all soft-deleted pages, ordered by deletion date.

**Returns**: `Page[]`

### `empty_trash`
Permanently delete all pages in trash.

**Returns**: `void`

### `search_pages`
Full-text search across page titles and content using SQLite FTS5.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | `string` | Yes | Search query |
| `limit` | `number` | No | Max results (default: 50) |

**Returns**: `SearchResult[]` (with snippet previews, `<mark>` highlighted matches)

---

## Tags

### `get_all_tags`
Get all tags, ordered by name.

**Returns**: `Tag[]`

### `create_tag`
Create a new tag.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.name` | `string` | Yes | Tag name (must be unique) |
| `input.color` | `string \| null` | No | Hex color string |

**Returns**: `Tag`

### `update_tag`
Update a tag's name or color.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Tag ID |
| `input.name` | `string` | No | New name |
| `input.color` | `string \| null` | No | New color |

**Returns**: `Tag`

### `delete_tag`
Delete a tag. Removes all page-tag associations.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Tag ID |

**Returns**: `void`

### `get_page_tags`
Get all tags assigned to a page.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |

**Returns**: `Tag[]`

### `set_page_tags`
Replace all tags on a page with the provided list.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |
| `tagIds` | `string[]` | Yes | Array of tag IDs |

**Returns**: `void`

---

## Links

### `get_backlinks`
Get all pages that link to the given page.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Target page ID |

**Returns**: `Backlink[]` (includes source page title, icon, and link text)

### `get_outgoing_links`
Get all links originating from the given page.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Source page ID |

**Returns**: `Link[]`

### `rebuild_links`
Rebuild the entire link table by scanning all page content for `[[wiki links]]`.

**Returns**: `void`

> This is automatically called when page content is updated. Use this command manually after bulk imports.

---

## Settings

### `get_setting`
Get a single setting value.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `key` | `string` | Yes | Setting key |

**Returns**: `string | null`

### `set_setting`
Set a setting value (upsert).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `key` | `string` | Yes | Setting key |
| `value` | `string` | Yes | Setting value |

**Returns**: `void`

### `get_all_settings`
Get all settings as key-value pairs.

**Returns**: `[string, string][]`

### Default Settings

| Key | Default | Values |
|-----|---------|--------|
| `theme` | `system` | `light`, `dark`, `system` |
| `language` | `en` | `en`, `fr`, `es`, `sw`, `hi`, `ar` |
| `font_size` | `16` | `12`–`24` |
| `line_spacing` | `1.5` | `1.0`–`2.5` |
| `default_block_type` | `paragraph` | any block type name |

---

## Export & Import

### `export_workspace`
Export the entire workspace (all pages, tags, page-tag associations, settings) as a JSON string.

**Returns**: `string` (JSON)

### `import_workspace`
Import a workspace from a JSON string. Replaces existing data (INSERT OR REPLACE).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `data` | `string` | Yes | JSON workspace export |

**Returns**: `number` (count of imported pages)

### `export_page_markdown`
Export a single page as Markdown text.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |

**Returns**: `string` (Markdown with `# {title}` header)

### `import_markdown`
Import a Markdown document as a new page.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | `string` | Yes | Page title |
| `content` | `string` | Yes | Markdown content |
| `parentId` | `string \| null` | No | Parent page ID |

**Returns**: `Page`

---

## Page Properties

### `get_page_properties`
Get all custom properties for a page, ordered by sort_order.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |

**Returns**: `PageProperty[]`

### `set_page_property`
Set (create or update) a property on a page. If a property with the same key exists, it is updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.pageId` | `string` | Yes | Page ID |
| `input.key` | `string` | Yes | Property key (e.g., "Author", "Status") |
| `input.value` | `string` | Yes | Property value |

**Returns**: `PageProperty`

### `delete_page_property`
Delete a property by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Property ID |

**Returns**: `void`

---

## Templates

### `get_all_templates`
Get all templates (built-in and custom), ordered by category then name.

**Returns**: `Template[]`

### `create_template`
Create a new custom template.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.name` | `string` | Yes | Template name |
| `input.icon` | `string` | Yes | Emoji icon |
| `input.content` | `string` | Yes | JSON ProseMirror document |
| `input.category` | `string` | Yes | Category (e.g., "custom") |

**Returns**: `Template`

### `update_template`
Update an existing template. Only provided fields are updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Template ID |
| `input.name` | `string` | No | New name |
| `input.icon` | `string` | No | New icon |
| `input.content` | `string` | No | New content |
| `input.category` | `string` | No | New category |

**Returns**: `Template`

### `delete_template`
Delete a template by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Template ID |

**Returns**: `void`

### `create_page_from_template`
Create a new page using a template's content and icon.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `templateId` | `string` | Yes | Template ID |
| `title` | `string` | Yes | Page title |
| `parentId` | `string \| null` | No | Parent page ID |

**Returns**: `Page`

### `get_or_create_daily_note`
Get today's daily note, or create it if it doesn't exist. Auto-links to the previous day's note.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `date` | `string` | Yes | Date in YYYY-MM-DD format |

**Returns**: `Page`

---

## Graph

### `get_graph_data`
Get all pages and links as graph nodes and edges for visualization.

**Returns**: `GraphData`

```typescript
{
  nodes: GraphNode[],   // one per non-deleted page
  edges: GraphEdge[],   // one per distinct link
}
```

---

## Types

### `Page`
```typescript
{
  id: string
  parentId: string | null
  title: string
  content: string          // JSON ProseMirror document
  icon: string | null      // emoji
  coverColor: string | null
  createdAt: string        // ISO 8601
  updatedAt: string        // ISO 8601
  deletedAt: string | null // ISO 8601 or null
  pinned: boolean
  sortOrder: number
}
```

### `PageTreeNode`
```typescript
{
  page: Page
  children: PageTreeNode[]
}
```

### `Tag`
```typescript
{
  id: string
  name: string
  color: string | null     // hex color
  createdAt: string        // ISO 8601
}
```

### `SearchResult`
```typescript
{
  id: string
  title: string
  snippet: string          // HTML with <mark> highlights
  icon: string | null
  updatedAt: string        // ISO 8601
}
```

### `Backlink`
```typescript
{
  id: string
  sourcePageId: string
  sourcePageTitle: string
  sourcePageIcon: string | null
  linkText: string
  createdAt: string        // ISO 8601
}
```

### `Link`
```typescript
{
  id: string
  sourcePageId: string
  targetPageId: string
  linkText: string
  createdAt: string        // ISO 8601
}
```

### `PageProperty`
```typescript
{
  id: string
  pageId: string
  key: string              // property name
  value: string            // property value
  sortOrder: number        // position in the list
}
```

### `Template`
```typescript
{
  id: string
  name: string
  icon: string             // emoji
  content: string          // JSON ProseMirror document
  category: string         // "built-in" or "custom"
  createdAt: string        // ISO 8601
  updatedAt: string        // ISO 8601
}
```

### `GraphNode`
```typescript
{
  id: string
  title: string
  icon: string | null
  tagCount: number         // number of tags on this page
  linkCount: number        // number of links to/from this page
  createdAt: string        // ISO 8601
  updatedAt: string        // ISO 8601
}
```

### `GraphEdge`
```typescript
{
  source: string           // source page ID
  target: string           // target page ID
}
```

### `GraphData`
```typescript
{
  nodes: GraphNode[]
  edges: GraphEdge[]
}
```

---

## Saved Searches

### `get_all_saved_searches`
Get all saved searches, ordered by pinned first then name.

**Returns**: `SavedSearch[]`

### `create_saved_search`
Create a new saved search.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.name` | `string` | Yes | Display name |
| `input.query` | `string` | Yes | FTS query text |
| `input.tagFilter` | `string \| null` | No | Optional tag filter |
| `input.pinned` | `boolean` | Yes | Pin to top |

**Returns**: `SavedSearch`

### `update_saved_search`
Update a saved search. Only provided fields are updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Search ID |
| `input.name` | `string?` | No | New name |
| `input.query` | `string?` | No | New query |
| `input.tagFilter` | `string \| null?` | No | New tag filter |
| `input.pinned` | `boolean?` | No | New pinned state |

**Returns**: `SavedSearch`

### `delete_saved_search`
Delete a saved search.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Search ID |

**Returns**: `void`

### `execute_saved_search`
Run a saved search and return matching results.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Search ID |

**Returns**: `SearchResult[]`

---

## Smart Folders

### `get_all_smart_folders`
Get all smart folders, ordered by sort_order then name.

**Returns**: `SmartFolder[]`

### `create_smart_folder`
Create a new smart folder.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.name` | `string` | Yes | Display name |
| `input.icon` | `string` | Yes | Emoji icon |
| `input.rules` | `string` | Yes | JSON array of `SmartFolderRule` |

**Returns**: `SmartFolder`

### `update_smart_folder`
Update a smart folder. Only provided fields are updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Folder ID |
| `input.name` | `string?` | No | New name |
| `input.icon` | `string?` | No | New icon |
| `input.rules` | `string?` | No | New rules JSON |
| `input.sortOrder` | `number?` | No | New sort order |

**Returns**: `SmartFolder`

### `delete_smart_folder`
Delete a smart folder.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Folder ID |

**Returns**: `void`

### `get_smart_folder_pages`
Get all pages matching a smart folder's rules, returned as a tree.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Folder ID |

**Returns**: `PageTreeNode[]`

---

## Additional Types

### `SavedSearch`
```typescript
{
  id: string
  name: string
  query: string
  tagFilter: string | null
  pinned: boolean
  createdAt: string        // ISO 8601
  updatedAt: string        // ISO 8601
}
```

### `SmartFolder`
```typescript
{
  id: string
  name: string
  icon: string             // emoji
  rules: string            // JSON array of SmartFolderRule
  sortOrder: number
  createdAt: string        // ISO 8601
  updatedAt: string        // ISO 8601
}
```

### `SmartFolderRule`
```typescript
{
  field: string            // "tag" | "title" | "created_after" | "created_before" | "has_property"
  operator: string         // "equals" | "contains" | "starts_with"
  value: string
}
```

---

## Tag Groups

### `get_all_tag_groups`
Get all tag groups, ordered by sort_order then name.

**Returns**: `TagGroup[]`

### `create_tag_group`
Create a new tag group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.name` | `string` | Yes | Group name |
| `input.color` | `string \| null` | No | Hex color |

**Returns**: `TagGroup`

### `update_tag_group`
Update a tag group. Only provided fields are updated.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.id` | `string` | Yes | Group ID |
| `input.name` | `string?` | No | New name |
| `input.color` | `string \| null?` | No | New color |
| `input.sortOrder` | `number?` | No | New sort order |

**Returns**: `TagGroup`

### `delete_tag_group`
Delete a tag group (cascades to tag_group_members).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Group ID |

**Returns**: `void`

### `add_tag_to_group`
Add a tag to a group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `groupId` | `string` | Yes | Group ID |
| `tagId` | `string` | Yes | Tag ID |

**Returns**: `void`

### `remove_tag_from_group`
Remove a tag from a group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `groupId` | `string` | Yes | Group ID |
| `tagId` | `string` | Yes | Tag ID |

**Returns**: `void`

### `get_tags_in_group`
Get all tags in a specific group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `groupId` | `string` | Yes | Group ID |

**Returns**: `Tag[]`

### `get_ungrouped_tags`
Get all tags not assigned to any group.

**Returns**: `Tag[]`

---

## Related Pages

### `get_related_pages`
Get pages related to the given page, scored by shared tags, backlinks, and forward links.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |
| `limit` | `number?` | No | Max results (default: 10) |

**Returns**: `RelatedPage[]`

### `RelatedPage`
```typescript
{
  id: string
  title: string
  icon: string | null
  score: number
  reasons: string[]      // ["shared tags", "backlink", "link"]
}
```

### `TagGroup`
```typescript
{
  id: string
  name: string
  color: string | null
  sortOrder: number
  createdAt: string
  updatedAt: string
}
```

---

## Attachments

### `create_attachment`
Store a file (image or any attachment) as a BLOB in SQLite.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input.pageId` | `string` | Yes | Owning page ID |
| `input.fileName` | `string` | Yes | Original file name |
| `input.mimeType` | `string` | Yes | MIME type |
| `input.data` | `number[]` | Yes | File content as byte array |

**Returns**: `Attachment`

### `get_attachment`
Retrieve attachment metadata and raw file data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Attachment ID |

**Returns**: `{ attachment: Attachment, data: number[] }`

### `get_attachments_for_page`
List all attachments for a page (metadata only, no BLOB data).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `pageId` | `string` | Yes | Page ID |

**Returns**: `Attachment[]`

### `delete_attachment`
Delete an attachment and its BLOB data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | `string` | Yes | Attachment ID |

**Returns**: `void`

### `Attachment`
```typescript
{
  id: string
  pageId: string
  fileName: string
  mimeType: string
  fileSize: number
  isImage: boolean
  createdAt: string
}
```

### PDF Export & OCR Commands

#### `export_page_pdf`
Exports a single page to PDF format.
- **Args**: `pageId: string`
- **Returns**: `number[]` (PDF bytes as array of bytes)

#### `export_workspace_pdf`
Exports all non-deleted pages to a single PDF document.
- **Args**: none
- **Returns**: `number[]` (PDF bytes as array of bytes)

#### `ocr_attachment`
Extracts text from an image attachment using Tesseract OCR.
- **Args**: `attachmentId: string`
- **Returns**: `string` (extracted text)
- **Errors**: Returns error message if Tesseract is not installed or OCR fails

### Audio Notes Commands

#### `create_audio_note`
Creates a new audio note record associated with an attachment.
- **Args**: `input: CreateAudioNoteInput` (`pageId`, `attachmentId`, `durationSec`, `title`)
- **Returns**: `AudioNote`

#### `get_audio_note`
Retrieves a single audio note by ID.
- **Args**: `id: string`
- **Returns**: `AudioNote`

#### `get_audio_notes_for_page`
Lists all audio notes for a page.
- **Args**: `pageId: string`
- **Returns**: `AudioNote[]`

#### `update_audio_note`
Updates an audio note's title and/or transcription.
- **Args**: `input: UpdateAudioNoteInput` (`id`, `title?`, `transcription?`)
- **Returns**: `AudioNote`

#### `delete_audio_note`
Deletes an audio note record (does not delete the associated attachment).
- **Args**: `id: string`
- **Returns**: `void`

### Sync Commands

#### `start_sync`
Starts local network sync — mDNS discovery + TCP server.
- **Args**: `deviceName?: string`, `port?: number` (default 9876), `pairingSecret: string` (minimum 12 characters)
- **Returns**: `SyncStatus` (`enabled`, `deviceId`, `deviceName`, `port`, `peers`, `lastSync`)
- **Details**: Registers `_900notes._tcp.local.` mDNS service, starts TCP listener, begins browsing for peers, and encrypts sync handshakes with AES-256-GCM using the pairing secret. Peers must use the same pairing secret.

#### `stop_sync`
Stops all sync activity — shuts down mDNS daemon and TCP server.
- **Args**: none
- **Returns**: `void`

#### `get_sync_status`
Returns current sync status and discovered peers.
- **Args**: none
- **Returns**: `SyncStatus`

#### `sync_with_peer`
Initiates a sync exchange with a specific discovered peer.
- **Args**: `peerId: string`
- **Returns**: `SyncConflict[]` (list of conflicts where local and remote timestamps differ)
- **Details**: Connects via TCP, exchanges page metadata, upserts newer pages (last-write-wins)

#### `sync_page_to_crdt`
Upserts a single page into the Automerge CRDT document and persists the doc to the `sync_state` table.
- **Args**: `pageId: string`
- **Returns**: `void`

#### `get_pending_sync_count`
Returns the number of pending operations in the sync queue.
- **Args**: none
- **Returns**: `number` (i64)

#### `apply_crdt_to_db`
Reads all pages from the Automerge CRDT document and upserts them into the SQLite database. Also updates the `last_sync` setting.
- **Args**: none
- **Returns**: `number` (count of pages applied)

#### `export_share_bundle`
Exports selected pages as an AES-256-GCM encrypted bundle (JSON with pages, tags, page_tags, properties).
- **Args**: `pageIds: string[]`, `passphrase: string`
- **Returns**: `string` (JSON-encoded `EncryptedBundle` with salt, nonce, ciphertext)

#### `import_share_bundle`
Imports pages from an encrypted share bundle. Requires the correct passphrase.
- **Args**: `encryptedData: string`, `passphrase: string`
- **Returns**: `number` (count of pages imported)

#### `export_page_html`
Exports a single page as a standalone HTML document with inline CSS.
- **Args**: `pageId: string`
- **Returns**: `string` (complete HTML document)

#### `export_pages_html`
Exports multiple pages as a single HTML document with navigation.
- **Args**: `pageIds: string[]`
- **Returns**: `string` (complete HTML document with nav)

#### `list_workspaces`
Lists all registered workspaces.
- **Args**: none
- **Returns**: `Workspace[]` (id, name, dbPath, createdAt, isDefault)

#### `get_active_workspace`
Returns the currently active workspace.
- **Args**: none
- **Returns**: `Workspace`

#### `create_workspace`
Creates a new workspace with its own SQLite database.
- **Args**: `name: string`
- **Returns**: `Workspace`

#### `delete_workspace`
Deletes a workspace and its database file. Cannot delete the default or active workspace.
- **Args**: `id: string`
- **Returns**: `void`

#### `rename_workspace`
Renames an existing workspace.
- **Args**: `id: string`, `name: string`
- **Returns**: `Workspace`

#### `switch_workspace`
Switches to a different workspace. Swaps the active database connection.
- **Args**: `id: string`
- **Returns**: `string` (workspace ID)

#### `is_encryption_enabled`
Checks if database encryption at rest is enabled.
- **Args**: none
- **Returns**: `boolean`

#### `enable_encryption`
Encrypts the SQLite database file at rest using AES-256-GCM with the given passphrase. WAL is checkpointed first, then the plain DB is encrypted and removed.
- **Args**: `passphrase: string`
- **Returns**: `void`

#### `unlock_database`
Decrypts the encrypted database to the plain DB path and swaps the active connection. Used on app startup when encryption is enabled.
- **Args**: `passphrase: string`
- **Returns**: `boolean` (true if unlock succeeded)

#### `disable_encryption`
Decrypts the database and removes encryption. The plain DB is opened after.
- **Args**: `passphrase: string`
- **Returns**: `void`

#### `change_passphrase`
Re-encrypts the database with a new passphrase. Requires the old passphrase.
- **Args**: `oldPassphrase: string`, `newPassphrase: string`
- **Returns**: `void`

#### `verify_passphrase`
Checks if a passphrase is correct without unlocking the database.
- **Args**: `passphrase: string`
- **Returns**: `boolean`

#### `export_encrypted_workspace`
Exports the entire workspace as a base64-encoded AES-256-GCM encrypted blob.
- **Args**: `passphrase: string`
- **Returns**: `string` (base64-encoded encrypted data)

#### `import_encrypted_workspace`
Imports an encrypted workspace export. Requires the passphrase used during export.
- **Args**: `encryptedData: string`, `passphrase: string`
- **Returns**: `number` (count of pages imported)

#### `secure_delete_page`
Permanently deletes a page by overwriting its content/title with zeros, purging all related data (revisions, links, tags, properties, attachments, audio notes), then running VACUUM to reclaim disk space. This cannot be undone.
- **Args**: `id: string`
- **Returns**: `void`

#### `secure_empty_trash`
Permanently deletes all trashed pages by overwriting their content with zeros, purging all related data, then running VACUUM. This cannot be undone.
- **Args**: none
- **Returns**: `void`

#### `get_page_tree_metadata`
Returns the page tree with lightweight metadata only (no content field). Use this instead of `get_page_tree` for sidebar rendering and other UI that doesn't need page content.
- **Args**: none
- **Returns**: `PageTreeNodeMeta[]` — each node has `PageMetadata` (id, parentId, title, icon, coverColor, createdAt, updatedAt, deletedAt, pinned, sortOrder) and `children`

#### `get_page_titles`
Returns all page IDs and titles (no content). Use this for wiki-link autocomplete and other title-only lookups.
- **Args**: none
- **Returns**: `[string, string][]` — array of `[id, title]` tuples

#### `get_recent_pages_metadata`
Returns recent pages with lightweight metadata only (no content field). Use this instead of `get_recent_pages` for sidebar recent list.
- **Args**: `limit?: number` (default: 10)
- **Returns**: `PageMetadata[]`
