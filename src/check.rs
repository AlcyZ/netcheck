use std::{
    error::Error,
    time::{Duration, Instant},
};

use reqwest::{Client, StatusCode};

use crate::DynResult;

use serde::{Serialize, Serializer};

fn serialize_status_code<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(status.as_u16())
}

fn serialize_option_status_code<S>(
    opt: &Option<StatusCode>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(status) => serializer.serialize_some(&status.as_u16()),
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Debug)]
enum CheckTarget {
    Google,
    Example,
    IP,
}

#[derive(Serialize, Debug)]
enum CheckError {
    Timeout,
    DnsFailure,
    ConnectionRefused,
    TlsError,
    #[serde(serialize_with = "serialize_status_code")]
    HttpStatus(StatusCode),
    Other(String),
    InvalidRequest,
}

#[derive(Serialize, Debug)]
enum LatencySpeed {
    Slow,
    Ok,
}

impl LatencySpeed {
    fn new(results: Vec<&TargetResult>, latency_threshold: Option<usize>) -> LatencySpeed {
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

#[derive(Serialize, Debug)]
struct Latency {
    duration: Duration,
    speed: LatencySpeed,
}

impl Latency {
    fn from_duration(duration: Duration, threshold: Option<u128>) -> Latency {
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

#[derive(Serialize, Debug)]
struct TargetResult {
    target: CheckTarget,
    success: bool,
    latency: Latency,
    #[serde(serialize_with = "serialize_option_status_code")]
    status_code: Option<StatusCode>,
    error: Option<CheckError>,
}

impl TargetResult {
    fn new(
        target: CheckTarget,
        success: bool,
        latency: Latency,
        status_code: Option<StatusCode>,
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
}

#[derive(Serialize, Debug)]
pub struct InternetCheckResult {
    internet_up: bool,
    speed: LatencySpeed,
    results: Vec<TargetResult>,
    avg: Duration,
}

impl InternetCheckResult {
    fn new(
        internet_up: bool,
        speed: LatencySpeed,
        results: Vec<TargetResult>,
        avg: Duration,
    ) -> InternetCheckResult {
        InternetCheckResult {
            internet_up,
            speed,
            results,
            avg,
        }
    }

    pub fn is_internet_up(&self) -> bool {
        self.internet_up
    }
}

pub async fn check_connection(
    client: Client,
    latency_threshold: Option<u128>,
) -> InternetCheckResult {
    let (google, example, ip) = tokio::join!(
        check_target(CheckTarget::Google, client.clone(), latency_threshold),
        check_target(CheckTarget::Example, client.clone(), latency_threshold),
        check_target(CheckTarget::IP, client.clone(), latency_threshold),
    );
    let results = vec![google, example, ip];

    let internet_up = results.iter().any(|r| r.success);
    let speed = LatencySpeed::new(results.iter().collect(), Some(100));
    let avg = avg_durations(results.iter().map(|r| r.latency.get_duration()).collect());

    InternetCheckResult::new(internet_up, speed, results, avg)
}

async fn check_target(
    target: CheckTarget,
    client: Client,
    latency_threshold: Option<u128>,
) -> TargetResult {
    let start = Instant::now();
    let response = client.get(get_endpoint(&target)).send().await;
    let latency = start.elapsed();

    match response {
        Ok(res) => {
            let status = res.status();

            TargetResult::new(
                target,
                status.is_success(),
                Latency::from_duration(latency, latency_threshold),
                Some(status),
                if status.is_success() {
                    None
                } else {
                    Some(CheckError::HttpStatus(status))
                },
            )
        }
        Err(err) => {
            let error = classify_reqwest_error(err);
            TargetResult::new(
                target,
                false,
                Latency::from_duration(latency, latency_threshold),
                None,
                Some(error),
            )
        }
    }
}

fn get_endpoint(target: &CheckTarget) -> String {
    match target {
        CheckTarget::Google => String::from("https://google.com/generate_204"),
        CheckTarget::Example => String::from("https://example.com"),
        CheckTarget::IP => String::from("https://1.1.1.1"),
    }
}

fn get_client(timeout: Option<u64>) -> DynResult<Client> {
    let timeout_duration = timeout.unwrap_or(5);

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_duration))
        .build()?;

    Ok(client)
}

fn avg_durations(durations: Vec<&Duration>) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }

    let sum: Duration = durations.iter().copied().sum();
    sum / durations.len() as u32
}

fn classify_reqwest_error(err: reqwest::Error) -> CheckError {
    if err.is_timeout() {
        return CheckError::Timeout;
    }

    if err.is_connect() {
        if let Some(source) = err.source() {
            let msg = source.to_string().to_lowercase();

            if msg.contains("dns") || msg.contains("resolve") {
                return CheckError::DnsFailure;
            }

            if msg.contains("refused") {
                return CheckError::ConnectionRefused;
            }
        }

        return CheckError::ConnectionRefused;
    }

    if err.is_status() {
        if let Some(status) = err.status() {
            return CheckError::HttpStatus(status);
        }
    }

    if err.is_request() {
        return CheckError::InvalidRequest;
    }

    CheckError::Other(err.to_string())
}
