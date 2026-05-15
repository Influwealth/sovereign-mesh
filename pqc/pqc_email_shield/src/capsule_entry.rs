use crate::{hybrid_decrypt, hybrid_encrypt, hybrid_sign, hybrid_verify, HybridCiphertext, HybridSignature, PqcError};

pub fn capsule_update_entry(method: &str, payload: Vec<u8>) -> Result<Vec<u8>, PqcError> {
    match method {
        "encrypt" => {
            let ct = hybrid_encrypt(b"recipient_pubkey", &payload)?;
            Ok(frame_ciphertext(ct))
        }
        "decrypt" => {
            let ct = parse_framed_ciphertext(&payload)?;
            hybrid_decrypt(b"recipient_privkey", &ct)
        }
        "sign" => {
            let sig = hybrid_sign(b"signing_privkey", &payload)?;
            Ok(sig.sig)
        }
        "verify" => {
            let ok = hybrid_verify(b"signing_pubkey", &payload, &HybridSignature { sig: payload.clone() })?;
            Ok(ok.to_string().into_bytes())
        }
        _ => Err(PqcError::new("unsupported method")),
    }
}

fn frame_ciphertext(ct: HybridCiphertext) -> Vec<u8> {
    let mut framed = Vec::new();
    framed.extend_from_slice(&ct.kem_ciphertext);
    framed.push(0);
    framed.extend_from_slice(&ct.payload_ciphertext);
    framed
}

fn parse_framed_ciphertext(framed: &[u8]) -> Result<HybridCiphertext, PqcError> {
    let Some(split) = framed.iter().position(|b| *b == 0) else {
        return Err(PqcError::new("invalid ciphertext frame"));
    };
    Ok(HybridCiphertext {
        kem_ciphertext: framed[..split].to_vec(),
        payload_ciphertext: framed[split + 1..].to_vec(),
    })
}

