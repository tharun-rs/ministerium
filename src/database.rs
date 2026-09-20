use rusqlite::{params, Connection};
use serde::Serialize;
use std::{env, fs, path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

#[derive(Clone)]
pub struct Database { path: Arc<PathBuf> }

#[derive(Debug, Serialize)]
pub struct Deployment {
    pub repository_name: String, pub repository_full_name: String, pub repository_path: String,
    pub image_tag: String, pub container_id: String, pub host_port: u16, pub container_port: u16,
    pub restart_policy: String, pub deployed_at: i64, pub updated_at: i64,
}

pub struct NewDeployment {
    pub repository_name: String, pub repository_full_name: String, pub repository_ssh_url: String,
    pub repository_path: String, pub image_tag: String, pub container_id: String,
    pub host_port: u16, pub container_port: u16, pub restart_policy: String,
}

impl Database {
    pub fn from_environment() -> Result<Self, String> {
        let path = env::var("DEPLOYMENTS_DB_PATH").map(PathBuf::from).unwrap_or_else(|_| {
            PathBuf::from(env::var("GITHUB_ROOT_FOLDER").expect("GITHUB_ROOT_FOLDER not set")).join("ministerium.sqlite3")
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("failed to create database directory: {error}"))?;
        }
        let database = Self { path: Arc::new(path) };
        database.initialize()?;
        Ok(database)
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open(self.path.as_ref()).map_err(|error| format!("failed to open deployment database: {error}"))
    }

    fn initialize(&self) -> Result<(), String> {
        self.connection()?.execute_batch(
            "CREATE TABLE IF NOT EXISTS deployments (
                repository_name TEXT PRIMARY KEY NOT NULL, repository_full_name TEXT NOT NULL,
                repository_ssh_url TEXT NOT NULL, repository_path TEXT NOT NULL, image_tag TEXT NOT NULL,
                container_id TEXT NOT NULL, host_port INTEGER NOT NULL, container_port INTEGER NOT NULL,
                restart_policy TEXT NOT NULL, deployed_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS deployment_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT, repository_name TEXT NOT NULL,
                repository_full_name TEXT NOT NULL, repository_path TEXT NOT NULL, image_tag TEXT NOT NULL,
                container_id TEXT NOT NULL, host_port INTEGER NOT NULL, container_port INTEGER NOT NULL,
                restart_policy TEXT NOT NULL, deployed_at INTEGER NOT NULL
            );"
        ).map_err(|error| format!("failed to initialize deployment database: {error}"))?;
        Ok(())
    }

    pub async fn save_deployment(&self, deployment: NewDeployment) -> Result<(), String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.save_deployment_blocking(deployment)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn save_deployment_blocking(&self, deployment: NewDeployment) -> Result<(), String> {
        let now = unix_timestamp()?;
        self.connection()?.execute(
            "INSERT INTO deployments (repository_name, repository_full_name, repository_ssh_url, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(repository_name) DO UPDATE SET
               repository_full_name=excluded.repository_full_name, repository_ssh_url=excluded.repository_ssh_url,
               repository_path=excluded.repository_path, image_tag=excluded.image_tag, container_id=excluded.container_id,
               host_port=excluded.host_port, container_port=excluded.container_port, restart_policy=excluded.restart_policy,
               updated_at=excluded.updated_at",
            params![deployment.repository_name, deployment.repository_full_name, deployment.repository_ssh_url,
                deployment.repository_path, deployment.image_tag, deployment.container_id, deployment.host_port,
                deployment.container_port, deployment.restart_policy, now, now]
        ).map_err(|error| format!("failed to save deployment: {error}"))?;
        self.connection()?.execute(
            "INSERT INTO deployment_versions (repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![deployment.repository_name, deployment.repository_full_name, deployment.repository_path,
                deployment.image_tag, deployment.container_id, deployment.host_port, deployment.container_port,
                deployment.restart_policy, now]
        ).map_err(|error| format!("failed to save deployment history: {error}"))?;
        Ok(())
    }

    pub async fn list_deployments(&self) -> Result<Vec<Deployment>, String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.list_deployments_blocking()).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn list_deployments_blocking(&self) -> Result<Vec<Deployment>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, updated_at FROM deployments ORDER BY repository_name")
            .map_err(|error| format!("failed to query deployments: {error}"))?;
        statement.query_map([], deployment_from_row).map_err(|error| format!("failed to query deployments: {error}"))?
            .collect::<Result<Vec<_>, _>>().map_err(|error| format!("failed to read deployments: {error}"))
    }

    pub async fn deployment(&self, repository_name: String) -> Result<Option<Deployment>, String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.deployment_blocking(&repository_name)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn deployment_blocking(&self, repository_name: &str) -> Result<Option<Deployment>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, updated_at FROM deployments WHERE repository_name = ?")
            .map_err(|error| format!("failed to query deployment: {error}"))?;
        match statement.query_row([repository_name], deployment_from_row) {
            Ok(deployment) => Ok(Some(deployment)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(format!("failed to read deployment: {error}")),
        }
    }

    pub async fn previous_deployment(&self, repository_name: String) -> Result<Option<Deployment>, String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.previous_deployment_blocking(&repository_name)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn previous_deployment_blocking(&self, repository_name: &str) -> Result<Option<Deployment>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, deployed_at
             FROM deployment_versions WHERE repository_name = ? ORDER BY id DESC LIMIT 1 OFFSET 1"
        ).map_err(|error| format!("failed to query deployment history: {error}"))?;
        match statement.query_row([repository_name], deployment_from_row) {
            Ok(deployment) => Ok(Some(deployment)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(format!("failed to read deployment history: {error}")),
        }
    }

    pub async fn deployment_versions(&self, repository_name: String) -> Result<Vec<Deployment>, String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.deployment_versions_blocking(&repository_name)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn deployment_versions_blocking(&self, repository_name: &str) -> Result<Vec<Deployment>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, deployed_at
             FROM deployment_versions WHERE repository_name = ? ORDER BY id DESC"
        ).map_err(|error| format!("failed to query deployment history: {error}"))?;
        statement.query_map([repository_name], deployment_from_row).map_err(|error| format!("failed to query deployment history: {error}"))?
            .collect::<Result<Vec<_>, _>>().map_err(|error| format!("failed to read deployment history: {error}"))
    }

    pub async fn deployment_version(&self, repository_name: String, image_tag: String) -> Result<Option<Deployment>, String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.deployment_version_blocking(&repository_name, &image_tag)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn deployment_version_blocking(&self, repository_name: &str, image_tag: &str) -> Result<Option<Deployment>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT repository_name, repository_full_name, repository_path, image_tag, container_id, host_port, container_port, restart_policy, deployed_at, deployed_at
             FROM deployment_versions WHERE repository_name = ? AND image_tag = ? ORDER BY id DESC LIMIT 1"
        ).map_err(|error| format!("failed to query deployment history: {error}"))?;
        match statement.query_row(params![repository_name, image_tag], deployment_from_row) {
            Ok(deployment) => Ok(Some(deployment)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(format!("failed to read deployment history: {error}")),
        }
    }

    pub async fn activate_deployment(&self, deployment: Deployment) -> Result<(), String> {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.activate_deployment_blocking(deployment)).await
            .map_err(|error| format!("deployment database task failed: {error}"))?
    }

    fn activate_deployment_blocking(&self, deployment: Deployment) -> Result<(), String> {
        let now = unix_timestamp()?;
        self.connection()?.execute(
            "UPDATE deployments SET repository_full_name=?, repository_path=?, image_tag=?, container_id=?, host_port=?, container_port=?, restart_policy=?, updated_at=? WHERE repository_name=?",
            params![deployment.repository_full_name, deployment.repository_path, deployment.image_tag,
                deployment.container_id, deployment.host_port, deployment.container_port,
                deployment.restart_policy, now, deployment.repository_name]
        ).map_err(|error| format!("failed to activate deployment: {error}"))?;
        Ok(())
    }
}

fn deployment_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deployment> {
    Ok(Deployment { repository_name: row.get(0)?, repository_full_name: row.get(1)?, repository_path: row.get(2)?, image_tag: row.get(3)?, container_id: row.get(4)?, host_port: row.get(5)?, container_port: row.get(6)?, restart_policy: row.get(7)?, deployed_at: row.get(8)?, updated_at: row.get(9)? })
}

fn unix_timestamp() -> Result<i64, String> {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs() as i64)
        .map_err(|error| format!("system time is before Unix epoch: {error}"))
}
