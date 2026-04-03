use serde::{Deserialize, Serialize};

use crate::user::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct DocumentTypeId(pub i32);

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentType {
    pub id: DocumentTypeId,
    pub slug: String,
    pub name: String,

    pub is_insensitive: Option<bool>,
    pub document_count: u32,
    pub owner: Option<UserId>,
    pub user_can_change: bool,
}

impl std::fmt::Display for DocumentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
