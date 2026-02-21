use tauri::Manager;

use crate::TrayState;
use crate::ui::types::{AlertLevel, DisplayMode, TrayConfig, ClassifiedSnapshot};

pub fn spawn_tray_renderer(app: tauri::AppHandle, metrics: crate::metrics::types::SharedMetrics) {
    let render_interval = 1;
    let rotation_interval = 5;

    tauri::async_runtime::spawn(async move {
        let mut render_ticker = tokio::time::interval(std::time::Duration::from_secs(render_interval));
        let mut rotation_tick = tokio::time::interval(std::time::Duration::from_secs(rotation_interval));
        let mut rotation_index = 0;
        loop {
            tokio::select! {
                _ = render_ticker.tick() => {
                    let cfg: TrayConfig = match app.try_state::<crate::ui::types::TrayUiState>() {
                        Some(st) => *st.config.lock().unwrap(),
                        None => continue,
                    };
                      
                    let snap = metrics.read().await.clone();
                    let snap_classified = crate::ui::types::ClassifiedSnapshot::new(snap);
                    
                    
                    let title = format_title(&cfg, &snap_classified, &mut rotation_index);
                    
                    if let Some(state) = app.try_state::<TrayState>() {
                        if let Ok(tray) = state.tray.lock() {
                            let _ = tray.set_title(Some(&title));
                        }
                    }
                },

                _ = rotation_tick.tick() => {
                    rotation_index = rotation_index.wrapping_add(1);
                    rotation_index %= 6; // TODO: magic number
                },
            }
        }
    });
}

pub fn format_title(
    cfg: &TrayConfig,
    s: &ClassifiedSnapshot,
    rotation_index: &mut usize,
) -> String {
    match cfg.mode {
        DisplayMode::List => format_list(cfg, s),
        DisplayMode::Rotation => format_rotation(cfg, s, rotation_index),
    }
}

fn icon(cfg: &TrayConfig, level: AlertLevel) -> &'static str {
    if cfg.is_alert { level.icon() } else { "" }
}

fn format_list(cfg: &TrayConfig, s: &ClassifiedSnapshot) -> String {
    let mut parts = Vec::new();
    if cfg.show_cpu {
        parts.push(match s.cpu {
            Some(m) => format!("{} CPU {}%", icon(cfg, m.level), m.value),
            None => "CPU --".into(),
        });
    }
    if cfg.show_mem {
        parts.push(match s.mem {
            Some(m) => format!("{} Mem {:.0}%", icon(cfg, m.level), m.value),
            None => "Mem --".into(),
        });
    }
    if cfg.show_nw {
        parts.push(match s.nw {
            Some(m) => format!("{} NW {:.0}ms", icon(cfg, m.level), m.value),
            None => "NW --".into(),
        });
    }

    let text = parts.join(" ");
    text
}

fn format_rotation(cfg: &TrayConfig, s: &ClassifiedSnapshot, rotation_index: &mut usize) -> String {
    let mut items: Vec<String> = Vec::new();

    if cfg.show_cpu {
        items.push(match s.cpu {
            Some(m) => format!("{}CPU {}%", icon(cfg, m.level), m.value),
            None => "CPU --".to_string(),
        });
    }

    if cfg.show_mem {
        items.push(match s.mem {
            Some(m) => format!("{}Mem {:.0}%", icon(cfg, m.level), m.value),
            None => "Mem --".to_string(),
        });
    }

    if cfg.show_nw {
        items.push(match s.nw {
            Some(m) => format!("{}NW {:.0}ms", icon(cfg, m.level), m.value),
            None => "NW --".to_string(),
        });
    }

    if items.is_empty() {
        return "—".to_string();
    }

    let idx = *rotation_index % items.len();
    *rotation_index = rotation_index.wrapping_add(1);

    items[idx].clone()
}
