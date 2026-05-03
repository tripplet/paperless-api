//! Types related to users.

use paperless_api_macros::{CreateDto, Item, UpdateDto};
use serde::Deserialize;

/// A paperless user
#[derive(Debug, Clone, Deserialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "users")]
pub struct User {
    /// Unique identifier of the user.
    #[dto(skip)]
    pub id: crate::id::UserId,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// First name of the user.
    pub first_name: String,

    /// Last name of the user.
    pub last_name: String,

    /// Whether the user is a superuser.
    pub is_superuser: bool,

    /// Whether the user is a staff member.
    pub is_staff: bool,

    /// Whether the user is active.
    pub is_active: bool,

    /// Groups the user belongs to.
    pub groups: Vec<crate::id::GroupId>,
}
