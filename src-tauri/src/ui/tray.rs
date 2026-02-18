use std::sync::Mutex;
use tauri::{
    App,
    Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu, CheckMenuItem},
    tray::TrayIconBuilder,
};
use tokio::fs;
use crate::ui::{self, types::{DisplayMode, TrayConfig, TrayUiState}};
use crate::TrayState;

pub fn build_tray(app: &App, initial_cfg: TrayConfig) -> tauri::Result<()> {
    // 表示オプションメニューの構築
    let mi_show_cpu = CheckMenuItem::with_id(app, "show_cpu", "CPU Only", true, false, None::<&str>)?;
    let mi_show_mem = CheckMenuItem::with_id(app, "show_mem", "Memory Only", true, false, None::<&str>)?;
    let mi_show_nw  = CheckMenuItem::with_id(app, "show_nw",  "Network Only", true, false, None::<&str>)?;
    let mi_show_cpu_mem = CheckMenuItem::with_id(app, "show_cpu_mem", "CPU + Memory", true, false, None::<&str>)?;
    let mi_show_mem_nw = CheckMenuItem::with_id(app, "show_mem_nw", "Memory + Network", true, false, None::<&str>)?;
    let mi_show_cpu_nw = CheckMenuItem::with_id(app, "show_cpu_nw", "CPU + Network", true, false, None::<&str>)?;
    let mi_show_all = CheckMenuItem::with_id(app, "show_all", "All Metrics", true, true, None::<&str>)?;
    let show_metrics_items: [&dyn tauri::menu::IsMenuItem<_>; 7] = [
        &mi_show_cpu,
        &mi_show_mem,
        &mi_show_nw,
        &mi_show_cpu_mem,
        &mi_show_mem_nw,
        &mi_show_cpu_nw,
        &mi_show_all,
    ];
    let show_metrics_sub = Submenu::with_items(app, "Show Metrics", true, &show_metrics_items)?;

    let mi_mode_list = CheckMenuItem::with_id(app, "mode_list", "List", true, true, None::<&str>)?;
    let mi_mode_rotation = CheckMenuItem::with_id(app, "mode_rotation", "Rotation", true, false, None::<&str>)?;
    let mode_items: [&dyn tauri::menu::IsMenuItem<_>; 2] = [&mi_mode_list, &mi_mode_rotation];
    let mode_sub  = Submenu::with_items(app, "Display Mode", true, &mode_items)?;

    let mi_is_alert = CheckMenuItem::with_id(app, "toggle_alert", "Show Alert", true, true, None::<&str>)?;

    let options_items: [&dyn tauri::menu::IsMenuItem<_>; 3] = [
        &show_metrics_sub,
        &mode_sub,
        &mi_is_alert,
    ];
    let options_sub = Submenu::with_items(app, "Options", true, &options_items)?;
    let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    // ルートメニュー
    let menu = Menu::with_items(app, &[&options_sub, &PredefinedMenuItem::separator(app)?, &exit_i])?;

    let ui_state = TrayUiState {
        config: Mutex::new(TrayConfig {
        show_cpu: initial_cfg.show_cpu,
        show_mem: initial_cfg.show_mem,
        show_nw: initial_cfg.show_nw,
        mode: initial_cfg.mode,
        is_alert: initial_cfg.is_alert,
        }),
        mi_show_cpu,
        mi_show_mem,
        mi_show_nw,
        mi_show_cpu_mem,
        mi_show_mem_nw,
        mi_show_cpu_nw,
        mi_show_all,
        mi_mode_list,
        mi_mode_rotation,
        mi_is_alert,
    };
    // 初期チェック同期
    ui_state.sync_menu_checks();
    app.manage(ui_state);

    let tray = TrayIconBuilder::with_id("tray-1")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "exit" {
                app.exit(0);
                return;
            }

            let ui_state = app.state::<TrayUiState>();

            // ロック内で状態更新とコピーを行い、ロック外でUI反映と保存を行う
            // こうしないとデッドロックやUIの更新漏れが発生するので注意
            let cfg_copy: TrayConfig = {
                let mut cfg = ui_state.config.lock().unwrap();

                match event.id.as_ref() {
                    "show_cpu" => { cfg.show_cpu = true;  cfg.show_mem = false; cfg.show_nw = false; }
                    "show_mem" => { cfg.show_cpu = false; cfg.show_mem = true;  cfg.show_nw = false; }
                    "show_nw"  => { cfg.show_cpu = false; cfg.show_mem = false; cfg.show_nw = true;  }
                    "show_cpu_mem" => { cfg.show_cpu = true;  cfg.show_mem = true;  cfg.show_nw = false; }
                    "show_mem_nw"  => { cfg.show_cpu = false; cfg.show_mem = true;  cfg.show_nw = true;  }
                    "show_cpu_nw"  => { cfg.show_cpu = true;  cfg.show_mem = false; cfg.show_nw = true;  }
                    "show_all" => { cfg.show_cpu = true; cfg.show_mem = true; cfg.show_nw = true; }
                    "mode_list" => { cfg.mode = DisplayMode::List; }
                    "mode_rotation" => { cfg.mode = DisplayMode::Rotation; }
                    "toggle_alert" => { cfg.is_alert = !cfg.is_alert; }
                    _ => {}
                }

                *cfg // ← ロック中にコピーして返す（ここでロック解放される）
            };

            // （ロック外） UI反映と保存
            ui_state.sync_menu_checks();
            save_config_async(app.app_handle().clone(), cfg_copy);
        })
        .title("Vital Monitor")
        .build(app)?;
    // app.manage(TrayState{...})
    app.manage(TrayState {
        tray: Mutex::new(tray),
    });
    Ok(())
}

fn save_config_async(app: tauri::AppHandle, cfg: TrayConfig) {
    tauri::async_runtime::spawn(async move {
        let Ok(dir) = app.path().app_config_dir() else { return; };
        let path = dir.join("tray_config.json");

        let Ok(json) = serde_json::to_string_pretty(&cfg) else { return; };

        let _ = fs::create_dir_all(&dir).await;
        let _ = fs::write(path, json).await;
    });
}