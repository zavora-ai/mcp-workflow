use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::{json, Value};
use crate::types::*;
use crate::store::Store;

fn now() -> String { chrono::Utc::now().to_rfc3339() }

// --- Input Types ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WorkflowDefInput {
    /// Workflow name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// List of states (e.g. ["draft", "review", "approved", "rejected"])
    pub states: Vec<String>,
    /// Initial state
    pub initial_state: String,
    /// Terminal states (workflow ends here)
    pub terminal_states: Vec<String>,
    /// Transitions: [{"from": "draft", "to": "review", "action": "submit", "requires_approval": false, "allowed_roles": ["author"]}]
    pub transitions: Vec<Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartInstanceInput {
    /// Workflow definition ID
    pub workflow_id: String,
    /// Entity type (e.g. "order", "claim", "document", "change_request")
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Metadata
    pub metadata: Option<Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TransitionInput {
    /// Workflow instance ID
    pub instance_id: String,
    /// Action to perform (must match a valid transition from current state)
    pub action: String,
    /// Actor performing the action
    pub actor: String,
    /// Comment
    pub comment: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InstanceIdInput {
    /// Instance ID
    pub instance_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskInput {
    /// Task title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Priority: low, medium, high, critical
    pub priority: Option<String>,
    /// Assignee
    pub assignee: Option<String>,
    /// Reporter
    pub reporter: String,
    /// Due date (ISO datetime)
    pub due_date: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Link to workflow instance
    pub workflow_instance_id: Option<String>,
    /// Parent task ID (for subtasks)
    pub parent_task_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskUpdateInput {
    /// Task ID
    pub task_id: String,
    /// New status: open, in_progress, blocked, completed, cancelled
    pub status: Option<String>,
    /// New assignee
    pub assignee: Option<String>,
    /// New priority
    pub priority: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskIdInput {
    /// Task ID
    pub task_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApprovalInput {
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Requester
    pub requester: String,
    /// Approvers (list of user IDs)
    pub approvers: Vec<String>,
    /// Entity type being approved
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Due date
    pub due_date: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApprovalDecisionInput {
    /// Approval ID
    pub approval_id: String,
    /// Approver making the decision
    pub approver: String,
    /// Decision: approved or rejected
    pub decision: String,
    /// Comment
    pub comment: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CaseInput {
    /// Case title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Case type (support, legal, claim, incident, change_request)
    pub case_type: String,
    /// Priority: low, medium, high, critical
    pub priority: Option<String>,
    /// Assignee
    pub assignee: Option<String>,
    /// Reporter
    pub reporter: String,
    /// Tags
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CaseNoteInput {
    /// Case ID
    pub case_id: String,
    /// Note author
    pub author: String,
    /// Note text
    pub text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CaseUpdateInput {
    /// Case ID
    pub case_id: String,
    /// New status: open, investigating, pending, resolved, closed
    pub status: Option<String>,
    /// New assignee
    pub assignee: Option<String>,
    /// New priority
    pub priority: Option<String>,
}

// --- Server ---

#[derive(Clone)]
pub struct WorkflowServer {
    pub store: Store,
}

impl WorkflowServer {
    pub fn new() -> Self { Self { store: Store::new() } }
}

#[tool_router(server_handler)]
impl WorkflowServer {
    // === Workflow Definitions ===

    #[tool(description = "Define a workflow (state machine) with states, transitions, and approval gates.")]
    async fn workflow_create(&self, Parameters(input): Parameters<WorkflowDefInput>) -> String {
        let transitions: Vec<Transition> = input.transitions.iter().filter_map(|t| {
            Some(Transition {
                from: t["from"].as_str()?.into(), to: t["to"].as_str()?.into(),
                action: t["action"].as_str()?.into(),
                requires_approval: t["requires_approval"].as_bool().unwrap_or(false),
                allowed_roles: t["allowed_roles"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            })
        }).collect();
        let wf = WorkflowDef {
            id: String::new(), name: input.name, description: input.description.unwrap_or_default(),
            states: input.states, initial_state: input.initial_state, terminal_states: input.terminal_states,
            transitions, created_at: String::new(),
        };
        let id = self.store.create_workflow(wf);
        json!({"status": "created", "workflow_id": id}).to_string()
    }

    #[tool(description = "List all workflow definitions.")]
    async fn workflow_list(&self) -> String {
        let wfs: Vec<_> = self.store.workflows.lock().unwrap().values().cloned().collect();
        json!({"count": wfs.len(), "workflows": wfs}).to_string()
    }

    // === Workflow Instances ===

    #[tool(description = "Start a new workflow instance for an entity (order, claim, document, etc.).")]
    async fn instance_start(&self, Parameters(input): Parameters<StartInstanceInput>) -> String {
        match self.store.start_instance(&input.workflow_id, &input.entity_type, &input.entity_id, input.metadata.unwrap_or(json!({}))) {
            Some(id) => json!({"status": "started", "instance_id": id, "entity_type": input.entity_type, "entity_id": input.entity_id}).to_string(),
            None => json!({"error": "WORKFLOW_NOT_FOUND", "workflow_id": input.workflow_id}).to_string(),
        }
    }

    #[tool(description = "Transition a workflow instance to a new state by performing an action.")]
    async fn instance_transition(&self, Parameters(input): Parameters<TransitionInput>) -> String {
        let mut instances = self.store.instances.lock().unwrap();
        let inst = match instances.get_mut(&input.instance_id) {
            Some(i) => i,
            None => return json!({"error": "INSTANCE_NOT_FOUND"}).to_string(),
        };
        let wfs = self.store.workflows.lock().unwrap();
        let wf = match wfs.get(&inst.workflow_id) {
            Some(w) => w,
            None => return json!({"error": "WORKFLOW_NOT_FOUND"}).to_string(),
        };
        // Find valid transition
        let transition = wf.transitions.iter().find(|t| t.from == inst.current_state && t.action == input.action);
        match transition {
            Some(t) => {
                let from = inst.current_state.clone();
                inst.current_state = t.to.clone();
                inst.updated_at = now();
                inst.history.push(StateChange { from: from.clone(), to: t.to.clone(), action: input.action.clone(), actor: input.actor, timestamp: now(), comment: input.comment });
                let is_terminal = wf.terminal_states.contains(&inst.current_state);
                json!({"status": "transitioned", "from": from, "to": t.to, "action": input.action, "is_terminal": is_terminal}).to_string()
            }
            None => {
                let valid: Vec<_> = wf.transitions.iter().filter(|t| t.from == inst.current_state).map(|t| &t.action).collect();
                json!({"error": "INVALID_TRANSITION", "current_state": inst.current_state, "attempted_action": input.action, "valid_actions": valid}).to_string()
            }
        }
    }

    #[tool(description = "Get the current state and history of a workflow instance.")]
    async fn instance_get(&self, Parameters(input): Parameters<InstanceIdInput>) -> String {
        match self.store.instances.lock().unwrap().get(&input.instance_id) {
            Some(i) => serde_json::to_string_pretty(i).unwrap_or_default(),
            None => json!({"error": "INSTANCE_NOT_FOUND"}).to_string(),
        }
    }

    // === Tasks ===

    #[tool(description = "Create a task (assignable unit of work). Can be linked to a workflow instance or be standalone.")]
    async fn task_create(&self, Parameters(input): Parameters<TaskInput>) -> String {
        let task = Task {
            id: String::new(), title: input.title, description: input.description.unwrap_or_default(),
            status: "open".into(), priority: input.priority.unwrap_or_else(|| "medium".into()),
            assignee: input.assignee, reporter: input.reporter, due_date: input.due_date,
            tags: input.tags.unwrap_or_default(), workflow_instance_id: input.workflow_instance_id,
            parent_task_id: input.parent_task_id, metadata: json!({}),
            created_at: String::new(), updated_at: String::new(),
        };
        let id = self.store.create_task(task);
        json!({"status": "created", "task_id": id}).to_string()
    }

    #[tool(description = "Update a task (status, assignee, priority).")]
    async fn task_update(&self, Parameters(input): Parameters<TaskUpdateInput>) -> String {
        let mut tasks = self.store.tasks.lock().unwrap();
        match tasks.get_mut(&input.task_id) {
            Some(t) => {
                if let Some(s) = input.status { t.status = s; }
                if let Some(a) = input.assignee { t.assignee = Some(a); }
                if let Some(p) = input.priority { t.priority = p; }
                t.updated_at = now();
                json!({"status": "updated", "task_id": input.task_id, "current_status": t.status}).to_string()
            }
            None => json!({"error": "TASK_NOT_FOUND"}).to_string(),
        }
    }

    #[tool(description = "List tasks. Optionally filter by status or assignee.")]
    async fn task_list(&self) -> String {
        let tasks: Vec<_> = self.store.tasks.lock().unwrap().values().cloned().collect();
        json!({"count": tasks.len(), "tasks": tasks}).to_string()
    }

    // === Approvals ===

    #[tool(description = "Create an approval request (routes to one or more approvers).")]
    async fn approval_create(&self, Parameters(input): Parameters<ApprovalInput>) -> String {
        let approval = Approval {
            id: String::new(), title: input.title, description: input.description.unwrap_or_default(),
            status: "pending".into(), requester: input.requester, approvers: input.approvers,
            decisions: vec![], entity_type: input.entity_type, entity_id: input.entity_id,
            due_date: input.due_date, metadata: json!({}), created_at: String::new(),
        };
        let id = self.store.create_approval(approval);
        json!({"status": "created", "approval_id": id}).to_string()
    }

    #[tool(description = "Submit an approval decision (approve or reject).")]
    async fn approval_decide(&self, Parameters(input): Parameters<ApprovalDecisionInput>) -> String {
        let mut approvals = self.store.approvals.lock().unwrap();
        match approvals.get_mut(&input.approval_id) {
            Some(a) => {
                if a.status != "pending" { return json!({"error": "APPROVAL_NOT_PENDING", "status": a.status}).to_string(); }
                if !a.approvers.contains(&input.approver) { return json!({"error": "NOT_AN_APPROVER"}).to_string(); }
                a.decisions.push(ApprovalDecision { approver: input.approver, decision: input.decision.clone(), comment: input.comment, timestamp: now() });
                // Check if all approvers have decided
                if a.decisions.len() >= a.approvers.len() {
                    a.status = if a.decisions.iter().all(|d| d.decision == "approved") { "approved".into() } else { "rejected".into() };
                }
                json!({"status": a.status, "approval_id": input.approval_id, "decisions": a.decisions.len(), "total_approvers": a.approvers.len()}).to_string()
            }
            None => json!({"error": "APPROVAL_NOT_FOUND"}).to_string(),
        }
    }

    #[tool(description = "List pending approvals (optionally for a specific approver).")]
    async fn approval_list(&self) -> String {
        let approvals: Vec<_> = self.store.approvals.lock().unwrap().values().cloned().collect();
        json!({"count": approvals.len(), "approvals": approvals}).to_string()
    }

    // === Cases ===

    #[tool(description = "Create a case (support ticket, legal matter, claim, incident, change request).")]
    async fn case_create(&self, Parameters(input): Parameters<CaseInput>) -> String {
        let case = Case {
            id: String::new(), title: input.title, description: input.description.unwrap_or_default(),
            case_type: input.case_type, status: "open".into(),
            priority: input.priority.unwrap_or_else(|| "medium".into()),
            assignee: input.assignee, reporter: input.reporter,
            tags: input.tags.unwrap_or_default(), notes: vec![], metadata: json!({}),
            created_at: String::new(), updated_at: String::new(),
        };
        let id = self.store.create_case(case);
        json!({"status": "created", "case_id": id}).to_string()
    }

    #[tool(description = "Add a note to a case.")]
    async fn case_add_note(&self, Parameters(input): Parameters<CaseNoteInput>) -> String {
        let mut cases = self.store.cases.lock().unwrap();
        match cases.get_mut(&input.case_id) {
            Some(c) => {
                c.notes.push(CaseNote { author: input.author, text: input.text, timestamp: now() });
                c.updated_at = now();
                json!({"status": "note_added", "case_id": input.case_id, "notes_count": c.notes.len()}).to_string()
            }
            None => json!({"error": "CASE_NOT_FOUND"}).to_string(),
        }
    }

    #[tool(description = "Update case status, assignee, or priority.")]
    async fn case_update(&self, Parameters(input): Parameters<CaseUpdateInput>) -> String {
        let mut cases = self.store.cases.lock().unwrap();
        match cases.get_mut(&input.case_id) {
            Some(c) => {
                if let Some(s) = input.status { c.status = s; }
                if let Some(a) = input.assignee { c.assignee = Some(a); }
                if let Some(p) = input.priority { c.priority = p; }
                c.updated_at = now();
                json!({"status": "updated", "case_id": input.case_id, "current_status": c.status}).to_string()
            }
            None => json!({"error": "CASE_NOT_FOUND"}).to_string(),
        }
    }

    #[tool(description = "List all cases.")]
    async fn case_list(&self) -> String {
        let cases: Vec<_> = self.store.cases.lock().unwrap().values().cloned().collect();
        json!({"count": cases.len(), "cases": cases}).to_string()
    }
}
