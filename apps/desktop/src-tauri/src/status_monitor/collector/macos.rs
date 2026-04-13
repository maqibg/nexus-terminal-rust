pub const SCRIPT: &str = r#"sh -lc '
OS=$(uname -s 2>/dev/null)
if [ "$OS" != "Darwin" ]; then
  echo "UNSUPPORTED_COLLECTOR=macos"
  exit 0
fi
NET_IF=$(route -n get default 2>/dev/null | awk "/interface:/{print \$2; exit}")
IP=""
if [ -n "$NET_IF" ]; then IP=$(ipconfig getifaddr "$NET_IF" 2>/dev/null); fi
if [ -z "$IP" ]; then IP=$(hostname 2>/dev/null); fi
CPU_MODEL=$(sysctl -n machdep.cpu.brand_string 2>/dev/null)
if [ -z "$CPU_MODEL" ]; then CPU_MODEL=$(uname -m 2>/dev/null); fi
OS_NAME=$(sw_vers -productName 2>/dev/null)
OS_VER=$(sw_vers -productVersion 2>/dev/null)
if [ -n "$OS_NAME" ] && [ -n "$OS_VER" ]; then OS_NAME="$OS_NAME $OS_VER"; fi
if [ -z "$OS_NAME" ]; then OS_NAME=$(uname -sr 2>/dev/null); fi
CPU_CORES=$(sysctl -n hw.logicalcpu 2>/dev/null || echo "1")
MEM_TOTAL=$(sysctl -n hw.memsize 2>/dev/null | awk "{printf \"%d\", \$1/1024/1024}")
PAGESIZE=$(sysctl -n hw.pagesize 2>/dev/null || echo "4096")
VMINFO=$(vm_stat 2>/dev/null)
MEM_FREE=$(echo "$VMINFO" | awk -v ps="$PAGESIZE" "/^Pages free:/{gsub(/[^0-9]/,\"\",\$NF); free=\$NF+0} /^Pages speculative:/{gsub(/[^0-9]/,\"\",\$NF); spec=\$NF+0} END{printf \"%d\", (free+spec)*ps/1024/1024}")
MEM_CACHED=$(echo "$VMINFO" | awk -v ps="$PAGESIZE" "/^Pages inactive:/{gsub(/[^0-9]/,\"\",\$NF); ina=\$NF+0} /^Pages purgeable:/{gsub(/[^0-9]/,\"\",\$NF); pur=\$NF+0} END{printf \"%d\", (ina+pur)*ps/1024/1024}")
MEM_BUFFERS=0
MEM_USED=$(( ${MEM_TOTAL:-0}-${MEM_FREE:-0}-${MEM_CACHED:-0} ))
if [ "$MEM_USED" -lt 0 ]; then MEM_USED=0; fi
SWAPRAW=$(sysctl vm.swapusage 2>/dev/null)
SWAP_TOTAL=$(echo "$SWAPRAW" | awk "{for(i=1;i<=NF;i++){if(\$i==\"total\"&&\$(i+1)==\"=\"){v=\$(i+2);m=1;if(v~/G/)m=1024;gsub(/[MmGg]/,\"\",v);st=v*m}}; printf \"%.0f\", st+0}")
SWAP_USED=$(echo "$SWAPRAW" | awk "{for(i=1;i<=NF;i++){if(\$i==\"used\"&&\$(i+1)==\"=\"){v=\$(i+2);m=1;if(v~/G/)m=1024;gsub(/[MmGg]/,\"\",v);su=v*m}}; printf \"%.0f\", su+0}")
CPULINE=$(top -l 1 -s 0 -n 0 2>/dev/null | grep "CPU usage:" | head -1)
CPU_PCT=$(echo "$CPULINE" | awk "{for(i=1;i<=NF;i++){if(\$(i+1)~/^idle/){v=\$i; gsub(/%/,\"\",v); idle=v+0; found=1}}; if(found) printf \"%.1f\", 100-idle; else printf \"0\"}")
df -k 2>/dev/null | awk "NR>1 && index(\$1,\"/dev/\")==1 && (\$NF==\"/\" || index(\$NF,\"/Volumes/\")==1){print \"disk_item=\" \$NF \"|\" \$2+0 \"|\" \$3+0}"
DISK_TOTAL=$(df -k / 2>/dev/null | awk "NR==2{print \$2+0}")
DISK_USED=$(df -k / 2>/dev/null | awk "NR==2{print \$3+0}")
ps -A -o pid=,%mem=,comm= 2>/dev/null | sort -k2 -rn | head -10 | awk "{gsub(/\\|/,\"/\",\$3); print \"process_item=\" \$1 \"|\" \$2 \"|\" \$3}"
netstat -ib 2>/dev/null | awk "/^[a-z]/ && \$1!~/^lo/ && \$3~/Link/{if(\$4~/:/){rx=\$7;tx=\$10}else{rx=\$6;tx=\$9}; gsub(/[*]/,\"\",\$1); latestRx[\$1]=rx; latestTx[\$1]=tx} END{for(iface in latestRx){print \"net_item=\" iface \"|\" latestRx[iface] \"|\" latestTx[iface]}}"
RX_TOTAL=0
TX_TOTAL=0
if [ -n "$NET_IF" ]; then
  RX_TOTAL=$(netstat -ib 2>/dev/null | awk -v target="$NET_IF" "/^[a-z]/ && \$1==target && \$3~/Link/{if(\$4~/:/){rx=\$7}else{rx=\$6}; last=rx} END{print last+0}")
  TX_TOTAL=$(netstat -ib 2>/dev/null | awk -v target="$NET_IF" "/^[a-z]/ && \$1==target && \$3~/Link/{if(\$4~/:/){tx=\$10}else{tx=\$9}; last=tx} END{print last+0}")
fi
echo "ip_address=${IP:-}"
echo "cpu_model=${CPU_MODEL:-}"
echo "os_name=${OS_NAME:-}"
echo "cpu_percent=${CPU_PCT:-0}"
echo "cpu_total_ticks=0"
echo "cpu_idle_ticks=0"
echo "cpu_cores=${CPU_CORES:-0}"
echo "mem_total_mb=${MEM_TOTAL:-0}"
echo "mem_used_mb=${MEM_USED:-0}"
echo "mem_free_mb=${MEM_FREE:-0}"
echo "mem_buffers_mb=${MEM_BUFFERS:-0}"
echo "mem_cached_mb=${MEM_CACHED:-0}"
echo "swap_total_mb=${SWAP_TOTAL:-0}"
echo "swap_used_mb=${SWAP_USED:-0}"
echo "disk_total_kb=${DISK_TOTAL:-0}"
echo "disk_used_kb=${DISK_USED:-0}"
echo "net_interface=${NET_IF:-}"
echo "net_rx_total=${RX_TOTAL:-0}"
echo "net_tx_total=${TX_TOTAL:-0}"
'"#;
