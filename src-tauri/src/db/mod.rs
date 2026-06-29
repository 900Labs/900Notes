use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::models::*;

const MAX_ATTACHMENT_BYTES: usize = 25 * 1024 * 1024;
const SEARCH_HIGHLIGHT_START: &str = "\u{001f}900notes_mark_start\u{001f}";
const SEARCH_HIGHLIGHT_END: &str = "\u{001f}900notes_mark_end\u{001f}";

#[derive(Debug)]
pub struct SyncQueueEntry {
    pub id: String,
    pub page_id: String,
    pub peer_id: Option<String>,
    pub operation: String,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA cache_size=-65536;
             PRAGMA mmap_size=268435456;
             PRAGMA temp_store=MEMORY;
             PRAGMA synchronous=NORMAL;",
        )?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn checkpoint(&self) -> Result<(), DbError> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS pages (
                id TEXT PRIMARY KEY,
                parent_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                icon TEXT,
                cover_color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                pinned INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS page_tags (
                page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (page_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS links (
                id TEXT PRIMARY KEY,
                source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                target_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                link_text TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_pages_parent ON pages(parent_id);
            CREATE INDEX IF NOT EXISTS idx_pages_deleted ON pages(deleted_at);
            CREATE INDEX IF NOT EXISTS idx_pages_updated ON pages(updated_at);
            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_page_id);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_page_id);

            CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
                page_id UNINDEXED,
                title,
                content,
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS pages_fts_insert AFTER INSERT ON pages BEGIN
                INSERT INTO pages_fts(page_id, title, content)
                VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS pages_fts_update AFTER UPDATE ON pages BEGIN
                DELETE FROM pages_fts WHERE page_id = old.id;
                INSERT INTO pages_fts(page_id, title, content)
                VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS pages_fts_delete AFTER DELETE ON pages BEGIN
                DELETE FROM pages_fts WHERE page_id = old.id;
            END;

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS page_properties (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                value TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_page_properties_page ON page_properties(page_id);

            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT '📄',
                content TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT 'custom',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS saved_searches (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                query TEXT NOT NULL DEFAULT '',
                tag_filter TEXT,
                pinned INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS smart_folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT '📁',
                rules TEXT NOT NULL DEFAULT '[]',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS page_revisions (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_page_revisions_page_id ON page_revisions(page_id);

            CREATE TABLE IF NOT EXISTS favorites (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL UNIQUE,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tag_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tag_group_members (
                group_id TEXT NOT NULL REFERENCES tag_groups(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (group_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                file_name TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                is_image INTEGER NOT NULL DEFAULT 0,
                data BLOB NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_attachments_page_id ON attachments(page_id);

            CREATE TABLE IF NOT EXISTS audio_notes (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                attachment_id TEXT NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
                duration_sec REAL NOT NULL DEFAULT 0,
                title TEXT NOT NULL DEFAULT '',
                transcription TEXT,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_audio_notes_page_id ON audio_notes(page_id);

            CREATE TABLE IF NOT EXISTS sync_state (
                id TEXT PRIMARY KEY DEFAULT 'workspace',
                doc BLOB NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_queue (
                id TEXT PRIMARY KEY,
                page_id TEXT NOT NULL,
                peer_id TEXT,
                operation TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sync_queue_status ON sync_queue(status);

            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                author TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                enabled INTEGER NOT NULL DEFAULT 0,
                entry_point TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            INSERT OR IGNORE INTO settings(key, value) VALUES
                ('theme', 'system'),
                ('language', 'en'),
                ('font_size', '16'),
                ('line_spacing', '1.5'),
                ('default_block_type', 'paragraph');
            ",
        )?;
        self.ensure_search_index_schema()?;
        self.seed_builtin_templates()?;
        Ok(())
    }

    fn ensure_search_index_schema(&self) -> Result<(), DbError> {
        let sql: Option<String> = self
            .conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'pages_fts'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if sql
            .as_deref()
            .is_some_and(|s| s.contains("content=''") || s.contains("content=\"\""))
        {
            self.rebuild_search_index_table()?;
        } else {
            self.conn.execute(
                "INSERT INTO pages_fts(page_id, title, content)
                 SELECT p.id, p.title, p.content
                 FROM pages p
                 WHERE NOT EXISTS (
                     SELECT 1 FROM pages_fts f WHERE f.page_id = p.id
                 )",
                [],
            )?;
        }

        Ok(())
    }

    fn rebuild_search_index_table(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "
            DROP TRIGGER IF EXISTS pages_fts_insert;
            DROP TRIGGER IF EXISTS pages_fts_update;
            DROP TRIGGER IF EXISTS pages_fts_delete;
            DROP TABLE IF EXISTS pages_fts;

            CREATE VIRTUAL TABLE pages_fts USING fts5(
                page_id UNINDEXED,
                title,
                content,
                tokenize='unicode61'
            );

            CREATE TRIGGER pages_fts_insert AFTER INSERT ON pages BEGIN
                INSERT INTO pages_fts(page_id, title, content)
                VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER pages_fts_update AFTER UPDATE ON pages BEGIN
                DELETE FROM pages_fts WHERE page_id = old.id;
                INSERT INTO pages_fts(page_id, title, content)
                VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER pages_fts_delete AFTER DELETE ON pages BEGIN
                DELETE FROM pages_fts WHERE page_id = old.id;
            END;

            INSERT INTO pages_fts(page_id, title, content)
            SELECT id, title, content FROM pages;
            ",
        )?;
        Ok(())
    }

    fn seed_builtin_templates(&self) -> Result<(), DbError> {
        let now = "2026-01-01T00:00:00Z";

        let meeting_content = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 1}, "content": [{"type": "text", "text": "Meeting"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Attendees"}]},
                {"type": "paragraph"},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Agenda"}]},
                {"type": "bullet_list", "content": [{"type": "list_item", "content": [{"type": "paragraph"}]}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Notes"}]},
                {"type": "paragraph"},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Action Items"}]},
                {"type": "todo_item", "attrs": {"checked": false}}
            ]
        }).to_string();

        let daily_content = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 1}, "content": [{"type": "text", "text": "Daily Journal"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Gratitude"}]},
                {"type": "paragraph"},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Priorities"}]},
                {"type": "todo_item", "attrs": {"checked": false}},
                {"type": "todo_item", "attrs": {"checked": false}},
                {"type": "todo_item", "attrs": {"checked": false}},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Notes"}]},
                {"type": "paragraph"}
            ]
        }).to_string();

        let project_content = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "attrs": {"level": 1}, "content": [{"type": "text", "text": "Project"}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Overview"}]},
                {"type": "paragraph"},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Goals"}]},
                {"type": "bullet_list", "content": [{"type": "list_item", "content": [{"type": "paragraph"}]}]},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Timeline"}]},
                {"type": "paragraph"},
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Resources"}]},
                {"type": "paragraph"}
            ]
        }).to_string();

        let blank_content = serde_json::json!({
            "type": "doc",
            "content": [{"type": "paragraph"}]
        })
        .to_string();

        self.conn.execute(
            "INSERT OR IGNORE INTO templates (id, name, icon, content, category, created_at, updated_at) VALUES
             ('tpl-meeting', 'Meeting Notes', '\u{1F4CB}', ?1, 'built-in', ?2, ?2),
             ('tpl-daily', 'Daily Journal', '\u{1F4C5}', ?3, 'built-in', ?2, ?2),
             ('tpl-project', 'Project Page', '\u{1F680}', ?4, 'built-in', ?2, ?2),
             ('tpl-blank', 'Blank Page', '\u{1F4C4}', ?5, 'built-in', ?2, ?2)",
            params![meeting_content, now, daily_content, project_content, blank_content],
        )?;
        Ok(())
    }

    // ── Pages ──

    pub fn create_page(&self, input: &CreatePageInput) -> Result<Page, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let content = input.content.clone().unwrap_or_else(|| {
            serde_json::json!({
                "type": "doc",
                "content": [{"type": "paragraph"}]
            })
            .to_string()
        });
        let sort_order = self.get_next_sort_order(input.parent_id.as_deref())?;

        self.conn.execute(
            "INSERT INTO pages (id, parent_id, title, content, icon, created_at, updated_at, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
            params![id, input.parent_id, input.title, content, input.icon, now, sort_order],
        )?;

        self.get_page_by_id(&id)
    }

    pub fn get_page_by_id(&self, id: &str) -> Result<Page, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE id = ?1"
        )?;
        let page = stmt
            .query_row(params![id], |row| {
                Ok(Page {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    icon: row.get(4)?,
                    cover_color: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    deleted_at: row.get(8)?,
                    pinned: row.get::<_, i64>(9)? != 0,
                    sort_order: row.get(10)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("page {}", id)),
                other => DbError::Sqlite(other),
            })?;
        Ok(page)
    }

    pub fn get_all_pages(&self) -> Result<Vec<Page>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE deleted_at IS NULL ORDER BY sort_order"
        )?;
        let pages = stmt.query_map([], |row| {
            Ok(Page {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                icon: row.get(4)?,
                cover_color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                sort_order: row.get(10)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn get_all_pages_metadata(&self) -> Result<Vec<PageMetadata>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE deleted_at IS NULL ORDER BY sort_order"
        )?;
        let pages = stmt.query_map([], |row| {
            Ok(PageMetadata {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                icon: row.get(3)?,
                cover_color: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                deleted_at: row.get(7)?,
                pinned: row.get::<_, i64>(8)? != 0,
                sort_order: row.get(9)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn get_page_tree_metadata(&self) -> Result<Vec<PageTreeNodeMeta>, DbError> {
        let pages = self.get_all_pages_metadata()?;
        Ok(build_tree_meta(pages, None))
    }

    pub fn get_page_titles(&self) -> Result<Vec<(String, String)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM pages WHERE deleted_at IS NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_recent_pages_metadata(&self, limit: i64) -> Result<Vec<PageMetadata>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE deleted_at IS NULL
             ORDER BY updated_at DESC LIMIT ?1"
        )?;
        let pages = stmt.query_map(params![limit], |row| {
            Ok(PageMetadata {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                icon: row.get(3)?,
                cover_color: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                deleted_at: row.get(7)?,
                pinned: row.get::<_, i64>(8)? != 0,
                sort_order: row.get(9)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn get_all_plugins(&self) -> Result<Vec<Plugin>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, author, description, enabled, entry_point, installed_at, updated_at
             FROM plugins ORDER BY name"
        )?;
        let plugins = stmt.query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get::<_, i64>(5)? != 0,
                entry_point: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for plugin in plugins {
            result.push(plugin?);
        }
        Ok(result)
    }

    pub fn get_enabled_plugins(&self) -> Result<Vec<Plugin>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, author, description, enabled, entry_point, installed_at, updated_at
             FROM plugins WHERE enabled = 1 ORDER BY name"
        )?;
        let plugins = stmt.query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get::<_, i64>(5)? != 0,
                entry_point: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for plugin in plugins {
            result.push(plugin?);
        }
        Ok(result)
    }

    pub fn install_plugin(&self, manifest: &PluginManifest) -> Result<Plugin, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO plugins (id, name, version, author, description, enabled, entry_point, installed_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?7)",
            params![
                manifest.id,
                manifest.name,
                manifest.version,
                manifest.author,
                manifest.description,
                manifest.entry_point,
                now,
            ],
        )?;
        self.get_plugin(&manifest.id)
    }

    pub fn get_plugin(&self, id: &str) -> Result<Plugin, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, author, description, enabled, entry_point, installed_at, updated_at
             FROM plugins WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get::<_, i64>(5)? != 0,
                entry_point: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| DbError::NotFound(e.to_string()))
    }

    pub fn set_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE plugins SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i64, now, id],
        )?;
        Ok(())
    }

    pub fn uninstall_plugin(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM plugins WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_all_pages_for_sync(&self) -> Result<Vec<Page>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages ORDER BY updated_at DESC"
        )?;
        let pages = stmt.query_map([], |row| {
            Ok(Page {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                icon: row.get(4)?,
                cover_color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                sort_order: row.get(10)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn upsert_page_from_sync(&self, meta: &PageSyncMeta) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO pages (id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                parent_id = ?2,
                title = ?3,
                content = ?4,
                icon = ?5,
                cover_color = ?6,
                created_at = ?7,
                updated_at = ?8,
                deleted_at = ?9,
                pinned = ?10,
                sort_order = ?11
             WHERE ?8 > (SELECT updated_at FROM pages WHERE id = ?1)",
            params![
                meta.id,
                meta.parent_id,
                meta.title,
                meta.content,
                meta.icon,
                meta.cover_color,
                meta.created_at,
                meta.updated_at,
                meta.deleted_at,
                meta.pinned as i64,
                meta.sort_order,
            ],
        )?;
        Ok(())
    }

    pub fn get_page_tree(&self) -> Result<Vec<PageTreeNode>, DbError> {
        let pages = self.get_all_pages()?;
        Ok(build_tree(pages, None))
    }

    pub fn update_page(&self, input: &UpdatePageInput) -> Result<Page, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut updates: Vec<String> = vec!["updated_at = ?1".to_string()];
        let mut param_idx = 2;

        if let Some(ref _title) = input.title {
            updates.push(format!("title = ?{}", param_idx));
            param_idx += 1;
        }
        if let Some(ref _content) = input.content {
            updates.push(format!("content = ?{}", param_idx));
            param_idx += 1;
        }
        if let Some(ref _icon) = input.icon {
            updates.push(format!("icon = ?{}", param_idx));
            param_idx += 1;
        }
        if let Some(ref _cover_color) = input.cover_color {
            updates.push(format!("cover_color = ?{}", param_idx));
            param_idx += 1;
        }
        if let Some(_pinned) = input.pinned {
            updates.push(format!("pinned = ?{}", param_idx));
            param_idx += 1;
        }

        let sql = format!(
            "UPDATE pages SET {} WHERE id = ?{}",
            updates.join(", "),
            param_idx
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
        if let Some(ref title) = input.title {
            params_vec.push(Box::new(title.clone()));
        }
        if let Some(ref content) = input.content {
            params_vec.push(Box::new(content.clone()));
        }
        if let Some(ref icon) = input.icon {
            params_vec.push(Box::new(icon.clone()));
        }
        if let Some(ref cover_color) = input.cover_color {
            params_vec.push(Box::new(cover_color.clone()));
        }
        if let Some(pinned) = input.pinned {
            params_vec.push(Box::new(pinned as i64));
        }
        params_vec.push(Box::new(input.id.clone()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = self.conn.execute(&sql, param_refs.as_slice())?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("page {}", input.id)));
        }

        // Rebuild links and create revision if content changed
        if input.content.is_some() {
            let page = self.get_page_by_id(&input.id)?;
            self.rebuild_links_for_page(&page.id, &page.content)?;
            let _ = self.create_revision(&page.id, &page.title, &page.content);
            if let Err(e) = self.prune_revisions(&page.id, 50) {
                eprintln!("Failed to prune revisions: {e}");
            }
        }

        self.get_page_by_id(&input.id)
    }

    pub fn delete_page(&self, id: &str) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE pages SET deleted_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("page {}", id)));
        }
        Ok(())
    }

    pub fn restore_page(&self, id: &str) -> Result<Page, DbError> {
        let rows = self.conn.execute(
            "UPDATE pages SET deleted_at = NULL WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("page {}", id)));
        }
        self.get_page_by_id(id)
    }

    pub fn duplicate_page(&self, id: &str) -> Result<Page, DbError> {
        let page = self.get_page_by_id(id)?;
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = self.get_next_sort_order(page.parent_id.as_deref())?;
        let title = format!("{} (copy)", page.title);

        self.conn.execute(
            "INSERT INTO pages (id, parent_id, title, content, icon, cover_color, created_at, updated_at, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
            params![new_id, page.parent_id, title, page.content, page.icon, page.cover_color, now, sort_order],
        )?;

        // Copy tags
        self.conn.execute(
            "INSERT INTO page_tags (page_id, tag_id)
             SELECT ?1, tag_id FROM page_tags WHERE page_id = ?2",
            params![new_id, id],
        )?;

        self.get_page_by_id(&new_id)
    }

    pub fn move_page(&self, input: &MovePageInput) -> Result<Page, DbError> {
        self.validate_page_move(&input.id, input.parent_id.as_deref())?;
        let rows = self.conn.execute(
            "UPDATE pages SET parent_id = ?1, sort_order = ?2 WHERE id = ?3",
            params![input.parent_id, input.sort_order, input.id],
        )?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("page {}", input.id)));
        }
        self.get_page_by_id(&input.id)
    }

    fn validate_page_move(&self, page_id: &str, parent_id: Option<&str>) -> Result<(), DbError> {
        let page_exists: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM pages WHERE id = ?1 AND deleted_at IS NULL",
                params![page_id],
                |row| row.get(0),
            )
            .optional()?;
        if page_exists.is_none() {
            return Err(DbError::NotFound(format!("page {page_id}")));
        }

        let mut current_parent = parent_id.map(str::to_string);
        while let Some(current_id) = current_parent {
            if current_id == page_id {
                return Err(DbError::InvalidInput(
                    "page cannot be moved under itself or its descendants".to_string(),
                ));
            }

            current_parent = self
                .conn
                .query_row(
                    "SELECT parent_id FROM pages WHERE id = ?1 AND deleted_at IS NULL",
                    params![current_id],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()?
                .ok_or_else(|| DbError::NotFound("parent page".to_string()))?;
        }

        Ok(())
    }

    pub fn get_recent_pages(&self, limit: i64) -> Result<Vec<Page>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE deleted_at IS NULL
             ORDER BY updated_at DESC LIMIT ?1"
        )?;
        let pages = stmt.query_map(params![limit], |row| {
            Ok(Page {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                icon: row.get(4)?,
                cover_color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                sort_order: row.get(10)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn get_trash(&self) -> Result<Vec<Page>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order
             FROM pages WHERE deleted_at IS NOT NULL
             ORDER BY deleted_at DESC"
        )?;
        let pages = stmt.query_map([], |row| {
            Ok(Page {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                icon: row.get(4)?,
                cover_color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                sort_order: row.get(10)?,
            })
        })?;
        let mut result = Vec::new();
        for page in pages {
            result.push(page?);
        }
        Ok(result)
    }

    pub fn empty_trash(&self) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM pages WHERE deleted_at IS NOT NULL", [])?;
        Ok(())
    }

    pub fn secure_delete_page(&self, id: &str) -> Result<(), DbError> {
        let overwrite = "\0".repeat(256);
        self.conn.execute_batch("BEGIN;")?;
        let result: Result<(), DbError> = (|| {
            self.conn.execute(
                "UPDATE pages SET content = ?1, title = ?1, icon = NULL, cover_color = NULL, deleted_at = ?2 WHERE id = ?3",
                params![&overwrite, chrono::Utc::now().to_rfc3339(), id],
            )?;

            self.conn
                .execute("DELETE FROM page_revisions WHERE page_id = ?1", params![id])?;
            self.conn.execute(
                "DELETE FROM links WHERE source_page_id = ?1 OR target_page_id = ?1",
                params![id],
            )?;
            self.conn
                .execute("DELETE FROM page_tags WHERE page_id = ?1", params![id])?;
            self.conn.execute(
                "DELETE FROM page_properties WHERE page_id = ?1",
                params![id],
            )?;
            self.conn
                .execute("DELETE FROM attachments WHERE page_id = ?1", params![id])?;
            self.conn
                .execute("DELETE FROM audio_notes WHERE page_id = ?1", params![id])?;

            self.conn
                .execute("DELETE FROM pages WHERE id = ?1", params![id])?;
            Ok(())
        })();
        if result.is_ok() {
            self.conn.execute_batch("COMMIT;")?;
            self.conn.execute_batch("VACUUM;")?;
        } else {
            let _ = self.conn.execute_batch("ROLLBACK;");
        }
        result
    }

    pub fn secure_empty_trash(&self) -> Result<(), DbError> {
        let overwrite = "\0".repeat(256);

        let deleted_ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM pages WHERE deleted_at IS NOT NULL")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row?);
            }
            ids
        };

        self.conn.execute_batch("BEGIN;")?;
        let result: Result<(), DbError> = (|| {
            for id in &deleted_ids {
                self.conn.execute(
                    "UPDATE pages SET content = ?1, title = ?1, icon = NULL, cover_color = NULL WHERE id = ?2",
                    params![&overwrite, id],
                )?;
                self.conn
                    .execute("DELETE FROM page_revisions WHERE page_id = ?1", params![id])?;
                self.conn.execute(
                    "DELETE FROM links WHERE source_page_id = ?1 OR target_page_id = ?1",
                    params![id],
                )?;
                self.conn
                    .execute("DELETE FROM page_tags WHERE page_id = ?1", params![id])?;
                self.conn.execute(
                    "DELETE FROM page_properties WHERE page_id = ?1",
                    params![id],
                )?;
                self.conn
                    .execute("DELETE FROM attachments WHERE page_id = ?1", params![id])?;
                self.conn
                    .execute("DELETE FROM audio_notes WHERE page_id = ?1", params![id])?;
            }

            self.conn
                .execute("DELETE FROM pages WHERE deleted_at IS NOT NULL", [])?;
            Ok(())
        })();
        if result.is_ok() {
            self.conn.execute_batch("COMMIT;")?;
            self.conn.execute_batch("VACUUM;")?;
        } else {
            let _ = self.conn.execute_batch("ROLLBACK;");
        }
        result
    }

    pub fn search_pages(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, DbError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let fts_query = format_fts_query(query);
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.title, snippet(pages_fts, 2, ?2, ?3, '...', 32) as snippet, p.icon, p.updated_at
             FROM pages_fts
             JOIN pages p ON p.id = pages_fts.page_id
             WHERE pages_fts MATCH ?1 AND p.deleted_at IS NULL
             ORDER BY rank
             LIMIT ?4"
        )?;
        let results = stmt.query_map(
            params![
                fts_query,
                SEARCH_HIGHLIGHT_START,
                SEARCH_HIGHLIGHT_END,
                limit
            ],
            |row| {
                let snippet: String = row.get(2)?;
                Ok(SearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    snippet: sanitize_search_snippet(&snippet),
                    icon: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    fn get_next_sort_order(&self, parent_id: Option<&str>) -> Result<i64, DbError> {
        let max: Option<i64> = self.conn.query_row(
            "SELECT MAX(sort_order) FROM pages WHERE parent_id IS ?1 AND deleted_at IS NULL",
            params![parent_id],
            |row| row.get(0),
        )?;
        Ok(max.unwrap_or(0) + 1)
    }

    // ── Tags ──

    pub fn get_all_tags(&self) -> Result<Vec<Tag>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t ORDER BY t.name",
        )?;
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut result = Vec::new();
        for tag in tags {
            result.push(tag?);
        }
        Ok(result)
    }

    pub fn create_tag(&self, input: &CreateTagInput) -> Result<Tag, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, input.name, input.color, now],
        )?;
        self.conn
            .query_row(
                "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .map_err(DbError::Sqlite)
    }

    pub fn update_tag(&self, input: &UpdateTagInput) -> Result<Tag, DbError> {
        if let Some(ref name) = input.name {
            self.conn.execute(
                "UPDATE tags SET name = ?1 WHERE id = ?2",
                params![name, input.id],
            )?;
        }
        if let Some(ref color) = input.color {
            self.conn.execute(
                "UPDATE tags SET color = ?1 WHERE id = ?2",
                params![color, input.id],
            )?;
        }
        self.conn
            .query_row(
                "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
                params![input.id],
                |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("tag {}", input.id))
                }
                other => DbError::Sqlite(other),
            })
    }

    pub fn delete_tag(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_page_tags(&self, page_id: &str) -> Result<Vec<Tag>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t
             JOIN page_tags pt ON pt.tag_id = t.id
             WHERE pt.page_id = ?1
             ORDER BY t.name",
        )?;
        let tags = stmt.query_map(params![page_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut result = Vec::new();
        for tag in tags {
            result.push(tag?);
        }
        Ok(result)
    }

    pub fn set_page_tags(&self, page_id: &str, tag_ids: &[String]) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM page_tags WHERE page_id = ?1", params![page_id])?;
        for tag_id in tag_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO page_tags (page_id, tag_id) VALUES (?1, ?2)",
                params![page_id, tag_id],
            )?;
        }
        Ok(())
    }

    // ── Links ──

    pub fn get_backlinks(&self, page_id: &str) -> Result<Vec<Backlink>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, l.source_page_id, p.title, p.icon, l.link_text, l.created_at
             FROM links l
             JOIN pages p ON p.id = l.source_page_id
             WHERE l.target_page_id = ?1 AND p.deleted_at IS NULL
             ORDER BY l.created_at DESC",
        )?;
        let results = stmt.query_map(params![page_id], |row| {
            Ok(Backlink {
                id: row.get(0)?,
                source_page_id: row.get(1)?,
                source_page_title: row.get(2)?,
                source_page_icon: row.get(3)?,
                link_text: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn get_outgoing_links(&self, page_id: &str) -> Result<Vec<Link>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, l.source_page_id, l.target_page_id, l.link_text, l.created_at
             FROM links l
             WHERE l.source_page_id = ?1
             ORDER BY l.created_at DESC",
        )?;
        let results = stmt.query_map(params![page_id], |row| {
            Ok(Link {
                id: row.get(0)?,
                source_page_id: row.get(1)?,
                target_page_id: row.get(2)?,
                link_text: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn rebuild_links_for_page(&self, page_id: &str, content: &str) -> Result<(), DbError> {
        // Delete existing links from this page
        self.conn.execute(
            "DELETE FROM links WHERE source_page_id = ?1",
            params![page_id],
        )?;

        // Extract [[link text]] from content
        let linked_titles = extract_wiki_links(content);
        if linked_titles.is_empty() {
            return Ok(());
        }

        // Get all page titles for fuzzy matching
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM pages WHERE deleted_at IS NULL")?;
        let pages: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let now = chrono::Utc::now().to_rfc3339();
        for link_text in linked_titles {
            // Find matching page by title (case-insensitive, trimmed)
            let matched = pages
                .iter()
                .find(|(_, title)| title.to_lowercase() == link_text.to_lowercase());
            if let Some((target_id, _)) = matched {
                let link_id = uuid::Uuid::new_v4().to_string();
                self.conn.execute(
                    "INSERT INTO links (id, source_page_id, target_page_id, link_text, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![link_id, page_id, target_id, link_text, now],
                )?;
            }
        }
        Ok(())
    }

    pub fn rebuild_all_links(&self) -> Result<(), DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content FROM pages WHERE deleted_at IS NULL")?;
        let pages: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        self.conn.execute("DELETE FROM links", [])?;

        let now = chrono::Utc::now().to_rfc3339();
        let mut title_stmt = self
            .conn
            .prepare("SELECT id, title FROM pages WHERE deleted_at IS NULL")?;
        let all_pages: Vec<(String, String)> = title_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (page_id, content) in pages {
            let linked_titles = extract_wiki_links(&content);
            for link_text in linked_titles {
                let matched = all_pages
                    .iter()
                    .find(|(_, title)| title.to_lowercase() == link_text.to_lowercase());
                if let Some((target_id, _)) = matched {
                    let link_id = uuid::Uuid::new_v4().to_string();
                    self.conn.execute(
                        "INSERT INTO links (id, source_page_id, target_page_id, link_text, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![link_id, page_id, target_id, link_text, now],
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn conn_execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<(), DbError> {
        self.conn.execute(sql, params)?;
        Ok(())
    }

    pub fn begin_transaction(&self) -> Result<(), DbError> {
        self.conn.execute_batch("BEGIN;")?;
        Ok(())
    }

    pub fn commit_transaction(&self) -> Result<(), DbError> {
        self.conn.execute_batch("COMMIT;")?;
        Ok(())
    }

    pub fn rollback_transaction(&self) -> Result<(), DbError> {
        self.conn.execute_batch("ROLLBACK;")?;
        Ok(())
    }

    // ── Settings ──

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>, DbError> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let results = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    // ── Page Properties ──

    pub fn get_page_properties(&self, page_id: &str) -> Result<Vec<PageProperty>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, key, value, sort_order FROM page_properties
             WHERE page_id = ?1 ORDER BY sort_order",
        )?;
        let results = stmt.query_map(params![page_id], |row| {
            Ok(PageProperty {
                id: row.get(0)?,
                page_id: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn set_page_property(&self, input: &SetPropertyInput) -> Result<PageProperty, DbError> {
        // Check if property with this key already exists for this page
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM page_properties WHERE page_id = ?1 AND key = ?2",
                params![input.page_id, input.key],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            self.conn.execute(
                "UPDATE page_properties SET value = ?1 WHERE id = ?2",
                params![input.value, id],
            )?;
            return self.get_page_property_by_id(&id);
        }

        let id = uuid::Uuid::new_v4().to_string();
        let sort_order = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM page_properties WHERE page_id = ?1",
            params![input.page_id],
            |row| row.get::<_, i64>(0),
        )?;

        self.conn.execute(
            "INSERT INTO page_properties (id, page_id, key, value, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, input.page_id, input.key, input.value, sort_order],
        )?;

        self.get_page_property_by_id(&id)
    }

    pub fn delete_page_property(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM page_properties WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_page_property_by_id(&self, id: &str) -> Result<PageProperty, DbError> {
        self.conn
            .query_row(
                "SELECT id, page_id, key, value, sort_order FROM page_properties WHERE id = ?1",
                params![id],
                |row| {
                    Ok(PageProperty {
                        id: row.get(0)?,
                        page_id: row.get(1)?,
                        key: row.get(2)?,
                        value: row.get(3)?,
                        sort_order: row.get(4)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("property {}", id))
                }
                other => DbError::Sqlite(other),
            })
    }

    // ── Templates ──

    pub fn get_all_templates(&self) -> Result<Vec<Template>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon, content, category, created_at, updated_at FROM templates ORDER BY category, name",
        )?;
        let results = stmt.query_map([], |row| {
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                content: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for r in results {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn get_template_by_id(&self, id: &str) -> Result<Template, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, icon, content, category, created_at, updated_at FROM templates WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Template {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        icon: row.get(2)?,
                        content: row.get(3)?,
                        category: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("template {}", id)),
                other => DbError::Sqlite(other),
            })
    }

    pub fn create_template(&self, input: &CreateTemplateInput) -> Result<Template, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO templates (id, name, icon, content, category, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id,
                input.name,
                input.icon,
                input.content,
                input.category,
                now
            ],
        )?;
        self.get_template_by_id(&id)
    }

    pub fn update_template(&self, input: &UpdateTemplateInput) -> Result<Template, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(ref name) = input.name {
            self.conn.execute(
                "UPDATE templates SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, input.id],
            )?;
        }
        if let Some(ref icon) = input.icon {
            self.conn.execute(
                "UPDATE templates SET icon = ?1, updated_at = ?2 WHERE id = ?3",
                params![icon, now, input.id],
            )?;
        }
        if let Some(ref content) = input.content {
            self.conn.execute(
                "UPDATE templates SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, now, input.id],
            )?;
        }
        if let Some(ref category) = input.category {
            self.conn.execute(
                "UPDATE templates SET category = ?1, updated_at = ?2 WHERE id = ?3",
                params![category, now, input.id],
            )?;
        }
        self.get_template_by_id(&input.id)
    }

    pub fn delete_template(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn create_page_from_template(
        &self,
        template_id: &str,
        title: &str,
        parent_id: Option<&str>,
    ) -> Result<Page, DbError> {
        let template = self.get_template_by_id(template_id)?;
        let input = CreatePageInput {
            parent_id: parent_id.map(|s| s.to_string()),
            title: title.to_string(),
            content: Some(template.content.clone()),
            icon: Some(template.icon.clone()),
        };
        self.create_page(&input)
    }

    pub fn get_or_create_daily_note(&self, date: &str) -> Result<Page, DbError> {
        let title = format!("Daily — {}", date);

        let existing = self.conn.query_row(
            "SELECT id FROM pages WHERE title = ?1 AND deleted_at IS NULL LIMIT 1",
            params![title],
            |row| row.get::<_, String>(0),
        );

        if let Ok(id) = existing {
            return self.get_page_by_id(&id);
        }

        let template = self.get_template_by_id("tpl-daily").ok();
        let content = template
            .map(|t| t.content)
            .unwrap_or_else(|| {
                serde_json::json!({
                    "type": "doc",
                    "content": [
                        {"type": "heading", "attrs": {"level": 1}, "content": [{"type": "text", "text": &title}]},
                        {"type": "paragraph"}
                    ]
                })
                .to_string()
            });

        let input = CreatePageInput {
            parent_id: None,
            title: title.clone(),
            content: Some(content),
            icon: Some("📅".to_string()),
        };
        let page = self.create_page(&input)?;

        let prev_date = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(1))
            .map(|d| d.format("%Y-%m-%d").to_string());
        if let Some(prev) = prev_date {
            let prev_title = format!("Daily — {}", prev);
            let prev_id = self
                .conn
                .query_row(
                    "SELECT id FROM pages WHERE title = ?1 AND deleted_at IS NULL LIMIT 1",
                    params![prev_title],
                    |row| row.get::<_, String>(0),
                )
                .ok();
            if let Some(pid) = prev_id {
                let link_id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                self.conn.execute(
                    "INSERT INTO links (id, source_page_id, target_page_id, link_text, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![link_id, page.id, pid, &prev, now],
                )?;
            }
        }

        Ok(page)
    }

    // ── Graph ──

    pub fn get_graph_data(&self) -> Result<GraphData, DbError> {
        let mut node_stmt = self.conn.prepare(
            "SELECT p.id, p.title, p.icon, p.created_at, p.updated_at,
                    (SELECT COUNT(*) FROM page_tags pt WHERE pt.page_id = p.id) AS tag_count,
                    (SELECT COUNT(*) FROM links l WHERE l.source_page_id = p.id OR l.target_page_id = p.id) AS link_count
             FROM pages p
             WHERE p.deleted_at IS NULL",
        )?;
        let nodes = node_stmt.query_map([], |row| {
            Ok(GraphNode {
                id: row.get(0)?,
                title: row.get(1)?,
                icon: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                tag_count: row.get(5)?,
                link_count: row.get(6)?,
            })
        })?;
        let mut node_list = Vec::new();
        for n in nodes {
            node_list.push(n?);
        }

        let mut edge_stmt = self
            .conn
            .prepare("SELECT DISTINCT source_page_id, target_page_id FROM links")?;
        let edges = edge_stmt.query_map([], |row| {
            Ok(GraphEdge {
                source: row.get(0)?,
                target: row.get(1)?,
            })
        })?;
        let mut edge_list = Vec::new();
        for e in edges {
            edge_list.push(e?);
        }

        Ok(GraphData {
            nodes: node_list,
            edges: edge_list,
        })
    }

    // ── Saved Searches ──

    pub fn get_all_saved_searches(&self) -> Result<Vec<SavedSearch>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, query, tag_filter, pinned, created_at, updated_at
             FROM saved_searches ORDER BY pinned DESC, name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SavedSearch {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                tag_filter: row.get(3)?,
                pinned: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_saved_search(
        &self,
        input: &CreateSavedSearchInput,
    ) -> Result<SavedSearch, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO saved_searches (id, name, query, tag_filter, pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, input.name, input.query, input.tag_filter, input.pinned as i64, now],
        )?;
        Ok(SavedSearch {
            id,
            name: input.name.clone(),
            query: input.query.clone(),
            tag_filter: input.tag_filter.clone(),
            pinned: input.pinned,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_saved_search(
        &self,
        input: &UpdateSavedSearchInput,
    ) -> Result<SavedSearch, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(name) = &input.name {
            self.conn.execute(
                "UPDATE saved_searches SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, input.id],
            )?;
        }
        if let Some(query) = &input.query {
            self.conn.execute(
                "UPDATE saved_searches SET query = ?1, updated_at = ?2 WHERE id = ?3",
                params![query, now, input.id],
            )?;
        }
        if let Some(tag_filter) = &input.tag_filter {
            self.conn.execute(
                "UPDATE saved_searches SET tag_filter = ?1, updated_at = ?2 WHERE id = ?3",
                params![tag_filter, now, input.id],
            )?;
        }
        if let Some(pinned) = input.pinned {
            self.conn.execute(
                "UPDATE saved_searches SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
                params![pinned as i64, now, input.id],
            )?;
        }
        self.get_saved_search_by_id(&input.id)
    }

    pub fn delete_saved_search(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_saved_search_by_id(&self, id: &str) -> Result<SavedSearch, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, query, tag_filter, pinned, created_at, updated_at
             FROM saved_searches WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SavedSearch {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        query: row.get(2)?,
                        tag_filter: row.get(3)?,
                        pinned: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .map_err(DbError::from)
    }

    pub fn execute_saved_search(&self, id: &str) -> Result<Vec<SearchResult>, DbError> {
        let search = self.get_saved_search_by_id(id)?;
        let fts_query = format_fts_query(&search.query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.title, snippet(pages_fts, 2, ?2, ?3, '...', 20) as snippet, p.icon, p.updated_at
             FROM pages_fts
             JOIN pages p ON p.id = pages_fts.page_id
             WHERE pages_fts MATCH ?1 AND p.deleted_at IS NULL
             ORDER BY rank
             LIMIT 50",
        )?;
        let rows = stmt.query_map(
            params![fts_query, SEARCH_HIGHLIGHT_START, SEARCH_HIGHLIGHT_END],
            |row| {
                let snippet: String = row.get(2)?;
                Ok(SearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    snippet: sanitize_search_snippet(&snippet),
                    icon: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    // ── Smart Folders ──

    pub fn get_all_smart_folders(&self) -> Result<Vec<SmartFolder>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon, rules, sort_order, created_at, updated_at
             FROM smart_folders ORDER BY sort_order ASC, name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SmartFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                rules: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_smart_folder(
        &self,
        input: &CreateSmartFolderInput,
    ) -> Result<SmartFolder, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM smart_folders",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO smart_folders (id, name, icon, rules, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, input.name, input.icon, input.rules, sort_order, now],
        )?;
        Ok(SmartFolder {
            id,
            name: input.name.clone(),
            icon: input.icon.clone(),
            rules: input.rules.clone(),
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_smart_folder(
        &self,
        input: &UpdateSmartFolderInput,
    ) -> Result<SmartFolder, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(name) = &input.name {
            self.conn.execute(
                "UPDATE smart_folders SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, input.id],
            )?;
        }
        if let Some(icon) = &input.icon {
            self.conn.execute(
                "UPDATE smart_folders SET icon = ?1, updated_at = ?2 WHERE id = ?3",
                params![icon, now, input.id],
            )?;
        }
        if let Some(rules) = &input.rules {
            self.conn.execute(
                "UPDATE smart_folders SET rules = ?1, updated_at = ?2 WHERE id = ?3",
                params![rules, now, input.id],
            )?;
        }
        if let Some(sort_order) = input.sort_order {
            self.conn.execute(
                "UPDATE smart_folders SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                params![sort_order, now, input.id],
            )?;
        }
        self.get_smart_folder_by_id(&input.id)
    }

    pub fn delete_smart_folder(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM smart_folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_smart_folder_by_id(&self, id: &str) -> Result<SmartFolder, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, icon, rules, sort_order, created_at, updated_at
             FROM smart_folders WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SmartFolder {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        icon: row.get(2)?,
                        rules: row.get(3)?,
                        sort_order: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .map_err(DbError::from)
    }

    pub fn get_smart_folder_pages(&self, id: &str) -> Result<Vec<PageTreeNode>, DbError> {
        let folder = self.get_smart_folder_by_id(id)?;
        let rules: Vec<SmartFolderRule> = serde_json::from_str(&folder.rules).unwrap_or_default();

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_idx = 1;

        for rule in &rules {
            match rule.field.as_str() {
                "tag" => {
                    conditions.push(format!(
                        "p.id IN (SELECT page_id FROM page_tags pt JOIN tags t ON pt.tag_id = t.id WHERE t.name = ?{}",
                        param_idx
                    ));
                    params.push(Box::new(rule.value.clone()));
                    param_idx += 1;
                }
                "title" => {
                    let op = match rule.operator.as_str() {
                        "contains" => "LIKE",
                        "equals" => "=",
                        "starts_with" => "LIKE",
                        _ => "LIKE",
                    };
                    let val = match rule.operator.as_str() {
                        "starts_with" => format!("{}%", rule.value),
                        "contains" => format!("%{}%", rule.value),
                        _ => rule.value.clone(),
                    };
                    conditions.push(format!("p.title {} ?{}", op, param_idx));
                    params.push(Box::new(val));
                    param_idx += 1;
                }
                "created_after" => {
                    conditions.push(format!("p.created_at >= ?{}", param_idx));
                    params.push(Box::new(rule.value.clone()));
                    param_idx += 1;
                }
                "created_before" => {
                    conditions.push(format!("p.created_at <= ?{}", param_idx));
                    params.push(Box::new(rule.value.clone()));
                    param_idx += 1;
                }
                "has_property" => {
                    conditions.push(format!(
                        "p.id IN (SELECT page_id FROM page_properties WHERE key = ?{})",
                        param_idx
                    ));
                    params.push(Box::new(rule.value.clone()));
                    param_idx += 1;
                }
                _ => {}
            }
        }

        let where_clause = if conditions.is_empty() {
            String::from("WHERE p.deleted_at IS NULL")
        } else {
            format!(
                "WHERE p.deleted_at IS NULL AND ({})",
                conditions.join(" AND ")
            )
        };

        let sql = format!(
            "SELECT p.id, p.parent_id, p.title, p.content, p.icon, p.cover_color, p.created_at, p.updated_at, p.deleted_at, p.pinned, p.sort_order
             FROM pages p {} ORDER BY p.updated_at DESC",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(Page {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                icon: row.get(4)?,
                cover_color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                sort_order: row.get(10)?,
            })
        })?;
        let mut pages = Vec::new();
        for r in rows {
            pages.push(r?);
        }
        Ok(build_tree(pages, None))
    }

    // ── Page Revisions ──

    pub fn create_revision(
        &self,
        page_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PageRevision, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO page_revisions (id, page_id, title, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, page_id, title, content, now],
        )?;
        Ok(PageRevision {
            id,
            page_id: page_id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            created_at: now,
        })
    }

    pub fn prune_revisions(&self, page_id: &str, keep_count: usize) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM page_revisions WHERE page_id = ?1 AND id NOT IN (
                SELECT id FROM page_revisions WHERE page_id = ?1 ORDER BY created_at DESC LIMIT ?2
            )",
            params![page_id, keep_count as i64],
        )?;
        Ok(())
    }

    pub fn get_page_revisions(&self, page_id: &str) -> Result<Vec<PageRevision>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, title, content, created_at FROM page_revisions WHERE page_id = ?1 ORDER BY created_at DESC LIMIT 50",
        )?;
        let rows = stmt.query_map(params![page_id], |row| {
            Ok(PageRevision {
                id: row.get(0)?,
                page_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut revisions = Vec::new();
        for r in rows {
            revisions.push(r?);
        }
        Ok(revisions)
    }

    pub fn get_revision(&self, id: &str) -> Result<PageRevision, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, title, content, created_at FROM page_revisions WHERE id = ?1",
        )?;
        let revision = stmt
            .query_row(params![id], |row| {
                Ok(PageRevision {
                    id: row.get(0)?,
                    page_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("revision {}", id))
                }
                other => DbError::Sqlite(other),
            })?;
        Ok(revision)
    }

    pub fn restore_revision(&self, revision_id: &str) -> Result<Page, DbError> {
        let revision = self.get_revision(revision_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE pages SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![revision.title, revision.content, now, revision.page_id],
        )?;
        let page = self.get_page_by_id(&revision.page_id)?;
        self.rebuild_links_for_page(&page.id, &page.content)?;
        Ok(page)
    }

    pub fn delete_revision(&self, id: &str) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM page_revisions WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("revision {}", id)));
        }
        Ok(())
    }

    // ── Favorites ──

    pub fn get_favorites(&self) -> Result<Vec<Favorite>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.page_id, f.sort_order, f.created_at FROM favorites f JOIN pages p ON p.id = f.page_id WHERE p.deleted_at IS NULL ORDER BY f.sort_order, f.created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Favorite {
                id: row.get(0)?,
                page_id: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut favorites = Vec::new();
        for r in rows {
            favorites.push(r?);
        }
        Ok(favorites)
    }

    pub fn add_favorite(&self, page_id: &str) -> Result<Favorite, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let max_sort: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM favorites",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        let sort_order = max_sort + 1;
        self.conn.execute(
            "INSERT OR REPLACE INTO favorites (id, page_id, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, page_id, sort_order, now],
        )?;
        Ok(Favorite {
            id,
            page_id: page_id.to_string(),
            sort_order,
            created_at: now,
        })
    }

    pub fn remove_favorite(&self, page_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM favorites WHERE page_id = ?1", params![page_id])?;
        Ok(())
    }

    pub fn is_favorite(&self, page_id: &str) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM favorites WHERE page_id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn reorder_favorites(&self, ordered_page_ids: &[String]) -> Result<(), DbError> {
        for (idx, page_id) in ordered_page_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE favorites SET sort_order = ?1 WHERE page_id = ?2",
                params![idx as i64, page_id],
            )?;
        }
        Ok(())
    }

    // ── Tag Groups ──

    pub fn get_all_tag_groups(&self) -> Result<Vec<TagGroup>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, sort_order, created_at, updated_at FROM tag_groups ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TagGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut groups = Vec::new();
        for r in rows {
            groups.push(r?);
        }
        Ok(groups)
    }

    pub fn create_tag_group(&self, input: &CreateTagGroupInput) -> Result<TagGroup, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let max_sort: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM tag_groups",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        let sort_order = max_sort + 1;
        self.conn.execute(
            "INSERT INTO tag_groups (id, name, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, input.name, input.color, sort_order, now, now],
        )?;
        Ok(TagGroup {
            id,
            name: input.name.clone(),
            color: input.color.clone(),
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_tag_group(&self, input: &UpdateTagGroupInput) -> Result<TagGroup, DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut updates: Vec<String> = vec!["updated_at = ?1".to_string()];
        let mut param_idx = 2;
        if input.name.is_some() {
            updates.push(format!("name = ?{}", param_idx));
            param_idx += 1;
        }
        if input.color.is_some() {
            updates.push(format!("color = ?{}", param_idx));
            param_idx += 1;
        }
        if input.sort_order.is_some() {
            updates.push(format!("sort_order = ?{}", param_idx));
            param_idx += 1;
        }
        let sql = format!(
            "UPDATE tag_groups SET {} WHERE id = ?{}",
            updates.join(", "),
            param_idx
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
        if let Some(ref name) = input.name {
            params_vec.push(Box::new(name.clone()));
        }
        if let Some(ref color) = input.color {
            params_vec.push(Box::new(color.clone()));
        }
        if let Some(sort_order) = input.sort_order {
            params_vec.push(Box::new(sort_order));
        }
        params_vec.push(Box::new(input.id.clone()));
        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = self.conn.execute(&sql, param_refs.as_slice())?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("tag group {}", input.id)));
        }
        self.get_tag_group_by_id(&input.id)
    }

    pub fn get_tag_group_by_id(&self, id: &str) -> Result<TagGroup, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, color, sort_order, created_at, updated_at FROM tag_groups WHERE id = ?1",
                params![id],
                |row| {
                    Ok(TagGroup {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        sort_order: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("tag group {}", id))
                }
                other => DbError::Sqlite(other),
            })
    }

    pub fn delete_tag_group(&self, id: &str) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM tag_groups WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("tag group {}", id)));
        }
        Ok(())
    }

    pub fn add_tag_to_group(&self, group_id: &str, tag_id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tag_group_members (group_id, tag_id) VALUES (?1, ?2)",
            params![group_id, tag_id],
        )?;
        Ok(())
    }

    pub fn remove_tag_from_group(&self, group_id: &str, tag_id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM tag_group_members WHERE group_id = ?1 AND tag_id = ?2",
            params![group_id, tag_id],
        )?;
        Ok(())
    }

    pub fn get_tags_in_group(&self, group_id: &str) -> Result<Vec<Tag>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at FROM tags t
             JOIN tag_group_members m ON m.tag_id = t.id
             WHERE m.group_id = ?1 ORDER BY t.name",
        )?;
        let rows = stmt.query_map(params![group_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut tags = Vec::new();
        for r in rows {
            tags.push(r?);
        }
        Ok(tags)
    }

    pub fn get_ungrouped_tags(&self) -> Result<Vec<Tag>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at FROM tags t
             WHERE t.id NOT IN (SELECT tag_id FROM tag_group_members)
             ORDER BY t.name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut tags = Vec::new();
        for r in rows {
            tags.push(r?);
        }
        Ok(tags)
    }

    // ── Related Pages ──

    pub fn get_related_pages(
        &self,
        page_id: &str,
        limit: i64,
    ) -> Result<Vec<RelatedPage>, DbError> {
        let mut scores: std::collections::HashMap<String, (i64, Vec<String>)> =
            std::collections::HashMap::new();

        // 1. Shared tags (weight: 3 per shared tag)
        let mut stmt = self.conn.prepare(
            "SELECT pt2.page_id, COUNT(*) as shared_count
             FROM page_tags pt1
             JOIN page_tags pt2 ON pt1.tag_id = pt2.tag_id AND pt2.page_id != pt1.page_id
             WHERE pt1.page_id = ?1
             GROUP BY pt2.page_id",
        )?;
        let shared_tag_rows = stmt.query_map(params![page_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for r in shared_tag_rows {
            let (pid, count) = r?;
            let entry = scores.entry(pid).or_insert((0, Vec::new()));
            entry.0 += count * 3;
            entry.1.push("shared tags".to_string());
        }

        // 2. Backlinks (weight: 5 per link)
        let mut stmt2 = self.conn.prepare(
            "SELECT source_page_id FROM links WHERE target_page_id = ?1 AND source_page_id != ?1",
        )?;
        let backlink_rows = stmt2.query_map(params![page_id], |row| row.get::<_, String>(0))?;
        for r in backlink_rows {
            let pid = r?;
            let entry = scores.entry(pid).or_insert((0, Vec::new()));
            entry.0 += 5;
            entry.1.push("backlink".to_string());
        }

        // 3. Forward links (weight: 4 per link)
        let mut stmt3 = self.conn.prepare(
            "SELECT target_page_id FROM links WHERE source_page_id = ?1 AND target_page_id != ?1",
        )?;
        let forward_rows = stmt3.query_map(params![page_id], |row| row.get::<_, String>(0))?;
        for r in forward_rows {
            let pid = r?;
            let entry = scores.entry(pid).or_insert((0, Vec::new()));
            entry.0 += 4;
            entry.1.push("link".to_string());
        }

        // Sort by score descending, take top N
        let mut sorted: Vec<(String, i64, Vec<String>)> = scores
            .into_iter()
            .map(|(pid, (score, reasons))| (pid, score, reasons))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit as usize);

        // Fetch page details and filter out deleted pages
        let mut result = Vec::new();
        for (pid, score, reasons) in sorted {
            if let Ok(page) = self.get_page_by_id(&pid) {
                if page.deleted_at.is_none() {
                    // Deduplicate reasons
                    let unique_reasons: Vec<String> = {
                        let mut seen = std::collections::HashSet::new();
                        reasons
                            .into_iter()
                            .filter(|r| seen.insert(r.clone()))
                            .collect()
                    };
                    result.push(RelatedPage {
                        id: page.id,
                        title: page.title,
                        icon: page.icon,
                        score,
                        reasons: unique_reasons,
                    });
                }
            }
        }
        Ok(result)
    }

    // ── Attachments ──

    pub fn create_attachment(&self, input: &CreateAttachmentInput) -> Result<Attachment, DbError> {
        if input.data.len() > MAX_ATTACHMENT_BYTES {
            return Err(DbError::InvalidInput(format!(
                "attachment exceeds {} MB limit",
                MAX_ATTACHMENT_BYTES / 1024 / 1024
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let is_image = input.mime_type.starts_with("image/");
        let file_size = input.data.len() as i64;
        self.conn.execute(
            "INSERT INTO attachments (id, page_id, file_name, mime_type, file_size, is_image, data, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, input.page_id, input.file_name, input.mime_type, file_size, is_image as i64, input.data, now],
        )?;
        Ok(Attachment {
            id,
            page_id: input.page_id.clone(),
            file_name: input.file_name.clone(),
            mime_type: input.mime_type.clone(),
            file_size,
            is_image,
            created_at: now,
        })
    }

    pub fn get_attachment(&self, id: &str) -> Result<(Attachment, Vec<u8>), DbError> {
        self.conn
            .query_row(
                "SELECT id, page_id, file_name, mime_type, file_size, is_image, data, created_at FROM attachments WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        Attachment {
                            id: row.get(0)?,
                            page_id: row.get(1)?,
                            file_name: row.get(2)?,
                            mime_type: row.get(3)?,
                            file_size: row.get(4)?,
                            is_image: row.get::<_, i64>(5)? != 0,
                            created_at: row.get(7)?,
                        },
                        row.get::<_, Vec<u8>>(6)?,
                    ))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("attachment {}", id))
                }
                other => DbError::Sqlite(other),
            })
    }

    pub fn get_attachments_for_page(&self, page_id: &str) -> Result<Vec<Attachment>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, file_name, mime_type, file_size, is_image, created_at FROM attachments WHERE page_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![page_id], |row| {
            Ok(Attachment {
                id: row.get(0)?,
                page_id: row.get(1)?,
                file_name: row.get(2)?,
                mime_type: row.get(3)?,
                file_size: row.get(4)?,
                is_image: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })?;
        let mut attachments = Vec::new();
        for r in rows {
            attachments.push(r?);
        }
        Ok(attachments)
    }

    pub fn delete_attachment(&self, id: &str) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM attachments WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("attachment {}", id)));
        }
        Ok(())
    }

    // ── Audio Notes ──

    pub fn create_audio_note(&self, input: &CreateAudioNoteInput) -> Result<AudioNote, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO audio_notes (id, page_id, attachment_id, duration_sec, title, transcription, created_at) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)",
            params![id, input.page_id, input.attachment_id, input.duration_sec, input.title, now],
        )?;
        Ok(AudioNote {
            id,
            page_id: input.page_id.clone(),
            attachment_id: input.attachment_id.clone(),
            duration_sec: input.duration_sec,
            title: input.title.clone(),
            transcription: None,
            created_at: now,
        })
    }

    pub fn get_audio_note(&self, id: &str) -> Result<AudioNote, DbError> {
        self.conn
            .query_row(
                "SELECT id, page_id, attachment_id, duration_sec, title, transcription, created_at FROM audio_notes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(AudioNote {
                        id: row.get(0)?,
                        page_id: row.get(1)?,
                        attachment_id: row.get(2)?,
                        duration_sec: row.get(3)?,
                        title: row.get(4)?,
                        transcription: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("audio_note {}", id))
                }
                other => DbError::Sqlite(other),
            })
    }

    pub fn get_audio_notes_for_page(&self, page_id: &str) -> Result<Vec<AudioNote>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, attachment_id, duration_sec, title, transcription, created_at FROM audio_notes WHERE page_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![page_id], |row| {
            Ok(AudioNote {
                id: row.get(0)?,
                page_id: row.get(1)?,
                attachment_id: row.get(2)?,
                duration_sec: row.get(3)?,
                title: row.get(4)?,
                transcription: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn update_audio_note(&self, input: &UpdateAudioNoteInput) -> Result<AudioNote, DbError> {
        let current = self.get_audio_note(&input.id)?;
        let title = input.title.as_ref().unwrap_or(&current.title);
        let transcription = input
            .transcription
            .as_ref()
            .or(current.transcription.as_ref());
        self.conn.execute(
            "UPDATE audio_notes SET title = ?1, transcription = ?2 WHERE id = ?3",
            params![title, transcription, input.id],
        )?;
        self.get_audio_note(&input.id)
    }

    pub fn delete_audio_note(&self, id: &str) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM audio_notes WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("audio_note {}", id)));
        }
        Ok(())
    }

    // ── Sync state (CRDT) ──

    pub fn get_sync_doc(&self) -> Result<Option<Vec<u8>>, DbError> {
        self.conn
            .query_row(
                "SELECT doc FROM sync_state WHERE id = 'workspace'",
                [],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(DbError::Sqlite(other)),
            })
    }

    pub fn save_sync_doc(&self, doc: &[u8]) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sync_state (id, doc, updated_at)
             VALUES ('workspace', ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET doc = ?1, updated_at = ?2",
            params![doc, now],
        )?;
        Ok(())
    }

    pub fn get_sync_doc_updated_at(&self) -> Result<Option<String>, DbError> {
        self.conn
            .query_row(
                "SELECT updated_at FROM sync_state WHERE id = 'workspace'",
                [],
                |row| row.get::<_, String>(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(DbError::Sqlite(other)),
            })
    }

    // ── Sync queue ──

    pub fn enqueue_sync_op(
        &self,
        page_id: &str,
        peer_id: Option<&str>,
        operation: &str,
    ) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sync_queue (id, page_id, peer_id, operation, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
            params![id, page_id, peer_id, operation, now],
        )?;
        Ok(())
    }

    pub fn get_pending_sync_ops(&self) -> Result<Vec<SyncQueueEntry>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, peer_id, operation FROM sync_queue WHERE status = 'pending' ORDER BY created_at"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SyncQueueEntry {
                id: row.get::<_, String>(0)?,
                page_id: row.get::<_, String>(1)?,
                peer_id: row.get::<_, Option<String>>(2)?,
                operation: row.get::<_, String>(3)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn complete_sync_op(&self, id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE sync_queue SET status = 'completed' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn get_pending_sync_count(&self) -> Result<i64, DbError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM sync_queue WHERE status = 'pending'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(DbError::Sqlite)
    }

    pub fn clear_completed_sync_ops(&self) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM sync_queue WHERE status = 'completed'", [])?;
        Ok(())
    }

    pub fn set_last_sync_time(&self, time: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES ('last_sync', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![time],
        )?;
        Ok(())
    }

    pub fn get_last_sync_time(&self) -> Result<Option<String>, DbError> {
        self.get_setting("last_sync")
    }
}

