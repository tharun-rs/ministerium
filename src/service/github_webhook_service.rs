use axum::body::Bytes;
use crate::models::webhook_payload;

pub async fn process_webhook(body: Bytes) {
    // Parse body
    let payload: webhook_payload::WebhookPayload = 
        match serde_json::from_slice(&body){
            Ok(p) => p,
            Err(_) => return,
        };
    
    print!("{:?}", payload);
}