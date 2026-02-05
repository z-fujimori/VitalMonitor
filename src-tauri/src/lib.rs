use tauri::tray::TrayIconBuilder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // ✅ with_id を使う
      let _tray = TrayIconBuilder::with_id("tray-1")
        // 文字（数値）をメニューバーに出す（macOSで有効）
        .title("42")
        // アイコン（無いと見つけづらいので一旦付ける）
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
