use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct TagId(pub i32);

/// A document tag
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    /// Unique identifier of the tag.
    pub id: TagId,

    /// Name of the tag.
    pub name: String,

    /// Number of documents associated with this tag.
    pub document_count: i32,
}

impl std::fmt::Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
