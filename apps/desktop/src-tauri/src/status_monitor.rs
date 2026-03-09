use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use ssh_core::manager::SshSessionManager;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration, MissedTickBehavior};

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(3);
const METRICS_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

const METRICS_SCRIPT_SH: &str = r#"sh -lc '
IP=$(hostname -I 2>/dev/null | awk "{print \$1}")
if [ -z "$IP" ]; then IP=$(hostname 2>/dev/null); fi

CPU_MODEL=$(grep -m1 "model name" /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed "s/^ *//")
if [ -z "$CPU_MODEL" ]; then
  CPU_MODEL=$(lscpu 2>/dev/null | awk -F: "/Model name:/{sub(/^[[:space:]]+/,\"\",\$2); print \$2; exit}")
fi
if [ -z "$CPU_MODEL" ]; then CPU_MODEL=$(uname -m 2>/dev/null); fi

OS_NAME=$(awk -F= "/^PRETTY_NAME=/{gsub(/\"/,\"\",\$2); print \$2; exit}" /etc/os-release 2>/dev/null)
if [ -z "$OS_NAME" ]; then OS_NAME=$(uname -sr 2>/dev/null); fi

MEM_TOTAL=$(free -m 2>/dev/null | awk "/^Mem:/{print \$2+0}")
MEM_USED=$(free -m 2>/dev/null | awk "/^Mem:/{print \$3+0}")
SWAP_TOTAL=$(free -m 2>/dev/null | awk "/^Swap:/{print \$2+0}")
SWAP_USED=$(free -m 2>/dev/null | awk "/^Swap:/{print \$3+0}")

DISK_TOTAL=$(df -kP / 2>/dev/null | awk "NR==2{print \$2+0}")
DISK_USED=$(df -kP / 2>/dev/null | awk "NR==2{print \$3+0}")

CPU_TOTAL_TICKS=0
CPU_IDLE_TICKS=0
if [ -r /proc/stat ]; then
  CPU_LINE=$(awk "/^cpu /{print \$2\" \"\$3\" \"\$4\" \"\$5\" \"\$6\" \"\$7\" \"\$8\" \"\$9; exit}" /proc/stat 2>/dev/null)
  if [ -n "$CPU_LINE" ]; then
    set -- $CPU_LINE
    CPU_TOTAL_TICKS=$(( ${1:-0} + ${2:-0} + ${3:-0} + ${4:-0} + ${5:-0} + ${6:-0} + ${7:-0} + ${8:-0} ))
    CPU_IDLE_TICKS=$(( ${4:-0} + ${5:-0} ))
  fi
fi

NET_IF=$(ip route get 1.1.1.1 2>/dev/null | awk "{for(i=1;i<=NF;i++) if(\$i==\"dev\"){print \$(i+1); exit}}")
if [ -z "$NET_IF" ]; then
  NET_IF=$(awk -F: "NR>2{gsub(/ /,\"\",\$1); if(\$1!=\"lo\"){print \$1; exit}}" /proc/net/dev 2>/dev/null)
fi

RX_TOTAL=0
TX_TOTAL=0
if [ -n "$NET_IF" ] && [ -r "/sys/class/net/$NET_IF/statistics/rx_bytes" ]; then
  RX_TOTAL=$(cat "/sys/class/net/$NET_IF/statistics/rx_bytes" 2>/dev/null)
  TX_TOTAL=$(cat "/sys/class/net/$NET_IF/statistics/tx_bytes" 2>/dev/null)
else
  RX_TOTAL=$(cat /sys/class/net/*/statistics/rx_bytes 2>/dev/null | awk "{s+=\$1} END{print s+0}")
  TX_TOTAL=$(cat /sys/class/net/*/statistics/tx_bytes 2>/dev/null | awk "{s+=\$1} END{print s+0}")
fi

echo "ip_address=${IP:-}"
echo "cpu_model=${CPU_MODEL:-}"
echo "os_name=${OS_NAME:-}"
echo "cpu_percent=0"
echo "cpu_total_ticks=${CPU_TOTAL_TICKS:-0}"
echo "cpu_idle_ticks=${CPU_IDLE_TICKS:-0}"
echo "mem_total_mb=${MEM_TOTAL:-0}"
echo "mem_used_mb=${MEM_USED:-0}"
echo "swap_total_mb=${SWAP_TOTAL:-0}"
echo "swap_used_mb=${SWAP_USED:-0}"
echo "disk_total_kb=${DISK_TOTAL:-0}"
echo "disk_used_kb=${DISK_USED:-0}"
echo "net_interface=${NET_IF:-}"
echo "net_rx_total=${RX_TOTAL:-0}"
echo "net_tx_total=${TX_TOTAL:-0}"
'"#;

