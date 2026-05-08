use std::collections::BTreeMap;

use crate::ingress::IngressMessage;
use crate::quantum::QuantumInference;
use crate::scheduler::ScheduleDecision;
use crate::{CapsuleId, ExecutionError, ExecutionMode, Request, Response, RuntimeContext};

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

pub trait StableStateMount {
    fn read_stable(&self, key: &str) -> Option<Vec<u8>>;
    fn write_stable(&mut self, key: &str, value: Vec<u8>);
}

pub trait EphemeralStateMount {
    fn read_ephemeral(&self, key: &str) -> Option<String>;
    fn write_ephemeral(&mut self, key: &str, value: String);
}

pub trait SealedStateMount {
    fn open_sealed(&self, key: &str) -> Option<Vec<u8>>;
    fn write_sealed(&mut self, key: &str, value: Vec<u8>);
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryStateMount {
    stable: BTreeMap<String, Vec<u8>>,
    ephemeral: BTreeMap<String, String>,
    sealed: BTreeMap<String, Vec<u8>>,
}

impl StableStateMount for InMemoryStateMount {
    fn read_stable(&self, key: &str) -> Option<Vec<u8>> {
        self.stable.get(key).cloned()
    }

    fn write_stable(&mut self, key: &str, value: Vec<u8>) {
        self.stable.insert(key.to_string(), value);
    }
}

impl EphemeralStateMount for InMemoryStateMount {
    fn read_ephemeral(&self, key: &str) -> Option<String> {
        self.ephemeral.get(key).cloned()
    }

    fn write_ephemeral(&mut self, key: &str, value: String) {
        self.ephemeral.insert(key.to_string(), value);
    }
}

impl SealedStateMount for InMemoryStateMount {
    fn open_sealed(&self, key: &str) -> Option<Vec<u8>> {
        self.sealed.get(key).cloned()
    }

    fn write_sealed(&mut self, key: &str, value: Vec<u8>) {
        self.sealed.insert(key.to_string(), value);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleRegistration {
    pub capsule_id: CapsuleId,
    pub wasm_module: String,
    pub update_methods: Vec<String>,
    pub query_methods: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CapsuleRegistry {
    entries: BTreeMap<CapsuleId, CapsuleRegistration>,
}

impl CapsuleRegistry {
    pub fn register(&mut self, registration: CapsuleRegistration) {
        self.entries
            .insert(registration.capsule_id.clone(), registration);
    }

    pub fn get(&self, capsule_id: &str) -> Option<&CapsuleRegistration> {
        self.entries.get(capsule_id)
    }
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
    // TODO: validate the capsule registry entry before execution.
    // TODO: instantiate the WASM module with deterministic host imports only.
    // TODO: expose audit, event, and quantum host calls through a narrow ABI.
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

pub fn execute_capsule_with_context(
    context: &RuntimeContext,
    req: Request,
) -> Result<Response, ExecutionError> {
    if let ExecutionMode::Replay { snapshot } = &context.execution_mode {
        context
            .state_store
            .restore(snapshot)
            .map_err(|error| ExecutionError::StateFailed(format!("{:?}", error)))?;
    }

    let state_key = format!("last_method_{}", req.method);
    if matches!(context.execution_mode, ExecutionMode::Normal) {
        context
            .state_store
            .set(&req.capsule_id, &state_key, req.payload.clone())
            .map_err(|error| ExecutionError::StateFailed(format!("{:?}", error)))?;
    }

    execute_capsule(req)
}
