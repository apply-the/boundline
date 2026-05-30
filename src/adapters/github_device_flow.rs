use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;
use thiserror::Error;

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;
const JSON_ACCEPT_VALUE: &str = "application/json";

#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    DEFAULT_POLL_INTERVAL_SECS
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Error)]
pub enum DeviceFlowError {
    #[error("failed to request device code: {0}")]
    DeviceCodeRequest(String),
    #[error("device flow expired before user authorized")]
    Expired,
    #[error("device flow was denied by user")]
    AccessDenied,
    #[error("failed to poll for token: {0}")]
    PollRequest(String),
    #[error("unexpected error from GitHub: {0}")]
    Unexpected(String),
}

pub fn start_device_flow(client: &Client) -> Result<DeviceCodeResponse, DeviceFlowError> {
    start_device_flow_with_client_id(client, GITHUB_COPILOT_CLIENT_ID)
}

pub fn start_device_flow_with_client_id(
    client: &Client,
    client_id: &str,
) -> Result<DeviceCodeResponse, DeviceFlowError> {
    let response = client
        .post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", JSON_ACCEPT_VALUE)
        .form(&[("client_id", client_id), ("scope", "")])
        .send()
        .map_err(|e| DeviceFlowError::DeviceCodeRequest(e.to_string()))?;

    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        return Err(DeviceFlowError::DeviceCodeRequest(format!("HTTP {status}: {body}")));
    }

    response
        .json::<DeviceCodeResponse>()
        .map_err(|e| DeviceFlowError::DeviceCodeRequest(format!("invalid JSON: {e}")))
}

pub fn poll_for_token(
    client: &Client,
    device_code: &str,
    interval_secs: u64,
    expires_in_secs: u64,
) -> Result<String, DeviceFlowError> {
    poll_for_token_with_client_id(
        client,
        GITHUB_COPILOT_CLIENT_ID,
        device_code,
        interval_secs,
        expires_in_secs,
    )
}

pub fn poll_for_token_with_client_id(
    client: &Client,
    client_id: &str,
    device_code: &str,
    interval_secs: u64,
    expires_in_secs: u64,
) -> Result<String, DeviceFlowError> {
    let interval = Duration::from_secs(interval_secs);
    let max_attempts = expires_in_secs / interval_secs.max(1);

    for _ in 0..max_attempts {
        thread::sleep(interval);

        let response = client
            .post(GITHUB_OAUTH_TOKEN_URL)
            .header("Accept", JSON_ACCEPT_VALUE)
            .form(&[
                ("client_id", client_id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .map_err(|e| DeviceFlowError::PollRequest(e.to_string()))?;

        let parsed: OAuthTokenResponse = response
            .json()
            .map_err(|e| DeviceFlowError::PollRequest(format!("invalid JSON: {e}")))?;

        if let Some(token) = parsed.access_token.filter(|t| !t.trim().is_empty()) {
            return Ok(token);
        }

        match parsed.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                thread::sleep(Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS));
                continue;
            }
            Some("expired_token") => return Err(DeviceFlowError::Expired),
            Some("access_denied") => return Err(DeviceFlowError::AccessDenied),
            Some(other) => {
                return Err(DeviceFlowError::Unexpected(other.to_string()));
            }
            None => continue,
        }
    }

    Err(DeviceFlowError::Expired)
}

/// Result of a successful device-login flow.
pub struct DeviceLoginResult {
    pub token: String,
}

/// Execute the full device-login flow: request a device code, print user instructions,
/// and poll until the user authorizes or the flow expires.
pub fn execute_device_login() -> Result<DeviceLoginResult, DeviceFlowError> {
    let client = Client::new();
    let device_response = start_device_flow(&client)?;

    eprintln!(
        "Open {} in your browser and enter code: {}",
        device_response.verification_uri, device_response.user_code
    );

    let token = poll_for_token(
        &client,
        &device_response.device_code,
        device_response.interval,
        device_response.expires_in,
    )?;

    Ok(DeviceLoginResult { token })
}
