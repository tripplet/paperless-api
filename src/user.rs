//! Types related to users.

use serde::Deserialize;

/// A paperless user
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: crate::id::UserId,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_superuser: bool,
    pub is_staff: bool,
    pub is_active: bool,
}
