
use tokio::process::Command;

pub fn generate_location_block(repo_name: &str, port: u16) -> String {
    format!(
r#"
location /{repo}/ {{
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

use tokio::fs;

pub async fn upsert_location(
    repo_name: &str,
    port: u16,
) -> Result<(), String> {
    let path = "/etc/nginx/conf.d/ministerium/locations.conf";

    let new_block = generate_location_block(repo_name, port);

    let existing = fs::read_to_string(path).await.unwrap_or_default();

    let start_marker = format!("location /{}/", repo_name);

    let updated = if existing.contains(&start_marker) {
        // Replace existing location block
        replace_location_block(&existing, repo_name, &new_block)?
    } else {
        // Append new block
        format!("{}\n{}", existing, new_block)
    };

    fs::write(path, updated)
        .await
        .map_err(|e| format!("failed to write nginx locations: {}", e))?;

    Ok(())
}

fn replace_location_block(
    contents: &str,
    repo_name: &str,
    new_block: &str,
) -> Result<String, String> {
    let start = format!("location /{}/", repo_name);

    let start_idx = contents
        .find(&start)
        .ok_or("location start not found")?;

    let brace_idx = contents[start_idx..]
        .find('}')
        .ok_or("location end not found")?
        + start_idx
        + 1;

    let mut result = String::new();
    result.push_str(&contents[..start_idx]);
    result.push_str(new_block);
    result.push_str(&contents[brace_idx..]);

    Ok(result)
}
