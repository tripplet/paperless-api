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

/// Macro for defining ID wrapper types.
macro_rules! define_ids {
    ($($def:tt),* $(,)?) => {
        $(define_ids!(@single $def);)*
    };

    (@single ($name:ident, $type:ty)) => {
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
    };
    (@single ($name:ident, $type:ty, noncopy)) => {
        #[derive(Clone, Display, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        /// ID type for a Paperless entity (non-copy).
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), *self)
            }
        }

        impl AsRef<str> for $name
        {
            #[inline]
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl PaperlessId for $name {}
    };
}

define_ids!(
    (CorrespondentId, u32),
    (CustomFieldId, u32),
    (DocumentId, u32),
    (DocumentTypeId, u32),
    (GroupId, u32),
    (NoteId, u32),
    (SavedViewId, u32),
    (SelectableOptionId, String, noncopy),
    (ShareLinkId, u32),
    (ShareLinkBundleId, u32),
    (StoragePathId, u32),
    (TagId, u32),
    (TaskId, String, noncopy),
    (UserId, u32),
    (WorkflowActionId, u32),
    (WorkflowId, u32),
    (WorkflowTriggerId, u32),
    (WebhookActionId, u32),
);
