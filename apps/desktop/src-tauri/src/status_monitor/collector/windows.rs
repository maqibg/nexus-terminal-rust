pub const SCRIPT: &str = r###"powershell -NoProfile -Command "$ErrorActionPreference='SilentlyContinue';
$ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike '169.254*' -and $_.IPAddress -ne '127.0.0.1' } | Select-Object -First 1 -ExpandProperty IPAddress);
if (-not $ip) { $ip = hostname; }
$cpuName = Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name;
$cpuPerf = Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor;
$cpuTotal = ($cpuPerf | Where-Object { $_.Name -eq '_Total' } | Select-Object -First 1 -ExpandProperty PercentProcessorTime);
$cpuCores = ($cpuPerf | Where-Object { $_.Name -match '^\d+$' }).Count;
$cpuTotalValue = 0;
if ($null -ne $cpuTotal) { $cpuTotalValue = [double]$cpuTotal; }
$osInfo = Get-CimInstance Win32_OperatingSystem;
$osName = $osInfo.Caption;
$memTotal = [int]([math]::Round($osInfo.TotalVisibleMemorySize / 1024));
$memFree = [int]([math]::Round($osInfo.FreePhysicalMemory / 1024));
$memUsed = [int]([math]::Max(0, $memTotal - $memFree));
$pageFiles = Get-CimInstance Win32_PageFileUsage;
$swapTotal = [int](($pageFiles | Measure-Object -Property AllocatedBaseSize -Sum).Sum);
$swapUsed = [int](($pageFiles | Measure-Object -Property CurrentUsage -Sum).Sum);
$disks = Get-CimInstance Win32_LogicalDisk | Where-Object { $_.DriveType -in 2,3 };
foreach ($disk in $disks) {
  if ($disk.Size -and $disk.Size -gt 0) {
    $totalKb = [int64]([math]::Round([double]$disk.Size / 1KB));
    $usedKb = [int64]([math]::Round(([double]$disk.Size - [double]$disk.FreeSpace) / 1KB));
    Write-Output ('disk_item=' + $disk.DeviceID + '|' + $totalKb + '|' + $usedKb);
  }
}
$primary = $disks | Where-Object { $_.DeviceID -eq 'C:' } | Select-Object -First 1;
if (-not $primary) { $primary = $disks | Select-Object -First 1; }
$diskTotal = 0;
$diskUsed = 0;
if ($primary -and $primary.Size -and $primary.Size -gt 0) {
  $diskTotal = [int64]([math]::Round([double]$primary.Size / 1KB));
  $diskUsed = [int64]([math]::Round(([double]$primary.Size - [double]$primary.FreeSpace) / 1KB));
}
$netStats = Get-NetAdapterStatistics | Where-Object { $_.Name -notmatch 'Loopback' };
$mainNet = $netStats | Select-Object -First 1;
$mainNetName = '';
$mainNetRx = 0;
$mainNetTx = 0;
if ($null -ne $mainNet) {
  $mainNetName = $mainNet.Name;
  $mainNetRx = [int64]$mainNet.ReceivedBytes;
  $mainNetTx = [int64]$mainNet.SentBytes;
}
foreach ($net in $netStats) {
  Write-Output ('net_item=' + $net.Name + '|' + [int64]$net.ReceivedBytes + '|' + [int64]$net.SentBytes);
}
$topProcesses = Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 10;
foreach ($proc in $topProcesses) {
  $pct = 0;
  if ($memTotal -gt 0) { $pct = [math]::Round((($proc.WorkingSet64 / 1MB) * 100) / $memTotal, 1); }
  Write-Output ('process_item=' + $proc.Id + '|' + $pct + '|' + $proc.ProcessName);
}
$coreItems = $cpuPerf | Where-Object { $_.Name -match '^\d+$' } | Sort-Object { [int]$_.Name };
foreach ($core in $coreItems) {
  Write-Output ('cpu_core_item=' + $core.Name + '|0|0|' + [double]$core.PercentProcessorTime);
}
Write-Output ('ip_address=' + $ip);
Write-Output ('cpu_model=' + $cpuName);
Write-Output ('os_name=' + $osName);
Write-Output ('cpu_percent=' + [math]::Round($cpuTotalValue, 1));
Write-Output 'cpu_total_ticks=0';
Write-Output 'cpu_idle_ticks=0';
Write-Output ('cpu_cores=' + $cpuCores);
Write-Output ('mem_total_mb=' + $memTotal);
Write-Output ('mem_used_mb=' + $memUsed);
Write-Output ('mem_free_mb=' + $memFree);
Write-Output 'mem_buffers_mb=0';
Write-Output 'mem_cached_mb=0';
Write-Output ('swap_total_mb=' + $swapTotal);
Write-Output ('swap_used_mb=' + $swapUsed);
Write-Output ('disk_total_kb=' + $diskTotal);
Write-Output ('disk_used_kb=' + $diskUsed);
Write-Output ('net_interface=' + $mainNetName);
Write-Output ('net_rx_total=' + $mainNetRx);
Write-Output ('net_tx_total=' + $mainNetTx)
""###;
