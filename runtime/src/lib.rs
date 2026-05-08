pub mod audit;
pub mod execution;
pub mod graph;
pub mod ingress;
pub mod policy;
pub mod policy_loader;
pub mod quantum;
pub mod scheduler;
pub mod state;

pub use audit::record_audit_event;
pub use execution::execute_capsule;
pub use graph::{load_graph_for_capsule, CapsuleEdge, CapsuleGraph, CapsuleNode};
pub use ingress::handle_ingress;
pub use policy::check_policy;
pub use quantum::run_quantum_job;
pub use scheduler::{schedule, ScheduleDecision};
pub use state::{FileStateStore, InMemoryStateStore, StateSnapshot, StateStore};

use std::sync::Arc;

use audit::{AuditEvent, AuditSink};
use execution::{ExecutionContext, ExecutionReceipt, WasmExecutor};
use ingress::IngressMessage;
use policy::{PolicyDecision, PolicyEngine};
use quantum::QuantumBoundary;
use scheduler::Scheduler;

pub type CapsuleId = String;
pub type MethodName = String;
pub type NodeId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubsidyClass {
    Youth,
    Underserved,
    Standard,
    Enterprise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeCompatibility {
    Eligible,
    NotEligible,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub trace_id: String,
    pub caller: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub payload: Vec<u8>,
    pub subsidy_class: SubsidyClass,
    pub edge_compatibility: EdgeCompatibility,
    pub preferred_node: Option<NodeId>,
}

impl Request {
    pub fn new(
        trace_id: impl Into<String>,
        caller: impl Into<String>,
        capsule_id: impl Into<CapsuleId>,
        method: impl Into<MethodName>,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            caller: caller.into(),
            capsule_id: capsule_id.into(),
            method: method.into(),
            payload: payload.into(),
            subsidy_class: SubsidyClass::Standard,
            edge_compatibility: EdgeCompatibility::NotEligible,
            preferred_node: None,
        }
    }

    pub fn with_scheduling(
        mut self,
        subsidy_class: SubsidyClass,
        edge_compatibility: EdgeCompatibility,
        preferred_node: Option<NodeId>,
    ) -> Self {
        self.subsidy_class = subsidy_class;
        self.edge_compatibility = edge_compatibility;
        self.preferred_node = preferred_node;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub trace_id: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub message: String,
    pub payload: Vec<u8>,
}

impl Response {
    pub fn new(
        trace_id: impl Into<String>,
        capsule_id: impl Into<CapsuleId>,
        method: impl Into<MethodName>,
        message: impl Into<String>,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            capsule_id: capsule_id.into(),
            method: method.into(),
            message: message.into(),
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidRequest(String),
    PolicyDenied(String),
    GovernanceEscalation(String),
    SchedulingFailed(String),
    ExecutionFailed(String),
    AuditFailed(String),
    QuantumFailed(String),
    StateFailed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Normal,
    Replay { snapshot: StateSnapshot },
}

#[derive(Clone)]
pub struct RuntimeContext {
    pub node_id: NodeId,
    pub execution_mode: ExecutionMode,
    pub state_store: Arc<dyn StateStore>,
}

impl RuntimeContext {
    pub fn new(node_id: impl Into<NodeId>, state_store: Arc<dyn StateStore>) -> Self {
        Self {
            node_id: node_id.into(),
            execution_mode: ExecutionMode::Normal,
            state_store,
        }
    }

    pub fn replay(
        node_id: impl Into<NodeId>,
        state_store: Arc<dyn StateStore>,
        snapshot: StateSnapshot,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            execution_mode: ExecutionMode::Replay { snapshot },
            state_store,
        }
    }
}

pub struct CapsuleRuntime<P, S, E, Q, A>
where
    P: PolicyEngine,
    S: Scheduler,
    E: WasmExecutor,
    Q: QuantumBoundary,
    A: AuditSink,
{
    policy: P,
    scheduler: S,
    executor: E,
    quantum: Q,
    audit: A,
}

impl<P, S, E, Q, A> CapsuleRuntime<P, S, E, Q, A>
where
    P: PolicyEngine,
    S: Scheduler,
    E: WasmExecutor,
    Q: QuantumBoundary,
    A: AuditSink,
{
    pub fn new(policy: P, scheduler: S, executor: E, quantum: Q, audit: A) -> Self {
        Self {
            policy,
            scheduler,
            executor,
            quantum,
            audit,
        }
    }

    pub fn handle_ingress(
        &mut self,
        ingress: IngressMessage,
    ) -> Result<ExecutionReceipt, ExecutionError> {
        ingress
            .validate()
            .map_err(ExecutionError::InvalidRequest)?;

        let policy_decision = self.policy.evaluate(&ingress);
        match &policy_decision {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(reason) => {
                self.audit_policy_stop(&ingress, &policy_decision)?;
                return Err(ExecutionError::PolicyDenied(reason.clone()));
            }
            PolicyDecision::Escalate(reason) => {
                self.audit_policy_stop(&ingress, &policy_decision)?;
                return Err(ExecutionError::GovernanceEscalation(reason.clone()));
            }
        }

        let schedule = self.scheduler.schedule(&ingress);
        let quantum_hint = self.quantum.infer("capsule-execution-readiness", &ingress.payload);
        let context = ExecutionContext {
            schedule,
            quantum_hint: Some(quantum_hint),
        };

        let receipt = self
            .executor
            .execute(&ingress, &context)
            .map_err(ExecutionError::ExecutionFailed)?;

        self.audit
            .record(AuditEvent::execution_completed(&ingress, &receipt))
            .map_err(ExecutionError::AuditFailed)?;

        Ok(receipt)
    }

    fn audit_policy_stop(
        &mut self,
        ingress: &IngressMessage,
        decision: &PolicyDecision,
    ) -> Result<(), ExecutionError> {
        self.audit
            .record(AuditEvent::policy_stopped(ingress, decision))
            .map_err(ExecutionError::AuditFailed)
    }
}
