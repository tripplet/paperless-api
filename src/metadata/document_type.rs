//! Types related to document types.

use serde::{Deserialize, Serialize};

use paperless_api_macros::{CreateDto, Item, UpdateDto};

use super::MatchAlgorithm;
use super::permission::ItemPermissions;

/// A document type
#[derive(Debug, Default, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "document_types")]
pub struct DocumentType {
    /// Unique identifier of the document type.
    #[dto(skip)]
    pub id: crate::id::DocumentTypeId,

    /// Slug of the document type.
    #[dto(skip)]
    pub slug: String,

    /// Name of the document type.
    pub name: String,

    /// Matching pattern for the document type.
    #[serde(rename = "match")]
    pub match_pattern: String,

    /// Matching algorithm for the document type.
    pub matching_algorithm: MatchAlgorithm,

    /// Whether the document type matching is case-insensitive.
    pub is_insensitive: Option<bool>,

    /// Number of documents with this type.
    #[dto(skip)]
    #[serde(default)]
    pub document_count: u32,

    /// Owner of the document type.
    pub owner: Option<crate::id::UserId>,

    /// The permissions for this tag.
    #[dto(skip)]
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
