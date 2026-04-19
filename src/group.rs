//! Types related to groups.

use serde::Deserialize;

/// A paperless group
#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    /// Unique identifier of the group.
    pub id: crate::id::GroupId,

    /// The name of the group.
    pub name: String,
}
