<template>
  <div class="statistics-page">
    <div class="page-header">
      <h2>统计分析</h2>
      <div class="header-actions">
        <div class="range-switch">
          <button v-for="option in timeRangeOptions" :key="option.value" class="range-btn" :class="{ active: timeRange === option.value }" @click="timeRange = option.value">
            {{ option.label }}
          </button>
        </div>
        <AppSelect
          v-model="displayCurrency"
          class="currency-select"
          :options="currencyOptions"
          title="显示币种"
          aria-label="显示币种"
        />
        <button class="icon-btn" :disabled="loading" title="刷新数据" @click="refreshData">
          <i class="fas fa-rotate-right" :class="{ 'fa-spin': loading }"></i>
        </button>
        <button class="icon-btn" title="显示设置" @click="showSettingsDialog = true">
          <i class="fas fa-sliders-h"></i>
        </button>
      </div>
    </div>

    <div v-if="visibleComponents.summaryCards" class="summary-grid">
      <section class="summary-card">
        <div class="label">总连接数</div>
        <div class="value">{{ totalConnections }}</div>
        <div class="sub">管理信息覆盖 {{ managedCoverage }}</div>
      </section>
      <section class="summary-card">
        <div class="label">月度费用（估算）</div>
        <div class="value">{{ formatCurrency(totalMonthlyCostCny) }}</div>
        <div class="sub">预计年度 {{ formatCurrency(totalMonthlyCostCny * 12) }}</div>
      </section>
      <section class="summary-card">
        <div class="label">累计连接次数</div>
        <div class="value">{{ totalUsage }}</div>
        <div class="sub">活跃连接 {{ activeConnections }} / 命令 {{ totalCommands }}</div>
      </section>
      <section class="summary-card">
        <div class="label">总连接时长（估算）</div>
        <div class="value">{{ formatDuration(totalConnectionTimeSeconds) }}</div>
        <div class="sub">平均 {{ formatDuration(averageConnectionTimeSeconds) }} · 即将到期 {{ expiringSoonCount }} · 已过期 {{ expiredConnectionCount }}</div>
      </section>
    </div>

    <div class="panel-grid">
      <section v-if="visibleComponents.regionDistribution" class="panel-card">
        <h3>主机地区分布</h3>
        <div v-if="regionStats.length" class="bar-list">
          <div v-for="item in regionStats" :key="item.region" class="bar-item">
            <div class="bar-title">
              <span><span v-if="getRegionFlag(item.region)" class="flag">{{ getRegionFlag(item.region) }}</span>{{ item.region || '未知地区' }}</span>
              <span>{{ item.count }} 次</span>
            </div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.count, totalConnections)}%`, backgroundColor: getRegionColor(item.region) }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无地区数据，请编辑连接补充地区信息</div>
      </section>

      <section v-if="visibleComponents.providerStats" class="panel-card">
        <h3>服务商统计</h3>
        <div v-if="providerStats.length" class="bar-list">
          <div v-for="item in providerStats" :key="item.provider" class="bar-item">
            <div class="bar-title"><span>{{ item.provider }}</span><span>{{ item.count }} 次</span></div>
            <div class="bar-sub">月度费用: {{ formatCurrency(item.cost) }}</div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.count, totalConnections)}%`, backgroundColor: getColor(item.provider) }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无服务商数据</div>
      </section>

      <section v-if="visibleComponents.costAnalysis" class="panel-card">
        <h3>服务商费用占比</h3>
        <div v-if="providerCosts.length" class="bar-list">
          <div v-for="item in providerCosts" :key="item.provider" class="bar-item">
            <div class="bar-title"><span>{{ item.provider || '未分类' }}</span><span>{{ formatCurrency(item.cost) }}</span></div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.cost, totalMonthlyCostCny)}%`, backgroundColor: getColor(item.provider) }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无费用数据，请编辑连接添加计费信息</div>
      </section>

      <section v-if="visibleComponents.usageAnalysis" class="panel-card">
        <h3>最常使用连接</h3>
        <div v-if="topUsedConnections.length" class="bar-list">
          <div v-for="item in topUsedConnections" :key="item.connectionId" class="bar-item">
            <div class="bar-title"><span class="truncate">{{ item.name }}</span><span>{{ item.commandCount }} 次</span></div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.commandCount, maxUsage)}%`, backgroundColor: 'var(--blue)' }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无使用数据</div>
      </section>

      <section v-if="visibleComponents.connectionTime" class="panel-card">
        <h3>连接时长统计</h3>
        <div v-if="topConnectionDurations.length" class="bar-list">
          <div v-for="item in topConnectionDurations" :key="item.connectionId" class="bar-item">
            <div class="bar-title"><span class="truncate">{{ item.name }}</span><span>{{ formatDuration(item.totalDurationSeconds) }}</span></div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.totalDurationSeconds, maxConnectionDuration)}%`, backgroundColor: '#4ECDC4' }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无连接时长数据</div>
      </section>

      <section v-if="visibleComponents.commandStats" class="panel-card">
        <h3>最常用命令</h3>
        <div v-if="topCommands.length" class="bar-list">
          <div v-for="item in topCommands" :key="item.command" class="bar-item">
            <div class="bar-title"><span class="truncate cmd">{{ item.command }}</span><span>{{ item.count }} 次</span></div>
            <div class="bar-track"><div class="bar-fill" :style="{ width: `${getPercentage(item.count, maxCommandCount)}%`, backgroundColor: '#FFE66D' }"></div></div>
          </div>
        </div>
        <div v-else class="empty">暂无命令统计数据</div>
      </section>

      <section v-if="visibleComponents.trafficStats" class="panel-card">
        <h3>流量统计</h3>
        <div v-if="trafficStats.totalBytes > 0" class="traffic-wrap">
          <div class="traffic-grid">
            <div class="traffic-item"><span>上传</span><strong>{{ formatBytes(trafficStats.totalBytesOut) }}</strong></div>
            <div class="traffic-item"><span>下载</span><strong>{{ formatBytes(trafficStats.totalBytesIn) }}</strong></div>
            <div class="traffic-item"><span>总流量</span><strong class="highlight">{{ formatBytes(trafficStats.totalBytes) }}</strong></div>
          </div>
          <div class="traffic-bar">
            <div class="traffic-segment upload" :style="{ width: `${getPercentage(trafficStats.totalBytesOut, trafficStats.totalBytes)}%` }"></div>
            <div class="traffic-segment download" :style="{ width: `${getPercentage(trafficStats.totalBytesIn, trafficStats.totalBytes)}%` }"></div>
          </div>
        </div>
        <div v-else class="empty">暂无流量数据</div>
      </section>

      <section v-if="visibleComponents.expiringSoon" class="panel-card panel-card-wide">
        <h3>到期提醒（30 天内）</h3>
        <table v-if="expiringConnections.length" class="table">
          <thead><tr><th>连接</th><th>到期时间</th><th>状态</th></tr></thead>
          <tbody>
            <tr v-for="item in expiringConnections" :key="item.id">
              <td>{{ item.name }}</td>
              <td>{{ formatDateTime(item.expiryDate) }}</td>
              <td><span class="expiry-tag" :class="`is-${item.status}`">{{ item.status === 'expired' ? '已过期' : item.daysLeft === 0 ? '今天到期' : `${item.daysLeft} 天后到期` }}</span></td>
            </tr>
          </tbody>
        </table>
        <div v-else class="empty">暂无即将到期连接</div>
      </section>

      <section v-if="visibleComponents.detailedTable" class="panel-card panel-card-wide">
        <h3>连接详情</h3>
        <div class="table-wrap">
          <table class="table">
            <thead><tr><th>名称</th><th>主机</th><th>服务商</th><th>地区</th><th>费用(月)</th><th>连接次</th><th>最后活跃</th></tr></thead>
            <tbody>
              <tr v-for="row in detailedRows" :key="row.id">
                <td class="truncate" :title="row.name">{{ row.name }}</td>
                <td class="truncate" :title="row.host">{{ row.host }}</td>
                <td>{{ row.provider }}</td>
                <td><span v-if="getRegionFlag(row.region)" class="flag">{{ getRegionFlag(row.region) }}</span>{{ row.region || '-' }}</td>
                <td>
                  <div>{{ formatAnyCost(row.rawConnection) }}</div>
                  <div v-if="row.billingCycle !== 'monthly' && row.monthlyCostCny > 0" class="table-sub">折合: {{ formatCurrency(row.monthlyCostCny) }}/月</div>
                </td>
                <td>{{ row.commandCount }}</td>
                <td>{{ formatDateTime(row.lastActiveAt) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <div v-if="showSettingsDialog" class="dialog-mask" @click.self="showSettingsDialog = false">
      <div class="dialog-card">
        <h3>统计组件显示设置</h3>
        <p class="hint">选择要显示的统计组件</p>
        <label class="toggle-row"><span>概览卡片</span><input v-model="visibleComponents.summaryCards" type="checkbox"></label>
        <label class="toggle-row"><span>主机地区分布</span><input v-model="visibleComponents.regionDistribution" type="checkbox"></label>
        <label class="toggle-row"><span>服务商统计</span><input v-model="visibleComponents.providerStats" type="checkbox"></label>
        <label class="toggle-row"><span>服务商费用占比</span><input v-model="visibleComponents.costAnalysis" type="checkbox"></label>
        <label class="toggle-row"><span>最常使用连接</span><input v-model="visibleComponents.usageAnalysis" type="checkbox"></label>
        <label class="toggle-row"><span>连接时长统计</span><input v-model="visibleComponents.connectionTime" type="checkbox"></label>
        <label class="toggle-row"><span>最常用命令</span><input v-model="visibleComponents.commandStats" type="checkbox"></label>
        <label class="toggle-row"><span>流量统计</span><input v-model="visibleComponents.trafficStats" type="checkbox"></label>
        <label class="toggle-row"><span>到期提醒</span><input v-model="visibleComponents.expiringSoon" type="checkbox"></label>
        <label class="toggle-row"><span>连接详情表格</span><input v-model="visibleComponents.detailedTable" type="checkbox"></label>
        <div class="dialog-actions">
          <button class="btn" @click="resetVisibility">全部显示</button>
          <button class="btn btn-primary" @click="saveVisibilitySettings">保存</button>
        </div>
      </div>
    </div>
  </div>
</template><script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import AppSelect from '@/components/AppSelect.vue';
import { historyApi, transferApi, type CommandHistory, type Connection, type TransferTaskDto } from '@/lib/api';
import { useConnectionsStore } from '@/stores/connections';

interface ProviderStatItem { provider: string; count: number; cost: number; }
interface RegionStatItem { region: string; count: number; }
interface CommandStatItem { command: string; count: number; }
interface ConnectionActivityItem {
  connectionId: number;
  name: string;
  commandCount: number;
  sessionCount: number;
  totalDurationSeconds: number;
  lastActiveAt: string;
}
interface ExpiringConnectionItem {
  id: number;
  name: string;
  expiryDate: string;
  daysLeft: number;
  status: 'expired' | 'danger' | 'warning' | 'normal';
}
interface ConnectionDetailRow {
  id: number;
  name: string;
  host: string;
  provider: string;
  region: string;
  commandCount: number;
  lastActiveAt: string | null;
  monthlyCostCny: number;
  billingCycle: string;
  rawConnection: Connection;
}
interface VisibleComponents {
  summaryCards: boolean;
  regionDistribution: boolean;
  providerStats: boolean;
  costAnalysis: boolean;
  usageAnalysis: boolean;
  connectionTime: boolean;
  commandStats: boolean;
  trafficStats: boolean;
  expiringSoon: boolean;
  detailedTable: boolean;
}

type TimeRangeValue = 'today' | 'week' | 'month' | 'all';
type DisplayCurrency = 'CNY' | 'USD';

const VISIBILITY_STORAGE_KEY = 'statistics-visibility';
const CONNECTION_SESSION_GAP_MS = 30 * 60 * 1000;
const MIN_SESSION_SECONDS = 60;

const CURRENCY_TO_CNY_RATE: Record<string, number> = { CNY: 1, USD: 7.2, EUR: 7.8 };
const REGION_CODE_MAP: Record<string, string> = {
  香港: 'HK', 'Hong Kong': 'HK', HK: 'HK', 美国: 'US', USA: 'US', US: 'US', 'Los Angeles': 'US', 洛杉矶: 'US',
  日本: 'JP', Japan: 'JP', JP: 'JP', Tokyo: 'JP', 东京: 'JP', 新加坡: 'SG', Singapore: 'SG', SG: 'SG',
  韩国: 'KR', Korea: 'KR', KR: 'KR', Seoul: 'KR', 中国: 'CN', China: 'CN', CN: 'CN', 上海: 'CN', 北京: 'CN',
  德国: 'DE', Germany: 'DE', DE: 'DE', Frankfurt: 'DE', 英国: 'GB', UK: 'GB', GB: 'GB', London: 'GB',
  法国: 'FR', France: 'FR', FR: 'FR', 俄罗斯: 'RU', Russia: 'RU', RU: 'RU', 加拿大: 'CA', Canada: 'CA', CA: 'CA',
};

const defaultVisibleComponents: VisibleComponents = {
  summaryCards: true,
  regionDistribution: true,
  providerStats: true,
  costAnalysis: true,
  usageAnalysis: true,
  connectionTime: true,
  commandStats: true,
  trafficStats: true,
  expiringSoon: true,
  detailedTable: true,
};

const timeRangeOptions: Array<{ label: string; value: TimeRangeValue }> = [
  { label: '今日', value: 'today' },
  { label: '本周', value: 'week' },
  { label: '本月', value: 'month' },
  { label: '全部', value: 'all' },
];

const currencyOptions = [
  { value: 'CNY', label: '人民币 ¥' },
  { value: 'USD', label: '美元 $' },
];

const connectionsStore = useConnectionsStore();
const { list: connections } = storeToRefs(connectionsStore);

const loading = ref(false);
const timeRange = ref<TimeRangeValue>('all');
const displayCurrency = ref<DisplayCurrency>('CNY');
const showSettingsDialog = ref(false);
const visibleComponents = ref<VisibleComponents>({ ...defaultVisibleComponents });
const historyItems = ref<CommandHistory[]>([]);
const transferItems = ref<TransferTaskDto[]>([]);

function parseDateTime(value: unknown): Date | null {
  if (typeof value !== 'string') return null;

  const trimmed = value.trim();
  if (!trimmed) return null;

  let normalized = trimmed.replace(/\//g, '-').replace(/\s+/, 'T');
  if (/^\d{4}-\d{2}-\d{2}$/.test(normalized)) {
    normalized = `${normalized}T00:00:00`;
  }
  if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(normalized)) {
    normalized = `${normalized}:00`;
  }

  const parsed = new Date(normalized);
  if (!Number.isNaN(parsed.getTime())) {
    return parsed;
  }

  const match = normalized.match(/^(\d{4})-(\d{1,2})-(\d{1,2})(?:T(\d{1,2})(?::(\d{1,2}))?(?::(\d{1,2}))?)?$/);
  if (!match) {
    return null;
  }

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const hour = Number(match[4] ?? 0);
  const minute = Number(match[5] ?? 0);
  const second = Number(match[6] ?? 0);
  const fallback = new Date(year, month - 1, day, hour, minute, second);
  return Number.isNaN(fallback.getTime()) ? null : fallback;
}

function isInCurrentTimeRange(date: Date): boolean {
  if (timeRange.value === 'all') return true;
  const now = new Date();
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  if (timeRange.value === 'today') return date >= todayStart;
  if (timeRange.value === 'week') {
    const weekAgo = new Date(todayStart);
    weekAgo.setDate(weekAgo.getDate() - 7);
    return date >= weekAgo;
  }
  const monthAgo = new Date(todayStart);
  monthAgo.setMonth(monthAgo.getMonth() - 1);
  return date >= monthAgo;
}

function estimateActivity(timestamps: number[]): { sessionCount: number; durationSeconds: number } {
  if (timestamps.length === 0) return { sessionCount: 0, durationSeconds: 0 };
  const sorted = [...timestamps].sort((a, b) => a - b);
  let sessionCount = 0;
  let durationSeconds = 0;
  let sessionStart = sorted[0];
  let sessionEnd = sorted[0];

  for (let i = 1; i < sorted.length; i += 1) {
    const ts = sorted[i];
    if (ts - sessionEnd <= CONNECTION_SESSION_GAP_MS) {
      sessionEnd = ts;
      continue;
    }
    sessionCount += 1;
    durationSeconds += Math.max(MIN_SESSION_SECONDS, Math.round((sessionEnd - sessionStart) / 1000));
    sessionStart = ts;
    sessionEnd = ts;
  }

  sessionCount += 1;
  durationSeconds += Math.max(MIN_SESSION_SECONDS, Math.round((sessionEnd - sessionStart) / 1000));
  return { sessionCount, durationSeconds };
}

function getMonthlyCostCny(item: Record<string, unknown>): number {
  const rawAmount = item.billing_amount;
  if (typeof rawAmount !== 'number' || !Number.isFinite(rawAmount) || rawAmount <= 0) return 0;
  const currency = String(item.billing_currency ?? 'CNY').toUpperCase();
  const rate = CURRENCY_TO_CNY_RATE[currency] ?? 1;
  const amountCny = rawAmount * rate;
  const cycle = String(item.billing_cycle ?? 'monthly');
  switch (cycle) {
    case 'monthly': return amountCny;
    case 'quarterly': return amountCny / 3;
    case 'semi-annually': return amountCny / 6;
    case 'annually': return amountCny / 12;
    case 'biennially': return amountCny / 24;
    case 'triennially': return amountCny / 36;
    case 'custom': return 0;
    default: return amountCny;
  }
}

function detectProvider(item: Connection): string {
  if (typeof item.provider === 'string' && item.provider.trim()) return item.provider.trim();
  const keywords: Record<string, string[]> = {
    '阿里云(Aliyun)': ['aliyun', '阿里云', 'ali'], '腾讯云(Tencent)': ['tencent', 'qcloud', '腾讯云'], '华为云(Huawei)': ['huawei', '华为云', 'hwcloud'],
    AWS: ['aws', 'amazon'], Azure: ['azure', 'microsoft'], 'Google Cloud': ['gcp', 'google cloud'], Vultr: ['vultr'],
    DigitalOcean: ['digitalocean', 'do'], Linode: ['linode', 'akamai'], Bandwagon: ['bandwagon', 'bwh'],
    Cloudflare: ['cloudflare'], Oracle: ['oracle'], DMIT: ['dmit'], FiberState: ['fiberstate', 'fs'], RackNerd: ['racknerd', 'rn'], CloudCone: ['cloudcone', 'cc'],
  };
  const source = `${item.name ?? ''} ${item.host ?? ''}`.toLowerCase();
  for (const [provider, aliases] of Object.entries(keywords)) {
    if (aliases.some(alias => source.includes(alias))) return provider;
  }
  return '未分类';
}

const filteredHistoryItems = computed(() => historyItems.value.filter((item) => {
  const parsed = parseDateTime(item.timestamp);
  return parsed ? isInCurrentTimeRange(parsed) : false;
}));

const connectionActivity = computed<ConnectionActivityItem[]>(() => {
  const grouped = new Map<number, { connectionId: number; name: string; commandCount: number; timestamps: number[]; lastActiveAt: string }>();

  filteredHistoryItems.value.forEach((item) => {
    if (typeof item.connection_id !== 'number') return;
    const timestampDate = parseDateTime(item.timestamp);
    if (!timestampDate) return;

    const current = grouped.get(item.connection_id) ?? {
      connectionId: item.connection_id,
      name: connections.value.find(conn => conn.id === item.connection_id)?.name ?? `连接 #${item.connection_id}`,
      commandCount: 0,
      timestamps: [],
      lastActiveAt: item.timestamp,
    };

    current.commandCount += 1;
    current.timestamps.push(timestampDate.getTime());
    if ((parseDateTime(current.lastActiveAt)?.getTime() ?? 0) < timestampDate.getTime()) {
      current.lastActiveAt = item.timestamp;
    }

    grouped.set(item.connection_id, current);
  });

  return [...grouped.values()].map((item) => {
    const estimate = estimateActivity(item.timestamps);
    return {
      connectionId: item.connectionId,
      name: item.name,
      commandCount: item.commandCount,
      sessionCount: estimate.sessionCount,
      totalDurationSeconds: estimate.durationSeconds,
      lastActiveAt: item.lastActiveAt,
    };
  });
});

