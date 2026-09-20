use crate::{database::{Database, Deployment}, executors::{docker, nginx::expose_app}};
use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use std::env;

pub async fn restart(
    State(database): State<Database>, headers: HeaderMap, Path(repository_name): Path<String>,
) -> Result<Json<Deployment>, StatusCode> {
    authorize(&headers)?;
    let deployment = database.deployment(repository_name).await.map_err(internal_error)?.ok_or(StatusCode::NOT_FOUND)?;
    docker::restart(&deployment.repository_name).await.map_err(internal_error)?;
    Ok(Json(deployment))
}

pub async fn rollback(
    State(database): State<Database>, headers: HeaderMap, Path(repository_name): Path<String>, Query(request): Query<RollbackRequest>,
) -> Result<Json<Deployment>, StatusCode> {
    authorize(&headers)?;
    let mut previous = match request.image_tag {
        Some(image_tag) => database.deployment_version(repository_name, image_tag).await.map_err(internal_error)?,
        None => database.previous_deployment(repository_name).await.map_err(internal_error)?,
    }.ok_or(StatusCode::CONFLICT)?;
    let container = docker::run(&previous.repository_name, &previous.image_tag).await.map_err(internal_error)?;
    expose_app(&previous.repository_name, container.host_port).await.map_err(internal_error)?;
    previous.container_id = container.container_id;
    previous.host_port = container.host_port;
    let repository_name = previous.repository_name.clone();
    database.activate_deployment(previous).await.map_err(internal_error)?;
    database.deployment(repository_name).await.map_err(internal_error)?.map(Json).ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct RollbackRequest { pub image_tag: Option<String> }

fn authorize(headers: &HeaderMap) -> Result<(), StatusCode> {
    let token = env::var("MINISTERIUM_API_TOKEN").map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let expected = format!("Bearer {token}");
    match headers.get("authorization").and_then(|value| value.to_str().ok()) {
        Some(value) if value == expected => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn internal_error(error: String) -> StatusCode {
    eprintln!("container control operation failed: {error}");
    StatusCode::INTERNAL_SERVER_ERROR
}
