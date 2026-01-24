use tokio::process::Command;
use std::path::Path;

use crate::utils::{
    git_utils::git_repos_root_folder,
    docker_utils::get_docker_port
};

pub async fn build(repo_name: &str) -> Result<(), String> {
    let repo_root = git_repos_root_folder();
    let repo_folder = Path::new(&repo_root).join(repo_name);
    let image_tag = format!("{}:latest", repo_name);


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

    Ok(())
}

pub async fn run(repo_name: &str) -> Result<u16, String> {
    // 1. Run container with dynamic port mapping
    let output = Command::new("docker")
        .args([
            "run",
            "-d",                 // detached
            "-p", "0:8080",       // random host port → container 8080
            repo_name,            // image name
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

    // 2. Extract container ID
    let container_id = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    // 3. Ask Docker which port was assigned
    get_docker_port(&container_id).await
}
