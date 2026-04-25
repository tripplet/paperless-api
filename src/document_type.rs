//! Types related to document types.

use serde::{Deserialize, Serialize};

use crate::permission::ItemPermissions;
use crate::util::MatchAlgorithm;

/// A document type
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentType {
    /// Unique identifier of the document type.
    pub id: crate::id::DocumentTypeId,

    /// Slug of the document type.
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
    #[serde(default)]
    pub document_count: u32,

    /// Owner of the document type.
    pub owner: Option<crate::id::UserId>,

    /// The permissions for this tag.
    #[serde(flatten)]
    pub permissions: ItemPermissions,
}
}
