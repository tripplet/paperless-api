use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::document::DocumentId;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct ShareLinkId(pub u32);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareLinkFileVersion {
    Archive,
    Original,
}

/// A share link
#[derive(Debug, Clone, Deserialize)]
pub struct ShareLink {
    /// Unique identifier of the share link.
    pub id: ShareLinkId,

    /// Document of the share link.
    pub document: DocumentId,

    /// File version of the share link.
    pub file_version: ShareLinkFileVersion,

    /// Slug of the share link.
    pub slug: String,

    #[serde(skip)]
    pub(crate) base_url: String,
}

impl ShareLink {
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}/share/{}", self.base_url, self.slug)
    }
}

impl std::fmt::Display for ShareLinkFileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareLinkFileVersion::Archive => write!(f, "archive"),
            ShareLinkFileVersion::Original => write!(f, "original"),
        }
    }
}
