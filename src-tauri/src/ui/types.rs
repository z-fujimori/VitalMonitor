use std::sync::Mutex;
use tauri::menu::CheckMenuItem;
use tauri::Wry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode { List, Rotation }

#[derive(Clone, Copy, Debug)]
pub struct TrayConfig {
  pub show_cpu: bool,
  pub show_mem: bool,
  pub show_nw: bool,
  pub mode: DisplayMode,
  pub is_alert: bool,
}

type CheckItem = CheckMenuItem<Wry>;

pub struct TrayUiState {
  // 真の状態（Mutexは1つ）
  pub config: Mutex<TrayConfig>,

  // UI ハンドル
  pub mi_show_cpu: CheckItem,
  pub mi_show_mem: CheckItem,
  pub mi_show_nw: CheckItem,

  pub mi_mode_list: CheckItem,
  pub mi_mode_rotation: CheckItem,
  pub mi_is_alert: CheckItem,
}

impl TrayUiState {
  pub fn sync_menu_checks(&self) {
    let cfg = *self.config.lock().unwrap();

    let _ = self.mi_show_cpu.set_checked(cfg.show_cpu);
    let _ = self.mi_show_mem.set_checked(cfg.show_mem);
    let _ = self.mi_show_nw.set_checked(cfg.show_nw);

    let _ = self.mi_mode_list.set_checked(cfg.mode == DisplayMode::List);
    let _ = self.mi_mode_rotation.set_checked(cfg.mode == DisplayMode::Rotation);

    let _ = self.mi_is_alert.set_checked(cfg.is_alert);
  }
}