fn build_tree(pages: Vec<Page>, parent_id: Option<&str>) -> Vec<PageTreeNode> {
    let mut children_map: HashMap<Option<String>, Vec<Page>> = HashMap::new();
    for page in pages {
        children_map
            .entry(page.parent_id.clone())
            .or_default()
            .push(page);
    }
    build_tree_from_map(&mut children_map, parent_id)
}

fn build_tree_from_map(
    map: &mut HashMap<Option<String>, Vec<Page>>,
    parent_id: Option<&str>,
) -> Vec<PageTreeNode> {
    let children = map
        .remove(&parent_id.map(|s| s.to_string()))
        .unwrap_or_default();
    children
        .into_iter()
        .map(|page| {
            let pid = page.id.clone();
            let child_nodes = build_tree_from_map(map, Some(&pid));
            PageTreeNode {
                page,
                children: child_nodes,
            }
        })
        .collect()
}

fn build_tree_meta(pages: Vec<PageMetadata>, parent_id: Option<&str>) -> Vec<PageTreeNodeMeta> {
    let mut children_map: HashMap<Option<String>, Vec<PageMetadata>> = HashMap::new();
    for page in pages {
        children_map
            .entry(page.parent_id.clone())
            .or_default()
            .push(page);
    }
    build_tree_meta_from_map(&mut children_map, parent_id)
}

