use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CheckTarget {
    Google,
    Example,
    IP,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CheckError {
    Timeout,
    DnsFailure,
    ConnectionRefused,
    TlsError,
    HttpStatus(u16),
    Other(String),
    InvalidRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LatencySpeed {
    Slow,
    Ok,
}

impl LatencySpeed {
    pub fn new(results: &[&TargetResult], latency_threshold: Option<usize>) -> LatencySpeed {
        if results.is_empty() {
            return LatencySpeed::Ok;
        }

        let threshold = latency_threshold.unwrap_or(500);

        let sum: usize = results
            .iter()
            .map(|r| r.latency.get_duration().as_millis() as usize)
            .sum();

        if (sum / results.len()) > threshold {
            LatencySpeed::Slow
        } else {
            LatencySpeed::Ok
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Latency {
    duration: Duration,
    speed: LatencySpeed,
}

impl Latency {
    pub fn from_duration(duration: Duration, threshold: Option<u128>) -> Latency {
        let treshold_value = threshold.unwrap_or(500);

        let speed = if duration.as_millis() > treshold_value {
            LatencySpeed::Slow
        } else {
            LatencySpeed::Ok
        };

        Latency { duration, speed }
    }

    fn get_duration(&self) -> &Duration {
        &self.duration
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TargetResult {
    target: CheckTarget,
    success: bool,
    latency: Latency,
    status_code: Option<u16>,
    error: Option<CheckError>,
}

impl TargetResult {
    pub fn new(
        target: CheckTarget,
        success: bool,
        latency: Latency,
        status_code: Option<u16>,
        error: Option<CheckError>,
    ) -> TargetResult {
        TargetResult {
            target,
            success,
            latency,
            status_code,
            error,
        }
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn latency_duration(&self) -> &Duration {
        &self.latency.duration
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    Online,
    Offline,
}

impl From<bool> for Connectivity {
    fn from(value: bool) -> Self {
        if value {
            Connectivity::Online
        } else {
            Connectivity::Offline
        }
    }
}

impl Display for Connectivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Connectivity::Online => write!(f, "Online"),
            Connectivity::Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InternetCheckResult {
    pub timestamp: DateTime<Utc>,
    connectivity: Connectivity,
    speed: LatencySpeed,
    results: Vec<TargetResult>,
    avg: Duration,
}

impl InternetCheckResult {
    pub fn new(
        connectivity: Connectivity,
        speed: LatencySpeed,
        results: Vec<TargetResult>,
        avg: Duration,
    ) -> InternetCheckResult {
        InternetCheckResult {
            timestamp: Utc::now(),
            connectivity,
            speed,
            results,
            avg,
        }
    }

    pub fn connectivity(&self) -> Connectivity {
        self.connectivity
    }

    pub fn get_time(&self) -> String {
        self.timestamp.format("%d.%m.%y - %H:%M").to_string()
    }
}
