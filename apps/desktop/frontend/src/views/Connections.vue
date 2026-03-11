<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import AddConnectionForm from '@/components/AddConnectionForm.vue';
import BatchEditConnectionForm from '@/components/BatchEditConnectionForm.vue';
import { useConnectionsStore } from '@/stores/connections';
import { useSettingsStore } from '@/stores/settings';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useAlertDialog } from '@/composables/useAlertDialog';
import { useSessionLifecycle } from '@/composables/useSessionLifecycle';
import { connectionsApi, type Connection, type Tag } from '@/lib/api';

type SortField = 'last_connected_at' | 'name' | 'type' | 'updated_at' | 'created_at';
type SortOrder = 'asc' | 'desc';

interface ExtendedConnection extends Connection {
  created_at?: number | null;
  updated_at?: number | null;
  last_connected_at?: number | null;
  notes?: string | null;
  provider?: string | null;
  region?: string | null;
  expiry_date?: string | null;
  billing_cycle?: string | null;
  billing_amount?: number | null;
  billing_currency?: string | null;
  tag_ids?: number[];
}

interface ConnectionTestState {
  status: 'idle' | 'testing' | 'success' | 'error';
  resultText: string;
  latency?: number;
  latencyColor?: string;
}

interface TestButtonInfo {
  text: string;
  iconClass: string;
  disabled: boolean;
  title: string;
}

interface ExpiryStatus {
  text: string;
  type: 'danger' | 'warning' | 'success';
}

const LS_SORT_BY_KEY = 'connections_view_sort_by';
const LS_SORT_ORDER_KEY = 'connections_view_sort_order';
const LS_FILTER_TAG_KEY = 'connections_view_filter_tag';

const sortOptions: { value: SortField; label: string }[] = [
  { value: 'last_connected_at', label: '最近连接' },
  { value: 'name', label: '名称' },
  { value: 'type', label: '类型' },
  { value: 'updated_at', label: '最近更新' },
  { value: 'created_at', label: '创建时间' },
];

const store = useConnectionsStore();
const settingsStore = useSettingsStore();
const { list: connections, tags, loading: isLoadingConnections } = storeToRefs(store);
const isLoadingTags = computed(() => isLoadingConnections.value);
const currentLocale = computed(() => settingsStore.locale);
const currentTimezone = computed(() => {
  const fallback = Intl.DateTimeFormat().resolvedOptions().timeZone || 'Asia/Shanghai';
  return settingsStore.get('timezone', fallback);
});
const showConnectionTags = computed(() => settingsStore.getBoolean('showConnectionTags', true));

const router = useRouter();
const { confirm } = useConfirmDialog();
const { alert } = useAlertDialog();
const { connectConnection } = useSessionLifecycle(alert);

const localSortBy = ref<SortField>((localStorage.getItem(LS_SORT_BY_KEY) as SortField) || 'last_connected_at');
if (!sortOptions.some(option => option.value === localSortBy.value)) {
  localSortBy.value = 'last_connected_at';
}

const localSortOrder = ref<SortOrder>(localStorage.getItem(LS_SORT_ORDER_KEY) === 'asc' ? 'asc' : 'desc');

const getInitialSelectedTagId = (): number | null => {
  const storedValue = localStorage.getItem(LS_FILTER_TAG_KEY);
  if (!storedValue || storedValue === 'null') {
    return null;
  }
  const parsedValue = Number.parseInt(storedValue, 10);
  return Number.isNaN(parsedValue) ? null : parsedValue;
};

const selectedTagId = ref<number | null>(getInitialSelectedTagId());
const searchQuery = ref('');

const tagDropdownRef = ref<HTMLElement | null>(null);
const sortDropdownRef = ref<HTMLElement | null>(null);
const isTagDropdownOpen = ref(false);
const isSortDropdownOpen = ref(false);
const tagCreateInput = ref('');
const isCreatingTag = ref(false);
const deletingTagId = ref<number | null>(null);

const sortedTags = computed<Tag[]>(() => [...(tags.value as Tag[])].sort((a, b) => a.name.localeCompare(b.name)));

const selectedTagLabel = computed(() => {
  if (selectedTagId.value === null) {
    return '所有标签';
  }
  return tags.value.find(tag => tag.id === selectedTagId.value)?.name ?? '所有标签';
});

const selectedSortLabel = computed(() => sortOptions.find(option => option.value === localSortBy.value)?.label ?? '最近连接');

const showAddEditConnectionForm = ref(false);
const connectionToEditId = ref<number | null>(null);
const addEditMode = computed<'create' | 'edit'>(() => (connectionToEditId.value === null ? 'create' : 'edit'));

const isBatchEditMode = ref(false);
const selectedConnectionIdsForBatch = ref<Set<number>>(new Set());
const showBatchEditForm = ref(false);
const isDeletingSelectedConnections = ref(false);

const connectionTestStates = ref<Map<number, ConnectionTestState>>(new Map());
const isTestingAll = ref(false);
const isConnectingAll = ref(false);

const isAscending = computed(() => localSortOrder.value === 'asc');

const getConnectionType = (conn: ExtendedConnection): string => String(conn.type ?? 'SSH').toUpperCase();

const getConnectionName = (conn: ExtendedConnection): string => {
  const normalizedName = String(conn.name ?? '').trim();
  if (normalizedName.length > 0) {
    return normalizedName;
  }
  return String(conn.host ?? '未命名连接');
};

