use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::Database;
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceExport {
    pub version: String,
    pub exported_at: String,
    pub pages: Vec<Page>,
    pub tags: Vec<Tag>,
    pub page_tags: HashMap<String, Vec<String>>,
    pub settings: Vec<(String, String)>,
}

pub fn export_workspace(db: &Database) -> Result<WorkspaceExport, crate::db::DbError> {
    let pages = db.get_all_pages()?;
    let tags = db.get_all_tags()?;
    let mut page_tags = HashMap::new();
    for page in &pages {
        let ptags = db.get_page_tags(&page.id)?;
        page_tags.insert(
            page.id.clone(),
            ptags.iter().map(|t| t.id.clone()).collect(),
        );
    }
    let settings = db.get_all_settings()?;
    Ok(WorkspaceExport {
        version: "0.1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        pages,
        tags,
        page_tags,
        settings,
    })
}

pub fn import_workspace(
    db: &Database,
    data: &WorkspaceExport,
) -> Result<usize, crate::db::DbError> {
    db.begin_transaction()?;
    let result: Result<usize, crate::db::DbError> = (|| {
        let mut count = 0;
        for page in &data.pages {
            db.conn_execute(
                "INSERT OR REPLACE INTO pages (id, parent_id, title, content, icon, cover_color, created_at, updated_at, deleted_at, pinned, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                &[&page.id, &page.parent_id, &page.title, &page.content, &page.icon, &page.cover_color, &page.created_at, &page.updated_at, &page.deleted_at, &(page.pinned as i64), &page.sort_order],
            )?;
            count += 1;
        }
        for tag in &data.tags {
            db.conn_execute(
                "INSERT OR REPLACE INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
                &[&tag.id, &tag.name, &tag.color, &tag.created_at],
            )?;
        }
        for (page_id, tag_ids) in &data.page_tags {
            for tag_id in tag_ids {
                db.conn_execute(
                    "INSERT OR IGNORE INTO page_tags (page_id, tag_id) VALUES (?1, ?2)",
                    &[page_id, tag_id],
                )?;
            }
        }
        for (key, value) in &data.settings {
            db.set_setting(key, value)?;
        }
        db.rebuild_all_links()?;
        Ok(count)
    })();
    if result.is_ok() {
        db.commit_transaction()?;
    } else {
        let _ = db.rollback_transaction();
    }
    result
}