fn build_tree_meta_from_map(
    map: &mut HashMap<Option<String>, Vec<PageMetadata>>,
    parent_id: Option<&str>,
) -> Vec<PageTreeNodeMeta> {
    let children = map
        .remove(&parent_id.map(|s| s.to_string()))
        .unwrap_or_default();
    children
        .into_iter()
        .map(|page| {
            let pid = page.id.clone();
            let child_nodes = build_tree_meta_from_map(map, Some(&pid));
            PageTreeNodeMeta {
                page,
                children: child_nodes,
            }
        })
        .collect()
}

fn format_fts_query(query: &str) -> String {
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() {
        return String::new();
    }
    terms
        .iter()
        .map(|t| {
            let cleaned: String = t
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if cleaned.is_empty() {
                String::new()
            } else {
                format!("\"{}\"*", cleaned)
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_search_snippet(snippet: &str) -> String {
    escape_html(snippet)
        .replace(SEARCH_HIGHLIGHT_START, "<mark>")
        .replace(SEARCH_HIGHLIGHT_END, "</mark>")
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn extract_wiki_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut chars = content.chars().peekable();
    let mut buffer = String::new();
    let mut in_link = false;

    while let Some(c) = chars.next() {
        if c == '[' && chars.peek() == Some(&'[') {
            chars.next();
            in_link = true;
            buffer.clear();
        } else if c == ']' && chars.peek() == Some(&']') && in_link {
            chars.next();
            in_link = false;
            let trimmed = buffer.trim().to_string();
            if !trimmed.is_empty() {
                links.push(trimmed);
            }
        } else if in_link {
            buffer.push(c);
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let db = Database {
            conn: Connection::open_in_memory().unwrap(),
        };
        db.run_migrations().unwrap();
        db
    }

    #[test]
    fn test_create_and_get_page() {
        let db = test_db();
        let page = db.create_page(&CreatePageInput {
            parent_id: None,
            title: "Test Page".to_string(),
            content: Some(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#.to_string()),
            icon: Some("📄".to_string()),
        }).unwrap();
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.icon, Some("📄".to_string()));

        let fetched = db.get_page_by_id(&page.id).unwrap();
        assert_eq!(fetched.id, page.id);
        assert_eq!(fetched.title, "Test Page");
    }

    #[test]
    fn test_update_page() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Original".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let updated = db
            .update_page(&UpdatePageInput {
                id: page.id.clone(),
                title: Some("Updated".to_string()),
                content: Some(r#"{"type":"doc","content":[]}"#.to_string()),
                icon: Some("📝".to_string()),
                cover_color: None,
                pinned: Some(true),
            })
            .unwrap();

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.icon, Some("📝".to_string()));
        assert!(updated.pinned);
    }

    #[test]
    fn test_soft_delete_and_restore() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "To Delete".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        db.delete_page(&page.id).unwrap();
        let deleted = db.get_page_by_id(&page.id).unwrap();
        assert!(deleted.deleted_at.is_some());

        db.restore_page(&page.id).unwrap();
        let restored = db.get_page_by_id(&page.id).unwrap();
        assert!(restored.deleted_at.is_none());
    }

    #[test]
    fn test_create_tag_and_set_page_tags() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Tagged Page".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let tag = db
            .create_tag(&CreateTagInput {
                name: "important".to_string(),
                color: Some("#ff0000".to_string()),
            })
            .unwrap();

        db.set_page_tags(&page.id, &[tag.id.clone()]).unwrap();
        let tags = db.get_page_tags(&page.id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "important");
    }

    #[test]
    fn test_rebuild_links() {
        let db = test_db();
        let target = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Target Page".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let source = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Source Page".to_string(),
                content: Some(
                    r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"[[Target Page]]"}]}]}"#
                        .to_string(),
                ),
                icon: None,
            })
            .unwrap();

        db.rebuild_links_for_page(&source.id, &source.content)
            .unwrap();

        let backlinks = db.get_backlinks(&target.id).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_page_id, source.id);
    }

    #[test]
    fn test_secure_delete_page() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Secure Delete".to_string(),
                content: Some(r#"{"type":"doc","content":[]}"#.to_string()),
                icon: Some("🔒".to_string()),
            })
            .unwrap();

        db.create_revision(&page.id, &page.title, &page.content)
            .unwrap();

        db.secure_delete_page(&page.id).unwrap();

        let result = db.get_page_by_id(&page.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_revision_pruning() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Prune Test".to_string(),
                content: Some(r#"{"type":"doc","content":[]}"#.to_string()),
                icon: None,
            })
            .unwrap();

        for i in 0..60 {
            db.create_revision(&page.id, &format!("Rev {i}"), &page.content)
                .unwrap();
        }

        db.prune_revisions(&page.id, 50).unwrap();
        let revisions = db.get_page_revisions(&page.id).unwrap();
        assert_eq!(revisions.len(), 50);
    }

    #[test]
    fn test_settings() {
        let db = test_db();
        db.set_setting("theme", "dark").unwrap();
        let value = db.get_setting("theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));

        let all = db.get_all_settings().unwrap();
        assert!(all.iter().any(|(k, v)| k == "theme" && v == "dark"));
    }

    #[test]
    fn test_page_tree() {
        let db = test_db();
        let parent = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Parent".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let child = db
            .create_page(&CreatePageInput {
                parent_id: Some(parent.id.clone()),
                title: "Child".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let tree = db.get_page_tree().unwrap();
        assert!(!tree.is_empty());
        let parent_node = tree.iter().find(|n| n.page.id == parent.id);
        assert!(parent_node.is_some());
        assert!(parent_node
            .unwrap()
            .children
            .iter()
            .any(|c| c.page.id == child.id));
    }

    #[test]
    fn test_move_page_rejects_self_and_descendant_parent() {
        let db = test_db();
        let parent = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Parent".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let child = db
            .create_page(&CreatePageInput {
                parent_id: Some(parent.id.clone()),
                title: "Child".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let self_move = db.move_page(&MovePageInput {
            id: parent.id.clone(),
            parent_id: Some(parent.id.clone()),
            sort_order: 1,
        });
        assert!(matches!(self_move, Err(DbError::InvalidInput(_))));

        let descendant_move = db.move_page(&MovePageInput {
            id: parent.id.clone(),
            parent_id: Some(child.id),
            sort_order: 1,
        });
        assert!(matches!(descendant_move, Err(DbError::InvalidInput(_))));
    }

    #[test]
    fn test_create_attachment_rejects_oversized_blob() {
        let db = test_db();
        let page = db
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Attachment Parent".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();

        let result = db.create_attachment(&CreateAttachmentInput {
            page_id: page.id,
            file_name: "too-large.bin".to_string(),
            mime_type: "application/octet-stream".to_string(),
            data: vec![0; MAX_ATTACHMENT_BYTES + 1],
        });

        assert!(matches!(result, Err(DbError::InvalidInput(_))));
    }

    #[test]
    fn test_search() {
        let db = test_db();
        db.create_page(&CreatePageInput {
            parent_id: None,
            title: "Searchable Note".to_string(),
            content: Some(
                r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"unique content with <img src=x onerror=alert(1)> here"}]}]}"#
                    .to_string(),
            ),
            icon: None,
        })
        .unwrap();

        let results = db.search_pages("unique", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].snippet.contains("<mark>unique</mark>"));
        assert!(!results[0].snippet.contains("<img"));
        assert!(results[0].snippet.contains("&lt;img"));
    }
}
