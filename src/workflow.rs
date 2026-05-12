//! Types related to paperless workflows.

use std::collections::HashMap;

use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use paperless_api_macros::UpdateDto;

/// A workflow.
#[derive(Debug, Clone, Deserialize, UpdateDto)]
pub struct Workflow {
    /// Unique identifier of the workflow.
    #[dto(skip)]
    pub id: crate::id::WorkflowId,

    /// Whether the workflow is enabled.
    pub enabled: bool,

    /// Name of the workflow.
    pub name: String,

    /// Order of the workflow in the list.
    pub order: Option<i32>,

    /// Triggers that determine when the workflow is executed.
    #[dto(skip)] // TODO
    pub triggers: Vec<WorkflowTrigger>,

    /// Actions that are executed when the workflow is triggered.
    #[dto(skip)] // TODO
    pub actions: Vec<WorkflowAction>,
}

/// A trigger that determines when a workflow is executed.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowTrigger {
    /// Unique identifier of the trigger.
    pub id: crate::id::WorkflowTriggerId,

    /// The type of trigger.
    #[serde(rename = "type")]
    pub trigger_type: WorkflowTriggerType,
}

/// An action that can be executed when a workflow is triggered.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowAction {
    /// Unique identifier of the workflow action.
    pub id: crate::id::WorkflowActionId,

    /// The type of action.
    #[serde(rename = "type")]
    pub action_type: WorkflowActionType,

    /// Webhook configuration, if the action type is a webhook.
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
    /// Unique identifier of the webhook action.
    pub id: crate::id::WebhookActionId,

    /// URL of the webhook.
    pub url: String,

    /// Whether to use query parameters.
    pub use_params: bool,

    /// Whether to send the body as JSON.
    pub as_json: bool,

    /// Whether to include the document in the request.
    pub include_document: bool,

    /// Body of the webhook request.
    pub body: Option<String>,

    /// Headers to include in the webhook request.
    pub headers: Option<HashMap<String, String>>,

    /// Query parameters to include in the webhook request.
    pub params: Option<HashMap<String, String>>,
}