const activityByConnectionId = computed(() => {
  const map = new Map<number, ConnectionActivityItem>();
  connectionActivity.value.forEach(item => map.set(item.connectionId, item));
  return map;
});

const totalConnections = computed(() => connections.value.length);
const managedConnections = computed(() => connections.value.filter(item => Boolean(String(item.provider ?? '').trim() || String(item.region ?? '').trim() || String(item.expiry_date ?? '').trim() || (typeof item.billing_amount === 'number' && Number.isFinite(item.billing_amount) && item.billing_amount > 0))).length);
const managedCoverage = computed(() => totalConnections.value === 0 ? '0.0%' : `${((managedConnections.value / totalConnections.value) * 100).toFixed(1)}%`);
const totalMonthlyCostCny = computed(() => connections.value.reduce((sum, item) => sum + getMonthlyCostCny(item as Record<string, unknown>), 0));

const regionStats = computed<RegionStatItem[]>(() => {
  const map = new Map<string, number>();
  connections.value.forEach((item) => {
    const region = String(item.region ?? '').trim() || '未知地区';
    map.set(region, (map.get(region) ?? 0) + 1);
  });
  return [...map.entries()].map(([region, count]) => ({ region, count })).sort((a, b) => b.count - a.count);
});

const providerStats = computed<ProviderStatItem[]>(() => {
  const map = new Map<string, ProviderStatItem>();
  connections.value.forEach((item) => {
    const provider = detectProvider(item);
    const current = map.get(provider) ?? { provider, count: 0, cost: 0 };
    current.count += 1;
    current.cost += getMonthlyCostCny(item as Record<string, unknown>);
    map.set(provider, current);
  });
  return [...map.values()].sort((a, b) => b.count - a.count || b.cost - a.cost);
});

