use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DurationUnit {
    Ns,
    Us,
    Ms,
    S,
    Min,
    H,
    D,
}

impl DurationUnit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ns" => Some(Self::Ns),
            "us" => Some(Self::Us),
            "ms" => Some(Self::Ms),
            "s" => Some(Self::S),
            "min" => Some(Self::Min),
            "h" => Some(Self::H),
            "d" => Some(Self::D),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ns => "ns",
            Self::Us => "us",
            Self::Ms => "ms",
            Self::S => "s",
            Self::Min => "min",
            Self::H => "h",
            Self::D => "d",
        }
    }

    /// Convert the duration value to nanoseconds.
    pub fn to_nanos(&self, value: f64) -> f64 {
        match self {
            Self::Ns => value,
            Self::Us => value * 1_000.0,
            Self::Ms => value * 1_000_000.0,
            Self::S => value * 1_000_000_000.0,
            Self::Min => value * 60_000_000_000.0,
            Self::H => value * 3_600_000_000_000.0,
            Self::D => value * 86_400_000_000_000.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Duration {
    pub value: f64,
    pub unit: DurationUnit,
}

impl Duration {
    pub fn new(value: f64, unit: DurationUnit) -> Self {
        Self { value, unit }
    }

    pub fn to_nanos(&self) -> f64 {
        self.unit.to_nanos(self.value)
    }

    pub fn to_std(&self) -> std::time::Duration {
        std::time::Duration::from_nanos(self.to_nanos() as u64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSizeUnit {
    B,
    Kb,
    Mb,
    Gb,
    Tb,
    Pb,
    Kib,
    Mib,
    Gib,
    Tib,
    Pib,
}

impl DataSizeUnit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "b" => Some(Self::B),
            "kb" => Some(Self::Kb),
            "mb" => Some(Self::Mb),
            "gb" => Some(Self::Gb),
            "tb" => Some(Self::Tb),
            "pb" => Some(Self::Pb),
            "kib" => Some(Self::Kib),
            "mib" => Some(Self::Mib),
            "gib" => Some(Self::Gib),
            "tib" => Some(Self::Tib),
            "pib" => Some(Self::Pib),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::B => "b",
            Self::Kb => "kb",
            Self::Mb => "mb",
            Self::Gb => "gb",
            Self::Tb => "tb",
            Self::Pb => "pb",
            Self::Kib => "kib",
            Self::Mib => "mib",
            Self::Gib => "gib",
            Self::Tib => "tib",
            Self::Pib => "pib",
        }
    }

    pub fn to_bytes(&self, value: f64) -> f64 {
        match self {
            Self::B => value,
            Self::Kb => value * 1_000.0,
            Self::Mb => value * 1_000_000.0,
            Self::Gb => value * 1_000_000_000.0,
            Self::Tb => value * 1_000_000_000_000.0,
            Self::Pb => value * 1_000_000_000_000_000.0,
            Self::Kib => value * 1_024.0,
            Self::Mib => value * 1_048_576.0,
            Self::Gib => value * 1_073_741_824.0,
            Self::Tib => value * 1_099_511_627_776.0,
            Self::Pib => value * 1_125_899_906_842_624.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSize {
    pub value: f64,
    pub unit: DataSizeUnit,
}

impl DataSize {
    pub fn new(value: f64, unit: DataSizeUnit) -> Self {
        Self { value, unit }
    }

    pub fn to_bytes(&self) -> f64 {
        self.unit.to_bytes(self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pair<A, B> {
    pub first: A,
    pub second: B,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntSeq {
    pub start: i64,
    pub end: i64,
    pub step: i64,
}

impl IntSeq {
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        Self { start, end, step }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PklRegex {
    pub pattern: String,
}
