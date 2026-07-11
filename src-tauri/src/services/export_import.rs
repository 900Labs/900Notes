use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::db::{BackupValue, Database, DbError};

pub const BACKUP_FORMAT_VERSION: &str = "1.0.0";

const EXPORT_TABLES: &[&str] = &[
    "pages",
    "tags",
    "page_tags",
    "page_properties",
    "templates",
    "saved_searches",
    "smart_folders",
    "page_revisions",
    "favorites",
    "tag_groups",
    "tag_group_members",
    "attachments",
    "audio_notes",
    "settings",
];

// Child tables first so an exact restore can clear them with foreign keys on.
const DELETE_ORDER: &[&str] = &[
    "sync_queue",
    "sync_state",
    "audio_notes",
    "attachments",
    "tag_group_members",
    "favorites",
    "page_revisions",
    "page_properties",
    "page_tags",
    "links",
    "smart_folders",
    "saved_searches",
    "templates",
    "tag_groups",
    "tags",
    "pages",
    "settings",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceExport {
    pub format: String,
    pub version: String,
    pub exported_at: String,
    pub restore_mode: String,
    pub tables: BTreeMap<String, Vec<BTreeMap<String, BackupValue>>>,
}

pub fn export_workspace(db: &Database) -> Result<WorkspaceExport, DbError> {
    let mut tables = BTreeMap::new();
    for table in EXPORT_TABLES {
        tables.insert((*table).to_string(), db.export_table(table)?);
    }
    Ok(WorkspaceExport {
        format: "900notes-workspace-backup".to_string(),
        version: BACKUP_FORMAT_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        restore_mode: "replace".to_string(),
        tables,
    })
}

pub fn import_workspace(db: &Database, data: &WorkspaceExport) -> Result<usize, DbError> {
    validate_workspace(db, data)?;

    db.begin_transaction()?;
    let result = (|| {
        db.conn_execute("PRAGMA defer_foreign_keys = ON", &[])?;
        for table in DELETE_ORDER {
            db.conn_execute(&format!("DELETE FROM {table}"), &[])?;
        }
        for table in EXPORT_TABLES {
            if let Some(rows) = data.tables.get(*table) {
                db.import_table_rows(table, rows)?;
            }
        }
        db.rebuild_all_links()?;
        Ok(data.tables.get("pages").map_or(0, Vec::len))
    })();
    match result {
        Ok(count) => match db.commit_transaction() {
            Ok(()) => Ok(count),
            Err(error) => {
                let _ = db.rollback_transaction();
                Err(error)
            }
        },
        Err(error) => {
            let _ = db.rollback_transaction();
            Err(error)
        }
    }
}

pub fn validate_workspace(db: &Database, data: &WorkspaceExport) -> Result<(), DbError> {
    if data.format != "900notes-workspace-backup" || data.version != BACKUP_FORMAT_VERSION {
        return Err(DbError::InvalidInput(format!(
            "Unsupported backup format or version. Expected 900notes-workspace-backup {BACKUP_FORMAT_VERSION}"
        )));
    }
    if data.restore_mode != "replace" {
        return Err(DbError::InvalidInput(
            "This release supports exact replace restores only".to_string(),
        ));
    }
    for table in data.tables.keys() {
        if !EXPORT_TABLES.contains(&table.as_str()) {
            return Err(DbError::InvalidInput(format!(
                "Unexpected backup table: {table}"
            )));
        }
    }
    for table in EXPORT_TABLES {
        if !data.tables.contains_key(*table) {
            return Err(DbError::InvalidInput(format!(
                "Backup is incomplete. Missing table: {table}"
            )));
        }
        db.validate_backup_rows(table, &data.tables[*table])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreatePageInput, CreateTagInput};

    fn test_db() -> Database {
        let root = std::env::temp_dir().join(format!("900notes-backup-{}", uuid::Uuid::new_v4()));
        Database::open(&root).unwrap()
    }

    #[test]
    fn exact_restore_round_trips_properties_favorites_and_attachment_bytes() {
        let source = test_db();
        let page = source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Portable note".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let tag = source
            .create_tag(&CreateTagInput {
                name: "portable".to_string(),
                color: None,
            })
            .unwrap();
        source.set_page_tags(&page.id, &[tag.id]).unwrap();
        source.conn_execute(
            "INSERT INTO page_properties (id, page_id, key, value, sort_order) VALUES ('property', ?1, 'status', 'ready', 0)",
            &[&page.id],
        ).unwrap();
        source.conn_execute(
            "INSERT INTO favorites (id, page_id, sort_order, created_at) VALUES ('favorite', ?1, 0, 'now')",
            &[&page.id],
        ).unwrap();
        source.conn_execute(
            "INSERT INTO attachments (id, page_id, file_name, mime_type, file_size, is_image, data, created_at) VALUES ('attachment', ?1, 'proof.bin', 'application/octet-stream', 4, 0, ?2, 'now')",
            &[&page.id, &vec![0_u8, 1, 2, 255]],
        ).unwrap();

        let export = export_workspace(&source).unwrap();
        let target = test_db();
        import_workspace(&target, &export).unwrap();
        assert_eq!(target.get_all_pages().unwrap()[0].title, "Portable note");
        assert_eq!(
            target.get_page_properties(&page.id).unwrap()[0].value,
            "ready"
        );
        assert_eq!(target.get_favorites().unwrap()[0].page_id, page.id);
        assert_eq!(
            target.get_attachment("attachment").unwrap().1,
            vec![0, 1, 2, 255]
        );
    }

    #[test]
    fn rejects_unknown_backup_versions_without_mutating_database() {
        let db = test_db();
        let mut export = export_workspace(&db).unwrap();
        export.version = "99.0.0".to_string();
        assert!(import_workspace(&db, &export).is_err());
    }

    #[test]
    fn restores_nested_and_trashed_pages_when_children_arrive_first() {
        let source = test_db();
        let parent = source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Parent".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let child = source
            .create_page(&CreatePageInput {
                parent_id: Some(parent.id.clone()),
                title: "Child".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        source.delete_page(&parent.id).unwrap();
        let mut export = export_workspace(&source).unwrap();
        export.tables.get_mut("pages").unwrap().reverse();

        let target = test_db();
        import_workspace(&target, &export).unwrap();
        assert_eq!(
            target.get_page_by_id(&child.id).unwrap().parent_id,
            Some(parent.id.clone())
        );
        assert!(target
            .get_page_by_id(&parent.id)
            .unwrap()
            .deleted_at
            .is_some());
    }

    #[test]
    fn foreign_key_failure_rolls_back_exact_restore() {
        let source = test_db();
        let page = source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Incoming".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let tag = source
            .create_tag(&CreateTagInput {
                name: "tag".to_string(),
                color: None,
            })
            .unwrap();
        source.set_page_tags(&page.id, &[tag.id]).unwrap();
        let mut export = export_workspace(&source).unwrap();
        export.tables.get_mut("page_tags").unwrap()[0].insert(
            "tag_id".to_string(),
            BackupValue::Text("missing-tag".to_string()),
        );

        let target = test_db();
        target
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Keep me".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        assert!(import_workspace(&target, &export).is_err());
        assert_eq!(target.get_all_pages().unwrap()[0].title, "Keep me");
    }

    #[test]
    fn malformed_value_types_are_rejected_before_mutation() {
        let source = test_db();
        source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Incoming".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let mut export = export_workspace(&source).unwrap();
        export.tables.get_mut("pages").unwrap()[0]
            .insert("title".to_string(), BackupValue::Integer(7));

        let target = test_db();
        target
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Keep me".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        assert!(import_workspace(&target, &export).is_err());
        assert_eq!(target.get_all_pages().unwrap()[0].title, "Keep me");
    }

    #[test]
    fn missing_and_null_required_columns_are_rejected() {
        let source = test_db();
        source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Incoming".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let export = export_workspace(&source).unwrap();
        let target = test_db();

        let mut missing = export.clone();
        missing.tables.get_mut("pages").unwrap()[0].remove("title");
        assert!(import_workspace(&target, &missing).is_err());

        let mut null = export;
        null.tables.get_mut("pages").unwrap()[0].insert("title".to_string(), BackupValue::Null);
        assert!(import_workspace(&target, &null).is_err());
        assert!(target.get_all_pages().unwrap().is_empty());
    }

    #[test]
    fn restore_clears_old_sync_state_and_rebuilds_crdt_from_restored_pages() {
        let source = test_db();
        source
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Restored".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let export = export_workspace(&source).unwrap();

        let target = test_db();
        let old = target
            .create_page(&CreatePageInput {
                parent_id: None,
                title: "Old sync page".to_string(),
                content: None,
                icon: None,
            })
            .unwrap();
        let mut old_crdt = crate::services::crdt::CrdtService::load_from_db(&target).unwrap();
        old_crdt.save_to_db(&target).unwrap();
        target
            .enqueue_sync_op(&old.id, Some("peer"), "update")
            .unwrap();

        import_workspace(&target, &export).unwrap();
        assert_eq!(target.get_pending_sync_count().unwrap(), 0);
        let restored_crdt = crate::services::crdt::CrdtService::load_from_db(&target).unwrap();
        let titles = restored_crdt
            .read_pages_from_crdt()
            .into_iter()
            .map(|page| page.title)
            .collect::<Vec<_>>();
        assert_eq!(titles, vec!["Restored"]);
    }
}
