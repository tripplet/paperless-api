use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct TagId(pub u32);

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
