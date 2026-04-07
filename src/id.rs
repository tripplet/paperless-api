use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Macro for defining ID wrapper types
macro_rules! define_ids {
    ($($def:tt),* $(,)?) => {
        $(define_ids!(@single $def);)*
    };

    (@single ($name:ident, $type:ty)) => {
        #[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        pub struct $name(pub $type);
    };
    (@single ($name:ident, $type:ty, noncopy)) => {
        #[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[repr(transparent)]
        pub struct $name(pub $type);
    };
}

define_ids!(
    (CorrespondentId, u32),
    (CustomFieldId, u32),
    (DocumentId, u32),
    (DocumentTypeId, u32),
    (ShareLinkId, u32),
    (TagId, u32),
    (TaskId, String, noncopy),
    (UserId, u32),
    (WorkflowActionId, u32),
    (WorkflowId, u32),
    (WorkflowTriggerId, u32),
    (WebhookActionId, u32),
);
