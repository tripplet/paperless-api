//! Paperless ID types.
//!
//! Paperless uses numeric IDs for most entities.
//! To avoid confusion, ID types are defined as wrappers around the underlying numeric type.
//! e.g. `DocumentId(u32)`.

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Marker trait for Paperless ID types.
pub trait PaperlessId:
    std::fmt::Display + PartialEq + Eq + std::hash::Hash + serde::de::DeserializeOwned + Serialize
{
}

/// Trait for all Paperless IDs which can be treated as items.
pub trait ItemId: PaperlessId {
    #[must_use]
    fn endpoint() -> &'static str;
}

/// Macro for defining ID wrapper types.
macro_rules! define_ids {
    ($($def:tt),* $(,)?) => {
        $(define_ids!(@parse $def);)*
    };

    (@parse ($name:ident, $type:ty)) => {
        define_ids!(@emit copy $name, $type);
    };
    (@parse ($name:ident, $type:ty, noncopy)) => {
        define_ids!(@emit noncopy $name, $type);
    };
    (@parse ($name:ident, $type:ty, $endpoint:literal)) => {
        define_ids!(@emit copy $name, $type, $endpoint);
    };
    (@parse ($name:ident, $type:ty, noncopy, $endpoint:literal)) => {
        define_ids!(@emit noncopy $name, $type, $endpoint);
    };

    (@emit copy $name:ident, $type:ty $(, $endpoint:literal)?) => {
        #[derive(Clone, Copy, Display, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        /// ID type for a Paperless entity.
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), *self)
            }
        }

        impl From<$name> for $type {
            #[inline]
            fn from(item: $name) -> Self {
                item.0
            }
        }

        impl PaperlessId for $name {}
        define_ids!(@maybe_item_id $name $(, $endpoint)?);
    };
    (@emit noncopy $name:ident, $type:ty $(, $endpoint:literal)?) => {
        #[derive(Clone, Display, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        /// ID type for a Paperless entity (non-copy).
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), *self)
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl PaperlessId for $name {}
        define_ids!(@maybe_item_id $name $(, $endpoint)?);
    };
    (@maybe_item_id $name:ident) => {};
    (@maybe_item_id $name:ident, $endpoint:literal) => {
        impl ItemId for $name {
            #[inline]
            fn endpoint() -> &'static str {
                $endpoint
            }
        }
    };
}

define_ids!(
    (CorrespondentId, u32, "correspondents"),
    (CustomFieldId, u32, "custom_fields"),
    (DocumentId, u32, "documents"),
    (DocumentTypeId, u32, "document_types"),
    (GroupId, u32, "groups"),
    (NoteId, u32),
    (SavedViewId, u32, "saved_views"),
    (SelectableOptionId, String, noncopy),
    (ShareLinkId, u32, "share_links"),
    (StoragePathId, u32, "storage_paths"),
    (TagId, u32, "tags"),
    (TaskId, String, noncopy),
    (UserId, u32, "users"),
    (WorkflowActionId, u32),
    (WorkflowId, u32, "workflows"),
    (WorkflowTriggerId, u32),
    (WebhookActionId, u32),
);
