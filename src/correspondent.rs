use serde::Deserialize;

/// A correspondent
#[derive(Debug, Clone, Deserialize)]
pub struct Correspondent {
    pub id: crate::id::CorrespondentId,
    pub slug: String,
    pub name: String,

    /// The number of documents associated with this correspondent.
    pub document_count: u32,

    /// The user who owns this correspondent, if any.
    pub owner: Option<crate::id::UserId>,

    /// Whether the current user can change this correspondent.
    pub user_can_change: bool,
}
