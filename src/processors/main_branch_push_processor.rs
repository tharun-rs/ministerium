use crate::{
    database::{Database, NewDeployment},
    executors::{
        docker::{build, run},
        git::{clone, pull}, nginx::expose_app,
    },
    models::webhook_payload::WebhookPayload,
    utils::git_utils::{git_repos_root_folder, repo_exist},
};

pub async fn process(payload: WebhookPayload, database: Database) -> Result<(), String> {
    if let Some(repo) = payload.repository.as_ref() {
        let git_repo_ssh_url = &repo.ssh_url;
        let git_repo_name = &repo.name;


        // 1. Pull/clone repo
        if !repo_exist(git_repo_name) {
            clone(git_repo_ssh_url, git_repo_name).await?;
            println!("Repo cloned successfully");
        } else {
            pull(git_repo_name).await?;
            println!("Repo pulled successfully");
        }

        // 2. Build using docker
        let image_tag = build(git_repo_name).await?;
        println!("Docker build completed");

        // 3. Start docker
        let container = run(git_repo_name, &image_tag).await?;
        println!("Docker running started on port {}", container.host_port);

        // 4. Expose on nginx
        expose_app(git_repo_name, container.host_port).await?;

        // 5. Persist the deployment inventory after the service is reachable through NGINX.
        database.save_deployment(NewDeployment {
            repository_name: git_repo_name.clone(),
            repository_full_name: repo.full_name.clone(),
            repository_ssh_url: git_repo_ssh_url.clone(),
            repository_path: format!("{}/{}", git_repos_root_folder(), git_repo_name),
            image_tag,
            container_id: container.container_id,
            host_port: container.host_port,
            container_port: 8080,
            restart_policy: "unless-stopped".to_string(),
        }).await?;
        print!("Processing completed");
    }
    Ok(())
}
