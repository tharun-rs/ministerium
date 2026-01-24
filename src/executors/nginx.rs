use crate::utils::nginx_utils::{generate_nginx_config, reload_nginx};
use tokio::fs;

pub async fn reload_nginx_if_needed(repo_name: &str, port: u16) -> Result<(), String> {
    // 1. Write the Nginx config
    let contents = &generate_nginx_config(repo_name, port);
    let path = format!("/etc/nginx/conf.d/ministerium/{}.conf", repo_name);

    fs::write(&path, contents)
        .await
        .map_err(|e| format!("failed to write nginx config: {}", e))?;
    println!("Config file written.");

    // 2. Validate and reload nginx
    reload_nginx().await?;
    print!("Nginx Reloaded");

    Ok(())
}