const getConnectionTags = (conn: ExtendedConnection): string[] => {
  if (Array.isArray(conn.tag_ids) && conn.tag_ids.length > 0) {
    const mappedTagNames = conn.tag_ids
      .map(tagId => tags.value.find(item => item.id === tagId)?.name)
      .filter((name): name is string => Boolean(name));
    if (mappedTagNames.length > 0) {
      return mappedTagNames;
    }
  }

  if (Array.isArray(conn.tags)) {
    return conn.tags.filter((name): name is string => Boolean(name));
  }

  return [];
};

const normalizeTimestampSeconds = (rawValue: number | null | undefined): number | null => {
  if (rawValue === null || rawValue === undefined || Number.isNaN(rawValue)) {
    return null;
  }
  if (rawValue > 1_000_000_000_000) {
    return Math.floor(rawValue / 1000);
  }
  if (rawValue < 0) {
    return null;
  }
  return rawValue;
};

const formatRelativeTime = (timestampInSeconds: number | null | undefined): string => {
  const normalizedTimestamp = normalizeTimestampSeconds(timestampInSeconds);
  if (!normalizedTimestamp) {
    return '从未连接';
  }

  const nowInSeconds = Math.floor(Date.now() / 1000);
  const deltaSeconds = normalizedTimestamp - nowInSeconds;
  const absDeltaSeconds = Math.abs(deltaSeconds);
  const formatter = new Intl.RelativeTimeFormat(currentLocale.value, { numeric: 'auto' });

  if (absDeltaSeconds < 60) {
    return formatter.format(deltaSeconds, 'second');
  }
  if (absDeltaSeconds < 3600) {
    return formatter.format(Math.round(deltaSeconds / 60), 'minute');
  }
  if (absDeltaSeconds < 86400) {
    return formatter.format(Math.round(deltaSeconds / 3600), 'hour');
  }
  if (absDeltaSeconds < 2592000) {
    return formatter.format(Math.round(deltaSeconds / 86400), 'day');
  }

  try {
    return new Date(normalizedTimestamp * 1000).toLocaleString(currentLocale.value, {
      timeZone: currentTimezone.value,
    });
  } catch {
    return new Date(normalizedTimestamp * 1000).toLocaleString(currentLocale.value);
  }
};

const getTruncatedNotes = (notes: string | null | undefined): string => {
  if (!notes || notes.trim() === '') {
    return '';
  }
  const maxLength = 100;
  if (notes.length <= maxLength) {
    return notes;
  }
  return `${notes.slice(0, maxLength)}...`;
};

