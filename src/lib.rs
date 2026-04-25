#![doc = include_str!("../README.md")]

pub mod client;
pub mod correspondent;
pub mod custom_field;
pub mod document;
pub mod document_type;
pub mod group;
pub mod id;
pub mod note;
pub mod permission;
pub mod share_link;
pub mod storage_path;
pub mod tag;
pub mod task;
pub mod user;
pub mod util;
pub mod workflow;

pub use client::{PaperlessClient, RefreshMetaData};
pub use correspondent::Correspondent;
pub use custom_field::{CustomField, DocumentCustomField};
pub use document::Document;
pub use document_type::DocumentType;
pub use group::Group;
pub use share_link::ShareLink;
pub use storage_path::StoragePath;
pub use tag::Tag;
pub use task::Task;
pub use user::User;
pub use workflow::Workflow;

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
