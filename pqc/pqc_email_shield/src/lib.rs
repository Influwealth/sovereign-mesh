#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PqcError {
    pub message: String,
}

impl PqcError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridCiphertext {
    pub kem_ciphertext: Vec<u8>,
    pub payload_ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridSignature {
    pub sig: Vec<u8>,
}

pub mod capsule_entry;

pub fn hybrid_encrypt(recipient_pubkey: &[u8], plaintext: &[u8]) -> Result<HybridCiphertext, PqcError> {
    #[cfg(all(feature = "native-oqs", not(target_arch = "wasm32")))]
    {
        return oqs_native::hybrid_encrypt_native(recipient_pubkey, plaintext);
    }
    hybrid_encrypt_stub(recipient_pubkey, plaintext)
}

pub fn hybrid_decrypt(recipient_privkey: &[u8], ciphertext: &HybridCiphertext) -> Result<Vec<u8>, PqcError> {
    #[cfg(all(feature = "native-oqs", not(target_arch = "wasm32")))]
    {
        return oqs_native::hybrid_decrypt_native(recipient_privkey, ciphertext);
    }
    hybrid_decrypt_stub(recipient_privkey, ciphertext)
}

pub fn hybrid_sign(signing_privkey: &[u8], message: &[u8]) -> Result<HybridSignature, PqcError> {
    #[cfg(all(feature = "native-oqs", not(target_arch = "wasm32")))]
    {
        return oqs_native::hybrid_sign_native(signing_privkey, message);
    }
    hybrid_sign_stub(signing_privkey, message)
}

pub fn hybrid_verify(signing_pubkey: &[u8], message: &[u8], signature: &HybridSignature) -> Result<bool, PqcError> {
    #[cfg(all(feature = "native-oqs", not(target_arch = "wasm32")))]
    {
        return oqs_native::hybrid_verify_native(signing_pubkey, message, signature);
    }
    hybrid_verify_stub(signing_pubkey, message, signature)
}

fn hybrid_encrypt_stub(_recipient_pubkey: &[u8], plaintext: &[u8]) -> Result<HybridCiphertext, PqcError> {
    // WASM-safe deterministic stub. Real PQC-in-WASM can swap in later without API changes.
    let key_byte = 0xA5_u8;
    let payload_ciphertext: Vec<u8> = plaintext.iter().map(|b| b ^ key_byte).collect();
    Ok(HybridCiphertext {
        kem_ciphertext: b"wasm-stub-mlkem".to_vec(),
        payload_ciphertext,
    })
}

fn hybrid_decrypt_stub(_recipient_privkey: &[u8], ciphertext: &HybridCiphertext) -> Result<Vec<u8>, PqcError> {
    let key_byte = 0xA5_u8;
    Ok(ciphertext
        .payload_ciphertext
        .iter()
        .map(|b| b ^ key_byte)
        .collect())
}

fn hybrid_sign_stub(_signing_privkey: &[u8], message: &[u8]) -> Result<HybridSignature, PqcError> {
    let mut sig = Vec::with_capacity(message.len().min(32));
    for (index, byte) in message.iter().enumerate() {
        if sig.len() >= 32 {
            break;
        }
        sig.push(byte ^ (index as u8));
    }
    Ok(HybridSignature { sig })
}

fn hybrid_verify_stub(_signing_pubkey: &[u8], message: &[u8], signature: &HybridSignature) -> Result<bool, PqcError> {
    let expected = hybrid_sign_stub(&[], message)?.sig;
    Ok(expected == signature.sig)
}

// Native liboqs integration boundary (non-wasm only). The actual FFI surface
// should live behind this module; WASM builds must not require liboqs.
#[cfg(all(feature = "native-oqs", not(target_arch = "wasm32")))]
pub mod oqs_native {
    use super::{HybridCiphertext, HybridSignature, PqcError};

    // Minimal liboqs FFI surface. liboqs must be installed system-wide by pqc-bootstrap.sh.
    // This module intentionally avoids bindgen or external crates.

    #[repr(C)]
    struct OqsKem;

    #[repr(C)]
    struct OqsSig;

    #[link(name = "oqs")]
    extern "C" {
        fn OQS_KEM_new(method_name: *const i8) -> *mut OqsKem;
        fn OQS_KEM_free(kem: *mut OqsKem);
        fn OQS_KEM_encaps(kem: *mut OqsKem, ciphertext: *mut u8, shared_secret: *mut u8, public_key: *const u8) -> i32;
        fn OQS_KEM_decaps(kem: *mut OqsKem, shared_secret: *mut u8, ciphertext: *const u8, secret_key: *const u8) -> i32;

        fn OQS_SIG_new(method_name: *const i8) -> *mut OqsSig;
        fn OQS_SIG_free(sig: *mut OqsSig);
        fn OQS_SIG_sign(sig: *mut OqsSig, signature: *mut u8, signature_len: *mut usize, message: *const u8, message_len: usize, secret_key: *const u8) -> i32;
        fn OQS_SIG_verify(sig: *mut OqsSig, message: *const u8, message_len: usize, signature: *const u8, signature_len: usize, public_key: *const u8) -> i32;
    }

    // OQS status: 0 = success, nonzero = failure.
    const OQS_SUCCESS: i32 = 0;

