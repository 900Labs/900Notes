use automerge::{
    sync::{self, SyncDoc},
    transaction::Transactable,
    AutoCommit, ObjId, ObjType, ReadDoc, ScalarValue, Value,
};

use crate::db::Database;
use crate::models::Page;

const PAGES_KEY: &str = "pages";

pub struct CrdtService {
    doc: AutoCommit,
}

impl CrdtService {
    pub fn new() -> Self {
        CrdtService {
            doc: AutoCommit::new(),
        }
    }

    pub fn load_from_db(db: &Database) -> Result<Self, String> {
        match db.get_sync_doc().map_err(|e| e.to_string())? {
            Some(bytes) => {
                let doc = AutoCommit::load(&bytes).map_err(|e| format!("Automerge load: {e}"))?;
                Ok(CrdtService { doc })
            }
            None => {
                let mut service = CrdtService::new();
                service.sync_all_pages_to_crdt(db)?;
                Ok(service)
            }
        }
    }

    pub fn save_to_db(&mut self, db: &Database) -> Result<(), String> {
        let bytes = self.doc.save();
        db.save_sync_doc(&bytes).map_err(|e| e.to_string())
    }

    pub fn sync_all_pages_to_crdt(&mut self, db: &Database) -> Result<(), String> {
        let pages = db.get_all_pages_for_sync().map_err(|e| e.to_string())?;
        for page in &pages {
            self.upsert_page_in_crdt(page);
        }
        Ok(())
    }

    pub fn upsert_page_in_crdt(&mut self, page: &Page) {
        let pages_map = self.ensure_pages_map();
        let page_obj = self.ensure_page_map(&pages_map, &page.id);

        let _ = self.doc.put(&page_obj, "id", page.id.as_str());
        let _ = self.doc.put(&page_obj, "title", page.title.as_str());
        let _ = self.doc.put(&page_obj, "content", page.content.as_str());
        let _ = self.doc.put(
            &page_obj,
            "parentId",
            page.parent_id.as_deref().unwrap_or(""),
        );
        let _ = self
            .doc
            .put(&page_obj, "icon", page.icon.as_deref().unwrap_or(""));
        let _ = self.doc.put(
            &page_obj,
            "coverColor",
            page.cover_color.as_deref().unwrap_or(""),
        );
        let _ = self
            .doc
            .put(&page_obj, "createdAt", page.created_at.as_str());
        let _ = self
            .doc
            .put(&page_obj, "updatedAt", page.updated_at.as_str());
        let _ = self.doc.put(
            &page_obj,
            "deletedAt",
            page.deleted_at.as_deref().unwrap_or(""),
        );
        let _ = self.doc.put(&page_obj, "pinned", page.pinned);
        let _ = self.doc.put(&page_obj, "sortOrder", page.sort_order);
    }

    fn ensure_pages_map(&mut self) -> ObjId {
        match self.doc.get(&automerge::ROOT, PAGES_KEY) {
            Ok(Some((Value::Object(ObjType::Map), obj_id))) => obj_id,
            _ => {
                let _ = self
                    .doc
                    .put_object(&automerge::ROOT, PAGES_KEY, ObjType::Map);
                self.doc
                    .get(&automerge::ROOT, PAGES_KEY)
                    .ok()
                    .flatten()
                    .and_then(|(v, id)| match v {
                        Value::Object(ObjType::Map) => Some(id),
                        _ => None,
                    })
                    .unwrap_or_else(|| automerge::ROOT.to_owned())
            }
        }
    }

    fn ensure_page_map(&mut self, pages_map: &ObjId, page_id: &str) -> ObjId {
        match self.doc.get(pages_map, page_id) {
            Ok(Some((Value::Object(ObjType::Map), obj_id))) => obj_id,
            _ => {
                let _ = self.doc.put_object(pages_map, page_id, ObjType::Map);
                self.doc
                    .get(pages_map, page_id)
                    .ok()
                    .flatten()
                    .and_then(|(v, id)| match v {
                        Value::Object(ObjType::Map) => Some(id),
                        _ => None,
                    })
                    .unwrap_or_else(|| pages_map.to_owned())
            }
        }
    }

