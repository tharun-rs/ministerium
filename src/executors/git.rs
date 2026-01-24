use std::path::Path;

use tokio::process::Command;

use crate::utils::git_utils::git_repos_root_folder;

pub async fn clone(repo_url: &String, repo_name: &String) -> Result<(), String> {
    let repo_path = format!("{}/{}",git_repos_root_folder(), repo_name);

    let output = Command::new("git")
        .args(["clone", repo_url, &repo_path])
        .output()
        .await
        .map_err(|e| format!("failed to spawn git clone: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub async fn pull(repo_name: &String) -> Result<(), String> {
    let repo_root = git_repos_root_folder();
    let repo_folder = Path::new(&repo_root).join(repo_name);

    let output = Command::new("git")
        .args(["-C",
            repo_folder.to_str().ok_or("invalid repo path")?,
            "pull"
        ])
        .output()
        .await
        .map_err(|e| format!("failed to spawn git pull: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}