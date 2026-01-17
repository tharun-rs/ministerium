

// Root model
#[derive(serde::Deserialize, Debug)]
pub struct WebhookPayload {
    action: Option<String>,
    repository: Option<Repository>,
}


#[derive(serde::Deserialize, Debug)]
struct Repository {
    name: String,
}
