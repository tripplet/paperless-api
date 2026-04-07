use serde::Deserialize;

use crate::id::TagId;

/// A document tag
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    /// Unique identifier of the tag.
    pub id: TagId,

    /// Name of the tag.
    pub name: String,

    /// Number of documents associated with this tag.
    pub document_count: u32,
}