const providerCosts = computed(() => providerStats.value.map(item => ({ provider: item.provider, cost: item.cost })).filter(item => item.cost > 0).sort((a, b) => b.cost - a.cost));

const totalCommands = computed(() => filteredHistoryItems.value.length);
const activeConnections = computed(() => connectionActivity.value.length);
const totalUsage = computed(() => connectionActivity.value.reduce((sum, item) => sum + item.sessionCount, 0));
const totalConnectionTimeSeconds = computed(() => connectionActivity.value.reduce((sum, item) => sum + item.totalDurationSeconds, 0));
const averageConnectionTimeSeconds = computed(() => totalUsage.value > 0 ? totalConnectionTimeSeconds.value / totalUsage.value : 0);

const topUsedConnections = computed(() => [...connectionActivity.value].sort((a, b) => b.commandCount - a.commandCount).slice(0, 5).filter(item => item.commandCount > 0));
const maxUsage = computed(() => Math.max(...topUsedConnections.value.map(item => item.commandCount), 1));

const topConnectionDurations = computed(() => [...connectionActivity.value].sort((a, b) => b.totalDurationSeconds - a.totalDurationSeconds).slice(0, 5).filter(item => item.totalDurationSeconds > 0));
const maxConnectionDuration = computed(() => Math.max(...topConnectionDurations.value.map(item => item.totalDurationSeconds), 1));

