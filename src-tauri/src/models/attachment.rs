use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub page_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub is_image: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttachmentInput {
    pub page_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}
