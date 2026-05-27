use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A workflow definition (state machine template)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub states: Vec<String>,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<Transition>,
    pub created_at: String,
}

/// A transition between states
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub action: String,
    pub requires_approval: bool,
    pub allowed_roles: Vec<String>,
}

/// A running workflow instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub current_state: String,
    pub history: Vec<StateChange>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateChange {
    pub from: String,
    pub to: String,
    pub action: String,
    pub actor: String,
    pub timestamp: String,
    pub comment: Option<String>,
}

/// A task (assignable unit of work)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String, // open, in_progress, blocked, completed, cancelled
    pub priority: String, // low, medium, high, critical
    pub assignee: Option<String>,
    pub reporter: String,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub workflow_instance_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

/// An approval request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String, // pending, approved, rejected, expired
    pub requester: String,
    pub approvers: Vec<String>,
    pub decisions: Vec<ApprovalDecision>,
    pub entity_type: String,
    pub entity_id: String,
    pub due_date: Option<String>,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub approver: String,
    pub decision: String, // approved, rejected
    pub comment: Option<String>,
    pub timestamp: String,
}

/// A case (for case management — support, legal, claims, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub title: String,
    pub description: String,
    pub case_type: String,
    pub status: String, // open, investigating, pending, resolved, closed
    pub priority: String,
    pub assignee: Option<String>,
    pub reporter: String,
    pub tags: Vec<String>,
    pub notes: Vec<CaseNote>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaseNote {
    pub author: String,
    pub text: String,
    pub timestamp: String,
}
