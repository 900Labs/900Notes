use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    pub source_page_id: String,
    pub target_page_id: String,
    pub link_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backlink {
    pub id: String,
    pub source_page_id: String,
    pub source_page_title: String,
    pub source_page_icon: Option<String>,
    pub link_text: String,
    pub created_at: String,
}
