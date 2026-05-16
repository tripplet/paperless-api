//! Types related to custom fields.

use serde::{Deserialize, Serialize};

use paperless_api_macros::{CreateDto, Item, UpdateDto};

use crate::id::{CustomFieldId, SelectableOptionId};

/// Custom field definition.
#[derive(Debug, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]

pub struct CustomField {
    /// Unique identifier of the custom field.
    #[dto(skip)]
    pub id: CustomFieldId,

    /// Name of the custom field.
    pub name: String,

    /// Data type of the custom field.
    pub data_type: CustomFieldDataType,

    /// Extra data for the custom field, such as currency or select options.
    pub extra_data: Option<CustomFieldExtraData>,

    /// Number of documents that have this custom field set.
    #[dto(skip)]
    #[serde(default)]
    pub document_count: u32,
}

/// Custom field value of an existing document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentCustomField {
    /// Unique identifier of the custom field.
    pub field: CustomFieldId,

    /// Value of the custom field.
    pub value: String,
}

/// Extra data for a custom field.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomFieldExtraData {
    /// Default currency for monetary fields.
    pub default_currency: Option<String>,

    /// Selectable options for select fields.
    pub select_options: Option<Vec<SelectableOption>>,
}

/// A selectable option for a custom field.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectableOption {
    /// Unique identifier of the option.
    pub id: SelectableOptionId,

    /// Label of the option.
    pub label: String,
}

/// Data type of a custom field.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldDataType {
    #[default]
    String,
    Url,
    Date,
    Boolean,
    Integer,
    Float,
    Monetary,
    Documentlink,
    Select,
    Longtext,
}
