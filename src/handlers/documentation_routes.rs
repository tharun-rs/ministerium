use axum::{response::Html, Json};
use serde_json::{json, Value};

pub async fn openapi() -> Json<Value> {
    Json(json!({
        "openapi": "3.0.3", "info": {"title": "Ministerium API", "version": "1.0.0"},
        "paths": {
            "/heartbeat": {"get": {"summary": "Health check"}},
            "/github/webhook": {"post": {"summary": "GitHub webhook receiver", "description": "Requires GitHub HMAC signature."}},
            "/api/deployments": {"get": {"summary": "List deployment inventory"}},
            "/api/deployments/{repository_name}": {"get": {"summary": "Get a deployment", "parameters": [{"name": "repository_name", "in": "path", "required": true, "schema": {"type": "string"}}]}},
            "/api/deployments/{repository_name}/versions": {"get": {"summary": "List immutable image versions"}},
            "/api/deployments/{repository_name}/restart": {"post": {"summary": "Restart a deployed container", "security": [{"bearerAuth": []}]}},
            "/api/deployments/{repository_name}/rollback": {"post": {"summary": "Roll back to the previous image, or ?image_tag= for a selected version", "security": [{"bearerAuth": []}]}},
            "/api/metrics": {"get": {"summary": "Get Raspberry Pi host metrics"}}
        },
        "components": {"securitySchemes": {"bearerAuth": {"type": "http", "scheme": "bearer"}}}
    }))
}

pub async fn swagger() -> Html<&'static str> {
    Html(r#"<!doctype html><html><head><title>Ministerium API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>SwaggerUIBundle({url:'/openapi.json',dom_id:'#swagger-ui',persistAuthorization:true});</script></body></html>"#)
}
