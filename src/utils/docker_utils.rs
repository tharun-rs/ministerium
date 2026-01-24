use tokio::process::Command;

pub async fn get_docker_port(container_id: &String) -> Result<u16, String> {
    let port_output = Command::new("docker")
        .args([
            "port",
            container_id,
            "8080",
        ])
        .output()
        .await
        .map_err(|e| format!("Error getting docker port: {}",e))?;

    if !port_output.status.success() {
        return Err(format!(
            "docker port failed:\n{}",
            String::from_utf8_lossy(&port_output.stderr)
        ));
    }

    // Example output: "0.0.0.0:49153\n"
    let port_str = String::from_utf8_lossy(&port_output.stdout);
    let host_port = port_str
        .split(':')
        .last()
        .ok_or("failed to parse port")?
        .trim()
        .parse::<u16>()
        .map_err(|_| "invalid port number")?;

    Ok(host_port)

}