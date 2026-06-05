//! HTTP transport for external capability providers.

use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::adapters::capability_provider_runtime::ProviderRequestEnvelope;
use crate::domain::capability_provider::HttpProviderTransport;

const PROVIDER_HTTP_TIMEOUT_SECS: u64 = 15;

pub(super) fn execute_http_call<T: DeserializeOwned>(
    transport: &HttpProviderTransport,
    envelope: &ProviderRequestEnvelope,
) -> Result<T, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(PROVIDER_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|error| format!("failed to build provider HTTP client: {error}"))?;
    let url = operation_url(&transport.endpoint_ref, envelope.operation_name());
    let response = client
        .post(url)
        .json(envelope)
        .send()
        .map_err(|error| format!("provider HTTP request failed: {error}"))?;
    let status = response.status();
    let body = response
        .bytes()
        .map_err(|error| format!("failed to read provider HTTP response: {error}"))?;
    parse_http_response_body(status, &body)
}

fn operation_url(endpoint_ref: &str, operation_name: &str) -> String {
    format!("{}/{}", endpoint_ref.trim_end_matches('/'), operation_name.trim_start_matches('/'))
}

fn parse_http_response_body<T: DeserializeOwned>(
    status: StatusCode,
    body: &[u8],
) -> Result<T, String> {
    if !status.is_success() {
        return Err(format!("provider HTTP response was not successful: {status}"));
    }
    serde_json::from_slice::<T>(body)
        .map_err(|error| format!("failed to parse provider HTTP response: {error}"))
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::{operation_url, parse_http_response_body};
    use crate::domain::capability_provider::{ProviderHealthSnapshot, ProviderReadinessState};

    #[test]
    fn operation_url_trims_duplicate_slashes() {
        assert_eq!(
            operation_url("http://localhost:7777/provider/", "/health"),
            "http://localhost:7777/provider/health"
        );
    }

    #[test]
    fn parse_http_response_body_surfaces_non_success_status() {
        let error = parse_http_response_body::<ProviderHealthSnapshot>(
            StatusCode::SERVICE_UNAVAILABLE,
            &[],
        )
        .err()
        .unwrap_or_default();
        assert!(error.contains("provider HTTP response was not successful"));
    }

    #[test]
    fn parse_http_response_body_parses_successful_response_body() {
        let response = parse_http_response_body::<ProviderHealthSnapshot>(
            StatusCode::OK,
            br#"{"provider_id":"demo-provider","readiness_state":"ready","checked_at":11}"#,
        );
        assert_eq!(
            response,
            Ok(ProviderHealthSnapshot {
                provider_id: "demo-provider".to_string(),
                readiness_state: ProviderReadinessState::Ready,
                missing_dependencies: Vec::new(),
                warnings: Vec::new(),
                runtime_environment: Vec::new(),
                checked_at: 11,
            })
        );
    }
}