const topCommands = computed<CommandStatItem[]>(() => {
  const map = new Map<string, number>();
  filteredHistoryItems.value.forEach((item) => {
    const normalized = item.command.replace(/\s+/g, ' ').trim();
    if (!normalized) return;
    map.set(normalized, (map.get(normalized) ?? 0) + 1);
  });
  return [...map.entries()].map(([command, count]) => ({ command, count })).sort((a, b) => b.count - a.count).slice(0, 10);
});
const maxCommandCount = computed(() => Math.max(...topCommands.value.map(item => item.count), 1));

const trafficStats = computed(() => {
  const stats = { totalBytesIn: 0, totalBytesOut: 0, totalBytes: 0 };
  transferItems.value.forEach((item) => {
    const transferred = Math.max(typeof item.transferred_bytes === 'number' ? item.transferred_bytes : 0, item.status === 'Completed' ? (typeof item.total_bytes === 'number' ? item.total_bytes : 0) : 0);
    if (item.kind === 'upload') stats.totalBytesOut += transferred;
    else stats.totalBytesIn += transferred;
  });
  stats.totalBytes = stats.totalBytesIn + stats.totalBytesOut;
  return stats;
});

const expiringConnections = computed<ExpiringConnectionItem[]>(() => {
  const now = Date.now();
  const dayMs = 24 * 60 * 60 * 1000;

  return connections.value.map((item) => {
    const parsed = parseDateTime(item.expiry_date);
    if (!parsed) return null;
    const diffMs = parsed.getTime() - now;
    const daysLeft = Math.ceil(diffMs / dayMs);
    if (daysLeft > 30) return null;

    let status: ExpiringConnectionItem['status'] = 'normal';
    if (daysLeft < 0) status = 'expired';
    else if (daysLeft <= 3) status = 'danger';
    else if (daysLeft <= 7) status = 'warning';

    return { id: item.id, name: item.name, expiryDate: item.expiry_date ?? '', daysLeft, status };
  }).filter((item): item is ExpiringConnectionItem => item !== null).sort((a, b) => a.daysLeft - b.daysLeft);
});

