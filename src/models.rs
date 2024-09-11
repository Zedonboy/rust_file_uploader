use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow)]
pub struct DocumentPart {
    pub id: i32,
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub file_id : String
}