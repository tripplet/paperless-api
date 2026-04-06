use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct CorrespondentId(pub u32);

#[derive(Debug, Clone, Deserialize)]
pub struct Correspondent {
    pub id: CorrespondentId,
    pub slug: String,
    pub name: String,

    pub document_count: u32,
    pub owner: Option<i32>,
    pub user_can_change: bool,
}