const METRICS_SCRIPT_POWERSHELL: &str = r#"powershell -NoProfile -Command "$ErrorActionPreference='SilentlyContinue'; $ip=(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike '169.254*' -and $_.IPAddress -ne '127.0.0.1' } | Select-Object -First 1 -ExpandProperty IPAddress); if (-not $ip) { $ip=(hostname) }; $cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name); $os=(Get-CimInstance Win32_OperatingSystem | Select-Object -First 1 -ExpandProperty Caption); $cpuPct=[double]((Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average); $osInfo=Get-CimInstance Win32_OperatingSystem; $memTotal=[int]([math]::Round($osInfo.TotalVisibleMemorySize / 1024)); $memFree=[int]([math]::Round($osInfo.FreePhysicalMemory / 1024)); $memUsed=[int]([math]::Max(0, $memTotal - $memFree)); $disk=Get-CimInstance Win32_LogicalDisk -Filter 'DeviceID=''C:'''; if (-not $disk) { $disk=Get-CimInstance Win32_LogicalDisk | Select-Object -First 1 }; $diskSize=[double]($disk | Select-Object -First 1 -ExpandProperty Size); $diskFree=[double]($disk | Select-Object -First 1 -ExpandProperty FreeSpace); $diskTotal=[int64]([math]::Round($diskSize / 1KB)); $diskUsed=[int64]([math]::Round(($diskSize - $diskFree) / 1KB)); $net=Get-NetAdapterStatistics | Where-Object { $_.Name -notmatch 'Loopback' } | Select-Object -First 1; $netIf=''; $rx=0; $tx=0; if ($net) { $netIf=$net.Name; $rx=[int64]$net.ReceivedBytes; $tx=[int64]$net.SentBytes; }; Write-Output ('ip_address=' + $ip); Write-Output ('cpu_model=' + $cpu); Write-Output ('os_name=' + $os); Write-Output ('cpu_percent=' + [math]::Round($cpuPct, 1)); Write-Output 'cpu_total_ticks=0'; Write-Output 'cpu_idle_ticks=0'; Write-Output ('mem_total_mb=' + $memTotal); Write-Output ('mem_used_mb=' + $memUsed); Write-Output 'swap_total_mb=0'; Write-Output 'swap_used_mb=0'; Write-Output ('disk_total_kb=' + $diskTotal); Write-Output ('disk_used_kb=' + $diskUsed); Write-Output ('net_interface=' + $netIf); Write-Output ('net_rx_total=' + $rx); Write-Output ('net_tx_total=' + $tx)""#;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusUpdatePayload {
    pub session_id: String,
    pub connection_id: i64,
    pub timestamp: i64,
    pub ip_address: String,
    pub cpu_model: String,
    pub os_name: String,
    pub cpu_percent: f64,
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_percent: f64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub swap_percent: f64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_percent: f64,
    pub net_interface: String,
    pub net_rx_total: u64,
    pub net_tx_total: u64,
    pub net_rx_rate: u64,
    pub net_tx_rate: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StatusErrorPayload {
    session_id: String,
    message: String,
    timestamp: i64,
}

#[derive(Debug, Default)]
struct RawMetrics {
    ip_address: String,
    cpu_model: String,
    os_name: String,
    cpu_percent: f64,
    cpu_total_ticks: u64,
    cpu_idle_ticks: u64,
    mem_used_mb: u64,
    mem_total_mb: u64,
    swap_used_mb: u64,
    swap_total_mb: u64,
    disk_used_kb: u64,
    disk_total_kb: u64,
    net_interface: String,
    net_rx_total: u64,
    net_tx_total: u64,
}

#[derive(Debug, Clone, Copy)]
struct PreviousCpuSample {
    total_ticks: u64,
    idle_ticks: u64,
    timestamp: i64,
}

#[derive(Debug, Clone, Copy)]
struct PreviousNetSample {
    rx_total: u64,
    tx_total: u64,
    timestamp: i64,
}

#[derive(Clone)]
pub struct StatusMonitorService {
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    poll_interval: Duration,
}

