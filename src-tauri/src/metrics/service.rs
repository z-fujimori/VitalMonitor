use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

use crate::metrics::types::{SharedMetrics, MetricsSnapshot};

pub fn spawn_metric_tasks(metrics: SharedMetrics) {
    let get_cpu_interval = 1;
    let get_mem_interval = 1;
    let get_nw_interval = 3;
    let get_nw_timeout = 2;

    // CPU
    tauri::async_runtime::spawn({
        let metrics = Arc::clone(&metrics);
        async move {
            let mut tick = tokio::time::interval(Duration::from_secs(get_cpu_interval));
            loop {
                tick.tick().await;
                let v = crate::mac_metrics::read_cpu_usage_pct().await.ok();
                let mut m = metrics.write().await;
                m.cpu_pct = v;
            }
        }
    });

    // MEM
    tauri::async_runtime::spawn({
        let metrics = Arc::clone(&metrics);
        async move {
            let mut tick = tokio::time::interval(Duration::from_secs(get_mem_interval));
            loop {
                tick.tick().await;
                let v  = crate::mac_metrics::read_memory_pressure_pct().await.ok().map(|x| x as f32);;
                let mut m = metrics.write().await;
                m.mem_pressure_pct = v;
            }
        }
    });

    // NW（timeoutあり・1回だけ）
    tauri::async_runtime::spawn({
        let metrics = Arc::clone(&metrics);
        async move {
            let mut tick = tokio::time::interval(Duration::from_secs(get_nw_interval));
            loop {
                tick.tick().await;

                let v = tokio::time::timeout(
                    Duration::from_secs(get_nw_timeout),
                    crate::mac_metrics::network_latency_ms(),
                )
                .await
                .ok()
                .and_then(|r| r.ok());

                let mut m = metrics.write().await;
                m.nw_ms = v;
            }
        }
    });
}