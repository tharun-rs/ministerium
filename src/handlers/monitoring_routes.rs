use crate::{database::{Database, Deployment}, utils::metrics_utils::{system_metrics, SystemMetrics}};
use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::env;

pub async fn deployments(State(database): State<Database>) -> Result<Json<Vec<Deployment>>, StatusCode> {
    database.list_deployments().await.map(Json).map_err(|error| {
        eprintln!("failed to list deployments: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn deployment(
    State(database): State<Database>,
    Path(repository_name): Path<String>,
) -> Result<Json<Deployment>, StatusCode> {
    match database.deployment(repository_name).await {
        Ok(Some(deployment)) => Ok(Json(deployment)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(error) => {
            eprintln!("failed to read deployment: {error}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn deployment_versions(
    State(database): State<Database>,
    Path(repository_name): Path<String>,
) -> Result<Json<Vec<Deployment>>, StatusCode> {
    database.deployment_versions(repository_name).await.map(Json).map_err(|error| {
        eprintln!("failed to list deployment history: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn metrics() -> Json<SystemMetrics> {
    let repository_root = env::var("GITHUB_ROOT_FOLDER").unwrap_or_else(|_| ".".to_string());
    Json(system_metrics(&repository_root))
}
