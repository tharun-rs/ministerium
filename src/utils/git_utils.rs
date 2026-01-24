use axum::http::HeaderMap;
use crate::models::webhook_payload::Event;
use std::env;
use std::path::Path;

pub fn git_repos_root_folder() -> String {
    env::var("GITHUB_ROOT_FOLDER")
        .expect("GITHUB_ROOT_FOLDER not set")
}

pub fn extract_event(headers: &HeaderMap) -> Option<Event> {
    match headers.get("X-GitHub-Event")?.to_str().ok()? {
        "push" => Some(Event::Push),
        "pull_request" => Some(Event::PullRequest),
        _ => None,
    }
}

pub fn repo_exist(repo_name: &String) -> bool {
    let repo_root = git_repos_root_folder();
    let repo_path = Path::new(&repo_root).join(repo_name);

    repo_path.exists() && repo_path.is_dir()
}