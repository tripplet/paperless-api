use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Correspondent {
    pub id: crate::id::CorrespondentId,
    pub slug: String,
    pub name: String,

    pub document_count: u32,
    pub owner: Option<crate::id::UserId>,
    pub user_can_change: bool,
}
