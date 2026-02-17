use tauri::{
    Manager,
    menu::{
        Menu,
        MenuItem,
        PredefinedMenuItem,
        Submenu,
        CheckMenuItem,
    },
    tray::{
        TrayIcon,
        TrayIconBuilder
    }
};
use std::sync::{Mutex, Arc};
use tokio::sync::RwLock;

mod mac_metrics;
mod metrics;
mod ui;

use crate::metrics::types::{SharedMetrics, MetricsSnapshot};

pub struct TrayState {
    pub tray: Mutex<TrayIcon>,
}

#[cfg(target_os = "macos")]
fn spawn_tray_updater(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(3));

        loop {
        ticker.tick().await;

        let pressure_pct = match crate::mac_metrics::read_memory_pressure_pct().await {
            Ok(v) => v,
            Err(e) => {
            eprintln!("memory_pressure error: {e}");
            continue;
            }
        };

        let cpu_pct = match crate::mac_metrics::read_cpu_usage_pct().await {
            Ok(v) => v,
            Err(e) => {
            eprintln!("cpu usage error: {e}");
            continue;
            }
        };

        let nw_ms = match crate::mac_metrics::network_latency_ms().await {
            Ok(v) => v,
            Err(e) => {
            eprintln!("network latency error: {e}");
            continue;
            }
        };

        let demo_metrics_cpu = crate::mac_metrics::read_cpu_usage_pct().await; // --- IGNORE ---
        let demo_metrics_mem = crate::metrics::collect_macos::read_memory_pressure_pct().await; // --- IGNORE ---
        let demo_metrics_nw = crate::mac_metrics::read_network_latency_ms(); // --- IGNORE ---
        println!("demo cpu: {:?}, mem: {:?}, nw: {:?}", demo_metrics_cpu, demo_metrics_mem, demo_metrics_nw); // --- IGNORE ---

        let mem_text = format!("Mem {:.0}%", pressure_pct);
        let cpu_text = format!("CPU {:.0}%", cpu_pct);
        let nw_text = format!("NW {:.0}ms", nw_ms);

        let text = format!("{} {} {}", cpu_text, mem_text, nw_text);

        if let Some(state) = app.try_state::<TrayState>() {
            if let Ok(tray) = state.tray.lock() {
            let _ = tray.set_title(Some(&text));
            }
        }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    tauri::Builder::default()
        .setup(|app| {
            ui::tray::build_tray(app)?;

            let metrics: SharedMetrics = Arc::new(RwLock::new(MetricsSnapshot::default()));
        
            #[cfg(target_os = "macos")]
            spawn_tray_updater(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

