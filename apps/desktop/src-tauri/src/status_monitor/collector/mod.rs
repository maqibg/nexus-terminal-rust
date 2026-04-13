mod linux;
mod macos;
mod windows;

use ssh_core::manager::SshSessionManager;
use tokio::time::Duration;

use crate::status_monitor::parser::{looks_like_metrics_output, parse_metrics_output};
use crate::status_monitor::types::RawMetrics;

const METRICS_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn collect_raw_metrics(
    ssh_manager: &SshSessionManager,
    session_id: &str,
) -> Result<RawMetrics, String> {
    let mut errors = Vec::new();
    for (name, script) in [
        ("linux", linux::SCRIPT),
        ("macos", macos::SCRIPT),
        ("windows", windows::SCRIPT),
    ] {
        match run_remote_metrics_command(ssh_manager, session_id, script).await {
            Ok(output) if looks_like_metrics_output(&output) => {
                return Ok(parse_metrics_output(&output));
            }
            Ok(output) if output.trim().contains("UNSUPPORTED_COLLECTOR=") => {
                errors.push(format!("{name} collector unsupported"));
            }
            Ok(_) => errors.push(format!("{name} metrics output empty")),
            Err(error) => errors.push(format!("{name} metrics failed: {error}")),
        }
    }
    Err(errors.join("; "))
}

async fn run_remote_metrics_command(
    ssh_manager: &SshSessionManager,
    session_id: &str,
    command: &str,
) -> Result<String, String> {
    let output = ssh_manager
        .exec_command(session_id, command, None, false, METRICS_COMMAND_TIMEOUT)
        .await?;
    if output.exit_code != 0 && output.stdout.trim().is_empty() {
        let stderr = output.stderr.trim();
        return Err(format!(
            "remote metrics command failed (exit={}): {}",
            output.exit_code,
            if stderr.is_empty() { "no stderr" } else { stderr }
        ));
    }
    Ok(output.stdout)
}
