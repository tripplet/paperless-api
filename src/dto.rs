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
