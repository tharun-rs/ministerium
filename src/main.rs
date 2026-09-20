mod executors;
mod database;
mod handlers;
mod processors;
mod utils;
mod models;

use axum::{
    Router, routing::{get, post}
};

use dotenv::dotenv;
use handlers::health_routes;
use handlers::github_webhooks;
use handlers::monitoring_routes;
use handlers::{control_routes, documentation_routes};
use std::env;


#[tokio::main]
async fn main() {
    //  Load .env variables
    dotenv().ok();
    let server_addr  = env::var("SERVER_ADDR").unwrap();
    let database = database::Database::from_environment().expect("failed to initialize deployment database");
    let app = Router::new()
        .route("/heartbeat", get(health_routes::heartbeat))
        .route("/github/webhook", post(github_webhooks::github_webhook_handler))
        .route("/api/deployments", get(monitoring_routes::deployments))
        .route("/api/deployments/{repository_name}", get(monitoring_routes::deployment))
        .route("/api/deployments/{repository_name}/versions", get(monitoring_routes::deployment_versions))
        .route("/api/deployments/{repository_name}/restart", post(control_routes::restart))
        .route("/api/deployments/{repository_name}/rollback", post(control_routes::rollback))
        .route("/api/metrics", get(monitoring_routes::metrics))
        .route("/openapi.json", get(documentation_routes::openapi))
        .route("/swagger", get(documentation_routes::swagger))
        .with_state(database);

    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