    pub fn read_pages_from_crdt(&self) -> Vec<PageSyncCrdt> {
        let pages_map = match self.doc.get(&automerge::ROOT, PAGES_KEY) {
            Ok(Some((Value::Object(ObjType::Map), obj_id))) => obj_id,
            _ => return Vec::new(),
        };

        let mut result = Vec::new();
        let keys: Vec<String> = self.doc.keys(&pages_map).collect();
        for key in keys {
            if let Ok(Some((Value::Object(ObjType::Map), page_obj))) =
                self.doc.get(&pages_map, &key)
            {
                let get_str = |field: &str| -> String {
                    self.doc
                        .get(&page_obj, field)
                        .ok()
                        .flatten()
                        .and_then(|(v, _)| match v {
                            Value::Scalar(s) => s.to_str().map(|s| s.to_string()),
                            _ => None,
                        })
                        .unwrap_or_default()
                };
                let get_opt_str = |field: &str| -> Option<String> {
                    let s = get_str(field);
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                };
                let get_bool = |field: &str| -> bool {
                    self.doc
                        .get(&page_obj, field)
                        .ok()
                        .flatten()
                        .and_then(|(v, _)| match v {
                            Value::Scalar(s) => match s.as_ref() {
                                ScalarValue::Boolean(b) => Some(*b),
                                _ => None,
                            },
                            _ => None,
                        })
                        .unwrap_or(false)
                };
                let get_i64 = |field: &str| -> i64 {
                    self.doc
                        .get(&page_obj, field)
                        .ok()
                        .flatten()
                        .and_then(|(v, _)| match v {
                            Value::Scalar(s) => match s.as_ref() {
                                ScalarValue::Int(n) => Some(*n),
                                ScalarValue::Uint(n) => Some(*n as i64),
                                _ => None,
                            },
                            _ => None,
                        })
                        .unwrap_or(0)
                };
                let page = PageSyncCrdt {
                    id: get_str("id"),
                    title: get_str("title"),
                    content: get_str("content"),
                    parent_id: get_opt_str("parentId"),
                    icon: get_opt_str("icon"),
                    cover_color: get_opt_str("coverColor"),
                    created_at: get_str("createdAt"),
                    updated_at: get_str("updatedAt"),
                    deleted_at: get_opt_str("deletedAt"),
                    pinned: get_bool("pinned"),
                    sort_order: get_i64("sortOrder"),
                };
                result.push(page);
            }
        }
        result
    }

    pub fn merge_remote_changes(&mut self, remote_bytes: &[u8]) -> Result<(), String> {
        let mut remote_doc =
            AutoCommit::load(remote_bytes).map_err(|e| format!("Automerge load remote: {e}"))?;
        self.doc
            .merge(&mut remote_doc)
            .map_err(|e| format!("Automerge merge: {e}"))?;
        Ok(())
    }

    pub fn generate_sync_message(&mut self, state: &mut sync::State) -> Option<sync::Message> {
        self.doc.sync().generate_sync_message(state)
    }

    pub fn receive_sync_message(
        &mut self,
        state: &mut sync::State,
        message: sync::Message,
    ) -> Result<(), String> {
        self.doc
            .sync()
            .receive_sync_message(state, message)
            .map_err(|e| format!("Automerge receive: {e}"))
    }

    pub fn get_doc_bytes(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    pub fn has_pending_changes(&self) -> bool {
        self.doc.pending_ops() > 0
    }
}

#[derive(Debug, Clone)]
pub struct PageSyncCrdt {
    pub id: String,
    pub title: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub cover_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub pinned: bool,
    pub sort_order: i64,
}

impl PageSyncCrdt {
    pub fn to_page_meta(&self) -> crate::models::PageSyncMeta {
        crate::models::PageSyncMeta {
            id: self.id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            parent_id: self.parent_id.clone(),
            icon: self.icon.clone(),
            cover_color: self.cover_color.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            deleted_at: self.deleted_at.clone(),
            pinned: self.pinned,
            sort_order: self.sort_order,
        }
    }
}
