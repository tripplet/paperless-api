use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::user::UserId;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct TaskId(pub String);

/// A paperless task
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: u32,

    /// The Celery-ID of the task.
    pub task_id: TaskId,

    /// The name/king of the task.
    #[serde(rename = "task_name")]
    pub name: TaskName,

    /// The type of the task.
    #[serde(rename = "type")]
    pub task_type: TaskType,

    /// The status of the task.
    pub status: TaskStatus,

    pub owner: UserId,

    pub acknowledged: bool,
    pub result: Option<String>,
    pub related_document: Option<String>,
}

/// The status of a task.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskStatus {
    Failure,
    Pending,
    Received,
    Retry,
    Revoked,
    Started,
    Success,
}

/// The name of a task.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskName {
    ConsumeFile,
    TrainClassifier,
    CheckSanity,
    IndexOptimize,
}

/// The type of a task.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AutoTask,
    ScheduledTask,
    ManualTask,
}
