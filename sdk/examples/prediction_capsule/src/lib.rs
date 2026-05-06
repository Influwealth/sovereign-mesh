use sovereign_mesh_sdk::{
    encode_text, Capsule, CapsuleIdentity, CapsuleRequest, CapsuleResponse,
};

pub struct PredictionCapsule;

impl Capsule for PredictionCapsule {
    fn identity(&self) -> CapsuleIdentity {
        CapsuleIdentity::new(
            "capsule.prediction.v1",
            "Prediction Capsule",
            "0.1.0",
        )
    }

    fn handle(&self, request: CapsuleRequest) -> CapsuleResponse {
        match request.method.as_str() {
            "predict" => predict(request.payload),
            "health" => CapsuleResponse::ok(encode_text("ok")),
            _ => CapsuleResponse::rejected("unsupported method"),
        }
    }
}

fn predict(payload: Vec<u8>) -> CapsuleResponse {
    let signal_strength = if payload.is_empty() { "low" } else { "medium" };
    let response = format!(
        "{{\"prediction\":\"hold\",\"confidence\":\"{}\",\"quantum_boundary\":\"mocked\"}}",
        signal_strength
    );

    CapsuleResponse::ok(response.into_bytes())
}

pub fn capsule_entry(method: &str, payload: Vec<u8>) -> CapsuleResponse {
    let capsule = PredictionCapsule;
    capsule.handle(CapsuleRequest::new(method, payload))
}
