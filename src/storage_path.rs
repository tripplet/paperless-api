use derive_more::Display;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

/// A storage path
#[derive(Debug, Clone, Deserialize)]
pub struct StoragePath {
    pub id: crate::id::StoragePathId,

    pub slug: String,

    pub name: String,

    pub path: String,

    #[serde(rename = "match")]
    pub match_pattern: Option<String>,

    pub matching_algorithm: MatchAlgorithm,

    pub is_insensitive: bool,

    pub document_count: u32,
    pub owner: Option<crate::id::UserId>,
    pub user_can_change: bool,
}

/// The matching algorithm to use for storage path matching
#[derive(Debug, Clone, Copy, Display, Deserialize_repr)]
#[repr(u8)]
pub enum MatchAlgorithm {
    None,
    AnyWord,
    AllWords,
    ExactMatch,
    Regex,
    Fuzzy,
    Automatic,
}
