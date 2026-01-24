
use tokio::process::Command;
use tokio::fs;

pub fn generate_location_block(repo_name: &str, port: u16) -> String {
    format!(
r#"location /{repo}/ {{
    proxy_pass http://127.0.0.1:{port}/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}}
"#,
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

pub async fn write_location(
    repo_name: &str,
    port: u16,
) -> Result<(), String> {
    let path = format!(
        "/etc/nginx/conf.d/ministerium/locations/{}.conf",
        repo_name
    );

    let contents = generate_location_block(repo_name, port);

    fs::write(&path, contents)
        .await
        .map_err(|e| format!("failed to write nginx location: {}", e))?;

    Ok(())
}
