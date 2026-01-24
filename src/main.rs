mod executors;
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
use std::env;


#[tokio::main]
async fn main() {
    //  Load .env variables
    dotenv().ok();
    let server_addr  = env::var("SERVER_ADDR").unwrap();
    let app = Router::new()
        .route("/heartbeat", get(health_routes::heartbeat))
        .route("/github/webhook", post(github_webhooks::github_webhook_handler));

    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
