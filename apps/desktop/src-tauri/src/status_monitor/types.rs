use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsageEntry {
    pub name: String,
    pub used_kb: u64,
    pub total_kb: u64,
    pub percent: f64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusProcessEntry {
    pub pid: String,
    pub mem_percent: f64,
    pub command: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetInterfaceEntry {
    pub name: String,
    pub rx_total: u64,
    pub tx_total: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

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
    pub cpu_cores: Option<u32>,
    pub cpu_per_core: Vec<f64>,
    pub mem_used: u64,
    pub mem_total: u64,
    pub mem_percent: f64,
    pub mem_free: u64,
    pub mem_buffers: u64,
    pub mem_cached: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub swap_percent: f64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_percent: f64,
    pub disks: Vec<DiskUsageEntry>,
    pub top_processes: Vec<StatusProcessEntry>,
    pub net_interface: String,
    pub net_rx_total: u64,
    pub net_tx_total: u64,
    pub net_rx_rate: u64,
    pub net_tx_rate: u64,
    pub net_interfaces: Vec<NetInterfaceEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusErrorPayload {
    pub session_id: String,
    pub message: String,
    pub timestamp: i64,
    pub degraded: bool,
    pub unsupported: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RawCoreCpuEntry {
    pub index: usize,
    pub total_ticks: u64,
    pub idle_ticks: u64,
    pub direct_percent: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct RawProcessEntry {
    pub pid: String,
    pub mem_percent: f64,
    pub command: String,
}

#[derive(Debug, Clone, Default)]
pub struct RawNetInterfaceEntry {
    pub name: String,
    pub rx_total: u64,
    pub tx_total: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RawMetrics {
    pub ip_address: String,
    pub cpu_model: String,
    pub os_name: String,
    pub cpu_percent: f64,
    pub cpu_total_ticks: u64,
    pub cpu_idle_ticks: u64,
    pub cpu_cores: Option<u32>,
    pub cpu_per_core: Vec<RawCoreCpuEntry>,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_free_mb: u64,
    pub mem_buffers_mb: u64,
    pub mem_cached_mb: u64,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
    pub disk_used_kb: u64,
    pub disk_total_kb: u64,
    pub disks: Vec<(String, u64, u64)>,
    pub top_processes: Vec<RawProcessEntry>,
    pub net_interface: String,
    pub net_rx_total: u64,
    pub net_tx_total: u64,
    pub net_interfaces: Vec<RawNetInterfaceEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct PreviousCpuSample {
    pub total_ticks: u64,
    pub idle_ticks: u64,
    pub per_core: Vec<RawCoreCpuEntry>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PreviousNetSample {
    pub rx_total: u64,
    pub tx_total: u64,
    pub interfaces: Vec<RawNetInterfaceEntry>,
    pub timestamp: i64,
}
