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
