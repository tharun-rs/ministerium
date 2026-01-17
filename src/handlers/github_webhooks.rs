use axum::{
    body::Bytes,
    http::{HeaderMap,StatusCode}
};
use crate::utils::crypto_utils;
use crate::service::github_webhook_service;


pub async fn github_webhook_handler(
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode  {
    // 1. Verify signature
    if !crypto_utils::verify_signature(&headers, &body) {
        return StatusCode::UNAUTHORIZED;
    }

    // 2. Spawn background task
    tokio::spawn(async move {
        github_webhook_service::process_webhook(body).await;
    });

    // 3. ACK immediately
    StatusCode::OK
}

