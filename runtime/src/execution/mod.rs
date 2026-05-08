use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use crate::ingress::IngressMessage;
use crate::quantum::QuantumInference;
use crate::scheduler::ScheduleDecision;
use crate::state::InMemoryStateStore;
use crate::{
    CapsuleId, ExecutionError as RuntimeExecutionError, ExecutionMode, MethodName, Request,
    Response, RuntimeContext,
};

#[derive(Clone)]
pub struct ExecutionContext {
    pub schedule: ScheduleDecision,
    pub quantum_hint: Option<QuantumInference>,
    pub runtime: Option<RuntimeContext>,
}

impl ExecutionContext {
    pub fn new(schedule: ScheduleDecision, quantum_hint: Option<QuantumInference>) -> Self {
        Self {
            schedule,
            quantum_hint,
            runtime: None,
        }
    }

    pub fn with_runtime(mut self, runtime: RuntimeContext) -> Self {
        self.runtime = Some(runtime);
        self
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SAPMessage {
    pub trace_id: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub is_update: bool,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub trace_id: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub payload: Vec<u8>,
    pub events: Vec<CapsuleExecutionEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleExecutionEvent {
    pub name: String,
    pub payload: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    CapsuleNotFound(CapsuleId),
    MethodNotFound(MethodName),
    State(String),
    PanicIsolated,
    InvalidMessage(String),
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

pub fn route_sap_message(
    registry: &CapsuleRegistry,
    message: &SAPMessage,
) -> Result<CapsuleRegistration, ExecutionError> {
    let registration = registry
        .get(&message.capsule_id)
        .ok_or_else(|| ExecutionError::CapsuleNotFound(message.capsule_id.clone()))?;

    let allowed = if message.is_update {
        registration
            .update_methods
            .iter()
            .any(|method| method == &message.method)
    } else {
        registration
            .query_methods
            .iter()
            .any(|method| method == &message.method)
    };

    if !allowed {
        return Err(ExecutionError::MethodNotFound(message.method.clone()));
    }

    Ok(registration.clone())
}

pub fn execute_capsule_call(
    context: &RuntimeContext,
    registry: &CapsuleRegistry,
    schedule: &ScheduleDecision,
    message: SAPMessage,
) -> Result<ExecutionResult, ExecutionError> {
    execution_started(&message.capsule_id, &schedule.lane, &context.node_id);
    let routed = route_sap_message(registry, &message)?;
    let fail_capsule_id = message.capsule_id.clone();
    let trace_id = message.trace_id.clone();
    let method = message.method.clone();
    let payload = message.payload.clone();
    let is_update = message.is_update;

    let result = catch_unwind(AssertUnwindSafe(|| {
        if let ExecutionMode::Replay { snapshot } = &context.execution_mode {
            context
                .state_store
                .restore(snapshot)
                .map_err(|error| ExecutionError::State(format!("{:?}", error)))?;
        }

        if is_update {
            let state_key = format!("{}:{}", routed.capsule_id, method);
            context
                .state_store
                .set(&routed.capsule_id, &state_key, payload)
                .map_err(|error| ExecutionError::State(format!("{:?}", error)))?;
        }

        let event = CapsuleExecutionEvent {
            name: format!("{}.{}", routed.capsule_id, method),
            payload: format!("lane={} node_id={}", schedule.lane, context.node_id),
        };

        Ok(ExecutionResult {
            trace_id,
            capsule_id: routed.capsule_id,
            method,
            payload: b"capsule_execution_result".to_vec(),
            events: vec![event],
        })
    }));

    match result {
        Ok(Ok(execution_result)) => {
            execution_completed(&execution_result.capsule_id, &schedule.lane, &context.node_id);
            Ok(execution_result)
        }
        Ok(Err(error)) => {
            execution_failed(&fail_capsule_id, &schedule.lane, &context.node_id);
            Err(error)
        }
        Err(_) => {
            execution_failed(&fail_capsule_id, &schedule.lane, &context.node_id);
            Err(ExecutionError::PanicIsolated)
        }
    }
}

pub fn execute_capsule(req: Request) -> Result<Response, RuntimeExecutionError> {
    // TODO: load the capsule WASM module declared by capsule.toml.
    // TODO: validate the capsule registry entry before execution.
    // TODO: instantiate the WASM module with deterministic host imports only.
    // TODO: expose audit, event, and quantum host calls through a narrow ABI.
    // TODO: mount stable, ephemeral, and sealed state before invocation.
    // TODO: dispatch to generated update/query methods through the SDK ABI.
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
) -> Result<Response, RuntimeExecutionError> {
    if let ExecutionMode::Replay { snapshot } = &context.execution_mode {
        context
            .state_store
            .restore(snapshot)
            .map_err(|error| RuntimeExecutionError::StateFailed(format!("{:?}", error)))?;
    }

    let state_key = format!("last_method_{}", req.method);
    if matches!(context.execution_mode, ExecutionMode::Normal) {
        context
            .state_store
            .set(&req.capsule_id, &state_key, req.payload.clone())
            .map_err(|error| RuntimeExecutionError::StateFailed(format!("{:?}", error)))?;
    }

    execute_capsule(req)
}

pub fn execution_started(capsule_id: &str, lane: &str, node_id: &str) {
    crate::metrics::execution_started(capsule_id);
    println!(
        "sovereign-mesh.execution_started capsule_id={} lane={} node_id={}",
        capsule_id, lane, node_id
    );
}

pub fn execution_completed(capsule_id: &str, lane: &str, node_id: &str) {
    crate::metrics::execution_completed(capsule_id);
    crate::metrics::global_metrics().observe_execution_duration(capsule_id, 0.0);
    println!(
        "sovereign-mesh.execution_completed capsule_id={} lane={} node_id={}",
        capsule_id, lane, node_id
    );
}

pub fn execution_failed(capsule_id: &str, lane: &str, node_id: &str) {
    crate::metrics::execution_failed(capsule_id);
    println!(
        "sovereign-mesh.execution_failed capsule_id={} lane={} node_id={}",
        capsule_id, lane, node_id
    );
}

#[cfg(test)]
mod tests {
    use super::{
        execute_capsule_call, CapsuleRegistry, CapsuleRegistration, SAPMessage,
    };
    use crate::scheduler::{ExecutionPriority, ScheduleDecision};
    use crate::{InMemoryStateStore, RuntimeContext};
    use std::sync::Arc;

    #[test]
    fn sap_update_routes_to_capsule_state_and_events() {
        let mut registry = CapsuleRegistry::default();
        registry.register(CapsuleRegistration {
            capsule_id: "capsule.test.v1".to_string(),
            wasm_module: "test.wasm".to_string(),
            update_methods: vec!["set".to_string()],
            query_methods: vec!["get".to_string()],
        });
        let context = RuntimeContext::new("node-test", Arc::new(InMemoryStateStore::new()));
        let schedule = ScheduleDecision {
            priority: ExecutionPriority::Normal,
            lane: "standard".to_string(),
            edge_eligible: false,
        };
        let message = SAPMessage {
            trace_id: "trace-1".to_string(),
            capsule_id: "capsule.test.v1".to_string(),
            method: "set".to_string(),
            is_update: true,
            payload: b"value".to_vec(),
        };

        let result = execute_capsule_call(&context, &registry, &schedule, message)
            .expect("execution should succeed");

        assert_eq!(result.capsule_id, "capsule.test.v1");
        assert_eq!(result.events.len(), 1);
        assert!(context
            .state_store
            .get(&"capsule.test.v1".to_string(), "capsule.test.v1:set")
            .unwrap()
            .is_some());
    }
}
