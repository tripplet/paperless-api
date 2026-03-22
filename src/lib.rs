pub mod client;
pub mod correspondent;
pub mod custom_field;
pub mod document;
pub mod document_type;
pub mod tag;
pub mod task;

pub use client::{PaperlessClient, RefreshData};
pub use custom_field::{CustomField, DocumentCustomField};
pub use document::Document;
pub use tag::Tag;
pub use task::Task;

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
}

type Result<T> = std::result::Result<T, Error>;
