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
    // PQC capsule fast-path (placeholder): routes encrypt/decrypt/sign/verify through runtime quantum hooks.
    // TODO: move into execution engine dispatch once capsule registry loads pqc manifests.
    if req.capsule_id == "capsule.pqc_email_shield.v1" {
        let payload = match req.method.as_str() {
            "encrypt" => crate::quantum::pqc_encrypt(b"recipient_pubkey", &req.payload)?,
            "decrypt" => crate::quantum::pqc_decrypt(b"recipient_privkey", &req.payload)?,
            "sign" => crate::quantum::pqc_sign(b"signing_privkey", &req.payload)?,
            "verify" => {
                let ok = crate::quantum::pqc_verify(b"signing_pubkey", &req.payload, b"sig")?;
                ok.to_string().into_bytes()
            }
            _ => {
                return Err(ExecutionError::ExecutionFailed(
                    "unsupported pqc method".to_string(),
                ))
            }
        };

        return Ok(Response::new(
            req.trace_id,
            req.capsule_id,
            req.method,
            "pqc_email_shield executed",
            payload,
        ));
    }

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
