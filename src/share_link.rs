use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareLinkFileVersion {
    Archive,
    Original,
}

/// A share link
#[derive(Debug, Clone, Deserialize)]
pub struct ShareLink {
    /// Unique identifier of the share link.
    pub id: crate::id::ShareLinkId,

    /// Document of the share link.
    pub document: crate::id::DocumentId,

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