impl StatusMonitorService {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }

    pub async fn start_session(
        &self,
        session_id: String,
        ssh_manager: SshSessionManager,
        app_handle: tauri::AppHandle,
        poll_interval_override: Option<Duration>,
    ) {
        self.prune_finished().await;
        self.stop_session(&session_id).await;

        let poll_interval = poll_interval_override.unwrap_or(self.poll_interval);
        let sid = session_id.clone();
        let task = tokio::spawn(async move {
            let mut ticker = interval(poll_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut previous_cpu: Option<PreviousCpuSample> = None;
            let mut previous_net: Option<PreviousNetSample> = None;

            loop {
                ticker.tick().await;

                if !ssh_manager.has_session(&sid).await {
                    break;
                }

                match collect_status_with_prev(
                    &ssh_manager,
                    &sid,
                    &mut previous_cpu,
                    &mut previous_net,
                )
                .await
                {
                    Ok(status) => {
                        let _ = app_handle.emit(&format!("status:update:{sid}"), &status);
                    }
                    Err(err) => {
                        let payload = StatusErrorPayload {
                            session_id: sid.clone(),
                            message: err.clone(),
                            timestamp: now_timestamp_ms(),
                        };
                        let _ = app_handle.emit(&format!("status:error:{sid}"), &payload);

                        if err.contains("session not found") {
                            break;
                        }
                    }
                }
            }
        });

        self.tasks.lock().await.insert(session_id, task);
    }

    pub async fn stop_session(&self, session_id: &str) {
        if let Some(task) = self.tasks.lock().await.remove(session_id) {
            task.abort();
        }
    }

    async fn prune_finished(&self) {
        self.tasks
            .lock()
            .await
            .retain(|_, task| !task.is_finished());
    }
}

pub async fn collect_status_once(
    ssh_manager: &SshSessionManager,
    session_id: &str,
) -> Result<StatusUpdatePayload, String> {
    let mut previous_cpu = None;
    let mut previous_net = None;
    collect_status_with_prev(
        ssh_manager,
        session_id,
        &mut previous_cpu,
        &mut previous_net,
    )
    .await
}

async fn collect_status_with_prev(
    ssh_manager: &SshSessionManager,
    session_id: &str,
    previous_cpu: &mut Option<PreviousCpuSample>,
    previous_net: &mut Option<PreviousNetSample>,
) -> Result<StatusUpdatePayload, String> {
    let connection_id = ssh_manager
        .get_connection_id(session_id)
        .await
        .ok_or_else(|| "session not found".to_string())?;

    let raw = collect_raw_metrics(ssh_manager, session_id).await?;
    let timestamp = now_timestamp_ms();

    let cpu_percent = calc_cpu_percent(&raw, previous_cpu, timestamp);
    let (net_rx_rate, net_tx_rate) = calc_network_rate(&raw, previous_net, timestamp);

    Ok(StatusUpdatePayload {
        session_id: session_id.to_string(),
        connection_id,
        timestamp,
        ip_address: raw.ip_address,
        cpu_model: raw.cpu_model,
        os_name: raw.os_name,
        cpu_percent,
        mem_used: raw.mem_used_mb,
        mem_total: raw.mem_total_mb,
        mem_percent: calc_percent(raw.mem_used_mb, raw.mem_total_mb),
        swap_used: raw.swap_used_mb,
        swap_total: raw.swap_total_mb,
        swap_percent: calc_percent(raw.swap_used_mb, raw.swap_total_mb),
        disk_used: raw.disk_used_kb,
        disk_total: raw.disk_total_kb,
        disk_percent: calc_percent(raw.disk_used_kb, raw.disk_total_kb),
        net_interface: raw.net_interface,
        net_rx_total: raw.net_rx_total,
        net_tx_total: raw.net_tx_total,
        net_rx_rate,
        net_tx_rate,
    })
}

fn calc_cpu_percent(
    raw: &RawMetrics,
    previous_cpu: &mut Option<PreviousCpuSample>,
    timestamp: i64,
) -> f64 {
    let fallback = round_one_decimal(raw.cpu_percent.clamp(0.0, 100.0));

    if raw.cpu_total_ticks == 0 {
        return fallback;
    }

    let next_sample = PreviousCpuSample {
        total_ticks: raw.cpu_total_ticks,
        idle_ticks: raw.cpu_idle_ticks,
        timestamp,
    };

    let percent = if let Some(prev) = previous_cpu {
        let elapsed_ms = timestamp.saturating_sub(prev.timestamp);
        let total_diff = raw.cpu_total_ticks.saturating_sub(prev.total_ticks);
        let idle_diff = raw.cpu_idle_ticks.saturating_sub(prev.idle_ticks);

        if elapsed_ms > 100 && total_diff > 0 {
            let usage_ratio = 1.0 - (idle_diff as f64 / total_diff as f64);
            round_one_decimal((usage_ratio * 100.0).clamp(0.0, 100.0))
        } else {
            fallback
        }
    } else {
        fallback
    };

    *previous_cpu = Some(next_sample);
    percent
}

