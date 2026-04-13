use std::time::{SystemTime, UNIX_EPOCH};

use ssh_core::manager::SshSessionManager;

use crate::status_monitor::collector::collect_raw_metrics;
use crate::status_monitor::types::{
    DiskUsageEntry, NetInterfaceEntry, PreviousCpuSample, PreviousNetSample, RawMetrics,
    StatusProcessEntry, StatusUpdatePayload,
};

pub async fn collect_status_once(
    ssh_manager: &SshSessionManager,
    session_id: &str,
) -> Result<StatusUpdatePayload, String> {
    let mut previous_cpu = None;
    let mut previous_net = None;
    collect_status_with_prev(ssh_manager, session_id, &mut previous_cpu, &mut previous_net).await
}

pub async fn collect_status_with_prev(
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
    let (cpu_percent, cpu_per_core) = calc_cpu_metrics(&raw, previous_cpu, timestamp);
    let (net_rx_rate, net_tx_rate, net_interfaces) =
        calc_network_metrics(&raw, previous_net, timestamp);

    Ok(StatusUpdatePayload {
        session_id: session_id.to_string(),
        connection_id,
        timestamp,
        ip_address: raw.ip_address,
        cpu_model: raw.cpu_model,
        os_name: raw.os_name,
        cpu_percent,
        cpu_cores: raw
            .cpu_cores
            .or_else(|| Some(cpu_per_core.len() as u32).filter(|count| *count > 0)),
        cpu_per_core,
        mem_used: raw.mem_used_mb,
        mem_total: raw.mem_total_mb,
        mem_percent: calc_percent(raw.mem_used_mb, raw.mem_total_mb),
        mem_free: raw.mem_free_mb,
        mem_buffers: raw.mem_buffers_mb,
        mem_cached: raw.mem_cached_mb,
        swap_used: raw.swap_used_mb,
        swap_total: raw.swap_total_mb,
        swap_percent: calc_percent(raw.swap_used_mb, raw.swap_total_mb),
        disk_used: raw.disk_used_kb,
        disk_total: raw.disk_total_kb,
        disk_percent: calc_percent(raw.disk_used_kb, raw.disk_total_kb),
        disks: raw
            .disks
            .iter()
            .map(|(name, total_kb, used_kb)| DiskUsageEntry {
                name: name.clone(),
                used_kb: *used_kb,
                total_kb: *total_kb,
                percent: calc_percent(*used_kb, *total_kb),
            })
            .collect(),
        top_processes: raw
            .top_processes
            .iter()
            .map(|item| StatusProcessEntry {
                pid: item.pid.clone(),
                mem_percent: round_one_decimal(item.mem_percent),
                command: item.command.clone(),
            })
            .collect(),
        net_interface: raw.net_interface.clone(),
        net_rx_total: raw.net_rx_total,
        net_tx_total: raw.net_tx_total,
        net_rx_rate,
        net_tx_rate,
        net_interfaces,
    })
}

pub fn now_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn calc_cpu_metrics(
    raw: &RawMetrics,
    previous_cpu: &mut Option<PreviousCpuSample>,
    timestamp: i64,
) -> (f64, Vec<f64>) {
    let fallback = round_one_decimal(raw.cpu_percent.clamp(0.0, 100.0));
    let overall = if raw.cpu_total_ticks == 0 {
        fallback
    } else if let Some(previous) = previous_cpu.as_ref() {
        let total_diff = raw.cpu_total_ticks.saturating_sub(previous.total_ticks);
        let idle_diff = raw.cpu_idle_ticks.saturating_sub(previous.idle_ticks);
        if timestamp.saturating_sub(previous.timestamp) > 100 && total_diff > 0 {
            round_one_decimal((1.0 - idle_diff as f64 / total_diff as f64) * 100.0)
        } else {
            fallback
        }
    } else {
        fallback
    };

    let per_core = raw
        .cpu_per_core
        .iter()
        .map(|current| {
            current
                .direct_percent
                .or_else(|| {
                    previous_cpu.as_ref().and_then(|previous| {
                        previous.per_core.iter().find(|item| item.index == current.index).and_then(
                            |prior| {
                                let total_diff =
                                    current.total_ticks.saturating_sub(prior.total_ticks);
                                let idle_diff = current.idle_ticks.saturating_sub(prior.idle_ticks);
                                (total_diff > 0)
                                    .then_some((1.0 - idle_diff as f64 / total_diff as f64) * 100.0)
                            },
                        )
                    })
                })
                .map(|value| round_one_decimal(value.clamp(0.0, 100.0)))
                .unwrap_or(0.0)
        })
        .collect::<Vec<_>>();

    *previous_cpu = Some(PreviousCpuSample {
        total_ticks: raw.cpu_total_ticks,
        idle_ticks: raw.cpu_idle_ticks,
        per_core: raw.cpu_per_core.clone(),
        timestamp,
    });

    (overall, per_core)
}

fn calc_network_metrics(
    raw: &RawMetrics,
    previous_net: &mut Option<PreviousNetSample>,
    timestamp: i64,
) -> (u64, u64, Vec<NetInterfaceEntry>) {
    let elapsed_seconds = previous_net.as_ref().and_then(|previous| {
        let elapsed_ms = timestamp.saturating_sub(previous.timestamp);
        (elapsed_ms > 100).then_some(elapsed_ms as f64 / 1000.0)
    });

    let summary_rates = if let (Some(previous), Some(seconds)) =
        (previous_net.as_ref(), elapsed_seconds)
    {
        (
            ((raw.net_rx_total.saturating_sub(previous.rx_total) as f64) / seconds)
                .max(0.0)
                .round() as u64,
            ((raw.net_tx_total.saturating_sub(previous.tx_total) as f64) / seconds)
                .max(0.0)
                .round() as u64,
        )
    } else {
        (0, 0)
    };

    let interfaces = raw
        .net_interfaces
        .iter()
        .map(|current| {
            let (rx_rate, tx_rate) = if let (Some(previous), Some(seconds)) =
                (previous_net.as_ref(), elapsed_seconds)
            {
                if let Some(prior) = previous
                    .interfaces
                    .iter()
                    .find(|item| item.name == current.name)
                {
                    (
                        ((current.rx_total.saturating_sub(prior.rx_total) as f64) / seconds)
                            .max(0.0)
                            .round() as u64,
                        ((current.tx_total.saturating_sub(prior.tx_total) as f64) / seconds)
                            .max(0.0)
                            .round() as u64,
                    )
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            NetInterfaceEntry {
                name: current.name.clone(),
                rx_total: current.rx_total,
                tx_total: current.tx_total,
                rx_rate,
                tx_rate,
            }
        })
        .collect::<Vec<_>>();

    *previous_net = Some(PreviousNetSample {
        rx_total: raw.net_rx_total,
        tx_total: raw.net_tx_total,
        interfaces: raw.net_interfaces.clone(),
        timestamp,
    });

    (summary_rates.0, summary_rates.1, interfaces)
}

fn calc_percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    round_one_decimal((used as f64 / total as f64) * 100.0)
}

fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
