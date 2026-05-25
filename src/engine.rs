use crate::domain::*;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Engine {
    pub definitions: Arc<RwLock<Vec<WorkflowDefinition>>>,
    pub instances: Arc<RwLock<Vec<WorkflowInstance>>>,
    pub approvals: Arc<RwLock<Vec<ApprovalRequest>>>,
}

impl Engine {
    pub fn seeded() -> Self {
        let definitions = vec![
            WorkflowDefinition {
                id: "wf-onboard".into(), name: "Employee Onboarding".into(),
                description: "New hire onboarding: IT setup → manager approval → welcome email".into(),
                created_by: "HR Team".into(),
                steps: vec![
                    StepDefinition { id: "s1".into(), name: "Create accounts".into(), step_type: StepType::Action, next: Some("s2".into()), on_reject: None, assignee: Some("IT Team".into()) },
                    StepDefinition { id: "s2".into(), name: "Manager approval".into(), step_type: StepType::Approval, next: Some("s3".into()), on_reject: Some("s1".into()), assignee: Some("manager".into()) },
                    StepDefinition { id: "s3".into(), name: "Send welcome email".into(), step_type: StepType::Notification, next: None, on_reject: None, assignee: None },
                ],
            },
            WorkflowDefinition {
                id: "wf-deploy".into(), name: "Production Deployment".into(),
                description: "Deploy: run tests → security scan → lead approval → deploy → notify".into(),
                created_by: "Engineering".into(),
                steps: vec![
                    StepDefinition { id: "s1".into(), name: "Run tests".into(), step_type: StepType::Action, next: Some("s2".into()), on_reject: None, assignee: None },
                    StepDefinition { id: "s2".into(), name: "Security scan".into(), step_type: StepType::Action, next: Some("s3".into()), on_reject: None, assignee: None },
                    StepDefinition { id: "s3".into(), name: "Tech lead approval".into(), step_type: StepType::Approval, next: Some("s4".into()), on_reject: None, assignee: Some("tech-lead".into()) },
                    StepDefinition { id: "s4".into(), name: "Deploy to production".into(), step_type: StepType::Action, next: Some("s5".into()), on_reject: None, assignee: None },
                    StepDefinition { id: "s5".into(), name: "Notify team".into(), step_type: StepType::Notification, next: None, on_reject: None, assignee: None },
                ],
            },
            WorkflowDefinition {
                id: "wf-expense".into(), name: "Expense Approval".into(),
                description: "Submit expense → manager review → finance approval → reimburse".into(),
                created_by: "Finance".into(),
                steps: vec![
                    StepDefinition { id: "s1".into(), name: "Submit expense".into(), step_type: StepType::Action, next: Some("s2".into()), on_reject: None, assignee: None },
                    StepDefinition { id: "s2".into(), name: "Manager review".into(), step_type: StepType::Approval, next: Some("s3".into()), on_reject: None, assignee: Some("manager".into()) },
                    StepDefinition { id: "s3".into(), name: "Finance approval".into(), step_type: StepType::Approval, next: Some("s4".into()), on_reject: None, assignee: Some("finance".into()) },
                    StepDefinition { id: "s4".into(), name: "Process reimbursement".into(), step_type: StepType::Action, next: None, on_reject: None, assignee: Some("finance".into()) },
                ],
            },
        ];
        let instances = vec![
            WorkflowInstance {
                id: "inst-1".into(), workflow_id: "wf-onboard".into(), workflow_name: "Employee Onboarding".into(),
                current_step: "s2".into(), status: InstanceStatus::WaitingApproval,
                context: serde_json::json!({"employee": "Frank Ochieng", "department": "Engineering"}),
                history: vec![StepExecution { step_id: "s1".into(), step_name: "Create accounts".into(), status: "completed".into(), completed_at: "2026-05-24T10:00:00Z".into(), result: Some(serde_json::json!({"accounts": ["github", "slack", "email"]})) }],
                started_at: "2026-05-24T09:00:00Z".into(),
            },
        ];
        let approvals = vec![
            ApprovalRequest { id: "apr-1".into(), instance_id: "inst-1".into(), step_id: "s2".into(), step_name: "Manager approval".into(), assignee: "alice@company.com".into(), status: "pending".into(), requested_at: "2026-05-24T10:05:00Z".into() },
        ];
        Self {
            definitions: Arc::new(RwLock::new(definitions)),
            instances: Arc::new(RwLock::new(instances)),
            approvals: Arc::new(RwLock::new(approvals)),
        }
    }

    pub async fn advance_instance(&self, instance_id: &str) -> String {
        let mut instances = self.instances.write().await;
        let defs = self.definitions.read().await;
        let inst = match instances.iter_mut().find(|i| i.id == instance_id) {
            Some(i) => i,
            None => return format!("Instance {} not found", instance_id),
        };
        let def = match defs.iter().find(|d| d.id == inst.workflow_id) {
            Some(d) => d,
            None => return "Workflow definition not found".into(),
        };
        let current = match def.steps.iter().find(|s| s.id == inst.current_step) {
            Some(s) => s,
            None => return "Current step not found".into(),
        };
        inst.history.push(StepExecution {
            step_id: current.id.clone(), step_name: current.name.clone(),
            status: "completed".into(), completed_at: Utc::now().to_rfc3339(),
            result: None,
        });
        match &current.next {
            Some(next_id) => {
                inst.current_step = next_id.clone();
                let next_step = def.steps.iter().find(|s| s.id == *next_id);
                if let Some(ns) = next_step {
                    if matches!(ns.step_type, StepType::Approval) {
                        inst.status = InstanceStatus::WaitingApproval;
                        return format!("Advanced to '{}' — waiting for approval", ns.name);
                    }
                }
                inst.status = InstanceStatus::Running;
                format!("Advanced to step '{}'", next_id)
            }
            None => {
                inst.status = InstanceStatus::Completed;
                "Workflow completed".into()
            }
        }
    }
}
