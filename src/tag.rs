//! Types related to document tags.

use serde::Deserialize;

use crate::{id::TagId, util::MatchAlgorithm};
use crate::{id::TagId, permission::ItemPermissions, util::MatchAlgorithm};

/// A document tag
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    /// Unique identifier of the tag.
    pub id: TagId,

    /// Slug of the tag.
    pub slug: String,

    /// Name of the tag.
    pub name: String,

    /// Color of the tag, in hex format.
    pub color: String,

    /// Color of the text on the tag, in hex format.
    pub text_color: String,

    /// Matching pattern for the tag.
    #[serde(rename = "match")]
    pub match_pattern: String,

    /// Matching algorithm for the tag.
    pub matching_algorithm: MatchAlgorithm,

    /// Whether the tag matching is case-insensitive.
    pub is_insensitive: bool,

    /// Whether the tag is an inbox tag.
    pub is_inbox_tag: bool,

    /// Number of documents associated with this tag.
    #[serde(default)]
    pub document_count: u32,

    /// Owner of the tag.
    pub owner: Option<crate::id::UserId>,

    /// Parent tag of this tag.
    pub parent: Option<TagId>,

    /// Children tags of this tag.
    pub children: Vec<Box<Tag>>,

    /// The permissions for this tag.
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
}
