//! Types related to paperless IDs.

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Macro for defining ID wrapper types
macro_rules! define_ids {
    ($($def:tt),* $(,)?) => {
        $(define_ids!(@single $def);)*
    };

    (@single ($name:ident, $type:ty)) => {
        #[derive(Clone, Copy, Display, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), *self)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
    (@single ($name:ident, $type:ty, noncopy)) => {
        #[derive(Clone, Display, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        pub struct $name(pub $type);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), *self)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
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
    (StoragePathId, u32),
    (TagId, u32),
    (TaskId, String, noncopy),
    (UserId, u32),
    (WorkflowActionId, u32),
    (WorkflowId, u32),
    (WorkflowTriggerId, u32),
    (WebhookActionId, u32),
);
