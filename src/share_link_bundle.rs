//! Types related to share links.

use chrono::{DateTime, Utc};
use derive_more::Display;
use paperless_api_macros::Item;
use serde::{Deserialize, Serialize};

use crate::{
    PaperlessClient,
    dto::{CreateDto, UpdateDto},
    id::{DocumentId, ShareLinkBundleId},
    share_link::ShareLinkFileVersion,
};

/// A share link bundle.
#[derive(Debug, Clone, Deserialize, Serialize, Item)]
pub struct ShareLinkBundle {
    /// Unique identifier of the share link bundle.
    pub id: crate::id::ShareLinkBundleId,

    /// The documents in the bundle.
    pub documents: Vec<DocumentId>,

    /// File version of the share link.
    pub file_version: ShareLinkFileVersion,

    /// Slug of the bundle.
    pub slug: String,

    /// When the bundle was created.
    pub created: DateTime<Utc>,

    /// When the bundle was built.
    pub built_at: DateTime<Utc>,

    /// When the bundle expires.
    pub expiration: DateTime<Utc>,

    /// Size of the bundle in number of bytes.
    pub size_bytes: u32,

    /// Number of documents in the bundle.
    pub document_count: u32,

    /// The status of the bundle.
    pub status: ShareLinkBundleStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareLinkBundleDto {
    /// Number of days until the bundles expires.
    pub expiration_days: u16,

    /// The version of the files in the bundle.
    pub file_version: ShareLinkFileVersion,

    /// The documents in the bundle.
    pub document_ids: Vec<DocumentId>,
}

impl CreateDto for ShareLinkBundle {
    type Id = ShareLinkBundleId;
    type BaseType = ShareLinkBundle;
}

impl UpdateDto for ShareLinkBundle {
    type Id = ShareLinkBundleId;
    type BaseType = ShareLinkBundle;
}

/// The status of a share link bundle.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareLinkBundleStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

impl ShareLinkBundle {
    /// Returns the full URL of the share link.
    #[must_use]
    pub fn url(&self, client: &PaperlessClient) -> String {
        format!("{}/share/{}", client.base_url, self.slug)
    }
}
