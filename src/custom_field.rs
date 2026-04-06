use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct CustomFieldId(pub u32);

/// Custom field definition.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomField {
    /// Unique identifier of the custom field.
    pub id: CustomFieldId,

    /// Name of the custom field.
    pub name: String,

    /// Data type of the custom field.
    pub data_type: String,
}

/// Custom field value of an existing document
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentCustomField {
    /// Unique identifier of the custom field.
    pub field: CustomFieldId,

    /// Value of the custom field.
    pub value: String,
}
