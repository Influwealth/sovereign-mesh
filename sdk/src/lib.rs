use std::collections::BTreeMap;

pub use sovereign_mesh_sdk_macros::{capsule, query, update};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
}

impl CapsuleIdentity {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleRequest {
    pub method: String,
    pub payload: Vec<u8>,
}

impl CapsuleRequest {
    pub fn new(method: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            method: method.into(),
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleResponse {
    pub status: CapsuleStatus,
    pub payload: Vec<u8>,
}

impl CapsuleResponse {
    pub fn ok(payload: impl Into<Vec<u8>>) -> Self {
        Self {
            status: CapsuleStatus::Ok,
            payload: payload.into(),
        }
    }

    pub fn rejected(message: impl Into<String>) -> Self {
        Self {
            status: CapsuleStatus::Rejected,
            payload: message.into().into_bytes(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapsuleStatus {
    Ok,
    Rejected,
    Failed,
}

pub trait Capsule {
    fn identity(&self) -> CapsuleIdentity;
    fn handle(&self, request: CapsuleRequest) -> CapsuleResponse;
}

pub fn encode_text(value: &str) -> Vec<u8> {
    value.as_bytes().to_vec()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleContext {
    pub caller: String,
    pub caller_class: CallerClass,
    pub logical_time: u64,
    pub trace_id: String,
}

impl CapsuleContext {
    pub fn new(
        caller: impl Into<String>,
        caller_class: CallerClass,
        logical_time: u64,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            caller: caller.into(),
            caller_class,
            logical_time,
            trace_id: trace_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallerClass {
    Agent,
    Partner,
    SovereignUser,
}

pub trait UpdateMethod {
    async fn call(&mut self, context: CapsuleContext, payload: Vec<u8>) -> CapsuleResponse;
}

pub trait QueryMethod {
    fn call(&self, context: CapsuleContext, payload: Vec<u8>) -> CapsuleResponse;
}

pub trait StableState {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: Vec<u8>);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyValueStableState {
    entries: BTreeMap<String, Vec<u8>>,
}

impl KeyValueStableState {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl StableState for KeyValueStableState {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.entries.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: Vec<u8>) {
        self.entries.insert(key.to_string(), value);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EphemeralState {
    entries: BTreeMap<String, String>,
}

impl EphemeralState {
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub trait SealedState {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, SealedStateError>;
    fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, SealedStateError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedStateError {
    pub message: String,
}

impl SealedStateError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NoopSealedState;

impl SealedState for NoopSealedState {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, SealedStateError> {
        Ok(plaintext.to_vec())
    }

    fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, SealedStateError> {
        Ok(sealed.to_vec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleEvent {
    pub name: String,
    pub payload: String,
}

pub fn emit_event(name: &str, payload: &str) -> CapsuleEvent {
    CapsuleEvent {
        name: name.to_string(),
        payload: payload.to_string(),
    }
}

pub mod quantum {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct QuantumError {
        pub message: String,
    }

    impl QuantumError {
        pub fn new(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    pub async fn run_job(params: &str) -> Result<String, QuantumError> {
        if params.trim().is_empty() {
            return Err(QuantumError::new("quantum job params cannot be empty"));
        }

        Ok(format!("mock_quantum_result:{}", params))
    }
}
