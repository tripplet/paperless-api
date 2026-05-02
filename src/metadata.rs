//! Metadata associated with documents.

use derive_more::Display;
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod correspondent;
pub mod custom_field;
pub mod document_type;
pub mod permission;
pub mod storage_path;
pub mod tag;

pub use correspondent::Correspondent;
pub use custom_field::CustomField;
pub use document_type::DocumentType;
pub use permission::Permission;
pub use storage_path::StoragePath;
pub use tag::Tag;

/// A matching algorithm
#[derive(Debug, Default, Clone, Copy, Display, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum MatchAlgorithm {
    None = 0,
    AnyWord = 1,
    AllWords = 2,
    ExactMatch = 3,
    Regex = 4,
    Fuzzy = 5,

    #[default]
    Automatic = 6,
}
