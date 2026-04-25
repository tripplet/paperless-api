//! Utility types.

use derive_more::Display;
use serde_repr::{Deserialize_repr, Serialize_repr};

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
