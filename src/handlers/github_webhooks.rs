use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap,StatusCode}
};
use crate::database::Database;
use crate::utils::crypto_utils;
use crate::processors::github_webhook_processor;


pub async fn github_webhook_handler(
    State(database): State<Database>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode  {
    // 1. Verify signature
    if !crypto_utils::verify_signature(&headers, &body) {
        return StatusCode::UNAUTHORIZED;
    }

    // 2. Spawn background task
    tokio::spawn(async move {
        github_webhook_processor::process_webhook(body, headers, database).await;
    });

    // 3. ACK immediately
    StatusCode::OK
}
