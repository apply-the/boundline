//! Command/stdio transport for external capability providers.

use std::io::Write;
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;

use crate::adapters::capability_provider_runtime::ProviderRequestEnvelope;
use crate::domain::capability_provider::CommandProviderTransport;

pub(super) fn execute_command_call<T: DeserializeOwned>(
    transport: &CommandProviderTransport,
    envelope: &ProviderRequestEnvelope,
) -> Result<T, String> {
    let mut command = Command::new(&transport.command_ref);
    command
        .args(&transport.args)
        .arg(envelope.operation_name())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(working_directory_ref) = &transport.working_directory_ref {
        command.current_dir(working_directory_ref);
    }

    let mut child =
        command.spawn().map_err(|error| format!("failed to spawn provider command: {error}"))?;
    let payload = serde_json::to_vec(envelope)
        .map_err(|error| format!("failed to encode provider request: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&payload)
            .map_err(|error| format!("failed to write provider request: {error}"))?;
    } else {
        return Err("provider command did not expose stdin".to_string());
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to read provider command output: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("provider command exited with status {}", output.status)
        } else {
            format!("provider command exited with status {}: {stderr}", output.status)
        };
        return Err(message);
    }

    serde_json::from_slice::<T>(&output.stdout)
        .map_err(|error| format!("failed to parse provider command response: {error}"))
}
