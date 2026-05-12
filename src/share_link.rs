//! Types related to share links.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use paperless_api_macros::CreateDto;

/// The version of the file provided by a share link.
#[derive(Debug, Display, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareLinkFileVersion {
    /// The "current" file version in the paperless archive.
    /// This version might be modified by paperless.
    #[default]
    Archive,

    /// The original file version initially uploaded to paperless.
    Original,
}

/// A share link.
#[derive(Debug, Clone, Deserialize, CreateDto)]
#[api_info(private, endpoint = "share_links")]
pub struct ShareLink {
    /// Unique identifier of the share link.
    #[dto(skip)]
    pub id: crate::id::ShareLinkId,

    /// Document of the share link.
    pub document: crate::id::DocumentId,

    /// When the share link was created.
    #[dto(skip)]
    pub created: DateTime<Utc>,

    /// When the share link expires.
    pub expiration: DateTime<Utc>,

    /// File version of the share link.
    pub file_version: ShareLinkFileVersion,

    /// Slug of the share link.
    #[dto(skip)]
    pub slug: String,

    /// Base URL of the Paperless instance, used to generate the share link URL.
    #[serde(skip)]
    #[dto(skip)]
    pub(crate) base_url: Arc<str>,
}

impl ShareLink {
    /// Returns the full URL of the share link.
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}/share/{}", self.base_url, self.slug)
    }
}