fn calc_network_rate(
    raw: &RawMetrics,
    previous_net: &mut Option<PreviousNetSample>,
    timestamp: i64,
) -> (u64, u64) {
    let rates = if let Some(prev) = previous_net {
        let elapsed_ms = timestamp.saturating_sub(prev.timestamp);
        if elapsed_ms > 100 {
            let elapsed_seconds = elapsed_ms as f64 / 1000.0;
            let rx_delta = raw.net_rx_total.saturating_sub(prev.rx_total) as f64;
            let tx_delta = raw.net_tx_total.saturating_sub(prev.tx_total) as f64;
            (
                (rx_delta / elapsed_seconds).max(0.0).round() as u64,
                (tx_delta / elapsed_seconds).max(0.0).round() as u64,
            )
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    *previous_net = Some(PreviousNetSample {
        rx_total: raw.net_rx_total,
        tx_total: raw.net_tx_total,
        timestamp,
    });

    rates
}

async fn collect_raw_metrics(
    ssh_manager: &SshSessionManager,
    session_id: &str,
) -> Result<RawMetrics, String> {
    let mut errors = Vec::new();

    match run_remote_metrics_command(ssh_manager, session_id, METRICS_SCRIPT_SH).await {
        Ok(output) if looks_like_metrics_output(&output) => {
            return Ok(parse_metrics_output(&output))
        }
        Ok(_) => errors.push("sh metrics output empty".to_string()),
        Err(err) => errors.push(format!("sh metrics failed: {err}")),
    }

    match run_remote_metrics_command(ssh_manager, session_id, METRICS_SCRIPT_POWERSHELL).await {
        Ok(output) if looks_like_metrics_output(&output) => {
            return Ok(parse_metrics_output(&output))
        }
        Ok(_) => errors.push("powershell metrics output empty".to_string()),
        Err(err) => errors.push(format!("powershell metrics failed: {err}")),
    }

    Err(errors.join("; "))
}

async fn run_remote_metrics_command(
    ssh_manager: &SshSessionManager,
    session_id: &str,
    command: &str,
) -> Result<String, String> {
    let output = ssh_manager
        .exec_command(session_id, command, METRICS_COMMAND_TIMEOUT)
        .await?;

    if output.exit_code != 0 && output.stdout.trim().is_empty() {
        let stderr = output.stderr.trim();
        return Err(format!(
            "remote metrics command failed (exit={}): {}",
            output.exit_code,
            if stderr.is_empty() {
                "no stderr"
            } else {
                stderr
            }
        ));
    }

    Ok(output.stdout)
}

fn looks_like_metrics_output(output: &str) -> bool {
    output.lines().any(|line| {
        line.starts_with("cpu_total_ticks=")
            || line.starts_with("cpu_percent=")
            || line.starts_with("mem_total_mb=")
            || line.starts_with("disk_total_kb=")
            || line.starts_with("net_rx_total=")
    })
}

fn parse_metrics_output(output: &str) -> RawMetrics {
    let mut raw = RawMetrics::default();

    for line in output.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim();

        match key {
            "ip_address" => raw.ip_address = value.to_string(),
            "cpu_model" => raw.cpu_model = value.to_string(),
            "os_name" => raw.os_name = value.to_string(),
            "cpu_percent" => raw.cpu_percent = parse_f64(value),
            "cpu_total_ticks" => raw.cpu_total_ticks = parse_u64(value),
            "cpu_idle_ticks" => raw.cpu_idle_ticks = parse_u64(value),
            "mem_total_mb" => raw.mem_total_mb = parse_u64(value),
            "mem_used_mb" => raw.mem_used_mb = parse_u64(value),
            "swap_total_mb" => raw.swap_total_mb = parse_u64(value),
            "swap_used_mb" => raw.swap_used_mb = parse_u64(value),
            "disk_total_kb" => raw.disk_total_kb = parse_u64(value),
            "disk_used_kb" => raw.disk_used_kb = parse_u64(value),
            "net_interface" => raw.net_interface = value.to_string(),
            "net_rx_total" => raw.net_rx_total = parse_u64(value),
            "net_tx_total" => raw.net_tx_total = parse_u64(value),
            _ => {}
        }
    }

    raw
}

fn parse_u64(value: &str) -> u64 {
    value.parse::<u64>().unwrap_or(0)
}

fn parse_f64(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}

fn calc_percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    let percent = (used as f64 / total as f64) * 100.0;
    round_one_decimal(percent.clamp(0.0, 100.0))
}

fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn now_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
