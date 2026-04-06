use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct UserId(pub u32);

/// A paperless user
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_superuser: bool,
    pub is_staff: bool,
    pub is_active: bool,
}
