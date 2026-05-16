//! DTO traits and helpers.

use serde::Serialize;

use crate::id::PaperlessId;

/// Marker trait for DTOs used to create new items.
pub trait CreateDto: Serialize {
    /// The ID type for this item.
    type Id: PaperlessId;

    /// The base type for the DTO.
    type BaseType: serde::de::DeserializeOwned;

    /// Returns the API endpoint for this item.
    #[must_use]
    fn endpoint() -> &'static str;
}

/// Marker trait for DTOs used to update existing items.
pub trait UpdateDto: Serialize {
    /// The ID type for this item.
    type Id: PaperlessId;

    /// The base type for the DTO.
    type BaseType: serde::de::DeserializeOwned;

    /// Returns the API endpoint for this item.
    #[must_use]
    fn endpoint() -> &'static str;
}

/// Trait for items that can be managed via the Paperless API.
pub trait Item {
    /// The ID type for this item.
    type Id: PaperlessId;

    /// The base type for the DTO.
    type BaseType: serde::de::DeserializeOwned;

    /// Returns the API endpoint for this item.
    #[must_use]
    fn endpoint() -> &'static str;

    /// Returns the ID of this item.
    #[must_use]
    fn id(&self) -> Self::Id;
}