const parseExpiryTimestamp = (rawValue: unknown): number | null => {
  if (typeof rawValue !== 'string') {
    return null;
  }

  const trimmed = rawValue.trim();
  if (!trimmed) {
    return null;
  }

  let normalized = trimmed.replace(/\//g, '-').replace(/\s+/, 'T');
  if (/^\d{4}-\d{2}-\d{2}$/.test(normalized)) {
    normalized = `${normalized}T00:00:00`;
  }
  if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(normalized)) {
    normalized = `${normalized}:00`;
  }

  const direct = new Date(normalized);
  if (!Number.isNaN(direct.getTime())) {
    return direct.getTime();
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
  return Number.isNaN(fallback.getTime()) ? null : fallback.getTime();
};

const buildExpiryStatus = (conn: ExtendedConnection): ExpiryStatus | null => {
  const expiryTimestamp = parseExpiryTimestamp(conn.expiry_date);
  if (expiryTimestamp === null) {
    return null;
  }

  const now = Date.now();
  const diffMs = expiryTimestamp - now;
  const dayMs = 24 * 60 * 60 * 1000;
  const hourMs = 60 * 60 * 1000;
  const diffDays = Math.floor(diffMs / dayMs);
  const diffHours = Math.max(0, Math.floor((diffMs % dayMs) / hourMs));

  if (diffMs < 0) {
    return { text: '已过期', type: 'danger' };
  }
  if (diffDays === 0) {
    return { text: `今天到期 (${diffHours}小时)`, type: 'danger' };
  }
  if (diffDays < 7) {
    return { text: `${diffDays}天${diffHours}小时后到期`, type: 'danger' };
  }
  if (diffDays < 30) {
    return { text: `${diffDays}天后到期`, type: 'warning' };
  }

  const months = Math.max(1, Math.floor(diffDays / 30));
  return { text: `${months}个月后到期`, type: 'success' };
};

const expiryStatusMap = computed(() => {
  const map = new Map<number, ExpiryStatus>();
  (connections.value as ExtendedConnection[]).forEach((conn) => {
    const status = buildExpiryStatus(conn);
    if (status) {
      map.set(conn.id, status);
    }
  });
  return map;
});

const getExpiryStatus = (connectionId: number): ExpiryStatus | null => {
  return expiryStatusMap.value.get(connectionId) ?? null;
};

const filteredAndSortedConnections = computed<ExtendedConnection[]>(() => {
  const query = searchQuery.value.trim().toLowerCase();
  const selectedTagName = selectedTagId.value === null
    ? null
    : tags.value.find(tag => tag.id === selectedTagId.value)?.name ?? null;
  const sortBy = localSortBy.value;
  const factor = localSortOrder.value === 'desc' ? -1 : 1;

  let filteredByTag = [...(connections.value as ExtendedConnection[])];
  if (selectedTagName) {
    filteredByTag = filteredByTag.filter(conn => getConnectionTags(conn).includes(selectedTagName));
  }

  let searchedConnections = filteredByTag;
  if (query) {
    searchedConnections = filteredByTag.filter(conn => {
      const noteText = String(conn.notes ?? '').toLowerCase();
      return (
        getConnectionName(conn).toLowerCase().includes(query)
        || String(conn.username ?? '').toLowerCase().includes(query)
        || String(conn.host ?? '').toLowerCase().includes(query)
        || String(conn.port ?? '').includes(query)
        || noteText.includes(query)
      );
    });
  }

  return searchedConnections.sort((a, b) => {
    let result = 0;
    switch (sortBy) {
      case 'name':
        result = getConnectionName(a).localeCompare(getConnectionName(b)) * factor;
        break;
      case 'type':
        result = getConnectionType(a).localeCompare(getConnectionType(b)) * factor;
        break;
      case 'created_at':
        result = (Number(a.created_at ?? 0) - Number(b.created_at ?? 0)) * factor;
        break;
      case 'updated_at':
        result = (Number(a.updated_at ?? 0) - Number(b.updated_at ?? 0)) * factor;
        break;
      case 'last_connected_at': {
        const emptyFallback = localSortOrder.value === 'desc' ? -Infinity : Infinity;
        const valA = normalizeTimestampSeconds(a.last_connected_at) ?? emptyFallback;
        const valB = normalizeTimestampSeconds(b.last_connected_at) ?? emptyFallback;
        if (valA === valB) {
          result = 0;
          break;
        }
        result = (valA < valB ? -1 : 1) * factor;
        break;
      }
      default:
        result = 0;
    }

    if (result !== 0) {
      return result;
    }

    const fallbackByName = getConnectionName(a).localeCompare(getConnectionName(b));
    if (fallbackByName !== 0) {
      return fallbackByName * factor;
    }

    return (Number(a.id) - Number(b.id)) * factor;
  });
});

const getErrorMessage = (error: unknown): string => {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  if (typeof error === 'string' && error.length > 0) {
    return error;
  }
  return '未知错误';
};

const handleCreateTagFromFilter = async () => {
  if (isCreatingTag.value || isLoadingTags.value) {
    return;
  }

  const name = tagCreateInput.value.trim();
  if (!name) {
    return;
  }

  isCreatingTag.value = true;
  try {
    const existing = (tags.value as Tag[]).find(tag => tag.name === name);
    const tagId = existing ? existing.id : await connectionsApi.tagCreate(name);
    await store.fetch();
    selectedTagId.value = tagId;
    tagCreateInput.value = '';
    isTagDropdownOpen.value = false;
  } catch (error: unknown) {
    await alert('创建标签失败', getErrorMessage(error));
  } finally {
    isCreatingTag.value = false;
  }
};

const handleDeleteTagFromFilter = async (tag: Tag) => {
  if (deletingTagId.value !== null || isLoadingTags.value) {
    return;
  }

  const confirmed = await confirm('删除标签', `确定删除标签“${tag.name}”吗？`);
  if (!confirmed) {
    return;
  }

  deletingTagId.value = tag.id;
  try {
    const deleted = await connectionsApi.tagDelete(tag.id);
    if (!deleted) {
      await alert('删除标签失败', '后端返回删除失败');
      return;
    }

    if (selectedTagId.value === tag.id) {
      selectedTagId.value = null;
    }
    await store.fetch();
  } catch (error: unknown) {
    await alert('删除标签失败', getErrorMessage(error));
  } finally {
    deletingTagId.value = null;
  }
};

const toggleSortOrder = () => {
  localSortOrder.value = localSortOrder.value === 'asc' ? 'desc' : 'asc';
};

const closeToolbarDropdowns = () => {
  isTagDropdownOpen.value = false;
  isSortDropdownOpen.value = false;
};

const toggleTagDropdown = () => {
  if (isLoadingTags.value) {
    return;
  }
  isTagDropdownOpen.value = !isTagDropdownOpen.value;
  if (isTagDropdownOpen.value) {
    isSortDropdownOpen.value = false;
  }
};

const toggleSortDropdown = () => {
  isSortDropdownOpen.value = !isSortDropdownOpen.value;
  if (isSortDropdownOpen.value) {
    isTagDropdownOpen.value = false;
  }
};

const selectTag = (tagId: number | null) => {
  selectedTagId.value = tagId;
  isTagDropdownOpen.value = false;
};

const selectSortBy = (sortBy: SortField) => {
  localSortBy.value = sortBy;
  isSortDropdownOpen.value = false;
};

const handleToolbarDropdownPointerDown = (event: MouseEvent) => {
  const target = event.target as Node | null;
  if (!target) {
    return;
  }

  if (tagDropdownRef.value && !tagDropdownRef.value.contains(target)) {
    isTagDropdownOpen.value = false;
  }

  if (sortDropdownRef.value && !sortDropdownRef.value.contains(target)) {
    isSortDropdownOpen.value = false;
  }
};

const handleToolbarDropdownKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    closeToolbarDropdowns();
  }
};

watch(localSortBy, (newValue) => {
  localStorage.setItem(LS_SORT_BY_KEY, newValue);
});

watch(localSortOrder, (newValue) => {
  localStorage.setItem(LS_SORT_ORDER_KEY, newValue);
});

watch(showConnectionTags, (value) => {
  if (!value) {
    selectedTagId.value = null;
  }
  closeToolbarDropdowns();
});

watch(selectedTagId, (newValue) => {
  localStorage.setItem(LS_FILTER_TAG_KEY, newValue === null ? 'null' : String(newValue));
});

const openAddConnectionForm = () => {
  connectionToEditId.value = null;
  showAddEditConnectionForm.value = true;
};

const openEditConnectionForm = (conn: ExtendedConnection) => {
  connectionToEditId.value = conn.id;
  showAddEditConnectionForm.value = true;
};

const handleFormClose = () => {
  showAddEditConnectionForm.value = false;
  connectionToEditId.value = null;
};

const handleConnectionModified = async () => {
  handleFormClose();
  await store.fetch();
};

