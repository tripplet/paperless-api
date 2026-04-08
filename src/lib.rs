//! A small async client for interacting with the Paperless-ngx API.
//!
//! This crate provides [`PaperlessClient`] for talking to a Paperless instance and
//! convenience types for working with documents, tags, custom fields, correspondents,
//! document types, and tasks.
//!
//! # Getting started
//!
//! Create a client with your Paperless base URL and API token:
//!
//! ```no_run
//! use paperless_api::PaperlessClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PaperlessClient::new(
//!     "https://paperless.example.com",
//!     "your-api-token",
//!     None,
//! )?;
//! # let _ = client;
//! # Ok(())
//! # }
//! ```
//!
//! # Refreshing cached metadata
//!
//! The client keeps some metadata cached locally, such as tags, custom fields,
//! correspondents, and document types.
//!
//! You can refresh individual caches:
//!
//! ```no_run
//! use paperless_api::PaperlessClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = PaperlessClient::new(
//!     "https://paperless.example.com",
//!     "your-api-token",
//!     None,
//! )?;
//!
//! client.refresh_tags().await?;
//! client.refresh_custom_fields().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Or refresh multiple datasets at once:
//!
//! ```no_run
//! use paperless_api::{PaperlessClient, RefreshData};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = PaperlessClient::new(
//!     "https://paperless.example.com",
//!     "your-api-token",
//!     None,
//! )?;
//!
//! client.refresh([RefreshData::Tags, RefreshData::CustomFields]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Working with tags
//!
//! After refreshing tags, you can look them up from the local cache:
//!
//! ```no_run
//! use paperless_api::PaperlessClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = PaperlessClient::new(
//!     "https://paperless.example.com",
//!     "your-api-token",
//!     None,
//! )?;
//!
//! client.refresh_tags().await?;
//!
//! if let Some(tag) = client.find_tag_by_name("invoice") {
//!     let docs = client.get_documents_by_tags(&[tag.id], true).await?;
//!     println!("found {} documents", docs.len());
//! }
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod correspondent;
pub mod custom_field;
pub mod document;
pub mod document_type;
pub mod id;
pub mod note;
pub mod share_link;
pub mod storage_path;
pub mod tag;
pub mod task;
pub mod user;
pub mod workflow;

pub use client::{PaperlessClient, RefreshData};
pub use custom_field::{CustomField, DocumentCustomField};
pub use document::Document;
pub use tag::Tag;
pub use task::Task;
pub use user::User;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("response error: status code {status_code}, body: {body}")]
    Response { status_code: u16, body: String },

    #[error(transparent)]
    Request(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("other error: {0}")]
    Other(String),

    #[error("invalid json: {0}")]
    InvalidJson(String),

    #[error("not found")]
    NotFound,

    #[error("not changeable")]
    NotChangeable,

    #[error("already deleted")]
    AlreadyDeleted,
}

type Result<T> = std::result::Result<T, Error>;
