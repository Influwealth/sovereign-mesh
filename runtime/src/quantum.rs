#[derive(Clone, Debug, PartialEq)]
pub struct QuantumInference {
    pub model: String,
    pub score: f64,
    pub explanation: String,
}

pub trait QuantumBoundary {
    fn infer(&self, model: &str, payload: &[u8]) -> QuantumInference;
}

#[derive(Clone, Debug)]
pub struct MockQuantumBoundary {
    pub deterministic_score: f64,
}

impl Default for MockQuantumBoundary {
    fn default() -> Self {
        Self {
            deterministic_score: 0.5,
        }
    }
}

impl QuantumBoundary for MockQuantumBoundary {
    fn infer(&self, model: &str, payload: &[u8]) -> QuantumInference {
        let payload_factor = if payload.is_empty() { 0.0 } else { 0.01 };
        let score = (self.deterministic_score + payload_factor).min(1.0);

        QuantumInference {
            model: model.to_string(),
            score,
            explanation: "mocked deterministic quantum boundary".to_string(),
        }
    }
}
