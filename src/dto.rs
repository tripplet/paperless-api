//! DTO traits and helpers.

use serde::Serialize;

use crate::id::PaperlessId;

pub trait CreateDtoObject: Serialize {}

pub trait Item {
    type Id: PaperlessId;
    type BaseType: serde::de::DeserializeOwned;
    type CreateDto: CreateDtoObject;
    type UpdateDto: UpdateDtoObject;

    fn endpoint() -> &'static str;

    fn id(&self) -> Self::Id;
}

/// Marker trait for update DTOs.
pub trait UpdateDtoObject: Serialize {}

// #[cfg(test)]
// mod tests {
//     use paperless_api_macros::{CreateDto as CreateDtoDerive, UpdateDto as UpdateDtoDerive};
//     use serde::{Deserialize, Serialize};

//     use super::{CreateDto, UpdateDto};

//     #[derive(Debug, Default, Clone, Deserialize, Serialize, CreateDtoDerive, UpdateDtoDerive)]
//     struct TestItem {
//         #[dto(skip)]
//         pub id: u32,
//         pub name: String,
//         #[serde(rename = "match")]
//         pub match_pattern: String,
//         pub optional: Option<bool>,
//         #[dto(skip)]
//         #[serde(default)]
//         pub document_count: u32,
//     }

//     #[test]
//     fn create_dto_has_correct_fields() {
//         let dto = CreateTestItem {
//             name: "hello".to_string(),
//             match_pattern: "pattern".to_string(),
//             optional: None,
//         };
//         let json = serde_json::to_string(&dto).unwrap();
//         assert!(json.contains("\"name\""));
//         assert!(json.contains("\"match\""));
//         assert!(!json.contains("\"id\""));
//         assert!(!json.contains("\"document_count\""));
//     }

//     #[test]
//     fn update_dto_wraps_fields_in_option() {
//         let dto = UpdateTestItem {
//             name: Some("hello".to_string()),
//             match_pattern: None,
//             optional: Some(Some(true)),
//         };
//         let json = serde_json::to_string(&dto).unwrap();
//         assert!(json.contains("\"name\":\"hello\""));
//         assert!(!json.contains("\"match\""));
//         assert!(json.contains("\"optional\":true"));
//         assert!(!json.contains("\"id\""));
//     }

//     #[test]
//     fn create_dto_implements_marker_trait() {
//         fn assert_create<T: CreateDto>() {}
//         assert_create::<CreateTestItem>();
//     }

//     #[test]
//     fn update_dto_implements_marker_trait() {
//         fn assert_update<T: UpdateDto>() {}
//         assert_update::<UpdateTestItem>();
//     }

//     #[test]
//     fn create_dto_default_works() {
//         let dto = CreateTestItem::default();
//         assert_eq!(dto.name, "");
//         assert_eq!(dto.match_pattern, "");
//         assert_eq!(dto.optional, None);
//     }

//     #[test]
//     fn update_dto_default_is_all_none() {
//         let dto = UpdateTestItem::default();
//         assert_eq!(dto.name, None);
//         assert_eq!(dto.match_pattern, None);
//         assert_eq!(dto.optional, None);
//     }
// }
