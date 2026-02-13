use tokio::process::Command;
use tokio::time::{timeout, Duration, Instant};
use regex::Regex;
use std::net::TcpStream;

#[cfg(target_os = "macos")]
pub async fn read_memory_pressure_pct() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
  let output = Command::new("memory_pressure")
    .arg("-Q")
    .output()
    .await?;

  if !output.status.success() {
    return Err("memory_pressure failed".into());
  }

  let stdout = String::from_utf8(output.stdout)?;

  // 例: "System-wide memory free percentage: 61%"
  let free_pct = stdout
    .split_whitespace()
    .find(|s| s.ends_with('%'))
    .ok_or("percent not found")?
    .trim_end_matches('%')
    .parse::<f64>()?;

  // free% → pressure% に変換（あなたが表示したいのが pressure 側なら）
  Ok((100.0 - free_pct).clamp(0.0, 100.0))
}

// #[cfg(not(target_os = "macos"))]
pub async fn read_cpu_usage_pct() -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
  // sysinfo クレートを使って CPU 使用率を取得
  let cpu_usage = get_cpu();
  Ok(cpu_usage)
}

fn get_cpu() -> f32 {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_cpu();
    sys.global_cpu_info().cpu_usage()
}


pub async fn network_latency_ms() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // DNS遅延を排除したいので IP 直指定が無難（Cloudflare）
    let host = "1.1.1.1";

    // ping が詰まるケース対策：外側で timeout をかける（OS差を吸収）
    let fut = Command::new("ping")
        .arg("-n")          // 逆引きDNSを抑制（macOSで有効）
        .arg("-c").arg("1") // 1回だけ
        .arg(host)
        .output();

    let output = timeout(Duration::from_secs(2), fut)
        .await
        .map_err(|_| format!("ping timeout (host={})", host))??;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ping failed (host={}): {}", host, stderr.trim()).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // macOS/Linuxで出力が微妙に違っても拾えるように正規表現
    // 例: time=14.2 ms / time=14.2ms など
    let re = Regex::new(r"time[=<]?\s*([0-9]+(?:\.[0-9]+)?)\s*ms")?;

    let caps = re
        .captures(&stdout)
        .ok_or_else(|| format!("latency not found in ping output: {}", stdout))?;

    let ms: f64 = caps
        .get(1)
        .ok_or("failed to get capture group")?
        .as_str()
        .parse()?;
    Ok(ms)
}



pub fn read_network_latency_ms() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    // DNS遅延を排除したいので IP 直指定が無難（Cloudflare）
    let latency_ms = tcp_latency().ok_or("tcp latency measurement failed")? as f64;

    Ok(latency_ms as u64)
}

fn tcp_latency() -> Option<u128> {
    let addr = "1.1.1.1:443".parse().ok()?;
    let start = Instant::now();

    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).ok()?;

    Some(start.elapsed().as_millis())
}
