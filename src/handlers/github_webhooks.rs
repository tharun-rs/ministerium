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
    if let Ok(body_str) = std::str::from_utf8(&body) {
        println!("GitHub webhook payload:\n{}", body_str);
    } else {
        println!("GitHub webhook payload: <non-utf8 body>");
    }

    if let Some(event) = headers.get("X-GitHub-Event") {
        println!("GitHub event: {:?}", event);
    }

    if let Some(delivery) = headers.get("X-GitHub-Delivery") {
        println!("GitHub delivery ID: {:?}", delivery);
    }
    
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

