use crate::TrayState;
use tauri::Manager;

pub fn spawn_tray_renderer(app: tauri::AppHandle, metrics: crate::metrics::types::SharedMetrics) {
    let render_interval = 1;
    let otation_interval = 3;

    tauri::async_runtime::spawn(async move {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(render_interval));
        loop {
            ticker.tick().await;

            let cfg = match app.try_state::<crate::ui::types::TrayUiState>() {
                Some(st) => *st.config.lock().unwrap(),
                None => continue,
            };

            let snap = metrics.read().await.clone();
            let snap_classified = crate::ui::types::ClassifiedSnapshot::new(snap);

            let title = format_title(&cfg, &snap_classified);

            if let Some(state) = app.try_state::<TrayState>() {
                if let Ok(tray) = state.tray.lock() {
                    let _ = tray.set_title(Some(&title));
                }
            }
        }
    });
}

fn format_title(cfg: &crate::ui::types::TrayConfig, s: &crate::ui::types::ClassifiedSnapshot) -> String {
    let mut parts = Vec::new();
    if cfg.show_cpu {
        let alertIcon = s.cpu.map(|m| match m.level {
            crate::ui::types::AlertLevel::Safe => "🔵",
            crate::ui::types::AlertLevel::Normal => "🟢",
            crate::ui::types::AlertLevel::Warning => "🟤",
            crate::ui::types::AlertLevel::Critical => "🔴",
        }).unwrap_or("");
        parts.push(match s.cpu { Some(v) => format!("{} CPU {:.0}%", alertIcon, v.value), None => "CPU --".into() });
    }
    if cfg.show_mem {
            let alertIcon = s.mem.map(|m| match m.level {
                crate::ui::types::AlertLevel::Safe => "🔵",
                crate::ui::types::AlertLevel::Normal => "🟢",
                crate::ui::types::AlertLevel::Warning => "🟤",
                crate::ui::types::AlertLevel::Critical => "🔴",
            }).unwrap_or("");
        parts.push(match s.mem { Some(v) => format!("{} Mem {:.0}%", alertIcon, v.value), None => "Mem --".into() });
    }
    if cfg.show_nw {
            let alertIcon = s.nw.map(|m| match m.level {
                crate::ui::types::AlertLevel::Safe => "🔵",
                crate::ui::types::AlertLevel::Normal => "🟢",
                crate::ui::types::AlertLevel::Warning => "🟤",
                crate::ui::types::AlertLevel::Critical => "🔴",
            }).unwrap_or("");
        parts.push(match s.nw { Some(v) => format!("{} NW {:.0}ms", alertIcon, v.value), None => "NW --".into() });
    }
    let mut text = parts.join(" ");

    text = text.trim().to_string();
    text
}
