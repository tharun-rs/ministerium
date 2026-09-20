use tokio::process::Command;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::{
    git_utils::git_repos_root_folder,
    docker_utils::get_docker_port
};

pub struct RunningContainer {
    pub container_id: String,
    pub host_port: u16,
}

pub async fn build(repo_name: &str) -> Result<String, String> {
    let repo_root = git_repos_root_folder();
    let repo_folder = Path::new(&repo_root).join(repo_name);
    let version = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to create image version: {error}"))?
        .as_millis();
    let image_tag = format!("{}:deployment-{}", repo_name, version);


    let output = Command::new("docker")
        .args([
            "build",
            "-t",
            &image_tag,
            repo_folder.to_str().ok_or("invalid repo path")?,
        ])
        .output()
        .await
        .map_err(|e| format!("failed to spawn docker build: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "docker build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let latest = format!("{}:latest", repo_name);
    let tag = Command::new("docker")
        .args(["tag", &image_tag, &latest])
        .output()
        .await
        .map_err(|error| format!("failed to tag docker image: {error}"))?;
    if !tag.status.success() {
        return Err(format!("docker tag failed:\n{}", String::from_utf8_lossy(&tag.stderr)));
    }

    Ok(image_tag)
}

pub async fn run(repo_name: &str, image_tag: &str) -> Result<RunningContainer, String> {
    // 0. Ensure old container is gone
    stop_and_remove_container(repo_name).await?;

    // 1. Run container with a fixed name and dynamic port
    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--restart", "unless-stopped",
            "--name", repo_name,
            "--label", "ministerium.managed=true",
            "--label", &format!("ministerium.repository={}", repo_name),
            "-p", "0:8080",
            image_tag,
        ])
        .output()
        .await
        .map_err(|e| format!("failed to start docker container: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "docker run failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    // 2. Ask Docker which port was assigned
    let host_port = get_docker_port(&container_id).await?;
    Ok(RunningContainer { container_id, host_port })
}

pub async fn restart(container_name: &str) -> Result<(), String> {
    let output = Command::new("docker").args(["restart", container_name]).output().await
        .map_err(|error| format!("failed to restart container: {error}"))?;
    if !output.status.success() {
        return Err(format!("docker restart failed:\n{}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
}


async fn stop_and_remove_container(container_name: &str) -> Result<(), String> {
    // Check if container exists
    let output = Command::new("docker")
        .args(["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.ID}}"])
        .output()
        .await
        .map_err(|e| format!("failed to check containers: {}", e))?;

    if output.stdout.is_empty() {
        // No existing container
        return Ok(());
    }

    // Stop container (ignore errors if already stopped)
    Command::new("docker")
        .args(["stop", container_name])
        .output()
        .await
        .ok();

    // Remove container
    let rm = Command::new("docker")
        .args(["rm", container_name])
        .output()
        .await
        .map_err(|e| format!("failed to remove container: {}", e))?;

    if !rm.status.success() {
        return Err(String::from_utf8_lossy(&rm.stderr).to_string());
    }

    Ok(())
}
