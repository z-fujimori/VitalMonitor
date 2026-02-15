// Metric/AlertLevel/Percent/Bytes/Bps + ReadError

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Percent(pub f32);
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
pub struct Threshold2 {
    pub normal_lower_limit: f32,
    pub warning_lower_limit: f32,
    pub critical_lower_limit: f32,
}

impl Threshold2 {
    pub const fn new(normal: f32, warning: f32, critical: f32) -> Self {
        Self { normal_lower_limit: normal, warning_lower_limit: warning, critical_lower_limit: critical }
    }
}

pub fn level_by_threshold(value: f32, t: Threshold2) -> AlertLevel {
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
    pub thresholds: Threshold2,
}

impl Default for CpuPolicy {
    fn default() -> Self {
        Self {
            thresholds: Threshold2::new(50.0, 75.0, 90.0),
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
    pub thresholds: Threshold2,
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self {
            thresholds: Threshold2::new(60.0, 75.0, 90.0),
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
    pub thresholds: Threshold2,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            thresholds: Threshold2::new(50.0, 200.0, 450.0),
        }
    }
}

impl Policy<Millisecond> for NetworkPolicy {
    fn level(&self, value: &Millisecond) -> AlertLevel {
        level_by_threshold(value.0, self.thresholds)
    }
}

#[derive(Debug)]
pub enum ReadError {
    CommandSpawn(&'static str),
    NonZeroExit(&'static str),
    Utf8(std::string::FromUtf8Error),
    Parse(&'static str),
    Io(std::io::Error),
}

impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self { ReadError::Io(e) }
}
impl From<std::string::FromUtf8Error> for ReadError {
    fn from(e: std::string::FromUtf8Error) -> Self { ReadError::Utf8(e) }
}