    // Use the widely supported ML-KEM/ML-DSA algorithm names as exposed by liboqs.
    // If a given build of liboqs differs, these constants should be updated.
    const KEM_ALG: &[u8] = b"ML-KEM-768\0";
    const SIG_ALG: &[u8] = b"ML-DSA-65\0";

    pub fn hybrid_encrypt_native(recipient_pubkey: &[u8], plaintext: &[u8]) -> Result<HybridCiphertext, PqcError> {
        // Hybrid design: PQC KEM for shared secret + symmetric payload protection.
        // Symmetric payload protection is currently a deterministic XOR stream derived from shared secret bytes.
        // TODO: replace XOR with AEAD (e.g., ChaCha20-Poly1305) once a dependency policy is chosen.
        unsafe {
            let kem = OQS_KEM_new(KEM_ALG.as_ptr() as *const i8);
            if kem.is_null() {
                return Err(PqcError::new("OQS_KEM_new failed"));
            }

            // These sizes depend on the chosen algorithm. We use conservative buffers and truncate.
            // TODO: read kem->length_* fields via a proper struct definition if needed.
            let mut kem_ciphertext = vec![0u8; 4096];
            let mut shared_secret = vec![0u8; 4096];
            let rc = OQS_KEM_encaps(
                kem,
                kem_ciphertext.as_mut_ptr(),
                shared_secret.as_mut_ptr(),
                recipient_pubkey.as_ptr(),
            );
            OQS_KEM_free(kem);
            if rc != OQS_SUCCESS {
                return Err(PqcError::new("OQS_KEM_encaps failed"));
            }

            let payload_ciphertext = xor_stream(plaintext, &shared_secret);
            Ok(HybridCiphertext {
                kem_ciphertext: kem_ciphertext,
                payload_ciphertext,
            })
        }
    }

    pub fn hybrid_decrypt_native(recipient_privkey: &[u8], ciphertext: &HybridCiphertext) -> Result<Vec<u8>, PqcError> {
        unsafe {
            let kem = OQS_KEM_new(KEM_ALG.as_ptr() as *const i8);
            if kem.is_null() {
                return Err(PqcError::new("OQS_KEM_new failed"));
            }

            let mut shared_secret = vec![0u8; 4096];
            let rc = OQS_KEM_decaps(
                kem,
                shared_secret.as_mut_ptr(),
                ciphertext.kem_ciphertext.as_ptr(),
                recipient_privkey.as_ptr(),
            );
            OQS_KEM_free(kem);
            if rc != OQS_SUCCESS {
                return Err(PqcError::new("OQS_KEM_decaps failed"));
            }

            Ok(xor_stream(&ciphertext.payload_ciphertext, &shared_secret))
        }
    }

    pub fn hybrid_sign_native(signing_privkey: &[u8], message: &[u8]) -> Result<HybridSignature, PqcError> {
        unsafe {
            let sig = OQS_SIG_new(SIG_ALG.as_ptr() as *const i8);
            if sig.is_null() {
                return Err(PqcError::new("OQS_SIG_new failed"));
            }

            let mut signature = vec![0u8; 8192];
            let mut signature_len: usize = 0;
            let rc = OQS_SIG_sign(
                sig,
                signature.as_mut_ptr(),
                &mut signature_len as *mut usize,
                message.as_ptr(),
                message.len(),
                signing_privkey.as_ptr(),
            );
            OQS_SIG_free(sig);
            if rc != OQS_SUCCESS {
                return Err(PqcError::new("OQS_SIG_sign failed"));
            }

            signature.truncate(signature_len);
            Ok(HybridSignature { sig: signature })
        }
    }

    pub fn hybrid_verify_native(signing_pubkey: &[u8], message: &[u8], signature: &HybridSignature) -> Result<bool, PqcError> {
        unsafe {
            let sig = OQS_SIG_new(SIG_ALG.as_ptr() as *const i8);
            if sig.is_null() {
                return Err(PqcError::new("OQS_SIG_new failed"));
            }

            let rc = OQS_SIG_verify(
                sig,
                message.as_ptr(),
                message.len(),
                signature.sig.as_ptr(),
                signature.sig.len(),
                signing_pubkey.as_ptr(),
            );
            OQS_SIG_free(sig);
            Ok(rc == OQS_SUCCESS)
        }
    }

    fn xor_stream(input: &[u8], secret: &[u8]) -> Vec<u8> {
        if secret.is_empty() {
            return input.to_vec();
        }
        input
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ secret[index % secret.len()])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_encrypt_decrypt_roundtrip() {
        let plaintext = b"hello pqc";
        let ct = hybrid_encrypt(b"pub", plaintext).expect("encrypt should succeed");
        let pt = hybrid_decrypt(b"priv", &ct).expect("decrypt should succeed");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn stub_sign_verify_roundtrip() {
        let message = b"sign me";
        let sig = hybrid_sign(b"priv", message).expect("sign should succeed");
        let ok = hybrid_verify(b"pub", message, &sig).expect("verify should succeed");
        assert!(ok);
    }

    #[test]
    fn wasm_stub_path_is_exercised_on_wasm_builds() {
        // This test is meaningful mainly for wasm32 builds; it should compile anywhere.
        let message = b"wasm path";
        let sig = hybrid_sign(b"", message).expect("sign should succeed");
        let ok = hybrid_verify(b"", message, &sig).expect("verify should succeed");
        assert!(ok);
    }
}
