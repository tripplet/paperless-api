//! Types related to document tags.

use serde::{Deserialize, Serialize};

use paperless_api_macros::{CreateDto, Item, UpdateDto};

use super::MatchAlgorithm;
use crate::id::TagId;

/// A document tag
#[derive(Debug, Default, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "tags")]
pub struct Tag {
    /// Unique identifier of the tag.
    #[dto(skip)]
    pub id: TagId,

    /// Slug of the tag.
    #[dto(skip)]
    pub slug: String,

    /// Name of the tag.
    pub name: String,

    /// Color of the tag, in hex format.
    pub color: String,

    /// Color of the text on the tag, in hex format.
    #[dto(skip)]
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
    #[dto(skip)]
    #[serde(default)]
    pub document_count: u32,

    /// Owner of the tag.
    #[dto(skip)]
    pub owner: Option<crate::id::UserId>,

    /// Parent tag of this tag.
    pub parent: Option<TagId>,

    /// Children tags of this tag.
    #[dto(skip)]
    pub children: Vec<Box<Tag>>,

    /// The permissions for this tag.
    #[dto(skip)]
    #[serde(flatten)]
    pub permissions: super::permission::ItemPermissions,
}