const toggleBatchEditMode = () => {
  isBatchEditMode.value = !isBatchEditMode.value;
  if (!isBatchEditMode.value) {
    selectedConnectionIdsForBatch.value.clear();
  }
};

const handleConnectionClick = (connectionId: number) => {
  if (!isBatchEditMode.value) {
    return;
  }
  if (selectedConnectionIdsForBatch.value.has(connectionId)) {
    selectedConnectionIdsForBatch.value.delete(connectionId);
  } else {
    selectedConnectionIdsForBatch.value.add(connectionId);
  }
};

const isConnectionSelectedForBatch = (connectionId: number): boolean => selectedConnectionIdsForBatch.value.has(connectionId);

const selectAllConnections = () => {
  if (!isBatchEditMode.value) {
    return;
  }
  filteredAndSortedConnections.value.forEach(conn => selectedConnectionIdsForBatch.value.add(conn.id));
};

const deselectAllConnections = () => {
  if (!isBatchEditMode.value) {
    return;
  }
  selectedConnectionIdsForBatch.value.clear();
};

const invertSelection = () => {
  if (!isBatchEditMode.value) {
    return;
  }
  filteredAndSortedConnections.value.forEach((conn) => {
    if (selectedConnectionIdsForBatch.value.has(conn.id)) {
      selectedConnectionIdsForBatch.value.delete(conn.id);
    } else {
      selectedConnectionIdsForBatch.value.add(conn.id);
    }
  });
};

const openBatchEditModal = async () => {
  if (selectedConnectionIdsForBatch.value.size === 0) {
    await alert('提示', '请至少选择一个连接进行编辑。');
    return;
  }
  showBatchEditForm.value = true;
};

const handleBatchEditSaved = async () => {
  showBatchEditForm.value = false;
  selectedConnectionIdsForBatch.value.clear();
  await store.fetch();
};

const handleBatchEditFormClose = () => {
  showBatchEditForm.value = false;
};

const handleBatchDeleteConnections = async () => {
  if (selectedConnectionIdsForBatch.value.size === 0 || isDeletingSelectedConnections.value) {
    return;
  }

  const confirmed = await confirm(
    '删除连接',
    `您确定要删除选中的 ${selectedConnectionIdsForBatch.value.size} 个连接吗？此操作无法撤销。`,
  );
  if (!confirmed) {
    return;
  }

  isDeletingSelectedConnections.value = true;
  try {
    const idsToDelete = Array.from(selectedConnectionIdsForBatch.value);
    await Promise.all(idsToDelete.map(id => connectionsApi.delete(id)));
    selectedConnectionIdsForBatch.value.clear();
    await store.fetch();
    await alert('成功', '选中的连接已成功删除。');
  } catch (error) {
    await alert('错误', `批量删除连接失败: ${getErrorMessage(error)}`);
  } finally {
    isDeletingSelectedConnections.value = false;
  }
};

const getLatencyColorString = (latencyMs?: number): string => {
  if (latencyMs === undefined) {
    return 'var(--text)';
  }
  if (latencyMs < 100) {
    return 'var(--green)';
  }
  if (latencyMs < 300) {
    return 'var(--yellow)';
  }
  return 'var(--red)';
};

const testableConnectionTypes = new Set(['SSH', 'RDP', 'VNC']);

const handleTestSingleConnection = async (conn: ExtendedConnection) => {
  const connType = getConnectionType(conn);
  if (!conn.id || !testableConnectionTypes.has(connType)) {
    return;
  }

  connectionTestStates.value.set(conn.id, {
    status: 'testing',
    resultText: '测试中...',
  });

  const startedAt = performance.now();
  try {
    const testSuccess = await connectionsApi.test(conn.id);
    if (testSuccess) {
      const latency = Math.max(1, Math.round(performance.now() - startedAt));
      connectionTestStates.value.set(conn.id, {
        status: 'success',
        resultText: `${latency}ms`,
        latency,
        latencyColor: getLatencyColorString(latency),
      });
    } else {
      connectionTestStates.value.set(conn.id, {
        status: 'error',
        resultText: '测试失败',
      });
    }
  } catch (error) {
    connectionTestStates.value.set(conn.id, {
      status: 'error',
      resultText: getErrorMessage(error),
    });
  }
};

const handleTestAllFilteredConnections = async () => {
  if (isTestingAll.value || isLoadingConnections.value) {
    return;
  }

  const connectionsToTest = filteredAndSortedConnections.value.filter(conn =>
    testableConnectionTypes.has(getConnectionType(conn)),
  );
  if (connectionsToTest.length === 0) {
    return;
  }

  isTestingAll.value = true;
  try {
    await Promise.all(connectionsToTest.map(conn => handleTestSingleConnection(conn)));
  } finally {
    isTestingAll.value = false;
  }
};

const getSingleTestButtonInfo = (connId: number, connType: string): TestButtonInfo => {
  if (!testableConnectionTypes.has(connType)) {
    return {
      text: '测试',
      iconClass: 'fas fa-plug',
      disabled: true,
      title: '当前连接类型不支持测试。',
    };
  }

  const state = connectionTestStates.value.get(connId);
  if (state?.status === 'testing') {
    return {
      text: '测试中',
      iconClass: 'fas fa-spinner fa-spin',
      disabled: true,
      title: '测试中',
    };
  }

  return {
    text: '测试',
    iconClass: 'fas fa-plug',
    disabled: false,
    title: '测试',
  };
};

const getConnectionIconClass = (conn: ExtendedConnection): string => {
  const connType = getConnectionType(conn);
  if (connType === 'VNC') {
    return 'fas fa-plug';
  }
  if (connType === 'RDP') {
    return 'fas fa-desktop';
  }
  return 'fas fa-server';
};

