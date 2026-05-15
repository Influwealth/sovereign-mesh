use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use pqc_email_shield as pqc;

fn main() -> std::io::Result<()> {
    let addr = std::env::var("PQCAAS_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let listener = TcpListener::bind(addr)?;
    eprintln!("pqcaas listening");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let _ = handle_connection(stream);
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = vec![0_u8; 64 * 1024];
    let read = stream.read(&mut buffer)?;
    buffer.truncate(read);
    let request = String::from_utf8_lossy(&buffer);
    let (method, path) = parse_request_line(&request).unwrap_or(("GET", "/"));

    if method != "POST" {
        return write_json(&mut stream, 405, "{\"error\":\"method_not_allowed\"}");
    }

    let body = extract_body(&request);
    match path {
        "/encrypt" => {
            let plaintext = json_bytes_field(body, "plaintext").unwrap_or_default();
            let out = pqc::hybrid_encrypt(b"recipient_pubkey", &plaintext)
                .map(|ct| frame_ciphertext(ct))
                .unwrap_or_else(|e| json_error(&e.message));
            write_json(&mut stream, 200, &out)
        }
        "/decrypt" => {
            let framed = json_bytes_field(body, "ciphertext").unwrap_or_default();
            let out = parse_framed_ciphertext(&framed)
                .and_then(|ct| pqc::hybrid_decrypt(b"recipient_privkey", &ct))
                .map(|pt| format!("{{\"plaintext\":{}}}", json_base64(&pt)))
                .unwrap_or_else(|e| json_error(&e.message));
            write_json(&mut stream, 200, &out)
        }
        "/sign" => {
            let message = json_bytes_field(body, "message").unwrap_or_default();
            let out = pqc::hybrid_sign(b"signing_privkey", &message)
                .map(|sig| format!("{{\"signature\":{}}}", json_base64(&sig.sig)))
                .unwrap_or_else(|e| json_error(&e.message));
            write_json(&mut stream, 200, &out)
        }
        "/verify" => {
            let message = json_bytes_field(body, "message").unwrap_or_default();
            let signature = json_bytes_field(body, "signature").unwrap_or_default();
            let sig = pqc::HybridSignature { sig: signature };
            let out = pqc::hybrid_verify(b"signing_pubkey", &message, &sig)
                .map(|ok| format!("{{\"ok\":{}}}", if ok { "true" } else { "false" }))
                .unwrap_or_else(|e| json_error(&e.message));
            write_json(&mut stream, 200, &out)
        }
        _ => write_json(&mut stream, 404, "{\"error\":\"not_found\"}"),
    }
}

fn parse_request_line(request: &str) -> Option<(&str, &str)> {
    let line = request.lines().next()?;
    let mut parts = line.split_whitespace();
    Some((parts.next()?, parts.next()?))
}

fn extract_body(request: &str) -> &str {
    request.split("\r\n\r\n").nth(1).unwrap_or("")
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {} OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
        status,
        body.as_bytes().len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn json_error(message: &str) -> String {
    format!("{{\"error\":{}}}", json_string(message))
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

// Minimal base64 (no deps): hex encoding instead, labeled as base64 for placeholder compatibility.
fn json_base64(bytes: &[u8]) -> String {
    format!("\"{}\"", to_hex(bytes))
}

fn json_bytes_field(body: &str, field: &str) -> Option<Vec<u8>> {
    let needle = format!("\"{}\":", field);
    let index = body.find(&needle)? + needle.len();
    let rest = body[index..].trim_start();
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')? + start;
    from_hex(&rest[start..end]).ok()
}

fn frame_ciphertext(ct: pqc::HybridCiphertext) -> String {
    let mut framed = Vec::new();
    framed.extend_from_slice(&ct.kem_ciphertext);
    framed.push(0);
    framed.extend_from_slice(&ct.payload_ciphertext);
    format!("{{\"ciphertext\":{}}}", json_base64(&framed))
}

fn parse_framed_ciphertext(framed: &[u8]) -> Result<pqc::HybridCiphertext, pqc::PqcError> {
    let Some(split) = framed.iter().position(|b| *b == 0) else {
        return Err(pqc::PqcError::new("invalid ciphertext frame"));
    };
    Ok(pqc::HybridCiphertext {
        kem_ciphertext: framed[..split].to_vec(),
        payload_ciphertext: framed[split + 1..].to_vec(),
    })
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn from_hex(hex: &str) -> Result<Vec<u8>, ()> {
    let mut bytes = Vec::new();
    let mut chars = hex.as_bytes().iter().cloned();
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        let h = from_hex_nibble(hi)?;
        let l = from_hex_nibble(lo)?;
        bytes.push((h << 4) | l);
    }
    Ok(bytes)
}

fn from_hex_nibble(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(()),
    }
}

