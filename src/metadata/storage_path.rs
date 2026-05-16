//! Types related to storage paths.

use serde::{Deserialize, Serialize};

use paperless_api_macros::{CreateDto, Item, UpdateDto};

use super::MatchAlgorithm;
use super::permission::ItemPermissions;

/// A storage path.
#[derive(Debug, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]

pub struct StoragePath {
    /// Unique identifier of the storage path.
    #[dto(skip)]
    pub id: crate::id::StoragePathId,

    /// Slug of the storage path.
    #[dto(skip)]
    pub slug: String,

    /// Name of the storage path.
    pub name: String,
    pub path: String,

    /// Matching pattern for the storage path.
    #[serde(rename = "match")]
    pub match_pattern: Option<String>,

    /// Matching algorithm for the storage path.
    pub matching_algorithm: MatchAlgorithm,

    /// Whether the storage path matching is case-insensitive.
    pub is_insensitive: bool,

    /// The number of documents associated with this storage path.
    #[dto(skip)]
    #[serde(default)]
    pub document_count: u32,

    /// The user who owns this storage path, if any.
    pub owner: Option<crate::id::UserId>,

    /// The permissions for this storage path.
    #[dto(skip)]
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
