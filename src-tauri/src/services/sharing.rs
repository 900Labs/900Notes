use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::db::Database;
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareBundle {
    pub version: String,
    pub exported_at: String,
    pub pages: Vec<Page>,
    pub tags: Vec<Tag>,
    pub page_tags: HashMap<String, Vec<String>>,
    pub page_properties: HashMap<String, Vec<PageProperty>>,
    pub attachments_meta: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMeta {
    pub id: String,
    pub page_id: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedBundle {
    pub version: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub fn export_share_bundle(
    db: &Database,
    page_ids: &[String],
) -> Result<ShareBundle, crate::db::DbError> {
    let mut pages = Vec::new();
    let mut page_tags: HashMap<String, Vec<String>> = HashMap::new();
    let mut page_properties: HashMap<String, Vec<PageProperty>> = HashMap::new();
    let attachments_meta = Vec::new();

    for page_id in page_ids {
        let page = db.get_page_by_id(page_id)?;
        pages.push(page);

        let tags = db.get_page_tags(page_id)?;
        page_tags.insert(
            page_id.to_string(),
            tags.iter().map(|t| t.id.clone()).collect(),
        );

        let props = db.get_page_properties(page_id)?;
        page_properties.insert(page_id.to_string(), props);
    }

    // Collect unique tag IDs
    let mut tag_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for ids in page_tags.values() {
        for id in ids.iter() {
            tag_ids.insert(id.clone());
        }
    }

    let all_tags = db.get_all_tags()?;
    let tags: Vec<Tag> = all_tags
        .into_iter()
        .filter(|t| tag_ids.contains(&t.id))
        .collect();

    Ok(ShareBundle {
        version: "1.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        pages,
        tags,
        page_tags,
        page_properties,
        attachments_meta,
    })
}

pub fn import_share_bundle(
    db: &Database,
    bundle: &ShareBundle,
) -> Result<usize, crate::db::DbError> {
    let mut count = 0;
    for page in &bundle.pages {
        db.upsert_page_from_sync(&PageSyncMeta {
            id: page.id.clone(),
            title: page.title.clone(),
            content: page.content.clone(),
            parent_id: page.parent_id.clone(),
            icon: page.icon.clone(),
            cover_color: page.cover_color.clone(),
            created_at: page.created_at.clone(),
            updated_at: page.updated_at.clone(),
            deleted_at: page.deleted_at.clone(),
            pinned: page.pinned,
            sort_order: page.sort_order,
        })?;
        count += 1;
    }

    for tag in &bundle.tags {
        db.conn_execute(
            "INSERT OR IGNORE INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![tag.id, tag.name, tag.color, tag.created_at],
        )?;
    }

    for (page_id, tag_ids) in &bundle.page_tags {
        for tag_id in tag_ids {
            db.conn_execute(
                "INSERT OR IGNORE INTO page_tags (page_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![page_id, tag_id],
            )?;
        }
    }

    for (page_id, props) in &bundle.page_properties {
        for prop in props {
            db.set_page_property(&SetPropertyInput {
                page_id: page_id.clone(),
                key: prop.key.clone(),
                value: prop.value.clone(),
            })?;
        }
    }

    db.rebuild_all_links()?;
    Ok(count)
}

fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut current = [0u8; 32];
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(salt);
    current.copy_from_slice(&hasher.finalize());

    for _ in 0..100_000 {
        let mut h = Sha256::new();
        h.update(current);
        current.copy_from_slice(&h.finalize());
    }
    current
}

pub fn encrypt_bundle(bundle: &ShareBundle, passphrase: &str) -> Result<String, String> {
    let json = serde_json::to_vec(bundle).map_err(|e| format!("Serialize: {e}"))?;

    let mut salt = [0u8; 32];
    getrandom::getrandom(&mut salt).map_err(|e| format!("CSPRNG: {e}"))?;
    let key = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| format!("CSPRNG: {e}"))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, json.as_ref())
        .map_err(|e| format!("AES encrypt: {e}"))?;

    let enc = EncryptedBundle {
        version: "1.0.0".to_string(),
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(&nonce_bytes[..12]),
        ciphertext: BASE64.encode(&ciphertext),
    };

    serde_json::to_string_pretty(&enc).map_err(|e| format!("Serialize enc: {e}"))
}

pub fn decrypt_bundle(enc_str: &str, passphrase: &str) -> Result<ShareBundle, String> {
    let enc: EncryptedBundle =
        serde_json::from_str(enc_str).map_err(|e| format!("Parse encrypted bundle: {e}"))?;

    let salt = BASE64
        .decode(&enc.salt)
        .map_err(|e| format!("Decode salt: {e}"))?;
    let nonce_bytes = BASE64
        .decode(&enc.nonce)
        .map_err(|e| format!("Decode nonce: {e}"))?;
    let ciphertext = BASE64
        .decode(&enc.ciphertext)
        .map_err(|e| format!("Decode ciphertext: {e}"))?;

    let key = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Invalid passphrase or corrupted bundle".to_string())?;

    serde_json::from_slice(&plaintext).map_err(|e| format!("Deserialize bundle: {e}"))
}
