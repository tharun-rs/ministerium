use std::path::Path;
use tokio::process::Command;

pub fn generate_nginx_config(repo_name: &str, port: u16) -> String {
    format!(
r#"server {{
    listen 80;

    location /{repo}/ {{
        proxy_pass http://127.0.0.1:{port}/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }}
}}"#,
        repo = repo_name,
        port = port
    )
}

pub fn nginx_config_exists(repo_name: &str) -> bool {
    let path_str = format!("/etc/nginx/conf.d/{}.conf", repo_name);
    let nginx_file = Path::new(&path_str);

    nginx_file.exists() && nginx_file.is_file()
}

pub async fn reload_nginx() -> Result<(), String> {
    let test = Command::new("nginx")
        .arg("-t")
        .output()
        .await
        .map_err(|e| format!("failed to test nginx config: {}", e))?;

    if !test.status.success() {
        return Err(String::from_utf8_lossy(&test.stderr).to_string());
    }

    let reload = Command::new("nginx")
        .args(["-s", "reload"])
        .output()
        .await
        .map_err(|e| format!("failed to reload nginx: {}", e))?;

    if !reload.status.success() {
        return Err(String::from_utf8_lossy(&reload.stderr).to_string());
    }

    Ok(())
}

