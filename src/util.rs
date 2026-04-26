//! Utility types.

use derive_more::Display;
use serde::Deserialize;
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

#[derive(Debug, Clone, Deserialize)]
pub struct Statistics {
    /// Total number of documents.
    pub documents_total: u32,

    /// Number of documents in the inbox.
    pub documents_inbox: u32,

    /// Tag used for documents in the inbox.
    pub inbox_tag: u32,

    /// Tags used for documents in the inbox.
    pub inbox_tags: Vec<u32>,

    /// Counts of document file types.
    pub document_file_type_counts: Vec<DocumentFileTypeCount>,

    /// Total number of characters in all documents.
    pub character_count: u64,

    /// Total number of tags.
    pub tag_count: u32,

    /// Total number of correspondents.
    pub correspondent_count: u32,

    /// Total number of document types.
    pub document_type_count: u32,

    /// Total number of storage paths.
    pub storage_path_count: u32,

    /// Current ASN.
    pub current_asn: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentFileTypeCount {
    pub mime_type: String,

    #[serde(rename = "mime_type_count")]
    pub count: u32,
}
