use tauri::tray::TrayIcon;
use std::sync::{Mutex, Arc};
use std::path::PathBuf;
use tokio::sync::RwLock;
use tauri::Manager;
use tauri::ActivationPolicy;

mod mac_metrics;
mod metrics;
mod ui;
mod updater;

use crate::metrics::types::{SharedMetrics, MetricsSnapshot};
use crate::ui::types::{TrayConfig, DisplayMode};

pub struct TrayState {
    pub tray: Mutex<TrayIcon>,
}

fn load_config(app: &tauri::App) -> Option<TrayConfig> {
    let dir: PathBuf = app.path().app_config_dir().ok()?;
    let path = dir.join("tray_config.json");
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    tauri::Builder::default()
        .setup(|app| {
            let initial_cfg = load_config(app)
                .unwrap_or(TrayConfig {
                    show_cpu: true,
                    show_mem: true,
                    show_nw: true,
                    mode: DisplayMode::List,
                    is_alert: true,
                });

            ui::tray::build_tray(app, initial_cfg)?;

            let metrics: SharedMetrics = Arc::new(RwLock::new(MetricsSnapshot::default()));
        
            // #[cfg(target_os = "macos")]
            // spawn_tray_updater(app.handle().clone());
            metrics::service::spawn_metric_tasks(metrics.clone());
            updater::tray_updater::spawn_tray_renderer(app.handle().clone(), metrics);

            // Dockに表示しない
            app.set_activation_policy(ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

