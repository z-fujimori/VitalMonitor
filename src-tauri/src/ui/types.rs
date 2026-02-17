use std::sync::Mutex;
use tauri::menu::CheckMenuItem;
use tauri::Wry;

use crate::metrics::types::{Percent, Millisecond, MetricsSnapshot};

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
  pub mi_show_cpu_mem: CheckItem,
  pub mi_show_cpu_nw: CheckItem,
  pub mi_show_mem_nw: CheckItem,
  pub mi_show_all: CheckItem,

  pub mi_mode_list: CheckItem,
  pub mi_mode_rotation: CheckItem,
  pub mi_is_alert: CheckItem,
}

impl TrayUiState {
  pub fn sync_menu_checks(&self) {
    let cfg = *self.config.lock().unwrap();

    let _ = self.mi_show_cpu.set_checked(false);
    let _ = self.mi_show_mem.set_checked(false);
    let _ = self.mi_show_nw.set_checked(false);
    let _ = self.mi_show_cpu_mem.set_checked(false);
    let _ = self.mi_show_cpu_nw.set_checked(false);
    let _ = self.mi_show_mem_nw.set_checked(false);
    let _ = self.mi_show_all.set_checked(false);
    match (cfg.show_cpu, cfg.show_mem, cfg.show_nw) {
      (true, false, false) => { let _ = self.mi_show_cpu.set_checked(true); },
      (false, true, false) => { let _ = self.mi_show_mem.set_checked(true); },
      (false, false, true) => { let _ = self.mi_show_nw.set_checked(true); },
      (true, true, false) => { let _ = self.mi_show_cpu_mem.set_checked(true); },
      (true, false, true) => { let _ = self.mi_show_cpu_nw.set_checked(true); },
      (false, true, true) => { let _ = self.mi_show_mem_nw.set_checked(true); },
      (true, true, true) => { let _ = self.mi_show_all.set_checked(true); },
      _ => {},
    }

    let _ = self.mi_mode_list.set_checked(cfg.mode == DisplayMode::List);
    let _ = self.mi_mode_rotation.set_checked(cfg.mode == DisplayMode::Rotation);

    let _ = self.mi_is_alert.set_checked(cfg.is_alert);
  }
}

/// 
///  
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertLevel { Safe, Normal, Warning, Critical }

impl AlertLevel {
    pub fn icon(self) -> &'static str {
        match self {
            AlertLevel::Safe => "🔵",
            AlertLevel::Normal => "🟢",
            AlertLevel::Warning => "🟤",
            AlertLevel::Critical => "🔴",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metric<V> {
    pub value: V,
    pub level: AlertLevel,
}

impl<V> Metric<V> {
    pub fn new(value: V, level: AlertLevel) -> Self {
        Self { value, level }
    }

    pub fn classify<P: Policy<V>>(value: V, policy: &P) -> Self {
        let level = policy.level(&value);
        Self { value, level }
    }
}


pub trait Policy<V> {
    fn level(&self, value: &V) -> AlertLevel;
}

#[derive(Clone, Copy, Debug)]
pub struct AlertThresholds {
    pub normal_lower_limit: f32,
    pub warning_lower_limit: f32,
    pub critical_lower_limit: f32,
}

impl AlertThresholds {
    pub const fn new(normal: f32, warning: f32, critical: f32) -> Self {
        Self { normal_lower_limit: normal, warning_lower_limit: warning, critical_lower_limit: critical }
    }
}

pub fn level_by_threshold(value: f32, t: AlertThresholds) -> AlertLevel {
    if value < t.normal_lower_limit {
        AlertLevel::Safe
    } else if value < t.warning_lower_limit {
        AlertLevel::Normal
    } else if value < t.critical_lower_limit {
        AlertLevel::Warning
    } else {
        AlertLevel::Critical
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CpuPolicy {
    pub thresholds: AlertThresholds,
}

impl Default for CpuPolicy {
    fn default() -> Self {
        Self {
            thresholds: AlertThresholds::new(50.0, 75.0, 90.0),
        }
    }
}

impl Policy<Percent> for CpuPolicy {
    fn level(&self, value: &Percent) -> AlertLevel {
        level_by_threshold(value.0, self.thresholds)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryPolicy {
    pub thresholds: AlertThresholds,
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self {
            thresholds: AlertThresholds::new(60.0, 75.0, 90.0),
        }
    }
}

impl Policy<Percent> for MemoryPolicy {
    fn level(&self, value: &Percent) -> AlertLevel {
        level_by_threshold(value.0, self.thresholds)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NetworkPolicy {
    pub thresholds: AlertThresholds,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            thresholds: AlertThresholds::new(50.0, 200.0, 450.0),
        }
    }
}

impl Policy<Millisecond> for NetworkPolicy {
    fn level(&self, value: &Millisecond) -> AlertLevel {
        level_by_threshold(value.0, self.thresholds)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Policies {
    pub cpu: CpuPolicy,
    pub mem: MemoryPolicy,
    pub nw: NetworkPolicy,
}

#[derive(Clone, Debug, Default)]
pub struct ClassifiedSnapshot {
    pub cpu: Option<Metric<Percent>>,
    pub mem: Option<Metric<Percent>>,
    pub nw: Option<Metric<Millisecond>>,
}
impl ClassifiedSnapshot {
    pub fn new(snapshot: MetricsSnapshot) -> Self {
        let policies = Policies::default();
        Self {
            cpu: snapshot.cpu_pct.map(|v| Metric::classify(Percent(v), &policies.cpu)),
            mem: snapshot.mem_pressure_pct.map(|v| Metric::classify(Percent(v), &policies.mem)),
            nw: snapshot.nw_ms.map(|v| Metric::classify(Millisecond(v as f32), &policies.nw)),
        }
    }
}

