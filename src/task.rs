//! Types related to tasks.

use std::time::Duration;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::id::{TaskId, UniqueTaskId};

/// A paperless task.
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    /// Unique identifier of the task.
    pub id: crate::id::TaskId,

    /// The Celery-ID of the task.
    pub task_id: crate::id::UniqueTaskId,

    /// The type of the task.
    pub task_type: TaskType,

    /// The source of the task.
    pub trigger_source: TaskTriggerSource,

    /// The status of the task.
    pub status: TaskStatus,

    /// The result of the task, if any.
    pub result: Option<String>,

    /// When the task was created.
    pub date_created: chrono::DateTime<chrono::Utc>,

    /// When the task was started.
    pub date_started: Option<chrono::DateTime<chrono::Utc>>,

    /// When the task was completed.
    pub date_done: Option<chrono::DateTime<chrono::Utc>>,

    /// The duration the task took to complete.
    #[serde(
        rename = "duration_seconds",
        deserialize_with = "deserialize_duration_seconds"
    )]
    pub duration: Option<Duration>,

    /// The time the task was queued.
    #[serde(
        rename = "wait_time_seconds",
        deserialize_with = "deserialize_duration_seconds"
    )]
    pub wait_time: Option<Duration>,

    /// The input data for the task.
    pub input_data: Option<serde_json::Value>,

    /// The result data for the task.
    pub result_data: Option<serde_json::Value>,

    /// IDs of related document records.
    ///
    /// A document upload reports a root [`DocumentId`](crate::id::DocumentId).
    /// A version upload reports a [`DocumentVersionId`](crate::id::DocumentVersionId)
    #[serde(rename = "related_document_ids")]
    pub related_documents: Option<Vec<crate::id::DocumentVersionId>>,

    /// Whether the task has been acknowledged.
    pub acknowledged: bool,

    /// The user who owns the task.
    pub owner: Option<crate::id::UserId>,
}

impl From<&Task> for TaskId {
    fn from(task: &Task) -> Self {
        task.id
    }
}

impl From<&TaskId> for TaskId {
    fn from(id: &TaskId) -> Self {
        *id
    }
}

impl From<Task> for TaskId {
    fn from(task: Task) -> Self {
        task.id
    }
}

impl From<Task> for UniqueTaskId {
    fn from(task: Task) -> Self {
        task.task_id
    }
}

impl From<&Task> for UniqueTaskId {
    fn from(task: &Task) -> Self {
        task.task_id.clone()
    }
}

/// The status of a task.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Started,
    Success,
    Failure,
    Revoked,
}

/// The type of a task.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    ConsumeFile,
    TrainClassifier,
    SanityCheck,
    IndexOptimize,
    MailFetch,
    LlmIndex,
    EmptyTrash,
    CheckWorkflows,
    BulkUpdate,
    ReprocessDocument,
    BuildShareLink,
    BulkDelete,
}

/// The source of a task trigger.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskTriggerSource {
    Scheduled,
    WebUi,
    ApiUpload,
    FolderConsume,
    EmailConsume,
    System,
    Manual,
}

#[derive(Serialize)]
pub(crate) struct AcknowledgeRequest {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tasks: Vec<TaskId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) all: Option<bool>,
}

fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let seconds: Option<f64> = serde::Deserialize::deserialize(deserializer)?;
    Ok(seconds.map(Duration::from_secs_f64))
}
