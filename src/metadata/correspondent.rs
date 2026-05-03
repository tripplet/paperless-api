//! Types related to correspondents.

use serde::{Deserialize, Serialize};

use paperless_api_macros::{CreateDto, Item, UpdateDto};

use super::MatchAlgorithm;
use super::permission::ItemPermissions;

/// A correspondent
#[derive(Debug, Default, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "correspondents")]
pub struct Correspondent {
    /// Unique identifier of the correspondent.
    #[dto(skip)]
    pub id: crate::id::CorrespondentId,

    /// Slug of the correspondent.
    #[dto(skip)]
    pub slug: String,

    /// Name of the correspondent.
    pub name: String,

    /// Matching pattern for the tag.
    #[serde(rename = "match")]
    pub match_pattern: String,

    /// Matching algorithm for the tag.
    pub matching_algorithm: MatchAlgorithm,

    /// Whether the tag matching is case-insensitive.
    pub is_insensitive: bool,

    /// The number of documents associated with this correspondent.
    #[dto(skip)]
    #[serde(default)]
    pub document_count: u32,

    /// The user who owns this correspondent, if any.
    #[dto(skip)]
    pub owner: Option<crate::id::UserId>,

    /// The permissions for this correspondent.
    #[dto(skip)]
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
