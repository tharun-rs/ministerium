use serde::Serialize;
use std::{fs, path::Path, process::Command};

#[derive(Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: Option<f64>,
    pub load_average: Option<LoadAverage>,
    pub cpu_cores: Option<usize>,
    pub memory: Option<MemoryMetrics>,
    pub repository_disk: Option<DiskMetrics>,
    pub cpu_temperature_celsius: Option<f64>,
}

#[derive(Serialize)]
pub struct LoadAverage { pub one_minute: f64, pub five_minutes: f64, pub fifteen_minutes: f64 }

#[derive(Serialize)]
pub struct MemoryMetrics { pub total_bytes: u64, pub available_bytes: u64, pub used_bytes: u64 }

#[derive(Serialize)]
pub struct DiskMetrics { pub path: String, pub total_bytes: u64, pub used_bytes: u64, pub available_bytes: u64 }

pub fn system_metrics(repository_root: &str) -> SystemMetrics {
    SystemMetrics {
        uptime_seconds: read_uptime(), load_average: read_load_average(),
        cpu_cores: std::thread::available_parallelism().ok().map(usize::from),
        memory: read_memory(), repository_disk: read_disk_metrics(repository_root),
        cpu_temperature_celsius: read_cpu_temperature(),
    }
}

fn read_uptime() -> Option<f64> {
    fs::read_to_string("/proc/uptime").ok()?.split_whitespace().next()?.parse().ok()
}

fn read_load_average() -> Option<LoadAverage> {
    let values: Vec<f64> = fs::read_to_string("/proc/loadavg").ok()?.split_whitespace().take(3)
        .map(str::parse).collect::<Result<_, _>>().ok()?;
    Some(LoadAverage { one_minute: values[0], five_minutes: values[1], fifteen_minutes: values[2] })
}

fn read_memory() -> Option<MemoryMetrics> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    let value_in_bytes = |key| contents.lines().find_map(|line| line.strip_prefix(key)
        .and_then(|value| value.split_whitespace().next()).and_then(|value| value.parse::<u64>().ok())
        .map(|value| value * 1024));
    let total_bytes = value_in_bytes("MemTotal:")?;
    let available_bytes = value_in_bytes("MemAvailable:")?;
    Some(MemoryMetrics { total_bytes, available_bytes, used_bytes: total_bytes.saturating_sub(available_bytes) })
}

fn read_disk_metrics(path: &str) -> Option<DiskMetrics> {
    let output = Command::new("df").args(["-B1", "--output=size,used,avail", path]).output().ok()?;
    if !output.status.success() { return None; }
    let values: Vec<u64> = String::from_utf8_lossy(&output.stdout).lines().nth(1)?.split_whitespace()
        .map(str::parse).collect::<Result<_, _>>().ok()?;
    Some(DiskMetrics { path: path.to_owned(), total_bytes: values[0], used_bytes: values[1], available_bytes: values[2] })
}

fn read_cpu_temperature() -> Option<f64> {
    let value = fs::read_to_string(Path::new("/sys/class/thermal/thermal_zone0/temp")).ok()?;
    value.trim().parse::<f64>().ok().map(|millidegrees| millidegrees / 1000.0)
}
