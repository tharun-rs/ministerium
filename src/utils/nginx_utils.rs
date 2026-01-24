use std::env;
use std::path::Path;
use tokio::process::Command;

pub fn generate_nginx_config(repo_name: &str, port: u16) -> String {
    let server_name = env::var("APPS_SERVER_ADDR").unwrap_or_else(|_| "localhost".to_string());

    format!(
        r#"server {{
    listen 80;
    server_name {server_name};

    location /{repo}/ {{
        proxy_pass http://127.0.0.1:{port}/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }}
}}"#,
        server_name = server_name,
        repo = repo_name,
        port = port
    )
}

pub async fn reload_nginx() -> Result<(), String> {
    let test = Command::new("sudo")
        .args(["nginx","-t"])
        .output()
        .await
        .map_err(|e| format!("failed to test nginx config: {}", e))?;

    if !test.status.success() {
        return Err(String::from_utf8_lossy(&test.stderr).to_string());
    }

    let reload = Command::new("sudo")
        .args(["nginx","-s", "reload"])
        .output()
        .await
        .map_err(|e| format!("failed to reload nginx: {}", e))?;

    if !reload.status.success() {
        return Err(String::from_utf8_lossy(&reload.stderr).to_string());
    }

    Ok(())
}

