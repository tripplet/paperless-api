use std::collections::HashMap;

use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// A workflow
#[derive(Debug, Clone, Deserialize)]
pub struct Workflow {
    /// Unique identifier of the workflow.
    pub id: crate::id::WorkflowId,

    /// Whether the workflow is enabled.
    pub enabled: bool,

    /// Name of the workflow.
    pub name: String,

    /// Order of the workflow in the list.
    pub order: Option<i32>,

    /// Triggers that determine when the workflow is executed.
    pub triggers: Vec<WorkflowTrigger>,

    /// Actions that are executed when the workflow is triggered.
    pub actions: Vec<WorkflowAction>,
}

/// A trigger that determines when a workflow is executed.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowTrigger {
    pub id: crate::id::WorkflowTriggerId,

    #[serde(rename = "type")]
    pub trigger_type: WorkflowTriggerType,
}

/// An action that can be executed when a workflow is triggered.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowAction {
    pub id: crate::id::WorkflowActionId,

    #[serde(rename = "type")]
    pub action_type: WorkflowActionType,

    pub webhook: Option<WebhookAction>,
}

/// The type of trigger that determines when a workflow is executed.
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WorkflowTriggerType {
    ProcessingStarted = 1,
    DocumentAdded = 2,
    DocumentUpdated = 3,
    Scheduled = 4,
}

/// The type of action that is executed when a workflow is triggered.
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WorkflowActionType {
    Assign = 1,
    Remove = 2,
    Email = 3,
    Webhook = 4,
}

/// A webhook action that can be executed when a workflow is triggered.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookAction {
    pub id: crate::id::WebhookActionId,
    pub url: String,

    pub use_params: bool,
    pub as_json: bool,
    pub include_document: bool,

    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub params: Option<HashMap<String, String>>,
}
