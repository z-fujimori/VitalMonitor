/// Metric/AlertLevel/Percent/
/// Policy/AlertThresholds/CpuPolicy/MemoryPolicy/NetworkPolicy/
/// ReadError
/// MetricsSnapshot
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Percent(pub f32);
impl Percent {
    pub fn clamp_0_100(self) -> Self {
        Percent(self.0.clamp(0.0, 100.0))
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Millisecond(pub f32);

impl fmt::Display for Millisecond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0}", self.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub cpu_pct: Option<f32>,
    pub mem_pressure_pct: Option<f32>,
    pub nw_ms: Option<f64>,
}

pub type SharedMetrics = Arc<RwLock<MetricsSnapshot>>;
