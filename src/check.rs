use std::{
    borrow::Borrow,
    error::Error,
    time::{Duration, Instant},
};

use reqwest::Client;

use crate::model::{
    CheckError, CheckTarget, InternetCheckResult, Latency, LatencySpeed, TargetResult,
};

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

    let internet_up = results.iter().any(|r| r.success());

    let speed = LatencySpeed::new(&results.iter().collect::<Vec<_>>(), None);
    let avg = avg_durations(results.iter().map(|r| r.latency_duration()));

    InternetCheckResult::new(internet_up.into(), speed, results, avg)
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
                Some(status.as_u16()),
                if status.is_success() {
                    None
                } else {
                    Some(CheckError::HttpStatus(status.as_u16()))
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

fn avg_durations<I>(durations: I) -> Duration
where
    I: IntoIterator,
    I::Item: Borrow<Duration>,
{
    let mut total = Duration::ZERO;
    let mut count = 0u32;

    for d in durations {
        total += *d.borrow();
        count += 1;
    }

    if count == 0 {
        return Duration::ZERO;
    }

    total / count
}

fn classify_reqwest_error(err: reqwest::Error) -> CheckError {
    if err.is_timeout() {
        return CheckError::Timeout;
    }

    if err.is_builder()
        || err.to_string().to_lowercase().contains("tls")
        || err.to_string().to_lowercase().contains("certificate")
    {
        return CheckError::TlsError;
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

            if msg.contains("ssl") || msg.contains("tls") || msg.contains("certificate") {
                return CheckError::TlsError;
            }
        }

        return CheckError::ConnectionRefused;
    }

    if err.is_status()
        && let Some(status) = err.status()
    {
        return CheckError::HttpStatus(status.as_u16());
    }

    if err.is_request() {
        return CheckError::InvalidRequest;
    }

    CheckError::Other(err.to_string())
}
