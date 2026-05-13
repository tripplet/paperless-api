//! DTO traits and helpers.

use serde::Serialize;

use crate::id::PaperlessId;

/// Marker trait for DTOs used to create new items.
pub trait CreateDtoObject: Serialize {}

/// Marker trait for DTOs used to update existing items.
pub trait UpdateDtoObject: Serialize {}

/// Trait for items that can be managed via the Paperless API.
pub trait Item {
    /// The ID type for this item.
    type Id: PaperlessId;

    /// The base type used for deserialization.
    type BaseType: serde::de::DeserializeOwned;

    /// The DTO type used for creating new items.
    type CreateDto: CreateDtoObject;

    /// The DTO type used for updating existing items.
    type UpdateDto: UpdateDtoObject;

    /// Returns the API endpoint for this item.
    fn endpoint() -> &'static str;

    /// Returns the ID of this item.
    fn id(&self) -> Self::Id;
}