const expiringSoonCount = computed(() => expiringConnections.value.filter(item => item.daysLeft >= 0).length);
const expiredConnectionCount = computed(() => {
  const now = Date.now();
  return connections.value.filter(item => {
    const parsed = parseDateTime(item.expiry_date);
    return parsed ? parsed.getTime() < now : false;
  }).length;
});

const detailedRows = computed<ConnectionDetailRow[]>(() => connections.value.map((item) => {
  const activity = activityByConnectionId.value.get(item.id);
  return {
    id: item.id,
    name: item.name,
    host: `${item.username}@${item.host}:${item.port}`,
    provider: detectProvider(item),
    region: String(item.region ?? '').trim(),
    commandCount: activity?.commandCount ?? 0,
    lastActiveAt: activity?.lastActiveAt ?? null,
    monthlyCostCny: getMonthlyCostCny(item as Record<string, unknown>),
    billingCycle: String(item.billing_cycle ?? '').trim(),
    rawConnection: item,
  };
}).sort((a, b) => b.commandCount - a.commandCount || a.name.localeCompare(b.name)));

function resolveRegionCode(region?: string): string | undefined {
  if (!region) return undefined;
  const trimmed = region.trim();
  if (!trimmed) return undefined;
  let code = REGION_CODE_MAP[trimmed] || (trimmed.length === 2 ? trimmed.toUpperCase() : undefined);
  if (!code) {
    const found = Object.keys(REGION_CODE_MAP).find(key => trimmed.includes(key));
    if (found) code = REGION_CODE_MAP[found];
  }
  return code;
}

