use crate::execution::SAPMessage;
use crate::{CapsuleId, MethodName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepFlexAdapter {
    pub endpoint: String,
    pub resource_classes: Vec<DeepFlexResourceClass>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeepFlexResourceClass {
    Cpu,
    Gpu,
    Quantum,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepFlexRequest {
    pub endpoint: String,
    pub job_id: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub payload: Vec<u8>,
    pub resource_classes: Vec<DeepFlexResourceClass>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepFlexResponse {
    pub job_id: String,
    pub capsule_id: CapsuleId,
    pub method: MethodName,
    pub payload: Vec<u8>,
}

impl DeepFlexAdapter {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            resource_classes: vec![DeepFlexResourceClass::Cpu],
        }
    }

    pub fn with_resource_classes(mut self, resource_classes: Vec<DeepFlexResourceClass>) -> Self {
        self.resource_classes = resource_classes;
        self
    }

    pub fn sap_to_deepflex(&self, message: &SAPMessage) -> DeepFlexRequest {
        DeepFlexRequest {
            endpoint: self.endpoint.clone(),
            job_id: message.trace_id.clone(),
            capsule_id: message.capsule_id.clone(),
            method: message.method.clone(),
            payload: message.payload.clone(),
            resource_classes: self.resource_classes.clone(),
        }
    }

    pub fn deepflex_to_sap(&self, response: &DeepFlexResponse) -> SAPMessage {
        SAPMessage {
            trace_id: response.job_id.clone(),
            capsule_id: response.capsule_id.clone(),
            method: response.method.clone(),
            is_update: true,
            payload: response.payload.clone(),
        }
    }

    pub fn execute_mock(&self, message: &SAPMessage) -> DeepFlexResponse {
        let request = self.sap_to_deepflex(message);
        DeepFlexResponse {
            job_id: request.job_id,
            capsule_id: request.capsule_id,
            method: request.method,
            payload: b"deepflex_execution_result".to_vec(),
        }
    }
}

pub fn backend_is_deepflex(runtime: &str, backend: Option<&str>) -> bool {
    runtime.eq_ignore_ascii_case("deepflex")
        || backend
            .map(|value| value.eq_ignore_ascii_case("deepflex"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{backend_is_deepflex, DeepFlexAdapter, DeepFlexResourceClass};
    use crate::execution::SAPMessage;

    #[test]
    fn maps_sap_to_deepflex_request() {
        let adapter = DeepFlexAdapter::new("http://mock.deepflex")
            .with_resource_classes(vec![DeepFlexResourceClass::Cpu, DeepFlexResourceClass::Gpu]);
        let message = SAPMessage {
            trace_id: "trace-1".to_string(),
            capsule_id: "capsule.deepflex.v1".to_string(),
            method: "run".to_string(),
            is_update: true,
            payload: b"payload".to_vec(),
        };

        let request = adapter.sap_to_deepflex(&message);

        assert_eq!(request.endpoint, "http://mock.deepflex");
        assert_eq!(request.job_id, "trace-1");
        assert_eq!(request.resource_classes.len(), 2);
    }

    #[test]
    fn maps_deepflex_response_to_sap_message() {
        let adapter = DeepFlexAdapter::new("http://mock.deepflex");
        let message = SAPMessage {
            trace_id: "trace-2".to_string(),
            capsule_id: "capsule.deepflex.v1".to_string(),
            method: "run".to_string(),
            is_update: true,
            payload: b"payload".to_vec(),
        };
        let response = adapter.execute_mock(&message);
        let mapped = adapter.deepflex_to_sap(&response);

        assert_eq!(mapped.trace_id, "trace-2");
        assert_eq!(mapped.payload, b"deepflex_execution_result".to_vec());
    }

    #[test]
    fn detects_deepflex_backend() {
        assert!(backend_is_deepflex("deepflex", None));
        assert!(backend_is_deepflex("sovereign-mesh-runtime", Some("deepflex")));
        assert!(!backend_is_deepflex("sovereign-mesh-runtime", None));
    }
}
