//! Types related to notes.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::id::{NoteId, UserId};

/// A note associated with a document.
#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    /// Unique identifier for the note.
    pub id: NoteId,

    /// When the note was created.
    pub created: DateTime<Utc>,

    /// The user who created the note.
    pub user: NoteUser,

    /// The content of the note.
    #[serde(rename = "note")]
    pub content: String,
}

/// The user who created the note.
#[derive(Debug, Clone, Deserialize)]
pub struct NoteUser {
    /// Unique identifier for the user.
    pub id: UserId,

    /// The username of the user.
    pub username: String,

    /// The first name of the user.
    pub first_name: String,

    /// The last name of the user.
    pub last_name: String,
}