const connectTo = async (conn: ExtendedConnection) => {
  await connectConnection(conn, {
    onSessionActivated: async () => {
      await router.push('/workspace');
    },
  });
};

const handleConnectAllFilteredConnections = async () => {
  if (isConnectingAll.value || isLoadingConnections.value) {
    return;
  }

  const sshConnectionsToConnect = filteredAndSortedConnections.value.filter(conn => getConnectionType(conn) === 'SSH');
  if (sshConnectionsToConnect.length === 0) {
    return;
  }

  isConnectingAll.value = true;
  try {
    await Promise.allSettled(sshConnectionsToConnect.map((conn) => connectTo(conn)));
  } finally {
    isConnectingAll.value = false;
  }
};

onMounted(async () => {
  document.addEventListener('mousedown', handleToolbarDropdownPointerDown);
  window.addEventListener('keydown', handleToolbarDropdownKeydown);

  await Promise.all([
    store.fetch(),
    settingsStore.loadAll().catch(() => undefined),
  ]);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleToolbarDropdownPointerDown);
  window.removeEventListener('keydown', handleToolbarDropdownKeydown);
});
</script>

<template>
  <div class="connections-page">
    <div class="connections-shell">
      <h1 class="page-title">连接管理</h1>

      <div class="connections-card">
        <div class="toolbar">
          <h2 class="toolbar-title">连接列表 ({{ filteredAndSortedConnections.length }})</h2>
          <div class="toolbar-controls">
            <div class="batch-toggle-group">
              <label for="batch-edit-toggle" class="batch-toggle-label">批量修改</label>
              <button
                id="batch-edit-toggle"
                class="batch-toggle"
                :class="{ 'is-active': isBatchEditMode }"
                role="switch"
                :aria-checked="isBatchEditMode"
                @click="toggleBatchEditMode"
              >
                <span class="batch-toggle-thumb"></span>
              </button>
            </div>

            <input
              v-model="searchQuery"
              type="text"
              placeholder="搜索连接..."
              class="search-input"
            />

            <div v-if="showConnectionTags" class="select-group">
              <div
                ref="tagDropdownRef"
                class="custom-select"
                :class="{ 'is-open': isTagDropdownOpen, 'is-disabled': isLoadingTags }"
              >
                <button
                  type="button"
                  class="custom-select-trigger"
                  :disabled="isLoadingTags"
                  :aria-expanded="isTagDropdownOpen ? 'true' : 'false'"
                  aria-haspopup="listbox"
                  aria-label="按标签过滤连接"
                  @click="toggleTagDropdown"
                >
                  <span class="custom-select-trigger-text">{{ selectedTagLabel }}</span>
                  <i class="fas fa-chevron-down custom-select-trigger-icon"></i>
                </button>
                <div v-if="isTagDropdownOpen" class="custom-select-menu" role="listbox" aria-label="按标签过滤连接">
                  <div class="custom-select-menu-toolbar" @click.stop>
                    <input
                      v-model="tagCreateInput"
                      class="custom-select-menu-input"
                      type="text"
                      placeholder="新建标签..."
                      :disabled="isLoadingTags || isCreatingTag"
                      @keydown.enter.prevent="handleCreateTagFromFilter"
                    />
                    <button
                      type="button"
                      class="custom-select-menu-btn"
                      title="新建标签"
                      :disabled="isLoadingTags || isCreatingTag || !tagCreateInput.trim()"
                      @click="handleCreateTagFromFilter"
                    >
                      <i v-if="isCreatingTag" class="fas fa-spinner fa-spin"></i>
                      <i v-else class="fas fa-plus"></i>
                    </button>
                  </div>
                  <div class="custom-select-menu-divider"></div>
                  <button
                    type="button"
                    class="custom-select-option"
                    :class="{ 'is-active': selectedTagId === null }"
                    @click="selectTag(null)"
                  >
                    所有标签
                  </button>
                  <button v-if="isLoadingTags" type="button" class="custom-select-option" disabled>
                    加载中...
                  </button>
                  <div
                    v-for="tag in sortedTags"
                    :key="tag.id"
                    class="custom-select-option-row"
                  >
                    <button
                      type="button"
                      class="custom-select-option custom-select-option-main"
                      :class="{ 'is-active': selectedTagId === tag.id }"
                      @click="selectTag(tag.id)"
                    >
                      {{ tag.name }}
                    </button>
                    <button
                      type="button"
                      class="custom-select-option-delete"
                      title="删除标签"
                      :disabled="isLoadingTags || deletingTagId === tag.id"
                      @click.stop="handleDeleteTagFromFilter(tag)"
                    >
                      <i v-if="deletingTagId === tag.id" class="fas fa-spinner fa-spin"></i>
                      <i v-else class="fas fa-trash-alt"></i>
                    </button>
                  </div>
                </div>
              </div>

              <div ref="sortDropdownRef" class="custom-select" :class="{ 'is-open': isSortDropdownOpen }">
                <button
                  type="button"
                  class="custom-select-trigger"
                  :aria-expanded="isSortDropdownOpen ? 'true' : 'false'"
                  aria-haspopup="listbox"
                  aria-label="连接排序"
                  @click="toggleSortDropdown"
                >
                  <span class="custom-select-trigger-text">{{ selectedSortLabel }}</span>
                  <i class="fas fa-chevron-down custom-select-trigger-icon"></i>
                </button>
                <div v-if="isSortDropdownOpen" class="custom-select-menu" role="listbox" aria-label="连接排序">
                  <button
                    v-for="option in sortOptions"
                    :key="option.value"
                    type="button"
                    class="custom-select-option"
                    :class="{ 'is-active': localSortBy === option.value }"
                    @click="selectSortBy(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
              </div>

              <button
                class="icon-button"
                :title="isAscending ? '升序' : '降序'"
                :aria-label="isAscending ? '升序' : '降序'"
                @click="toggleSortOrder"
              >
                <i :class="['fas', isAscending ? 'fa-arrow-up-a-z' : 'fa-arrow-down-z-a']"></i>
              </button>
            </div>

            <button class="square-primary-button" title="新建连接" @click="openAddConnectionForm">
              <i class="fas fa-plus"></i>
            </button>

            <button
              class="toolbar-action-button"
              :disabled="isTestingAll || isLoadingConnections || !filteredAndSortedConnections.some(conn => getConnectionType(conn) === 'SSH')"
              title="测试全部筛选的 SSH 连接"
              @click="handleTestAllFilteredConnections"
            >
              <i v-if="isTestingAll" class="fas fa-spinner fa-spin"></i>
              <i v-else class="fas fa-check-double"></i>
              <span class="button-label">测试全部</span>
            </button>

            <button
              class="toolbar-action-button"
              :disabled="isConnectingAll || isLoadingConnections || !filteredAndSortedConnections.some(conn => getConnectionType(conn) === 'SSH')"
              @click="handleConnectAllFilteredConnections"
            >
              <i v-if="isConnectingAll" class="fas fa-spinner fa-spin"></i>
              <i v-else class="fas fa-network-wired"></i>
              <span class="button-label">连接全部</span>
            </button>
          </div>
        </div>

        <div v-if="isBatchEditMode" class="batch-actions">
          <button class="batch-ghost-button" @click="selectAllConnections">全选 ({{ selectedConnectionIdsForBatch.size }})</button>
          <button class="batch-ghost-button" @click="deselectAllConnections">取消全选</button>
          <button class="batch-ghost-button" @click="invertSelection">反选</button>
          <button
            class="batch-primary-button"
            :disabled="selectedConnectionIdsForBatch.size === 0"
            @click="openBatchEditModal"
          >
            <i class="fas fa-edit"></i>
            <span>编辑选中</span>
          </button>
          <button
            class="batch-danger-button"
            :disabled="selectedConnectionIdsForBatch.size === 0 || isDeletingSelectedConnections"
            title="删除选中的连接"
            @click="handleBatchDeleteConnections"
          >
            <i v-if="isDeletingSelectedConnections" class="fas fa-spinner fa-spin"></i>
            <i v-else class="fas fa-trash-alt"></i>
            <span>删除选中</span>
          </button>
        </div>

        <div class="list-body">
          <div v-if="isLoadingConnections && filteredAndSortedConnections.length === 0" class="empty-state">加载中...</div>

          <ul v-else-if="filteredAndSortedConnections.length > 0" class="connection-list">
            <li
              v-for="conn in filteredAndSortedConnections"
              :key="conn.id"
              class="connection-item"
              :class="{
                'is-selectable': isBatchEditMode,
                'is-selected': isBatchEditMode && isConnectionSelectedForBatch(conn.id),
              }"
              @click="handleConnectionClick(conn.id)"
            >
              <div class="connection-main">
                <span class="connection-name" :title="getConnectionName(conn)">
                  <i :class="[getConnectionIconClass(conn), 'connection-type-icon']"></i>
                  <span class="connection-name-text">{{ getConnectionName(conn) }}</span>
                </span>

                <span class="connection-detail" :title="`${conn.username}@${conn.host}:${conn.port}`">
                  {{ conn.username }}@{{ conn.host }}:{{ conn.port }}
                </span>

                <span class="connection-last-time">上次连接: {{ formatRelativeTime(conn.last_connected_at) }}</span>

                <div v-if="conn.notes && conn.notes.trim() !== ''" class="connection-notes">
                  <span class="connection-notes-label">备注:</span>
                  <span class="connection-notes-text" :title="conn.notes">
                    {{ getTruncatedNotes(conn.notes) }}
                  </span>
                </div>

                <div v-if="showConnectionTags && getConnectionTags(conn).length > 0" class="connection-tags">
                  <span
                    v-for="tagName in getConnectionTags(conn)"
                    :key="tagName"
                    class="connection-tag"
                  >
                    {{ tagName }}
                  </span>
                </div>

                <div
                  v-if="getConnectionType(conn) === 'SSH' && connectionTestStates.get(conn.id) && connectionTestStates.get(conn.id)?.status !== 'idle'"
                  class="connection-test-result"
                >
                  <div v-if="connectionTestStates.get(conn.id)?.status === 'testing'" class="test-result testing">
                    <i class="fas fa-spinner fa-spin"></i>
                    <span>测试中...</span>
                  </div>
                  <div
                    v-else-if="connectionTestStates.get(conn.id)?.status === 'success'"
                    class="test-result success"
                    :style="{ color: connectionTestStates.get(conn.id)?.latencyColor || 'inherit' }"
                  >
                    <i class="fas fa-check-circle"></i>
                    <span>{{ connectionTestStates.get(conn.id)?.resultText }}</span>
                  </div>
                  <div v-else class="test-result error">
                    <i class="fas fa-times-circle"></i>
                    <span>错误: {{ connectionTestStates.get(conn.id)?.resultText }}</span>
                  </div>
                </div>
              </div>

              <div class="connection-actions">
                <div v-if="getExpiryStatus(conn.id)" class="connection-expiry">
                  <span class="expiry-tag" :class="`is-${getExpiryStatus(conn.id)?.type}`">
                    {{ getExpiryStatus(conn.id)?.text }}
                  </span>
                </div>

                <div class="connection-action-buttons">
                  <button
                    v-if="getConnectionType(conn) === 'SSH'"
                    class="row-ghost-button"
                    :disabled="isBatchEditMode || getSingleTestButtonInfo(conn.id, getConnectionType(conn)).disabled"
                    :title="getSingleTestButtonInfo(conn.id, getConnectionType(conn)).title"
                    @click.stop="handleTestSingleConnection(conn)"
                  >
                    <i :class="getSingleTestButtonInfo(conn.id, getConnectionType(conn)).iconClass"></i>
                    <span v-if="getSingleTestButtonInfo(conn.id, getConnectionType(conn)).text !== '测试中'">
                      {{ getSingleTestButtonInfo(conn.id, getConnectionType(conn)).text }}
                    </span>
                  </button>

                  <button
                    class="row-ghost-button"
                    :disabled="isBatchEditMode"
                    @click.stop="openEditConnectionForm(conn)"
                  >
                    <i class="fas fa-pencil-alt"></i>
                    <span>编辑</span>
                  </button>

                  <button
                    class="row-primary-button"
                    :disabled="isBatchEditMode"
                    @click.stop="connectTo(conn)"
                  >
                    <span>连接</span>
                  </button>
                </div>
              </div>
            </li>
          </ul>

          <div v-else-if="!isLoadingConnections && searchQuery && filteredAndSortedConnections.length === 0" class="empty-state">
            没有连接匹配搜索条件
          </div>
          <div v-else-if="!isLoadingConnections && selectedTagId !== null && filteredAndSortedConnections.length === 0" class="empty-state">
            该标签下没有连接记录
          </div>
          <div v-else class="empty-state">没有连接记录</div>
        </div>
      </div>
    </div>

    <AddConnectionForm
      :visible="showAddEditConnectionForm"
      :mode="addEditMode"
      :connection-id="connectionToEditId ?? undefined"
      @close="handleFormClose"
      @saved="handleConnectionModified"
    />

    <BatchEditConnectionForm
      :visible="showBatchEditForm"
      :connection-ids="Array.from(selectedConnectionIdsForBatch)"
      @saved="handleBatchEditSaved"
      @cancel="handleBatchEditFormClose"
    />
  </div>
