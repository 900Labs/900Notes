# Database Schema

All data is stored in a single SQLite file at `{APP_DATA_DIR}/900notes.db`.

## Configuration

- **Journal mode**: WAL (Write-Ahead Logging) for concurrent reads during writes.
- **Foreign keys**: ON (cascade deletes for page_tags and links).
- **Driver**: `rusqlite` with the `bundled` feature (SQLite compiled into the binary).

## Tables

### `pages`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `parent_id` | TEXT | FK → pages(id) ON DELETE SET NULL | Parent page for nesting |
| `title` | TEXT | NOT NULL DEFAULT '' | Page title |
| `content` | TEXT | NOT NULL DEFAULT '' | JSON ProseMirror document |
| `icon` | TEXT | nullable | Emoji icon |
| `cover_color` | TEXT | nullable | Hex color for cover |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `deleted_at` | TEXT | nullable | ISO 8601 timestamp (null = not deleted) |
| `pinned` | INTEGER | NOT NULL DEFAULT 0 | Boolean (0/1) |
| `sort_order` | INTEGER | NOT NULL DEFAULT 0 | Position within parent |

**Indexes**: `idx_pages_parent` (parent_id), `idx_pages_deleted` (deleted_at), `idx_pages_updated` (updated_at)

### `tags`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL UNIQUE | Tag name |
| `color` | TEXT | nullable | Hex color |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |

### `page_tags`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `page_id` | TEXT | NOT NULL FK → pages(id) ON DELETE CASCADE | |
| `tag_id` | TEXT | NOT NULL FK → tags(id) ON DELETE CASCADE | |

**Primary key**: composite (page_id, tag_id)

### `links`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `source_page_id` | TEXT | NOT NULL FK → pages(id) ON DELETE CASCADE | Page containing the link |
| `target_page_id` | TEXT | NOT NULL FK → pages(id) ON DELETE CASCADE | Page being linked to |
| `link_text` | TEXT | NOT NULL DEFAULT '' | The `[[link text]]` content |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes**: `idx_links_source` (source_page_id), `idx_links_target` (target_page_id)

### `pages_fts` (FTS5 Virtual Table)

| Column | Type | Description |
|--------|------|-------------|
| `page_id` | UNINDEXED | Foreign reference to pages.id |
| `title` | indexed | Page title for full-text search |
| `content` | indexed | Plain-text extraction of page content |

**Tokenizer**: `unicode61` (Unicode-aware tokenization, supports non-ASCII languages).

### `settings`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `key` | TEXT | PRIMARY KEY | Setting key |
| `value` | TEXT | NOT NULL | Setting value (stored as string) |

**Default rows**: theme, language, font_size, line_spacing, default_block_type

### `page_properties`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `page_id` | TEXT | NOT NULL FK → pages(id) ON DELETE CASCADE | Page this property belongs to |
| `key` | TEXT | NOT NULL | Property name (e.g., "Author", "Status") |
| `value` | TEXT | NOT NULL DEFAULT '' | Property value |
| `sort_order` | INTEGER | NOT NULL DEFAULT 0 | Display order within the page |

**Indexes**: `idx_page_properties_page` (page_id)

### `templates`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 (custom) or fixed ID (built-in: tpl-meeting, tpl-daily, tpl-project, tpl-blank) |
| `name` | TEXT | NOT NULL | Template display name |
| `icon` | TEXT | NOT NULL DEFAULT '📄' | Emoji icon |
| `content` | TEXT | NOT NULL DEFAULT '' | JSON ProseMirror document |
| `category` | TEXT | NOT NULL DEFAULT 'custom' | "built-in" or "custom" |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Built-in templates**: Meeting Notes, Daily Journal, Project Page, Blank Page (seeded on first run via `seed_builtin_templates()`)

### `saved_searches`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL | Display name |
| `query` | TEXT | NOT NULL DEFAULT '' | FTS query text |
| `tag_filter` | TEXT | | Optional tag name filter |
| `pinned` | INTEGER | NOT NULL DEFAULT 0 | 0 or 1 (pinned sorts first) |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

### `smart_folders`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL | Display name |
| `icon` | TEXT | NOT NULL DEFAULT '📁' | Emoji icon |
| `rules` | TEXT | NOT NULL DEFAULT '[]' | JSON array of SmartFolderRule |
| `sort_order` | INTEGER | NOT NULL DEFAULT 0 | Display order |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**SmartFolderRule format**: `{ "field": "tag|title|created_after|created_before|has_property", "operator": "equals|contains|starts_with", "value": "..." }`

### `tag_groups`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL | Group display name |
| `color` | TEXT | | Optional color (hex) |
| `sort_order` | INTEGER | NOT NULL DEFAULT 0 | Display order |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

### `tag_group_members`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `group_id` | TEXT | FK → tag_groups(id) ON DELETE CASCADE | Group reference |
| `tag_id` | TEXT | FK → tags(id) ON DELETE CASCADE | Tag reference |
| | | PRIMARY KEY (group_id, tag_id) | Composite PK |

### `attachments`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `page_id` | TEXT | FK → pages(id) ON DELETE CASCADE | Owning page |
| `file_name` | TEXT | NOT NULL | Original file name |
| `mime_type` | TEXT | NOT NULL | MIME type (e.g. `image/png`) |
| `file_size` | INTEGER | NOT NULL | Size in bytes |
| `is_image` | INTEGER | NOT NULL DEFAULT 0 | 1 if MIME starts with `image/` |
| `data` | BLOB | NOT NULL | Raw file content |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Index**: `idx_attachments_page_id` on `page_id`.

## Triggers

### `pages_fts_insert`
Fires `AFTER INSERT ON pages`. Inserts the new page's title and content into `pages_fts`.

### `pages_fts_update`
Fires `AFTER UPDATE ON pages`. Deletes the old FTS row and inserts the new one. This keeps the search index in sync with any page edit.

### `pages_fts_delete`
Fires `AFTER DELETE ON pages`. Removes the page from the FTS index.

## Link Engine

The link engine is implemented in `Database::rebuild_links_for_page()` and `Database::rebuild_all_links()`.

### How it works

1. When a page's `content` is updated (via `update_page`), the engine:
   - Deletes all existing `links` rows where `source_page_id = page_id`
   - Scans the content JSON for `[[wiki link]]` syntax (the `extract_wiki_links` function)
   - For each link text, finds a page with a matching title (case-insensitive)
   - Inserts a new `links` row connecting source → target

2. `rebuild_all_links()` does the same for all pages — used after bulk imports.

### Wiki Link Extraction

The `extract_wiki_links()` function scans raw JSON content for `[[` and `]]` delimiters. It extracts the text between them as the link target. This works on the raw JSON string (not the parsed ProseMirror document) for simplicity and reliability.

### Title Matching

Link text is matched against page titles case-insensitively. If no match is found, the link is silently dropped (the `[[text]]` remains in the content but no link row is created). When a page is renamed, links are not automatically rebuilt — run `rebuild_links` to update.

## Migrations

Migrations are run on startup in `Database::open()` via `run_migrations()`. The method uses `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` for idempotency. No separate migration files are needed for the MVP — all schema is in a single `execute_batch` call.

### Adding Migrations (Post-MVP)

For future schema changes, add a `schema_version` setting and conditional migration blocks:

```rust
let version: i64 = self.conn.query_row(
    "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'schema_version'",
    [], |row| row.get(0)
).unwrap_or(1);

if version < 2 {
    self.conn.execute_batch("ALTER TABLE pages ADD COLUMN new_column TEXT;")?;
    self.set_setting("schema_version", "2")?;
}
```
