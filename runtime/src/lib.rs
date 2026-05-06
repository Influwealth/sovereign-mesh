pub mod audit;
pub mod execution;
pub mod ingress;
pub mod policy;
pub mod quantum;
pub mod scheduler;

use audit::{AuditEvent, AuditSink};
use execution::{ExecutionContext, ExecutionReceipt, WasmExecutor};
use ingress::IngressMessage;
use policy::{PolicyDecision, PolicyEngine};
use quantum::QuantumBoundary;
use scheduler::Scheduler;

#[derive(Debug)]
pub enum RuntimeError {
    InvalidIngress(String),
    PolicyDenied(String),
    GovernanceEscalation(String),
    ExecutionFailed(String),
    AuditFailed(String),
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

    pub fn handle_ingress(&mut self, ingress: IngressMessage) -> Result<ExecutionReceipt, RuntimeError> {
        ingress
            .validate()
            .map_err(RuntimeError::InvalidIngress)?;

        let policy_decision = self.policy.evaluate(&ingress);
        match &policy_decision {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(reason) => {
                self.audit_policy_stop(&ingress, &policy_decision)?;
                return Err(RuntimeError::PolicyDenied(reason.clone()));
            }
            PolicyDecision::Escalate(reason) => {
                self.audit_policy_stop(&ingress, &policy_decision)?;
                return Err(RuntimeError::GovernanceEscalation(reason.clone()));
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
            .map_err(RuntimeError::ExecutionFailed)?;

        self.audit
            .record(AuditEvent::execution_completed(&ingress, &receipt))
            .map_err(RuntimeError::AuditFailed)?;

        Ok(receipt)
    }

    fn audit_policy_stop(
        &mut self,
        ingress: &IngressMessage,
        decision: &PolicyDecision,
    ) -> Result<(), RuntimeError> {
        self.audit
            .record(AuditEvent::policy_stopped(ingress, decision))
            .map_err(RuntimeError::AuditFailed)
    }
}
