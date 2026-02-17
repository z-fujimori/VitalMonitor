/// Metric/AlertLevel/Percent/
/// Policy/AlertThresholds/CpuPolicy/MemoryPolicy/NetworkPolicy/
/// ReadError
/// MetricsSnapshot

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Percent(pub f32);
impl Percent {
    pub fn clamp_0_100(self) -> Self {
        Percent(self.0.clamp(0.0, 100.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Millisecond(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertLevel { Safe, Normal, Warning, Critical }

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


#[derive(Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub cpu_pct: Option<f32>,
    pub mem_pressure_pct: Option<f32>,
    pub nw_ms: Option<f64>,
}
impl MetricsSnapshot {
    pub fn classify(&self, policy: &Policies) -> ClassifiedSnapshot {
        ClassifiedSnapshot {
            cpu: self.cpu_pct
                .map(|v| Percent(v).clamp_0_100())
                .map(|p| Metric::classify(p, &policy.cpu)),
            mem: self.mem_pressure_pct
                .map(|v| Percent(v).clamp_0_100())
                .map(|p| Metric::classify(p, &policy.mem)),
            nw:  self.nw_ms
                .map(|v| Millisecond(v as f32))
                .map(|ms| Metric::classify(ms, &policy.nw)),
        }
    }
}
pub type SharedMetrics = Arc<RwLock<MetricsSnapshot>>;
