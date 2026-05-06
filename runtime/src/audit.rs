use crate::execution::{ExecutionReceipt, ExecutionStatus};
use crate::ingress::IngressMessage;
use crate::policy::PolicyDecision;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditKind {
    PolicyStopped,
    ExecutionCompleted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEvent {
    pub trace_id: String,
    pub capsule_id: String,
    pub kind: AuditKind,
    pub message: String,
}

impl AuditEvent {
    pub fn policy_stopped(ingress: &IngressMessage, decision: &PolicyDecision) -> Self {
        Self {
            trace_id: ingress.trace_id.clone(),
            capsule_id: ingress.capsule_id.clone(),
            kind: AuditKind::PolicyStopped,
            message: format!("policy decision: {:?}", decision),
        }
    }

    pub fn execution_completed(ingress: &IngressMessage, receipt: &ExecutionReceipt) -> Self {
        let status = match &receipt.status {
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Rejected => "rejected",
            ExecutionStatus::Failed => "failed",
        };

        Self {
            trace_id: ingress.trace_id.clone(),
            capsule_id: ingress.capsule_id.clone(),
            kind: AuditKind::ExecutionCompleted,
            message: format!("execution {}", status),
        }
    }
}

pub trait AuditSink {
    fn record(&mut self, event: AuditEvent) -> Result<(), String>;
}

#[derive(Clone, Debug, Default)]
pub struct MemoryAuditSink {
    pub events: Vec<AuditEvent>,
}

impl AuditSink for MemoryAuditSink {
    fn record(&mut self, event: AuditEvent) -> Result<(), String> {
        self.events.push(event);
        Ok(())
    }
}