</template>

<style scoped>
.connections-page {
  height: 100%;
  overflow-y: auto;
  padding: 20px 24px 24px;
  background: var(--bg-base);
  color: var(--text);
}

.connections-shell {
  margin: 0 auto;
  max-width: 1120px;
}

.page-title {
  margin-bottom: 24px;
  font-size: calc(26px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text);
}

.connections-card {
  min-height: 400px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-surface0);
  overflow: hidden;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}

.toolbar-title {
  flex-shrink: 0;
  font-size: calc(18px + var(--ui-font-size-offset));
  font-weight: 500;
  color: var(--text);
}

.toolbar-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.batch-toggle-group {
  display: flex;
  align-items: center;
  margin-right: 6px;
}

.batch-toggle-label {
  margin-right: 8px;
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text-sub);
}

.batch-toggle {
  position: relative;
  width: 44px;
  height: 24px;
  border: 2px solid transparent;
  border-radius: 999px;
  background: var(--ui-switch-off);
  cursor: pointer;
  transition: background 0.2s ease;
}

.batch-toggle.is-active {
  background: var(--blue);
}

.batch-toggle-thumb {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--ui-switch-knob);
  transition: transform 0.2s ease;
}

.batch-toggle.is-active .batch-toggle-thumb {
  transform: translateX(20px);
}

