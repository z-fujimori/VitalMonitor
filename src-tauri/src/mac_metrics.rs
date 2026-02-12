use std::process::Command;

#[cfg(target_os = "macos")]
pub async fn read_memory_pressure_pct() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
  use tokio::process::Command;

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

