

pub fn spawn_tray_renderer(app: tauri::AppHandle, metrics: SharedMetrics) {
    let render_interval = 1;
    let otation_interval = 3;

    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(render_interval));
    loop {
        tick.tick().await;

        let cfg = match app.try_state::<crate::ui::types::TrayUiState>() {
            Some(st) => *st.config.lock().unwrap(),
            None => continue,
        };

        let snap = metrics.read().await.clone();

        let title = format_title(&cfg, &snap);
    }
}

fn format_title(cfg: &crate::ui::types::TrayConfig, s: &MetricsSnapshot) -> String {
    let mut parts = Vec::new();
    if cfg.show_cpu {
        parts.push(match s.cpu_pct { Some(v) => format!("CPU {:.0}%", v), None => "CPU --".into() });
    }
    if cfg.show_mem {
        parts.push(match s.mem_pressure_pct { Some(v) => format!("Mem {:.0}%", v), None => "Mem --".into() });
    }
    if cfg.show_nw {
        parts.push(match s.nw_ms { Some(v) => format!("NW {:.0}ms", v), None => "NW --".into() });
    }
    let mut text = parts.join(" ");

}
