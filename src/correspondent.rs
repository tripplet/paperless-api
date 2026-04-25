//! Types related to correspondents.

use serde::Deserialize;

use crate::permission::ItemPermissions;
use crate::util::MatchAlgorithm;

/// A correspondent
#[derive(Debug, Clone, Deserialize)]
pub struct Correspondent {
    /// Unique identifier of the correspondent.
    pub id: crate::id::CorrespondentId,

    /// Slug of the correspondent.
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
    #[serde(default)]
    pub document_count: u32,

    /// The user who owns this correspondent, if any.
    pub owner: Option<crate::id::UserId>,

    /// The permissions for this correspondent.
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
}
