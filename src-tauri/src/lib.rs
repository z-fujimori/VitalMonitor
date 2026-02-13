use tauri::{
  Manager,
  menu::{
    Menu,
    MenuItem,
    PredefinedMenuItem
  },
  tray::{
    TrayIcon,
    TrayIconBuilder
  }
};
use std::sync::Mutex;

mod mac_metrics;

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

      let mem_text = format!("Mem {:.0}%", pressure_pct);
      let cpu_text = format!("CPU {:.0}%", cpu_pct);

      let text = format!("{} {}", cpu_text, mem_text);

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
      let menu = Menu::with_items(app, &[&hide_i, &separator, &quit_i])?;
      // ✅ with_id を使う
      let tray = TrayIconBuilder::with_id("tray-1")
        .menu(&menu)
        .show_menu_on_left_click(true)
          .on_menu_event(|app, event| match event.id.as_ref() {
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
          })
        // 文字（数値）をメニューバーに出す（macOSで有効）
        .title("...")
        // アイコン（無いと見つけづらいので一旦付ける）
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
      
      app.manage(TrayState { tray: Mutex::new(tray) });
      
      #[cfg(target_os = "macos")]
      spawn_tray_updater(app.handle().clone());


      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

