//! Utility types.

use derive_more::Display;
use serde_repr::Deserialize_repr;

/// A matching algorithm
#[derive(Debug, Clone, Copy, Display, Deserialize_repr)]
#[repr(u8)]
pub enum MatchAlgorithm {
    None = 0,
    AnyWord = 1,
    AllWords = 2,
    ExactMatch = 3,
    Regex = 4,
    Fuzzy = 5,
    Automatic = 6,
}
