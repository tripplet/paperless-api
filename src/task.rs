//! Types related to tasks.

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// A paperless task.
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    /// Unique identifier of the task.
    pub id: u32,

    /// The Celery-ID of the task.
    pub task_id: crate::id::TaskId,

    /// The name/kind of the task.
    #[serde(rename = "task_name")]
    pub name: TaskName,

    /// The type of the task.
    #[serde(rename = "type")]
    pub task_type: TaskType,

    /// The status of the task.
    pub status: TaskStatus,

    /// The user who owns the task.
    pub owner: crate::id::UserId,

    /// Whether the task has been acknowledged.
    pub acknowledged: bool,

    /// The result of the task, if any.
    pub result: Option<String>,

    /// The ID of the related document, if any.
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
