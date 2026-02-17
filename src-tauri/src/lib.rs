use tauri::tray::TrayIcon;
use std::sync::{Mutex, Arc};
use tokio::sync::RwLock;

mod mac_metrics;
mod metrics;
mod ui;
mod updater;

use crate::metrics::types::{SharedMetrics, MetricsSnapshot};

pub struct TrayState {
    pub tray: Mutex<TrayIcon>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    tauri::Builder::default()
        .setup(|app| {
            ui::tray::build_tray(app)?;

            let metrics: SharedMetrics = Arc::new(RwLock::new(MetricsSnapshot::default()));
        
            // #[cfg(target_os = "macos")]
            // spawn_tray_updater(app.handle().clone());
            metrics::service::spawn_metric_tasks(metrics.clone());
            updater::tray_updater::spawn_tray_renderer(app.handle().clone(), metrics);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

