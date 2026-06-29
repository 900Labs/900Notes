use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProperty {
    pub id: String,
    pub page_id: String,
    pub key: String,
    pub value: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPropertyInput {
    pub page_id: String,
    pub key: String,
    pub value: String,
}
