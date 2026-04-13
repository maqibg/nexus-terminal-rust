use crate::status_monitor::types::{
    RawCoreCpuEntry, RawMetrics, RawNetInterfaceEntry, RawProcessEntry,
};

pub fn looks_like_metrics_output(output: &str) -> bool {
    output
        .lines()
        .filter_map(|line| line.split_once('='))
        .any(|(key, _)| matches!(key.trim(), "ip_address" | "cpu_model" | "os_name" | "mem_total_mb"))
}

pub fn parse_metrics_output(output: &str) -> RawMetrics {
    let mut raw = RawMetrics::default();
    for line in output.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "ip_address" => raw.ip_address = value.trim().to_string(),
            "cpu_model" => raw.cpu_model = value.trim().to_string(),
            "os_name" => raw.os_name = value.trim().to_string(),
            "cpu_percent" => raw.cpu_percent = parse_f64(value),
            "cpu_total_ticks" => raw.cpu_total_ticks = parse_u64(value),
            "cpu_idle_ticks" => raw.cpu_idle_ticks = parse_u64(value),
            "cpu_cores" => raw.cpu_cores = parse_u64(value).try_into().ok(),
            "mem_total_mb" => raw.mem_total_mb = parse_u64(value),
            "mem_used_mb" => raw.mem_used_mb = parse_u64(value),
            "mem_free_mb" => raw.mem_free_mb = parse_u64(value),
            "mem_buffers_mb" => raw.mem_buffers_mb = parse_u64(value),
            "mem_cached_mb" => raw.mem_cached_mb = parse_u64(value),
            "swap_total_mb" => raw.swap_total_mb = parse_u64(value),
            "swap_used_mb" => raw.swap_used_mb = parse_u64(value),
            "disk_total_kb" => raw.disk_total_kb = parse_u64(value),
            "disk_used_kb" => raw.disk_used_kb = parse_u64(value),
            "net_interface" => raw.net_interface = value.trim().to_string(),
            "net_rx_total" => raw.net_rx_total = parse_u64(value),
            "net_tx_total" => raw.net_tx_total = parse_u64(value),
            "disk_item" => {
                let mut parts = value.split('|').map(str::trim);
                let Some(name) = parts.next().filter(|item| !item.is_empty()) else { continue; };
                let total_kb = parts.next().map(parse_u64).unwrap_or_default();
                let used_kb = parts.next().map(parse_u64).unwrap_or_default();
                if total_kb > 0 {
                    raw.disks.push((name.to_string(), total_kb, used_kb));
                }
            }
            "process_item" => {
                let mut parts = value.split('|').map(str::trim);
                let Some(pid) = parts.next().filter(|item| !item.is_empty()) else { continue; };
                let mem_percent = parts.next().map(parse_f64).unwrap_or_default();
                raw.top_processes.push(RawProcessEntry {
                    pid: pid.to_string(),
                    mem_percent,
                    command: parts.next().unwrap_or_default().to_string(),
                });
            }
            "net_item" => {
                let mut parts = value.split('|').map(str::trim);
                let Some(name) = parts.next().filter(|item| !item.is_empty()) else { continue; };
                raw.net_interfaces.push(RawNetInterfaceEntry {
                    name: name.to_string(),
                    rx_total: parts.next().map(parse_u64).unwrap_or_default(),
                    tx_total: parts.next().map(parse_u64).unwrap_or_default(),
                });
            }
            "cpu_core_item" => {
                let mut parts = value.split('|').map(str::trim);
                raw.cpu_per_core.push(RawCoreCpuEntry {
                    index: parts.next().map(parse_u64).unwrap_or_default() as usize,
                    total_ticks: parts.next().map(parse_u64).unwrap_or_default(),
                    idle_ticks: parts.next().map(parse_u64).unwrap_or_default(),
                    direct_percent: parts.next().map(parse_f64).filter(|item| item.is_finite()),
                });
            }
            _ => {}
        }
    }
    raw.cpu_per_core.sort_by_key(|item| item.index);
    raw.net_interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    raw.top_processes
        .sort_by(|left, right| right.mem_percent.total_cmp(&left.mem_percent));
    raw
}

fn parse_u64(value: &str) -> u64 {
    value.trim().parse::<u64>().unwrap_or_default()
}

fn parse_f64(value: &str) -> f64 {
    value.trim().parse::<f64>().unwrap_or_default()
}