function toFlagEmoji(code: string): string {
  if (!/^[A-Z]{2}$/.test(code)) return '';
  return String.fromCodePoint(0x1F1E6 + (code.charCodeAt(0) - 65), 0x1F1E6 + (code.charCodeAt(1) - 65));
}

function getRegionFlag(region?: string): string { const code = resolveRegionCode(region); return code ? toFlagEmoji(code) : ''; }

function getRegionColor(region: string): string {
  const map: Record<string, string> = { 香港: '#FF6B6B', 美国: '#4ECDC4', 日本: '#FFE66D', 新加坡: '#95E1D3', 韩国: '#F38181', 中国: '#AA96DA', 德国: '#FCBAD3', 英国: '#A8D8EA', 法国: '#FFAAA7', 俄罗斯: '#C7CEEA' };
  return map[region] || getColor(region);
}

function getColor(value: string): string {
  let hash = 0;
  for (let i = 0; i < value.length; i += 1) hash = value.charCodeAt(i) + ((hash << 5) - hash);
  const color = (hash & 0x00ffffff).toString(16).toUpperCase();
  return `#${'00000'.substring(0, 6 - color.length)}${color}`;
}

function getPercentage(value: number, total: number): number {
  if (!Number.isFinite(value) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.min(100, (value / total) * 100);
}

function convertCycleLabel(cycle?: string | null): string {
  switch (cycle) {
    case 'monthly': return '月'; case 'quarterly': return '季'; case 'semi-annually': return '半年'; case 'annually': return '年';
    case 'biennially': return '两年'; case 'triennially': return '三年'; case 'custom': return '自定义'; default: return '';
  }
}

function formatAnyCost(item: Connection): string {
  if (typeof item.billing_amount !== 'number' || !Number.isFinite(item.billing_amount) || item.billing_amount <= 0) return '-';
  const currency = String(item.billing_currency ?? 'CNY').toUpperCase();
  const cycleLabel = convertCycleLabel(item.billing_cycle);
  return cycleLabel ? `${item.billing_amount.toFixed(2)} ${currency}/${cycleLabel}` : `${item.billing_amount.toFixed(2)} ${currency}`;
}

function formatCurrency(amountCny: number): string {
  if (!Number.isFinite(amountCny)) return displayCurrency.value === 'USD' ? '$0.00' : '¥0.00';
  if (displayCurrency.value === 'USD') return `$${(amountCny / 7.2).toFixed(2)}`;
  return `¥${amountCny.toFixed(2)}`;
}

function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return '0秒';
  if (seconds < 60) return `${Math.round(seconds)}秒`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}分钟`;
  if (seconds < 86400) return `${(seconds / 3600).toFixed(1)}小时`;
  return `${(seconds / 86400).toFixed(1)}天`;
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  if (bytes < 1024) return `${bytes.toFixed(0)} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDateTime(value: string | null | undefined): string {
  if (!value) return '-';
  const parsed = parseDateTime(value);
  if (!parsed) return value;
  return parsed.toLocaleString('zh-CN', { hour12: false, month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
}

function loadVisibilitySettings(): void {
  const raw = localStorage.getItem(VISIBILITY_STORAGE_KEY);
  if (!raw) return;
  try {
    const parsed = JSON.parse(raw) as Partial<VisibleComponents>;
    const merged = { ...defaultVisibleComponents };
    (Object.keys(defaultVisibleComponents) as Array<keyof VisibleComponents>).forEach((key) => {
      if (typeof parsed[key] === 'boolean') merged[key] = parsed[key] as boolean;
    });
    visibleComponents.value = merged;
  } catch (error) {
    console.error('load statistics visibility settings failed:', error);
  }
}

function saveVisibilitySettings(): void {
  localStorage.setItem(VISIBILITY_STORAGE_KEY, JSON.stringify(visibleComponents.value));
  showSettingsDialog.value = false;
}

function resetVisibility(): void {
  visibleComponents.value = { ...defaultVisibleComponents };
}

async function refreshData(): Promise<void> {
  loading.value = true;
  try {
    await connectionsStore.fetch();
    const [historyResult, transferResult] = await Promise.allSettled([historyApi.list(5000, 0), transferApi.list()]);

    if (historyResult.status === 'fulfilled') {
      historyItems.value = historyResult.value;
    } else {
      historyItems.value = [];
      console.error('load command history failed:', historyResult.reason);
    }

    if (transferResult.status === 'fulfilled') {
      transferItems.value = transferResult.value;
    } else {
      transferItems.value = [];
      console.error('load transfer stats failed:', transferResult.reason);
    }
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadVisibilitySettings();
  void refreshData();
});
</script>

<style scoped>
.statistics-page {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--border) 80%, transparent) transparent;
}

