use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioNote {
    pub id: String,
    pub page_id: String,
    pub attachment_id: String,
    pub duration_sec: f64,
    pub title: String,
    pub transcription: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAudioNoteInput {
    pub page_id: String,
    pub attachment_id: String,
    pub duration_sec: f64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAudioNoteInput {
    pub id: String,
    pub title: Option<String>,
    pub transcription: Option<String>,
}
