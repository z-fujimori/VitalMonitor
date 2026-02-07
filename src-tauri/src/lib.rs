use tauri::{Manager,menu::{Menu,MenuItem,PredefinedMenuItem},tray::{TrayIcon,TrayIconBuilder}};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  struct TrayState(TrayIcon);
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
              app.hide().unwrap();
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
          })
        // 文字（数値）をメニューバーに出す（macOSで有効）
        .title("42")
        // アイコン（無いと見つけづらいので一旦付ける）
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
      
      app.manage(TrayState(tray));

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
