use serde::{Deserialize, Serialize};

use crate::id::{GroupId, UserId};

/// The permissions for a paperless item.
///
/// If full permissions are not explicitly requested, only the `Simple` variant is available.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ItemPermissions {
    /// Full permissions
    Full { permissions: FullPermissions },

    /// Simple permissions
    Simple { user_can_change: bool },
}

/// The full permissions for a paperless item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullPermissions {
    /// The users and groups that can view the item.
    pub view: Permission,

    /// The users and groups that can change the item.
    pub change: Permission,
}

/// A detailed permission for a paperless item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Permission {
    pub users: Vec<UserId>,
    pub groups: Vec<GroupId>,
}

impl Default for ItemPermissions {
    fn default() -> Self {
        Self::Full {
            permissions: FullPermissions {
                view: Permission {
                    users: Vec::new(),
                    groups: Vec::new(),
                },
                change: Permission {
                    users: Vec::new(),
                    groups: Vec::new(),
                },
            },
        }
    }
}
