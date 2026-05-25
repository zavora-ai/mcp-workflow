use crate::domain::*;
use crate::engine::Engine;
use chrono::Utc;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateWorkflowInput {
    pub name: String,
    pub description: String,
    pub steps: Vec<StepInput>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StepInput {
    pub name: String,
    pub step_type: String, // "action", "approval", "condition", "notification", "wait"
    pub assignee: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartWorkflowInput {
    pub workflow_id: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApprovalInput {
    pub approval_id: String,
    pub approved: bool,
    pub comment: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatusFilterInput {
    pub status: Option<String>,
}

#[derive(Clone)]
pub struct WorkflowServer {
    pub engine: Engine,
}

#[tool_router(server_handler)]
impl WorkflowServer {
    #[tool(description = "List all workflow definitions")]
    async fn list_workflows(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let defs = self.engine.definitions.read().await;
        let summary: Vec<serde_json::Value> = defs.iter().map(|d| {
            serde_json::json!({"id": d.id, "name": d.name, "steps": d.steps.len(), "created_by": d.created_by})
        }).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Get workflow definition with all steps")]
    async fn get_workflow(&self, Parameters(input): Parameters<IdInput>) -> String {
        let defs = self.engine.definitions.read().await;
        match defs.iter().find(|d| d.id == input.id) {
            Some(d) => serde_json::to_string_pretty(d).unwrap(),
            None => format!("Workflow {} not found", input.id),
        }
    }

    #[tool(description = "Create a new workflow definition with steps")]
    async fn create_workflow(&self, Parameters(input): Parameters<CreateWorkflowInput>) -> String {
        let id = format!("wf-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let steps: Vec<StepDefinition> = input.steps.iter().enumerate().map(|(i, s)| {
            let step_type = match s.step_type.as_str() {
                "approval" => StepType::Approval,
                "condition" => StepType::Condition,
                "notification" => StepType::Notification,
                "wait" => StepType::Wait,
                _ => StepType::Action,
            };
            let next = if i + 1 < input.steps.len() { Some(format!("s{}", i + 2)) } else { None };
            StepDefinition { id: format!("s{}", i + 1), name: s.name.clone(), step_type, next, on_reject: None, assignee: s.assignee.clone() }
        }).collect();
        let def = WorkflowDefinition { id: id.clone(), name: input.name.clone(), description: input.description, steps, created_by: "AI Agent".into() };
        self.engine.definitions.write().await.push(def);
        format!("Created workflow '{}' (id: {})", input.name, id)
    }

    #[tool(description = "Start a new workflow instance")]
    async fn start_workflow(&self, Parameters(input): Parameters<StartWorkflowInput>) -> String {
        let defs = self.engine.definitions.read().await;
        let def = match defs.iter().find(|d| d.id == input.workflow_id) {
            Some(d) => d,
            None => return format!("Workflow {} not found", input.workflow_id),
        };
        let id = format!("inst-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let first_step = &def.steps[0];
        let status = if matches!(first_step.step_type, StepType::Approval) { InstanceStatus::WaitingApproval } else { InstanceStatus::Running };
        let inst = WorkflowInstance {
            id: id.clone(), workflow_id: input.workflow_id, workflow_name: def.name.clone(),
            current_step: first_step.id.clone(), status,
            context: input.context.unwrap_or(serde_json::json!({})),
            history: Vec::new(), started_at: Utc::now().to_rfc3339(),
        };
        self.engine.instances.write().await.push(inst);
        format!("Started workflow '{}' (instance: {})", def.name, id)
    }

    #[tool(description = "List workflow instances, optionally filtered by status")]
    async fn list_instances(&self, Parameters(input): Parameters<StatusFilterInput>) -> String {
        let instances = self.engine.instances.read().await;
        let filtered: Vec<serde_json::Value> = instances.iter()
            .filter(|i| input.status.as_ref().map_or(true, |s| format!("{:?}", i.status).to_lowercase().contains(&s.to_lowercase())))
            .map(|i| serde_json::json!({"id": i.id, "workflow": i.workflow_name, "current_step": i.current_step, "status": format!("{:?}", i.status), "started_at": i.started_at}))
            .collect();
        serde_json::to_string_pretty(&filtered).unwrap()
    }

    #[tool(description = "Get workflow instance details with history")]
    async fn get_instance(&self, Parameters(input): Parameters<IdInput>) -> String {
        let instances = self.engine.instances.read().await;
        match instances.iter().find(|i| i.id == input.id) {
            Some(i) => serde_json::to_string_pretty(i).unwrap(),
            None => format!("Instance {} not found", input.id),
        }
    }

    #[tool(description = "Advance a workflow instance to the next step")]
    async fn advance_step(&self, Parameters(input): Parameters<IdInput>) -> String {
        self.engine.advance_instance(&input.id).await
    }

    #[tool(description = "Cancel a running workflow instance")]
    async fn cancel_instance(&self, Parameters(input): Parameters<IdInput>) -> String {
        let mut instances = self.engine.instances.write().await;
        match instances.iter_mut().find(|i| i.id == input.id) {
            Some(i) => { i.status = InstanceStatus::Cancelled; format!("Instance {} cancelled", input.id) }
            None => format!("Instance {} not found", input.id),
        }
    }

    #[tool(description = "List pending approval requests")]
    async fn list_approvals(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let approvals = self.engine.approvals.read().await;
        let pending: Vec<&ApprovalRequest> = approvals.iter().filter(|a| a.status == "pending").collect();
        serde_json::to_string_pretty(&pending).unwrap()
    }

    #[tool(description = "Approve or reject a pending approval")]
    async fn resolve_approval(&self, Parameters(input): Parameters<ApprovalInput>) -> String {
        let mut approvals = self.engine.approvals.write().await;
        match approvals.iter_mut().find(|a| a.id == input.approval_id) {
            Some(a) => {
                a.status = if input.approved { "approved".into() } else { "rejected".into() };
                let instance_id = a.instance_id.clone();
                let result = if input.approved {
                    drop(approvals);
                    self.engine.advance_instance(&instance_id).await
                } else {
                    format!("Approval {} rejected", input.approval_id)
                };
                result
            }
            None => format!("Approval {} not found", input.approval_id),
        }
    }

    #[tool(description = "Get workflow execution history for an instance")]
    async fn get_history(&self, Parameters(input): Parameters<IdInput>) -> String {
        let instances = self.engine.instances.read().await;
        match instances.iter().find(|i| i.id == input.id) {
            Some(i) => serde_json::to_string_pretty(&i.history).unwrap(),
            None => format!("Instance {} not found", input.id),
        }
    }

    #[tool(description = "Retry a failed workflow instance from the current step")]
    async fn retry_instance(&self, Parameters(input): Parameters<IdInput>) -> String {
        let mut instances = self.engine.instances.write().await;
        match instances.iter_mut().find(|i| i.id == input.id) {
            Some(i) => {
                if matches!(i.status, InstanceStatus::Failed) {
                    i.status = InstanceStatus::Running;
                    format!("Instance {} retrying from step '{}'", input.id, i.current_step)
                } else {
                    format!("Instance {} is not in failed state", input.id)
                }
            }
            None => format!("Instance {} not found", input.id),
        }
    }

    #[tool(description = "Get summary stats: running, waiting, completed, failed counts")]
    async fn get_stats(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let instances = self.engine.instances.read().await;
        let running = instances.iter().filter(|i| matches!(i.status, InstanceStatus::Running)).count();
        let waiting = instances.iter().filter(|i| matches!(i.status, InstanceStatus::WaitingApproval)).count();
        let completed = instances.iter().filter(|i| matches!(i.status, InstanceStatus::Completed)).count();
        let failed = instances.iter().filter(|i| matches!(i.status, InstanceStatus::Failed)).count();
        serde_json::to_string_pretty(&serde_json::json!({"running": running, "waiting_approval": waiting, "completed": completed, "failed": failed, "total": instances.len()})).unwrap()
    }
}
