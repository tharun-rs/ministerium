use axum::{
    body::Bytes, 
    http::HeaderMap
};
use crate::{
    models::webhook_payload::{
        Event,
        WebhookPayload
    }, 
    processors::main_branch_push_processor,
    utils::git_utils::extract_event
};

pub async fn process_webhook(body: Bytes, headers: HeaderMap) {
    // 1. Extract event type
    let event: Option<Event> = extract_event(&headers);

    // 2. Parse body
    let payload: WebhookPayload = 
        match serde_json::from_slice(&body){
            Ok(p) => p,
            Err(_) => return,
        };
    
    // 3. Call respective processors
    match event {
        Some(Event::Push) => {
            // Push events always have `ref`
            if let Some(git_ref) = payload.git_ref.as_deref() {
                // Only act on main branch
                if git_ref == "refs/heads/main" {
                    if let Err(err) = main_branch_push_processor::process(payload).await {
                        eprintln!("Pipeline failed: {}",err);
                    };
                }
            }
        }

        Some(Event::PullRequest) => {
            // TODO: handle pull request events
        }

        None => {
            // Unknown or unsupported event
        }
    }
    
}
