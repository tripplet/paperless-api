//! Types related to groups.

use paperless_api_macros::{CreateDto, Item, UpdateDto};
use serde::Deserialize;

/// A paperless group
#[derive(Debug, Clone, Deserialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "groups")]
pub struct Group {
    /// Unique identifier of the group.
    #[dto(skip)]
    pub id: crate::id::GroupId,

    /// Name of the group.
    pub name: String,
}
