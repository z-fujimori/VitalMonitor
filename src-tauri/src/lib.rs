use crate::ui::{TrayUiState, TrayConfig, DisplayMode};
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
use std::sync::Mutex;

mod mac_metrics;
mod ui;

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
      let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
      let separator = PredefinedMenuItem::separator(app)?;
      let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

      // 表示操作メニュー
      let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
      let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      // --- 表示項目（チェックボックス） ---
      let mi_show_cpu = CheckMenuItem::with_id(app, "toggle_cpu", "CPU", true, true, None::<&str>)?;
      let mi_show_mem = CheckMenuItem::with_id(app, "toggle_mem", "MEM", true, true, None::<&str>)?;
      let mi_show_nw  = CheckMenuItem::with_id(app, "toggle_nw",  "NW",  true, false, None::<&str>)?;
      // --- 表示モード（ラジオ風） ---
      let mi_mode_list     = CheckMenuItem::with_id(app, "mode_list",     "List",     true, true, None::<&str>)?;
      let mi_mode_rotation = CheckMenuItem::with_id(app, "mode_rotation", "Rotation", true, false, None::<&str>)?;
      let mode_items: [&dyn tauri::menu::IsMenuItem<_>; 2] = [&mi_mode_list, &mi_mode_rotation];
      let mode_sub  = Submenu::with_items(app, "Display Mode", true, &mode_items)?;
      // --- アラート表示（チェックボックス） ---
      let mi_is_alert = CheckMenuItem::with_id(app, "toggle_alert", "Show Alert", true, true, None::<&str>)?;
      // --- Options サブメニュー ---
      let sep1 = PredefinedMenuItem::separator(app)?;
      let sep2 = PredefinedMenuItem::separator(app)?;
      let option_items: [&dyn tauri::menu::IsMenuItem<_>; 7] = [
        &mi_show_cpu,
        &mi_show_mem,
        &mi_show_nw,
        &sep1,
        &mode_sub,
        &sep2,
        &mi_is_alert
      ];
      let options_sub = Submenu::with_items(app, "Options", true, &option_items)?;
      // --- ルートメニュー ---
      let menu = Menu::with_items(
        app,
        &[
          &options_sub,
          &PredefinedMenuItem::separator(app)?,
          &hide_i,
          &quit_i,
        ],
      )?;
      // let menu = Menu::with_items(app, &[&hide_i, &separator, &quit_i])?;
      
      let ui_state = TrayUiState {
        config: Mutex::new(TrayConfig {
          show_cpu: true,
          show_mem: true,
          show_nw: false,
          mode: DisplayMode::List,
          is_alert: true,
        }),
        mi_show_cpu,
        mi_show_mem,
        mi_show_nw,
        mi_mode_list,
        mi_mode_rotation,
        mi_is_alert,
      };
      // 初期チェック同期
      ui_state.sync_menu_checks();
      app.manage(ui_state);
      
      // ✅ with_id を使う
      let tray = TrayIconBuilder::with_id("tray-1")
        .menu(&menu)
        .show_menu_on_left_click(true)
          .on_menu_event(|app, event| {
            match event.id.as_ref() {
              "quit" => {
                println!("quit menu item was clicked");
                app.exit(0);
              }
              "hide" => {
                println!("hide menu item was clicked");
                // app.hide().unwrap();
              }
              _ => {
                  println!("menu item {:?} not handled", event.id);
              }
            }

            let st = app.state::<TrayUiState>();
            {
              let mut cfg = st.config.lock().unwrap();
              match event.id.as_ref() {
                "toggle_cpu" => cfg.show_cpu = !cfg.show_cpu,
                "toggle_mem" => cfg.show_mem = !cfg.show_mem,
                "toggle_nw"  => cfg.show_nw  = !cfg.show_nw,
                "toggle_alert" => cfg.is_alert = !cfg.is_alert,
                "mode_list" => cfg.mode = DisplayMode::List,
                "mode_rotation" => cfg.mode = DisplayMode::Rotation,
                _ => return,
              }
            }
          st.sync_menu_checks();
        })
        // 文字（数値）をメニューバーに出す（macOSで有効）
        .title("...")
        // アイコン（無いと見つけづらいので一旦付ける）
        // .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
      
      app.manage(TrayState { tray: Mutex::new(tray) });
      
      #[cfg(target_os = "macos")]
      spawn_tray_updater(app.handle().clone());


      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

