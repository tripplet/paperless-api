//! Types related to storage paths.

use serde::Deserialize;

use crate::permission::ItemPermissions;
use crate::util::MatchAlgorithm;

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

    #[serde(default)]
    pub document_count: u32,

    pub owner: Option<crate::id::UserId>,

    /// The permissions for this tag.
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
