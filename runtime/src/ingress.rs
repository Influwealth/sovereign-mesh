use crate::audit::record_audit_event;
use crate::execution::execute_capsule;
use crate::policy::check_policy;
use crate::scheduler::schedule;
use crate::{ExecutionError, Request, Response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallerClass {
    Agent,
    Partner,
    SovereignUser,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngressMessage {
    pub trace_id: String,
    pub caller_id: String,
    pub caller_class: CallerClass,
    pub capsule_id: String,
    pub method: String,
    pub payload: Vec<u8>,
}

impl IngressMessage {
    pub fn validate(&self) -> Result<(), String> {
        if self.trace_id.trim().is_empty() {
            return Err("trace_id is required".to_string());
        }
        if self.caller_id.trim().is_empty() {
            return Err("caller_id is required".to_string());
        }
        if self.capsule_id.trim().is_empty() {
            return Err("capsule_id is required".to_string());
        }
        if self.method.trim().is_empty() {
            return Err("method is required".to_string());
        }
        Ok(())
    }
}

pub fn handle_ingress(req: Request) -> Result<Response, ExecutionError> {
    record_audit_event(
        "ingress.received",
        &format!("trace_id={} capsule={}", req.trace_id, req.capsule_id),
    );

    check_policy(&req)?;
    let schedule_decision = schedule(&req)?;
    record_audit_event(
        "scheduler.selected",
        &format!(
            "trace_id={} lane={} edge_eligible={}",
            req.trace_id, schedule_decision.lane, schedule_decision.edge_eligible
        ),
    );
    let response = execute_capsule(req)?;

    record_audit_event(
        "ingress.completed",
        &format!(
            "trace_id={} capsule={} method={}",
            response.trace_id, response.capsule_id, response.method
        ),
    );

    Ok(response)
}
