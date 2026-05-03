#![doc = include_str!("../README.md")]

pub mod client;
pub mod document;
pub mod document_query;
pub mod dto;
pub mod group;
pub mod id;
pub mod metadata;
pub mod note;
pub mod saved_view;
pub mod share_link;
pub mod task;
pub mod user;
pub mod util;
pub mod workflow;

pub use client::{PaperlessClient, RefreshMetaData};
pub use document::Document;
pub use document_query::DocumentQuery;
pub use group::Group;
pub use saved_view::{CreateSavedView, SavedView};
pub use share_link::ShareLink;
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
