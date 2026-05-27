use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::*;

fn now() -> String { chrono::Utc::now().to_rfc3339() }
fn uid() -> String { uuid::Uuid::new_v4().to_string()[..8].to_string() }

#[derive(Clone)]
pub struct Store {
    pub workflows: Arc<Mutex<HashMap<String, WorkflowDef>>>,
    pub instances: Arc<Mutex<HashMap<String, WorkflowInstance>>>,
    pub tasks: Arc<Mutex<HashMap<String, Task>>>,
    pub approvals: Arc<Mutex<HashMap<String, Approval>>>,
    pub cases: Arc<Mutex<HashMap<String, Case>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            instances: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            approvals: Arc::new(Mutex::new(HashMap::new())),
            cases: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_workflow(&self, mut w: WorkflowDef) -> String {
        w.id = format!("wf_{}", uid());
        w.created_at = now();
        let id = w.id.clone();
        self.workflows.lock().unwrap().insert(id.clone(), w);
        id
    }

    pub fn start_instance(&self, workflow_id: &str, entity_type: &str, entity_id: &str, metadata: serde_json::Value) -> Option<String> {
        let wf = self.workflows.lock().unwrap().get(workflow_id)?.clone();
        let inst = WorkflowInstance {
            id: format!("inst_{}", uid()),
            workflow_id: workflow_id.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            current_state: wf.initial_state.clone(),
            history: vec![],
            metadata,
            created_at: now(),
            updated_at: now(),
        };
        let id = inst.id.clone();
        self.instances.lock().unwrap().insert(id.clone(), inst);
        Some(id)
    }

    pub fn create_task(&self, mut t: Task) -> String {
        t.id = format!("task_{}", uid());
        t.created_at = now();
        t.updated_at = now();
        let id = t.id.clone();
        self.tasks.lock().unwrap().insert(id.clone(), t);
        id
    }

    pub fn create_approval(&self, mut a: Approval) -> String {
        a.id = format!("appr_{}", uid());
        a.created_at = now();
        let id = a.id.clone();
        self.approvals.lock().unwrap().insert(id.clone(), a);
        id
    }

    pub fn create_case(&self, mut c: Case) -> String {
        c.id = format!("case_{}", uid());
        c.created_at = now();
        c.updated_at = now();
        let id = c.id.clone();
        self.cases.lock().unwrap().insert(id.clone(), c);
        id
    }
}
