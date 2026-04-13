pub const SCRIPT: &str = r#"sh -lc '
OS=$(uname -s 2>/dev/null)
if [ "$OS" != "Linux" ]; then
  echo "UNSUPPORTED_COLLECTOR=linux"
  exit 0
fi
IP=$(hostname -I 2>/dev/null | awk "{print \$1}")
if [ -z "$IP" ]; then IP=$(hostname 2>/dev/null); fi
CPU_MODEL=$(grep -m1 "model name" /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed "s/^ *//")
if [ -z "$CPU_MODEL" ]; then CPU_MODEL=$(lscpu 2>/dev/null | awk -F: "/Model name:/{sub(/^[[:space:]]+/,\"\",\$2); print \$2; exit}"); fi
if [ -z "$CPU_MODEL" ]; then CPU_MODEL=$(uname -m 2>/dev/null); fi
OS_NAME=$(awk -F= "/^PRETTY_NAME=/{gsub(/\"/,\"\",\$2); print \$2; exit}" /etc/os-release 2>/dev/null)
if [ -z "$OS_NAME" ]; then OS_NAME=$(uname -sr 2>/dev/null); fi
CPU_CORES=$(nproc 2>/dev/null || grep -c "^processor" /proc/cpuinfo 2>/dev/null || echo "1")
CPU_LINE=$(awk "/^cpu /{print \$2\" \"\$3\" \"\$4\" \"\$5\" \"\$6\" \"\$7\" \"\$8\" \"\$9; exit}" /proc/stat 2>/dev/null)
CPU_TOTAL=0
CPU_IDLE=0
if [ -n "$CPU_LINE" ]; then set -- $CPU_LINE; CPU_TOTAL=$(( ${1:-0}+${2:-0}+${3:-0}+${4:-0}+${5:-0}+${6:-0}+${7:-0}+${8:-0} )); CPU_IDLE=$(( ${4:-0}+${5:-0} )); fi
awk "/^cpu[0-9]/{idx=substr(\$1,4); total=0; for(i=2;i<=NF;i++) total+=\$i; idle=\$5+\$6; printf \"cpu_core_item=%s|%d|%d\n\", idx, total, idle }" /proc/stat 2>/dev/null
MEM_TOTAL=$(awk "/^MemTotal:/{print int(\$2/1024)}" /proc/meminfo 2>/dev/null)
MEM_FREE=$(awk "/^MemFree:/{print int(\$2/1024)}" /proc/meminfo 2>/dev/null)
MEM_BUFFERS=$(awk "/^Buffers:/{print int(\$2/1024)}" /proc/meminfo 2>/dev/null)
MEM_CACHED=$(awk "/^Cached:/{c=\$2} /^SReclaimable:/{s=\$2} END{print int((c+s)/1024)}" /proc/meminfo 2>/dev/null)
MEM_USED=$(( ${MEM_TOTAL:-0}-${MEM_FREE:-0}-${MEM_BUFFERS:-0}-${MEM_CACHED:-0} ))
if [ "$MEM_USED" -lt 0 ]; then MEM_USED=0; fi
SWAP_TOTAL=$(awk "/^SwapTotal:/{print int(\$2/1024)}" /proc/meminfo 2>/dev/null)
SWAP_FREE=$(awk "/^SwapFree:/{print int(\$2/1024)}" /proc/meminfo 2>/dev/null)
SWAP_USED=$(( ${SWAP_TOTAL:-0}-${SWAP_FREE:-0} ))
if [ "$SWAP_USED" -lt 0 ]; then SWAP_USED=0; fi
DISK_TOTAL=$(df -kP / 2>/dev/null | awk "NR==2{print \$2+0}")
DISK_USED=$(df -kP / 2>/dev/null | awk "NR==2{print \$3+0}")
df -kP 2>/dev/null | awk "NR>1 && \$2+0>0 && \$1!~/^(tmpfs|devtmpfs|overlay|squashfs)\$/{print \"disk_item=\" \$6 \"|\" \$2+0 \"|\" \$3+0}"
ps -eo pid,%mem,comm --sort=-%mem 2>/dev/null | awk "NR>1 && NR<=11{gsub(/\\|/,\"/\",\$3); print \"process_item=\" \$1 \"|\" \$2 \"|\" \$3}"
NET_IF=$(ip route get 1.1.1.1 2>/dev/null | awk "{for(i=1;i<=NF;i++) if(\$i==\"dev\"){print \$(i+1); exit}}")
if [ -z "$NET_IF" ]; then NET_IF=$(awk -F: "NR>2{gsub(/ /,\"\",\$1); if(\$1!=\"lo\"){print \$1; exit}}" /proc/net/dev 2>/dev/null); fi
RX_TOTAL=0
TX_TOTAL=0
if [ -n "$NET_IF" ] && [ -r "/sys/class/net/$NET_IF/statistics/rx_bytes" ]; then
  RX_TOTAL=$(cat "/sys/class/net/$NET_IF/statistics/rx_bytes" 2>/dev/null)
  TX_TOTAL=$(cat "/sys/class/net/$NET_IF/statistics/tx_bytes" 2>/dev/null)
fi
awk -F: "NR>2{gsub(/^[ \t]+/,\"\",\$1); iface=\$1; if(iface!=\"lo\" && iface!~/^veth/ && iface!~/^docker/ && iface!~/^br-/){split(\$2,b); print \"net_item=\" iface \"|\" b[1] \"|\" b[9]}}" /proc/net/dev 2>/dev/null
echo "ip_address=${IP:-}"
echo "cpu_model=${CPU_MODEL:-}"
echo "os_name=${OS_NAME:-}"
echo "cpu_percent=0"
echo "cpu_total_ticks=${CPU_TOTAL:-0}"
echo "cpu_idle_ticks=${CPU_IDLE:-0}"
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
