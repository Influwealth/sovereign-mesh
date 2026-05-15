use crate::ExecutionError;
use pqc_email_shield as pqc;

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

pub async fn run_quantum_job(params: &str) -> Result<String, ExecutionError> {
    // TODO: integrate real backend adapters without binding the runtime to one vendor.
    // TODO: account for latency budgets and deterministic replay requirements.
    // TODO: anonymize capsule payloads before remote quantum or hybrid inference.
    if params.trim().is_empty() {
        return Err(ExecutionError::QuantumFailed(
            "quantum job params cannot be empty".to_string(),
        ));
    }

    Ok("quantum_result".to_string())
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

pub fn pqc_encrypt(recipient_pubkey: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, ExecutionError> {
    let ct = pqc::hybrid_encrypt(recipient_pubkey, plaintext)
        .map_err(|e| ExecutionError::QuantumFailed(e.message))?;
    // Simple framing: kem || 0x00 || payload
    let mut out = Vec::new();
    out.extend_from_slice(&ct.kem_ciphertext);
    out.push(0);
    out.extend_from_slice(&ct.payload_ciphertext);
    Ok(out)
}

pub fn pqc_decrypt(recipient_privkey: &[u8], framed: &[u8]) -> Result<Vec<u8>, ExecutionError> {
    let Some(split) = framed.iter().position(|b| *b == 0) else {
        return Err(ExecutionError::QuantumFailed("invalid pqc frame".to_string()));
    };
    let ct = pqc::HybridCiphertext {
        kem_ciphertext: framed[..split].to_vec(),
        payload_ciphertext: framed[split + 1..].to_vec(),
    };
    pqc::hybrid_decrypt(recipient_privkey, &ct)
        .map_err(|e| ExecutionError::QuantumFailed(e.message))
}

pub fn pqc_sign(signing_privkey: &[u8], message: &[u8]) -> Result<Vec<u8>, ExecutionError> {
    let sig = pqc::hybrid_sign(signing_privkey, message)
        .map_err(|e| ExecutionError::QuantumFailed(e.message))?;
    Ok(sig.sig)
}

pub fn pqc_verify(signing_pubkey: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, ExecutionError> {
    let sig = pqc::HybridSignature { sig: signature.to_vec() };
    pqc::hybrid_verify(signing_pubkey, message, &sig)
        .map_err(|e| ExecutionError::QuantumFailed(e.message))
}

#[cfg(test)]
mod tests {
    use super::{MockQuantumBoundary, QuantumBoundary};

    #[test]
    fn mock_quantum_boundary_returns_deterministic_score() {
        let boundary = MockQuantumBoundary {
            deterministic_score: 0.7,
        };

        let inference = boundary.infer("capsule-execution-readiness", b"payload");

        assert_eq!(inference.model, "capsule-execution-readiness");
        assert!((inference.score - 0.71).abs() < f64::EPSILON);
        assert_eq!(inference.explanation, "mocked deterministic quantum boundary");
    }
}
