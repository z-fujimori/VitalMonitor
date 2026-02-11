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
