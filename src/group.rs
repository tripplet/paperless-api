//! User groups.
//!
//! Fetch with [`load_items`](crate::PaperlessClient::load_items) or
//! [`load_by_id`](crate::PaperlessClient::load_by_id), using [`Group`].
//! For cached access, refresh [`Groups`](crate::RefreshAttributes::Groups) before
//! reading [`groups`](crate::PaperlessClient::groups).

use paperless_api_macros::{CreateDto, Item, UpdateDto};
use serde::Deserialize;

/// A paperless group.
#[derive(Debug, Clone, Deserialize, CreateDto, UpdateDto, Item)]
pub struct Group {
    /// Unique identifier of the group.
    #[dto(skip)]
    pub id: crate::id::GroupId,

    /// Name of the group.
    pub name: String,
}
