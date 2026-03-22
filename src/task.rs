use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct TaskId(pub i32);

/// A paperless task
#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: TaskId,

    #[serde(rename = "task_name")]
    pub name: String,

    pub owner: i32,
    pub task_id: String,
    pub status: String,
    pub acknowledged: bool,
    pub result: Option<String>,
    pub related_document: Option<String>,
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
