// macOSでの取得（CPU/Memory/NW）※OS依存まとめる
use crate::metrics::types::{Percent, Millisecond};
use sysinfo::System;
use regex::Regex;
use std::net::TcpStream;
use tokio::process::Command;
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug)]
pub enum ReadError {
    CommandSpawn(&'static str),
    NonZeroExit(&'static str),
    Utf8(std::string::FromUtf8Error),
    Parse(&'static str),
    Io(std::io::Error),
    Timeout(&'static str),
    Other(String),

}

impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self { ReadError::Io(e) }
}
impl From<std::string::FromUtf8Error> for ReadError {
    fn from(e: std::string::FromUtf8Error) -> Self { ReadError::Utf8(e) }
}

pub async fn read_cpu_usage_pct() -> Result<Percent, ReadError> {
    let mut sys = System::new_all();

    // 1回目の更新（初期化）
    sys.refresh_cpu_usage();
    sleep(Duration::from_millis(120)).await;

    // 2回目の更新（差分から使用率が出る）
    sys.refresh_cpu_usage();

    let usage = sys.global_cpu_info().cpu_usage(); // f32 (0..=100)
    Ok(Percent(usage).clamp_0_100())
}

#[cfg(target_os = "macos")]
pub async fn read_memory_pressure_pct() -> Result<crate::metrics::types::Percent, ReadError> {
    let output = Command::new("memory_pressure")
        .arg("-Q")
        .output()
        .await
        .map_err(|_| ReadError::CommandSpawn("memory_pressure"))?;

    if !output.status.success() {
        return Err(ReadError::NonZeroExit("memory_pressure"));
    }

    let stdout = String::from_utf8(output.stdout)?; // ReadError::Utf8

    // 例: "System-wide memory free percentage: 61%"
    let free_pct: f32 = stdout
        .split_whitespace()
        .find(|s| s.ends_with('%'))
        .ok_or(ReadError::Parse("percent not found"))?
        .trim_end_matches('%')
        .parse::<f32>()
        .map_err(|_| ReadError::Parse("percent parse failed"))?;

    // free% → pressure% に変換
    Ok(Percent(100.0 - free_pct).clamp_0_100())
}

pub fn network_latency_ms_tcp() -> Result<Millisecond, ReadError> {
    let addr = "1.1.1.1:443".parse().map_err(|_| ReadError::Parse("invalid addr"))?;

    let start = Instant::now();
    TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .map_err(|_| ReadError::Timeout("tcp connect"))?;

    let ms = start.elapsed().as_secs_f32() * 1000.0;
    Ok(Millisecond(ms))
}