.statistics-page::-webkit-scrollbar {
  width: 10px;
}

.statistics-page::-webkit-scrollbar-track {
  background: transparent;
}

.statistics-page::-webkit-scrollbar-thumb {
  border-radius: 999px;
  border: 2px solid transparent;
  background-clip: padding-box;
  background: color-mix(in srgb, var(--border) 85%, transparent);
}

.statistics-page::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text-sub) 55%, transparent);
}
.page-header { display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap; }
.page-header h2 { margin: 0; font-size: calc(18px + var(--ui-font-size-offset)); color: var(--text); }
.header-actions { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.range-switch { display: inline-flex; border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
.range-btn { height: 32px; padding: 0 12px; border: none; border-right: 1px solid var(--border); background: transparent; color: var(--text-sub); font-size: calc(12px + var(--ui-font-size-offset)); cursor: pointer; }
.range-btn:last-child { border-right: none; }
.range-btn.active { background: var(--blue); color: var(--button-text-color); }
.icon-btn, .btn { height: 32px; border-radius: 6px; font-size: calc(12px + var(--ui-font-size-offset)); }
.currency-select { min-width: 110px; }
.currency-select :deep(.app-select-trigger) {
  height: 32px;
  min-height: 32px;
  border-radius: 6px;
  font-size: calc(12px + var(--ui-font-size-offset));
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 0 10px;
}
.icon-btn { width: 32px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; }
.icon-btn:hover { background: var(--bg-surface1); }
.summary-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
.summary-card { border: 1px solid var(--border); border-radius: 8px; background: color-mix(in srgb, var(--bg-surface1) 45%, transparent); padding: 12px; }
.summary-card .label { font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text-sub); }
.summary-card .value { margin-top: 8px; font-size: calc(22px + var(--ui-font-size-offset)); font-weight: 600; color: var(--text); }
.summary-card .sub { margin-top: 6px; font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text-dim); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.panel-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.panel-card { border: 1px solid var(--border); border-radius: 8px; background: color-mix(in srgb, var(--bg-surface1) 45%, transparent); padding: 12px; }
.panel-card-wide { grid-column: 1 / -1; }
.panel-card h3 { margin: 0 0 10px; font-size: calc(14px + var(--ui-font-size-offset)); color: var(--text); }
.bar-list { display: flex; flex-direction: column; gap: 10px; }
.bar-title { display: flex; justify-content: space-between; gap: 10px; font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text-sub); }
.bar-sub, .table-sub { font-size: calc(11px + var(--ui-font-size-offset)); color: var(--text-dim); }
.bar-track { width: 100%; height: 7px; border-radius: 999px; background: color-mix(in srgb, var(--bg-base) 65%, transparent); overflow: hidden; }
.bar-fill { height: 100%; border-radius: inherit; }
.flag { margin-right: 4px; }
.table-wrap { overflow-x: auto; }
.table { width: 100%; border-collapse: collapse; }
.table th, .table td { padding: 8px 10px; border-bottom: 1px solid color-mix(in srgb, var(--border) 80%, transparent); text-align: left; font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text); }
.table th { color: var(--text-sub); font-weight: 500; }
.truncate { max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cmd { max-width: 320px; }
.empty { padding: 20px 8px; text-align: center; color: var(--text-dim); font-size: calc(12px + var(--ui-font-size-offset)); }
.traffic-wrap { display: flex; flex-direction: column; gap: 10px; }
.traffic-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
.traffic-item { border: 1px solid var(--border); border-radius: 8px; padding: 8px; display: flex; flex-direction: column; gap: 4px; }
.traffic-item span { color: var(--text-sub); font-size: calc(12px + var(--ui-font-size-offset)); }
.traffic-item strong { color: var(--text); font-size: calc(14px + var(--ui-font-size-offset)); }
.traffic-item .highlight { color: var(--blue); }
.traffic-bar { height: 8px; border-radius: 999px; overflow: hidden; background: color-mix(in srgb, var(--bg-base) 65%, transparent); display: flex; }
.traffic-segment.upload { background: var(--teal); }
.traffic-segment.download { background: var(--yellow); }
.expiry-tag { display: inline-flex; align-items: center; min-height: 22px; border-radius: 999px; border: 1px solid transparent; padding: 0 8px; font-size: calc(11px + var(--ui-font-size-offset)); font-weight: 600; }
.expiry-tag.is-expired, .expiry-tag.is-danger { color: color-mix(in srgb, var(--red) 70%, #fff); border-color: color-mix(in srgb, var(--red) 45%, transparent); background: color-mix(in srgb, var(--red) 22%, transparent); }
.expiry-tag.is-warning { color: color-mix(in srgb, var(--yellow) 72%, #fff); border-color: color-mix(in srgb, var(--yellow) 45%, transparent); background: color-mix(in srgb, var(--yellow) 20%, transparent); }
.expiry-tag.is-normal { color: color-mix(in srgb, var(--green) 72%, #fff); border-color: color-mix(in srgb, var(--green) 42%, transparent); background: color-mix(in srgb, var(--green) 18%, transparent); }
.dialog-mask { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.45); display: flex; align-items: center; justify-content: center; z-index: 3000; }
.dialog-card { width: min(520px, calc(100vw - 32px)); max-height: calc(100vh - 40px); overflow-y: auto; border: 1px solid var(--border); border-radius: 10px; background: var(--bg-surface0); padding: 16px; }
.dialog-card h3 { margin: 0; color: var(--text); font-size: calc(16px + var(--ui-font-size-offset)); }
.dialog-card .hint { margin: 8px 0 12px; color: var(--text-sub); font-size: calc(12px + var(--ui-font-size-offset)); }
.toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 6px 0; font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); }
.toggle-row input { accent-color: var(--blue); }
.dialog-actions { margin-top: 12px; display: flex; justify-content: flex-end; gap: 8px; }
.btn { border: 1px solid var(--border); background: transparent; color: var(--text); padding: 0 12px; cursor: pointer; }
.btn-primary { border-color: var(--blue); background: var(--blue); color: var(--button-text-color); }

@media (max-width: 1150px) { .summary-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 920px) { .panel-grid { grid-template-columns: 1fr; } .panel-card-wide { grid-column: auto; } .traffic-grid { grid-template-columns: 1fr; } }
@media (max-width: 680px) { .summary-grid { grid-template-columns: 1fr; } .header-actions { width: 100%; } .range-switch { width: 100%; } .range-btn { flex: 1; padding: 0 6px; } }
</style>
