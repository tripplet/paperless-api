//! Types for working with Paperless documents.
//!
//! Document mutations are applied locally first.
//! Methods such as [`set_title`](Document::set_title),
//! [`set_content`](Document::set_content),
//! [`add_tag`](Document::add_tag), etc..
//! only update the in-memory [`Document`] value and mark it as changed.
//! The changes are only sent to the Paperless server when
//! [`patch`](Document::patch) is called.

use std::{fmt::Display, io, path::Path, sync::Arc, time::Duration};

use chrono::{DateTime, NaiveDate, Utc};
use enumflags2::{BitFlags, bitflags};
use futures_util::TryStreamExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;

use crate::{
    DocumentCustomField, Error, Result,
    client::PaperlessClient,
    correspondent::CorrespondentId,
    custom_field::CustomFieldId,
    document_type::DocumentTypeId,
    share_link::{ShareLink, ShareLinkFileVersion},
    tag::TagId,
    user::UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct DocumentId(pub i32);

/// Represents a document.
///
/// Changes made through mutating methods such as
/// [`set_title`](Document::set_title),
/// [`set_content`](Document::set_content),
/// [`add_tag`](Document::add_tag), and
/// [`set_custom_field`](Document::set_custom_field)
/// are only tracked locally at first.
///
/// They are not sent to the Paperless server until
/// [`patch`](Document::patch) is called.
#[derive(Debug, Clone)]
pub struct Document {
    data: DocumentData,
    client: Arc<PaperlessClient>,
    content_is_truncated: bool,
    changed_values: BitFlags<ChangedAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DocumentData {
    id: DocumentId,
    archive_serial_number: Option<ArchiveSerialNumber>,
    original_file_name: String,
    added: DateTime<Utc>,
    created: Option<NaiveDate>,
    modified: DateTime<Utc>,
    page_count: u32,
    title: String,
    content: String,
    tags: Vec<TagId>,
    owner: Option<UserId>,
    correspondent: Option<CorrespondentId>,
    custom_fields: Vec<DocumentCustomField>,
    document_type: Option<DocumentTypeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct ArchiveSerialNumber(pub u32);

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum ChangedAttributes {
    Title,
    Content,
    Tags,
    CustomFields,
    Correspondent,
    DocumentType,
    Created,
    Owner,

    Deleted,
}

/// The content (OCR) of a document, either full or truncated.
#[derive(Debug, Clone)]
pub enum Content<'a> {
    /// Full content of the document.
    Full(&'a str),

    /// Truncated content of the document.
    Truncated(&'a str),
}

#[derive(Debug, Serialize)]
struct PatchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<TagId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    custom_fields: Option<Vec<DocumentCustomField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    correspondent: Option<CorrespondentId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    document_type: Option<DocumentTypeId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<NaiveDate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<UserId>,
}

#[derive(Debug, Serialize)]
struct ShareLinkRequest {
    document: DocumentId,
    file_version: ShareLinkFileVersion,
    expiration: DateTime<Utc>,
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ArchiveSerialNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Document {
    pub(crate) fn new(
        data: DocumentData,
        client: Arc<PaperlessClient>,
        content_is_truncated: bool,
    ) -> Self {
        Self {
            data,
            client,
            content_is_truncated,
            changed_values: BitFlags::default(),
        }
    }

    /// Get the id of the document
    #[inline]
    #[must_use]
    pub fn id(&self) -> DocumentId {
        self.data.id
    }

    /// Get the archive serial number of the document.
    #[inline]
    #[must_use]
    pub fn archive_serial_number(&self) -> Option<ArchiveSerialNumber> {
        self.data.archive_serial_number
    }

    /// Get the timestamp when the document was added.
    #[inline]
    #[must_use]
    pub fn added(&self) -> &DateTime<Utc> {
        &self.data.added
    }

    /// Get the created timestamp of the document.
    #[inline]
    #[must_use]
    pub fn created(&self) -> Option<&NaiveDate> {
        self.data.created.as_ref()
    }

    /// Get the modified timestamp of the document.
    #[inline]
    #[must_use]
    pub fn modified(&self) -> &DateTime<Utc> {
        &self.data.modified
    }

    /// Get the title of the document.
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.data.title
    }

    /// Get the original file name of the document.
    #[inline]
    #[must_use]
    pub fn original_file_name(&self) -> &str {
        &self.data.original_file_name
    }

    /// Get the correspondent id of the document.
    #[inline]
    #[must_use]
    pub fn correspondent(&self) -> Option<CorrespondentId> {
        self.data.correspondent
    }

    /// Get the owner id of the document.
    #[inline]
    #[must_use]
    pub fn owner(&self) -> Option<UserId> {
        self.data.owner
    }

    /// Get the document type id of the document.
    #[inline]
    #[must_use]
    pub fn document_type(&self) -> Option<DocumentTypeId> {
        self.data.document_type
    }

    /// Get the number of pages in the document.
    #[inline]
    #[must_use]
    pub fn page_count(&self) -> u32 {
        self.data.page_count
    }

    /// Get all tag-ids for the document.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[TagId] {
        &self.data.tags
    }

    /// Get all custom fields for the document.
    #[inline]
    #[must_use]
    pub fn custom_fields(&self) -> &[DocumentCustomField] {
        &self.data.custom_fields
    }

    /// Get the content of the document.
    #[inline]
    #[must_use]
    pub fn content(&self) -> Content<'_> {
        if self.content_is_truncated {
            Content::Truncated(&self.data.content)
        } else {
            Content::Full(&self.data.content)
        }
    }

    /// Add a tag to the document.
    pub fn add_tag(&mut self, tag_id: TagId) {
        if !self.data.tags.contains(&tag_id) {
            self.data.tags.push(tag_id);
            self.changed_values |= ChangedAttributes::Tags;
        }
    }

    pub fn remove_tag(&mut self, tag_id: TagId) {
        if let Some(index) = self.data.tags.iter().position(|id| *id == tag_id) {
            self.data.tags.remove(index);
            self.changed_values |= ChangedAttributes::Tags;
        }
    }

    /// Set the title of the document.
    pub fn set_title(&mut self, title: &str) {
        self.data.title = title.to_string();
        self.changed_values |= ChangedAttributes::Title;
    }

    /// Set the content of the document.
    pub fn set_content(&mut self, content: &str) {
        self.data.content = content.to_string();
        self.content_is_truncated = false;
        self.changed_values |= ChangedAttributes::Content;
    }

    /// Set a custom field for the document.
    pub fn set_custom_field(&mut self, field: CustomFieldId, value: &str) {
        for custom_field in &mut self.data.custom_fields {
            if custom_field.field == field {
                custom_field.value = value.to_string();
                self.changed_values |= ChangedAttributes::CustomFields;
                return;
            }
        }

        self.data.custom_fields.push(DocumentCustomField {
            field,
            value: value.to_string(),
        });
        self.changed_values |= ChangedAttributes::CustomFields;
    }

    /// Remove a custom field from the document.
    pub fn remove_custom_field(&mut self, field: CustomFieldId) {
        if let Some(index) = self
            .data
            .custom_fields
            .iter()
            .position(|custom_field| custom_field.field == field)
        {
            self.data.custom_fields.remove(index);
            self.changed_values |= ChangedAttributes::CustomFields;
        }
    }

    /// Set the created date of the document.
    pub fn set_created(&mut self, created: NaiveDate) {
        self.data.created = Some(created);
        self.changed_values |= ChangedAttributes::Created;
    }

    /// Set the owner of the document.
    pub fn set_owner(&mut self, owner: UserId) {
        self.data.owner = Some(owner);
        self.changed_values |= ChangedAttributes::Owner;
    }

    /// Set the correspondent of the document.
    pub fn set_correspondent(&mut self, correspondent: CorrespondentId) {
        self.data.correspondent = Some(correspondent);
        self.changed_values |= ChangedAttributes::Correspondent;
    }

    /// Set the document type of the document.
    pub fn set_document_type(&mut self, document_type: DocumentTypeId) {
        self.data.document_type = Some(document_type);
        self.changed_values |= ChangedAttributes::DocumentType;
    }

    /// Returns `true` if the document has unsaved changes.
    #[inline]
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.changed_values.is_empty() && !self.changed_values.contains(ChangedAttributes::Deleted)
    }

    /// Returns `true` if the document was deleted.
    #[inline]
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.changed_values.contains(ChangedAttributes::Deleted)
    }

    fn fail_if_deleted(&self) -> Result<()> {
        if self.is_deleted() {
            Err(Error::AlreadyDeleted)
        } else {
            Ok(())
        }
    }

    /// Refresh the document from the server.
    ///
    /// This will discard any local changes and replace them with the server's state.
    pub async fn reload(&mut self) -> Result<()> {
        let document_data = self
            .client
            .as_ref()
            .get_document_data_by_id(self.data.id)
            .await?;

        self.data = document_data;

        self.changed_values = BitFlags::empty();
        self.content_is_truncated = false;
        Ok(())
    }

    /// Update the document on the server.
    ///
    /// This applies the currently tracked local changes to the remote Paperless document.
    pub async fn patch(&mut self) -> Result<()> {
        if !self.is_dirty() {
            return Ok(());
        }

        self.fail_if_deleted()?;

        let patch = PatchRequest {
            title: self
                .changed_values
                .contains(ChangedAttributes::Title)
                .then_some(self.data.title.clone()),

            content: self
                .changed_values
                .contains(ChangedAttributes::Content)
                .then_some(self.data.content.clone()),

            tags: self
                .changed_values
                .contains(ChangedAttributes::Tags)
                .then_some(self.data.tags.clone()),

            custom_fields: self
                .changed_values
                .contains(ChangedAttributes::CustomFields)
                .then_some(
                    self.data
                        .custom_fields
                        .iter()
                        .map(|field| DocumentCustomField {
                            field: field.field,
                            value: field.value.clone(),
                        })
                        .collect(),
                ),
            correspondent: self
                .changed_values
                .contains(ChangedAttributes::Correspondent)
                .then_some(self.data.correspondent)
                .flatten(),

            document_type: self
                .changed_values
                .contains(ChangedAttributes::DocumentType)
                .then_some(self.data.document_type)
                .flatten(),

            created: self
                .changed_values
                .contains(ChangedAttributes::Created)
                .then_some(self.data.created)
                .flatten(),

            owner: self
                .changed_values
                .contains(ChangedAttributes::Owner)
                .then_some(self.data.owner)
                .flatten(),
        };

        self.client
            .request(
                Method::PATCH,
                &format!("/api/documents/{}/", self.data.id),
                Some(&serde_json::to_value(patch).expect("Patch request")),
            )
            .await?;

        self.changed_values = BitFlags::empty();
        Ok(())
    }

    pub async fn delete(&mut self) -> Result<()> {
        self.client
            .request(
                Method::DELETE,
                &format!("/api/documents/{}/", self.data.id),
                None,
            )
            .await?;

        self.changed_values = BitFlags::from(ChangedAttributes::Deleted);
        Ok(())
    }

    /// Get the full content of the document, replacing any truncated content.
    pub async fn get_full_content(&mut self) -> Result<()> {
        self.fail_if_deleted()?;

        if !self.content_is_truncated {
            return Ok(());
        }

        let doc = self.client.get_document_data_by_id(self.data.id).await?;
        self.data.content = doc.content;
        self.content_is_truncated = false;
        Ok(())
    }

    /// Download the document to a file.
    pub async fn download_to_file(&self, path: &Path) -> Result<()> {
        self.fail_if_deleted()?;

        let resp = self
            .client
            .request(
                Method::GET,
                &format!("/api/documents/{}/download/", self.data.id),
                None,
            )
            .await?;

        if !resp.status().is_success() {
            return Err(Error::Other(format!(
                "Failed to download document: {}",
                resp.status()
            )));
        }

        let mut stream = StreamReader::new(
            resp.bytes_stream()
                .map_err(|e| io::Error::other(format!("Failed to read response body: {e}"))),
        );

        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|e| Error::Other(format!("Failed to create file: {e}")))?;

        tokio::io::copy(&mut stream, &mut file)
            .await
            .map_err(|e| Error::Other(format!("Failed to write file: {e}")))?;

        Ok(())
    }

    /// Download the document to a buffer.
    pub async fn download_to_buffer(&self) -> Result<Vec<u8>> {
        self.fail_if_deleted()?;

        let resp = self
            .client
            .request(
                Method::GET,
                &format!("/api/documents/{}/download/", self.data.id),
                None,
            )
            .await?;

        if resp.status().is_success() {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| Error::Other(format!("Failed to read response body: {e}")))?;
            Ok(bytes.to_vec())
        } else {
            Err(Error::Other(format!(
                "Failed to download document: {}",
                resp.status()
            )))
        }
    }

    /// Generates a share link for the document that expires after the specified duration.
    pub async fn generate_share_link_duration(
        &self,
        valid_for: Duration,
        version: ShareLinkFileVersion,
    ) -> Result<ShareLink> {
        let expires = Utc::now() + valid_for;
        self.generate_share_link_expires(expires, version).await
    }

    /// Generates a share link for the document that expires at the specified time.
    pub async fn generate_share_link_expires(
        &self,
        expires: DateTime<Utc>,
        version: ShareLinkFileVersion,
    ) -> Result<ShareLink> {
        self.fail_if_deleted()?;

        let resp = self
            .client
            .request(
                Method::POST,
                "/api/share_links/",
                Some(
                    &serde_json::to_value(ShareLinkRequest {
                        document: self.data.id,
                        file_version: version,
                        expiration: expires,
                    })
                    .expect("Share link request"),
                ),
            )
            .await?;

        let mut share_link: ShareLink = resp
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to generate share link: {e}")))?;

        share_link.base_url.clone_from(&self.client.base_url);
        Ok(share_link)
    }
}

impl Display for Content<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Full(text) => write!(f, "{text}"),
            Content::Truncated(text) => write!(f, "{text}..."),
        }
    }
}

impl AsRef<str> for Content<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Content::Full(text) | Content::Truncated(text) => text,
        }
    }
}
