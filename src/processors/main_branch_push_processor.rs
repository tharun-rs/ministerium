use crate::{
    executors::{
        docker::{build, run},
        git::{clone, pull}, nginx::reload_nginx_if_needed,
    },
    models::webhook_payload::WebhookPayload,
    utils::git_utils::repo_exist,
};

pub async fn process(payload: WebhookPayload) -> Result<(), String> {
    if let Some(repo) = payload.repository.as_ref() {
        let git_repo_ssh_url = &repo.ssh_url;
        let git_repo_name = &repo.name;


        // 1. Pull/clone repo
        if !repo_exist(git_repo_name) {
            clone(git_repo_ssh_url).await?;
            println!("Repo cloned successfully");
        } else {
            pull(git_repo_name).await?;
            println!("Repo pulled successfully");
        }

        // 2. Build using docker
        build(git_repo_name).await?;
        println!("Docker build completed");

        // 3. Start docker
        let port = run(git_repo_name).await?;
        print!("Docker running started");

        // 4. Expose on nginx if not configured already
        reload_nginx_if_needed(git_repo_name, port).await?;
        print!("Processing completed");
    }
    Ok(())
}
