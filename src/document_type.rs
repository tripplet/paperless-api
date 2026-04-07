use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentType {
    pub id: crate::id::DocumentTypeId,
    pub slug: String,
    pub name: String,

    pub is_insensitive: Option<bool>,
    pub document_count: u32,
    pub owner: Option<crate::id::UserId>,
    pub user_can_change: bool,
}
