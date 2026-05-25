use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<StepDefinition>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub next: Option<String>,          // next step id
    pub on_reject: Option<String>,     // step id if rejected
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Action,
    Approval,
    Condition,
    Notification,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub current_step: String,
    pub status: InstanceStatus,
    pub context: serde_json::Value,
    pub history: Vec<StepExecution>,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub completed_at: String,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub instance_id: String,
    pub step_id: String,
    pub step_name: String,
    pub assignee: String,
    pub status: String, // "pending", "approved", "rejected"
    pub requested_at: String,
}