.search-input,
.custom-select-trigger,
.icon-button,
.square-primary-button,
.toolbar-action-button {
  height: 32px;
}

.search-input,
.custom-select-trigger {
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  outline: none;
}

.search-input {
  width: 192px;
  padding: 0 10px;
}

.search-input:focus,
.custom-select-trigger:focus-visible,
.icon-button:focus,
.square-primary-button:focus,
.toolbar-action-button:focus,
.batch-ghost-button:focus,
.batch-primary-button:focus,
.batch-danger-button:focus,
.row-ghost-button:focus,
.row-primary-button:focus {
  box-shadow: 0 0 0 1px var(--blue);
}

.select-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.custom-select {
  position: relative;
  min-width: 116px;
}

.custom-select-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
  cursor: pointer;
  text-align: left;
}

.custom-select.is-open .custom-select-trigger {
  border-color: var(--blue);
}

.custom-select.is-disabled .custom-select-trigger {
  cursor: not-allowed;
  opacity: 0.65;
}

.custom-select-trigger-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-trigger-icon {
  color: var(--text-sub);
  font-size: calc(11px + var(--ui-font-size-offset));
  transition: transform 0.15s ease;
}

.custom-select.is-open .custom-select-trigger-icon {
  transform: rotate(180deg);
}

.custom-select-menu {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  border: 1px solid var(--border);
  border-radius: 0 0 10px 10px;
  background: var(--bg-base);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  z-index: 80;
  max-height: 260px;
  overflow-y: auto;
}

.custom-select-option {
  width: 100%;
  min-height: 30px;
  border: none;
  background: transparent;
  color: var(--text);
  text-align: left;
  padding: 5px 10px;
  font-size: calc(13px + var(--ui-font-size-offset));
  line-height: 1.2;
  cursor: pointer;
}

.custom-select-option:hover {
  background: color-mix(in srgb, var(--blue) 20%, var(--bg-base));
}

.custom-select-option.is-active {
  background: var(--blue);
  color: var(--button-text-color);
}

.custom-select-option:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.custom-select-menu-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px;
}

.custom-select-menu-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  outline: none;
}

.custom-select-menu-input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.custom-select-menu-btn {
  width: 30px;
  height: 30px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}

