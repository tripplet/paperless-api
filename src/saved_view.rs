//! Types related to saved views in the paperless UI.

use derive_more::Display;
use paperless_api_macros::{CreateDto, Item, UpdateDto};
use serde::{Deserialize, Serialize};

/// A saved view in the paperless UI.
#[derive(Debug, Default, Clone, Deserialize, Serialize, CreateDto, UpdateDto, Item)]
#[api_info(endpoint = "saved_views")]
pub struct SavedView {
    /// The ID of the saved view.
    #[dto(skip)]
    pub id: crate::id::SavedViewId,

    /// The name of the saved view.
    pub name: String,

    /// Whether the saved view should be shown on the dashboard.
    pub show_on_dashboard: bool,

    /// Whether the saved view should be shown in the sidebar.
    pub show_in_sidebar: bool,

    /// The field to sort the view by.
    pub sort_field: Option<String>,

    /// Whether to sort the view in reverse order.
    pub sort_reverse: Option<bool>,

    /// The filter rules determining which documents are shown in the view.
    pub filter_rules: Option<Vec<FilterRule>>,

    /// The display mode of the view.
    pub display_mode: Option<DisplayMode>,

    /// The fields to display in the view.
    pub display_fields: Option<Vec<String>>,

    /// The number of documents to show per page.
    pub page_size: Option<u32>,

    /// The user who owns the saved view.
    #[dto(skip)]
    pub owner: Option<crate::id::UserId>,

    /// Whether the user can change the saved view.
    #[dto(skip)]
    pub user_can_change: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DisplayMode {
    Table,
    SmallCards,
    LargeCards,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterRule {
    #[serde(rename = "rule_type")]
    pub rule: FilterRuleType,

    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, Display)]
#[repr(u8)]
pub enum FilterRuleType {
    TitleContains = 0,
    ContentContains = 1,
    AsnIs = 2,
    CorrespondentIs = 3,
    DocumentTypeIs = 4,
    IsInInbox = 5,
    HasTag = 6,
    HasAnyTag = 7,
    CreatedBefore = 8,
    CreatedAfter = 9,
    CreatedInYear = 10,
    CreatedInMonth = 11,
    CreatedDayIs = 12,
    AddedBefore = 13,
    AddedAfter = 14,
    ModifiedBefore = 15,
    ModifiedAfter = 16,
    DoesNotHaveTag = 17,
    DocumentHasNoAsn = 18,
    TitleOrContentContains = 19,
    FullTextSearch = 20,
    SimilarDocuments = 21,
    HasTagsIn = 22,
    AsnGreaterThan = 23,
    AsnLessThan = 24,
    StoragePathIs = 25,
    HasCorrespondentIn = 26,
    HasNoCorrespondentIn = 27,
    HasDocumentTypeIn = 28,
    HasNoDocumentTypeIn = 29,
    HasStoragePathIn = 30,
    HasNoStoragePathIn = 31,
    OwnerIs = 32,
    HasOwnerIn = 33,
    HasNoOwner = 34,
    HasNoOwnerIn = 35,
    HasCustomFieldValue = 36,
    IsSharedByMe = 37,
    HasCustomFields = 38,
    HasTheCustomFields = 39,
    DoesNotHaveCustomFields = 40,
    DoesNotHaveCustomField = 41,
    CustomFieldQuery = 42,
    CreateDto = 43,
    CreatedBy = 44,
    AddedTo = 45,
    AddedBy = 46,
    MimeTypeIs = 47,

    Unknown(u8),
}

impl Serialize for FilterRuleType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = if let FilterRuleType::Unknown(unknown) = self {
            *unknown
        } else {
            // SAFETY: FilterRuleType must be a valid 0..=47
            unsafe { *(std::ptr::from_ref::<FilterRuleType>(self)).cast::<u8>() }
        };

        serializer.serialize_u8(value)
    }
}

impl<'de> serde::Deserialize<'de> for FilterRuleType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;

        let enum_value = match value {
            0..=47 => unsafe { std::mem::transmute::<u16, FilterRuleType>(u16::from(value)) },
            _ => FilterRuleType::Unknown(value),
        };

        Ok(enum_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_rule_type_boundary_values() {
        let zero: FilterRuleType = serde_json::from_str("0").unwrap();
        assert!(matches!(zero, FilterRuleType::TitleContains));

        let forty_seven: FilterRuleType = serde_json::from_str("47").unwrap();
        assert!(matches!(forty_seven, FilterRuleType::MimeTypeIs));

        let forty_seven: FilterRuleType = serde_json::from_str("48").unwrap();
        assert!(matches!(forty_seven, FilterRuleType::Unknown(48)));
    }

    #[test]
    fn filter_rule_type_unknown_value_roundtrip() {
        let unknown: FilterRuleType = serde_json::from_str("200").unwrap();
        assert!(matches!(unknown, FilterRuleType::Unknown(200)));

        let serialized = serde_json::to_string(&unknown).unwrap();
        assert_eq!(serialized, "200");
    }

    #[test]
    fn filter_rule_roundtrip_with_value() {
        let rule = FilterRule {
            rule: FilterRuleType::FullTextSearch,
            value: Some("created:[-3 month to now]".to_string()),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: FilterRule = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.rule, FilterRuleType::FullTextSearch));
        assert_eq!(
            deserialized.value,
            Some("created:[-3 month to now]".to_string())
        );
    }
}
