use crate::utils::nginx_utils::{write_location, reload_nginx};

pub async fn expose_app(
    repo_name: &str,
    port: u16,
) -> Result<(), String> {
    write_location(repo_name, port).await?;
    reload_nginx().await?;
    println!("NGINX updated for {}", repo_name);
    Ok(())
}


