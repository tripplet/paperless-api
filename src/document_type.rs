use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::user::UserId;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct DocumentTypeId(pub u32);

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
