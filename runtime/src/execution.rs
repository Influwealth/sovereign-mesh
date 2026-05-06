use crate::ingress::IngressMessage;
use crate::quantum::QuantumInference;
use crate::scheduler::ScheduleDecision;

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
