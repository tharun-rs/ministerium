use axum::{
    body::Bytes,
    http::HeaderMap
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

fn github_secret() -> String {
    env::var("GITHUB_WEBHOOK_SECRET")
        .expect("GITHUB_WEBHOOK_SECRET not set")
}


pub fn verify_signature(headers: &HeaderMap, body: &Bytes) -> bool {
    // 1. Get signature header
    let signature = match headers.get("X-Hub-Signature-256") {
        Some(sig) => sig,
        None => return false,
    };

    let signature = match signature.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // 2. Signature must start with "sha256="
    let signature = match signature.strip_prefix("sha256=") {
        Some(s) => s,
        None => return false,
    };

    // 3. Create HMAC instance with secret
    let secret = github_secret();
    let mut mac: HmacSha256 = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    // 4. Feed raw body bytes
    mac.update(body);

    // 5. Decode GitHub's hex signature
    let expected = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // 6. Constant-time comparison
    mac.verify_slice(&expected).is_ok()
}