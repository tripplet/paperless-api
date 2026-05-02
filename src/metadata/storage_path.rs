//! Types related to storage paths.

use serde::{Deserialize, Serialize};

use paperless_api_macros::CreateDto;

use super::MatchAlgorithm;
use super::permission::ItemPermissions;

/// A storage path
#[derive(Debug, Clone, Deserialize, Serialize, CreateDto)]
#[api_info(endpoint = "storage_paths")]
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