.custom-select-menu-btn:hover {
  background: var(--bg-surface1);
}

.custom-select-menu-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.custom-select-menu-divider {
  height: 1px;
  background: var(--border);
}

.custom-select-option-row {
  display: flex;
  align-items: stretch;
}

.custom-select-option-main {
  flex: 1;
}

.custom-select-option-delete {
  width: 34px;
  border: none;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.custom-select-option-delete:hover {
  background: color-mix(in srgb, var(--red) 14%, transparent);
  color: var(--red);
}

.custom-select-option-delete:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.icon-button {
  width: 32px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}

.icon-button:hover {
  background: var(--bg-surface1);
}

.square-primary-button {
  width: 32px;
  border: none;
  border-radius: 6px;
  background: var(--blue);
  color: var(--button-text-color);
  cursor: pointer;
}

.square-primary-button:hover {
  filter: brightness(1.05);
}

.toolbar-action-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  border: none;
  border-radius: 6px;
  background: var(--blue);
  color: var(--button-text-color);
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
}

.toolbar-action-button:hover {
  filter: brightness(1.05);
}

.toolbar-action-button:disabled,
.batch-primary-button:disabled,
.batch-danger-button:disabled,
.row-ghost-button:disabled,
.row-primary-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.batch-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface0);
}

.batch-ghost-button,
.batch-primary-button,
.batch-danger-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border-radius: 6px;
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
}

.batch-ghost-button {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-sub);
}

.batch-ghost-button:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.batch-primary-button {
  border: none;
  background: var(--blue);
  color: var(--button-text-color);
}

.batch-danger-button {
  border: 1px solid var(--red);
  background: var(--red);
  color: var(--button-text-color);
}

.batch-danger-button:hover {
  filter: brightness(0.95);
}

.list-body {
  padding: 16px;
}

.connection-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.connection-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 48%, transparent);
  transition: background 0.15s ease, border-color 0.15s ease;
}

.connection-item:hover {
  background: color-mix(in srgb, var(--bg-surface1) 75%, transparent);
}

.connection-item.is-selectable {
  cursor: pointer;
}

.connection-item.is-selected {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.connection-main {
  flex: 1;
  min-width: 0;
  margin-right: 6px;
}

.connection-name {
  display: flex;
  align-items: center;
  font-size: calc(14px + var(--ui-font-size-offset));
  font-weight: 500;
  color: var(--text);
}

.connection-type-icon {
  width: 16px;
  margin-right: 8px;
  text-align: center;
  color: var(--text-sub);
}

.connection-name-text,
.connection-detail,
.connection-last-time {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.connection-detail {
  margin-top: 2px;
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text-sub);
}

.connection-last-time {
  margin-top: 1px;
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-dim);
}

.connection-notes {
  margin-top: 6px;
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-sub);
}

.connection-notes-label {
  color: var(--text-dim);
  font-weight: 500;
}

.connection-notes-text {
  margin-left: 4px;
  word-break: break-word;
}

.connection-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 6px;
}

.connection-tag {
  padding: 2px 6px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-base);
  color: var(--text-sub);
  font-size: calc(11px + var(--ui-font-size-offset));
}

.connection-test-result {
  margin-top: 7px;
  padding-top: 6px;
  border-top: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
  font-size: calc(12px + var(--ui-font-size-offset));
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
}

.test-result.testing {
  color: var(--text-sub);
}

.test-result.error {
  color: var(--red);
}

.connection-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.connection-action-buttons {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
}

.connection-expiry {
  width: 100%;
  display: flex;
  justify-content: flex-end;
}

.expiry-tag {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  white-space: nowrap;
}

.expiry-tag.is-danger {
  color: color-mix(in srgb, var(--red) 70%, #fff);
  border-color: color-mix(in srgb, var(--red) 45%, transparent);
  background: color-mix(in srgb, var(--red) 22%, transparent);
}

.expiry-tag.is-warning {
  color: color-mix(in srgb, var(--yellow) 72%, #fff);
  border-color: color-mix(in srgb, var(--yellow) 45%, transparent);
  background: color-mix(in srgb, var(--yellow) 20%, transparent);
}

.expiry-tag.is-success {
  color: color-mix(in srgb, var(--green) 72%, #fff);
  border-color: color-mix(in srgb, var(--green) 42%, transparent);
  background: color-mix(in srgb, var(--green) 18%, transparent);
}

.row-ghost-button,
.row-primary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 36px;
  border-radius: 6px;
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
  cursor: pointer;
}

.row-ghost-button {
  padding: 0 12px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
}

.row-ghost-button:hover {
  background: var(--bg-surface1);
}

.row-primary-button {
  padding: 0 16px;
  border: none;
  background: var(--blue);
  color: var(--button-text-color);
}

.row-primary-button:hover {
  filter: brightness(1.05);
}

.empty-state {
  padding: 34px 8px;
  text-align: center;
  color: var(--text-dim);
  font-size: calc(14px + var(--ui-font-size-offset));
}

@media (max-width: 960px) {
  .connections-page {
    padding: 16px;
  }

  .toolbar-controls {
    width: 100%;
  }

  .search-input {
    width: 100%;
    min-width: 0;
  }

  .select-group {
    flex: 1;
    min-width: 0;
  }

  .custom-select {
    flex: 1;
    min-width: 0;
  }

  .button-label {
    display: none;
  }

  .connection-item {
    flex-wrap: wrap;
  }

  .connection-actions {
    width: 100%;
    align-items: flex-start;
  }

  .connection-action-buttons {
    justify-content: flex-start;
  }

  .connection-expiry {
    justify-content: flex-start;
  }
}
</style>
