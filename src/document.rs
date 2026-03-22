use std::{fmt::Display, io, path::Path, sync::Arc};

use enumflags2::{BitFlags, bitflags};
use futures_util::TryStreamExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;

use crate::{
    DocumentCustomField, Error, Result, client::PaperlessClient, correspondent::CorrespondentId,
    custom_field::CustomFieldId, document_type::DocumentTypeId, tag::TagId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct DocumentId(pub i32);

/// Represents a document
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document {
    /// Unique identifier of the document.
    pub id: DocumentId,
    title: String,
    content: String,
    tags: Vec<TagId>,
    owner: i32,
    correspondent: Option<CorrespondentId>,
    document_type: Option<DocumentTypeId>,

    /// Original file name of the document.
    pub original_file_name: String,

    /// Number of pages in the document.
    pub page_count: u32,

    custom_fields: Vec<DocumentCustomField>,

    #[serde(skip)]
    pub(crate) client: Option<Arc<PaperlessClient>>,

    #[serde(skip)]
    pub(crate) content_is_truncated: bool,

    #[serde(skip)]
    changed_values: BitFlags<ChangedAttributes>,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum ChangedAttributes {
    Title,
    Content,
    Tags,
    CustomFields,
    Correspondent,
    DocumentType,
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
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Document {
    /// Add a tag to the document.
    pub fn add_tag(&mut self, tag_id: TagId) {
        if !self.tags.contains(&tag_id) {
            self.tags.push(tag_id);
            self.changed_values |= ChangedAttributes::Tags;
        }
    }

    /// Get all tag-ids for the document.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[TagId] {
        &self.tags
    }

    /// Set the title of the document.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.changed_values |= ChangedAttributes::Title;
    }

    /// Get the title of the document.
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the content of the document.
    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.content_is_truncated = false;
        self.changed_values |= ChangedAttributes::Content;
    }

    /// Get the content of the document.
    #[inline]
    #[must_use]
    pub fn content(&self) -> Content<'_> {
        if self.content_is_truncated {
            Content::Truncated(&self.content)
        } else {
            Content::Full(&self.content)
        }
    }

    /// Get all custom fields for the document.
    #[inline]
    #[must_use]
    pub fn custom_fields(&self) -> &[DocumentCustomField] {
        &self.custom_fields
    }

    /// Returns `true` if the document has unsaved changes.
    #[inline]
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.changed_values.is_empty()
    }

    /// Set a custom field for the document.
    pub fn set_custom_field(&mut self, field: CustomFieldId, value: &str) {
        for custom_field in &mut self.custom_fields {
            if custom_field.field == field {
                custom_field.value = value.to_string();
                self.changed_values |= ChangedAttributes::CustomFields;
                return;
            }
        }

        self.custom_fields.push(DocumentCustomField {
            field: field,
            value: value.to_string(),
        });
        self.changed_values |= ChangedAttributes::CustomFields;
    }

    /// Update the document on the server.
    pub async fn update(&mut self) -> Result<()> {
        if !self.is_dirty() {
            return Ok(());
        }

        let patch = PatchRequest {
            title: self
                .changed_values
                .contains(ChangedAttributes::Title)
                .then_some(self.title.clone()),

            content: self
                .changed_values
                .contains(ChangedAttributes::Content)
                .then_some(self.content.clone()),

            tags: self
                .changed_values
                .contains(ChangedAttributes::Tags)
                .then_some(self.tags.clone()),

            custom_fields: self
                .changed_values
                .contains(ChangedAttributes::CustomFields)
                .then_some(
                    self.custom_fields
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
                .then_some(self.correspondent)
                .flatten(),

            document_type: self
                .changed_values
                .contains(ChangedAttributes::DocumentType)
                .then_some(self.document_type)
                .flatten(),
        };

        self.client
            .as_ref()
            .unwrap()
            .request(
                Method::PATCH,
                &format!("/api/documents/{}/", self.id),
                Some(&serde_json::to_value(patch).expect("Patch request")),
            )
            .await?;

        self.changed_values = BitFlags::empty();
        Ok(())
    }

    /// Get the full content of the document, replacing any truncated content.
    pub async fn get_full_content(&mut self) -> Result<()> {
        if !self.content_is_truncated {
            return Ok(());
        }

        let doc = self
            .client
            .as_ref()
            .unwrap()
            .get_document_by_id(self.id)
            .await?;

        self.content = doc.content;
        self.content_is_truncated = false;
        Ok(())
    }

    /// Download the document to a file.
    pub async fn download_to_file(&self, path: &Path) -> Result<()> {
        let resp = self
            .client
            .as_ref()
            .unwrap()
            .request(
                Method::GET,
                &format!("/api/documents/{}/download/", self.id),
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
        let resp = self
            .client
            .as_ref()
            .unwrap()
            .request(
                Method::GET,
                &format!("/api/documents/{}/download/", self.id),
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
