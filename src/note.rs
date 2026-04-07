use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::id::{NoteId, UserId};

#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    pub id: NoteId,

    pub created: DateTime<Utc>,

    pub user: NoteUser,

    #[serde(rename = "note")]
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoteUser {
    pub id: UserId,
    pub username: String,

    pub first_name: String,
    pub last_name: String,
}
