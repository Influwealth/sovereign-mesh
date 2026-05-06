use crate::ingress::IngressMessage;
use crate::quantum::QuantumInference;
use crate::scheduler::ScheduleDecision;
use crate::{ExecutionError, Request, Response};

#[derive(Clone, Debug)]
pub struct ExecutionContext {
    pub schedule: ScheduleDecision,
    pub quantum_hint: Option<QuantumInference>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Completed,
    Rejected,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub trace_id: String,
    pub capsule_id: String,
    pub method: String,
    pub status: ExecutionStatus,
    pub output: Vec<u8>,
}

pub trait WasmExecutor {
    fn execute(
        &self,
        ingress: &IngressMessage,
        context: &ExecutionContext,
    ) -> Result<ExecutionReceipt, String>;
}

#[derive(Clone, Debug, Default)]
pub struct MockWasmExecutor;

impl WasmExecutor for MockWasmExecutor {
    fn execute(
        &self,
        ingress: &IngressMessage,
        context: &ExecutionContext,
    ) -> Result<ExecutionReceipt, String> {
        let mut output = Vec::new();
        output.extend_from_slice(b"mock-wasm-output:");
        output.extend_from_slice(context.schedule.lane.as_bytes());

        Ok(ExecutionReceipt {
            trace_id: ingress.trace_id.clone(),
            capsule_id: ingress.capsule_id.clone(),
            method: ingress.method.clone(),
            status: ExecutionStatus::Completed,
            output,
        })
    }
}

pub fn execute_capsule(req: Request) -> Result<Response, ExecutionError> {
    // TODO: load the capsule WASM module declared by capsule.toml.
    // TODO: mount stable, ephemeral, and sealed state before invocation.
    // TODO: dispatch to registered update/query methods through the SDK ABI.
    let message = format!(
        "executed placeholder capsule method {} on {}",
        req.method, req.capsule_id
    );

    Ok(Response::new(
        req.trace_id,
        req.capsule_id,
        req.method,
        message.clone(),
        message.into_bytes(),
    ))
}
