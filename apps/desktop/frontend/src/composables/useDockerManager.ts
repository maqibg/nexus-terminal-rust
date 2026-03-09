import { computed, onMounted, onUnmounted, readonly, ref, watch, type ComputedRef } from 'vue';
import { sshApi } from '@/lib/api';
import type { SshExecResult } from '@/lib/api-ssh';
import { useSettingsStore } from '@/stores/settings';
import { useUINotificationStore } from '@/stores/uiNotifications';

interface PortInfo {
  IP?: string;
  PrivatePort: number;
  PublicPort?: number;
  Type: string;
}

export interface DockerStats {
  ID: string;
  Name: string;
  CPUPerc: string;
  MemUsage: string;
  MemPerc: string;
  NetIO: string;
  BlockIO: string;
  PIDs: string;
}

export interface DockerContainer {
  id: string;
  Names: string[];
  Image: string;
  ImageID: string;
  Command: string;
  Created: number | string;
  State: string;
  Status: string;
  Ports: PortInfo[];
  Labels: Record<string, string>;
  stats?: DockerStats | null;
}

type DockerCommand = 'start' | 'stop' | 'restart' | 'remove';
type SshConnectionStatus = 'connecting' | 'connected' | 'disconnected';

const STATUS_TIMEOUT_MS = 15_000;
const COMMAND_TIMEOUT_MS = 30_000;
const DEFAULT_REFRESH_SECONDS = 5;
const SUDO_COMMAND_PREFIX = 'sudo -n ';
const SUDO_PASSWORD_REQUIRED_PATTERNS = [
  'sudo: a password is required',
  'sudo: no tty present',
  'sudo: a terminal is required',
  'sudo: no password was provided',
  'sorry, try again',
];
const DOCKER_PERMISSION_PATTERNS = [
  'permission denied while trying to connect to the docker daemon socket',
  'Cannot connect to the Docker daemon',
];
const dockerSudoPasswordCache = new Map<string, string>();
const dockerSudoPromptedSessions = new Set<string>();

function getSessionKey(sessionId: string | null): string | null {
  return sessionId?.trim() ? sessionId : null;
}

function getCachedSudoPassword(sessionId: string | null): string {
  const key = getSessionKey(sessionId);
  return key ? dockerSudoPasswordCache.get(key) ?? '' : '';
}

function setCachedSudoPassword(sessionId: string | null, password: string): void {
  const key = getSessionKey(sessionId);
  if (!key) {
    return;
  }
  dockerSudoPasswordCache.set(key, password);
  dockerSudoPromptedSessions.delete(key);
}

function clearCachedSudoPassword(sessionId: string | null): void {
  const key = getSessionKey(sessionId);
  if (key) {
    dockerSudoPasswordCache.delete(key);
  }
}

function markSudoPrompted(sessionId: string | null): void {
  const key = getSessionKey(sessionId);
  if (key) {
    dockerSudoPromptedSessions.add(key);
  }
}

function clearSudoPromptState(sessionId: string | null): void {
  const key = getSessionKey(sessionId);
  if (key) {
    dockerSudoPromptedSessions.delete(key);
  }
}

function hasPromptedForSudo(sessionId: string | null): boolean {
  const key = getSessionKey(sessionId);
  return key ? dockerSudoPromptedSessions.has(key) : false;
}

function parsePortsString(portsString: string | undefined | null): PortInfo[] {
  if (!portsString) {
    return [];
  }

  const ports: PortInfo[] = [];
  for (const entry of portsString.split(', ')) {
    const parts = entry.split('->');
    const publicPart = parts.length === 2 ? parts[0] : '';
    const privatePart = parts.length === 2 ? parts[1] : parts[0];
    const privateMatch = privatePart.match(/^(\d+)\/(tcp|udp|\w+)$/);
    if (!privateMatch) {
      continue;
    }

    const privatePort = Number.parseInt(privateMatch[1], 10);
    const type = privateMatch[2];
    const publicMatch = publicPart ? publicPart.match(/^(?:([\d.:a-fA-F]+):)?(\d+)$/) : null;

    ports.push({
      IP: publicMatch?.[1] || undefined,
      PublicPort: publicMatch ? Number.parseInt(publicMatch[2], 10) : undefined,
      PrivatePort: privatePort,
      Type: type,
    });
  }

  return ports;
}

function parseContainerLine(line: string): DockerContainer | null {
  try {
    const data = JSON.parse(line) as Record<string, unknown>;
    return {
      id: String(data.ID ?? ''),
      Names: typeof data.Names === 'string' ? String(data.Names).split(',') : [],
      Image: String(data.Image ?? ''),
      ImageID: String(data.ImageID ?? ''),
      Command: String(data.Command ?? ''),
      Created: (data.CreatedAt as string | number | undefined) ?? '',
      State: String(data.State ?? 'unknown'),
      Status: String(data.Status ?? ''),
      Ports: parsePortsString(data.Ports as string | undefined),
      Labels: typeof data.Labels === 'object' && data.Labels !== null ? data.Labels as Record<string, string> : {},
      stats: null,
    };
  } catch {
    return null;
  }
}

function parseStatsMap(stdout: string): Map<string, DockerStats> {
  const statsMap = new Map<string, DockerStats>();
  for (const line of stdout.split('\n').map((item) => item.trim()).filter(Boolean)) {
    try {
      const stats = JSON.parse(line) as DockerStats;
      if (stats.ID) {
        statsMap.set(stats.ID, stats);
      }
    } catch {
      // ignore malformed stats line
    }
  }
  return statsMap;
}

function isDockerUnavailable(result: SshExecResult): boolean {
  const stderr = result.stderr.trim();
  return stderr.includes('command not found')
    || stderr.includes('permission denied')
    || stderr.includes('Cannot connect to the Docker daemon')
    || SUDO_PASSWORD_REQUIRED_PATTERNS.some((pattern) => stderr.includes(pattern))
    || (!result.stdout.trim() && result.exit_code !== 0);
}

function isDockerPermissionIssue(result: SshExecResult): boolean {
  const stderr = result.stderr.trim();
  return DOCKER_PERMISSION_PATTERNS.some((pattern) => stderr.includes(pattern))
    || SUDO_PASSWORD_REQUIRED_PATTERNS.some((pattern) => stderr.includes(pattern));
}

function sanitizeContainerId(containerId: string): string {
  return containerId.replace(/[^a-zA-Z0-9_.-]/g, '');
}

async function execRemote(sessionId: string, command: string, timeoutMs: number): Promise<SshExecResult> {
  return sshApi.executeCommand(sessionId, command, timeoutMs);
}

export function useDockerManager(
  sessionId: ComputedRef<string | null>,
  sshConnectionStatus: ComputedRef<SshConnectionStatus>,
) {
  const settingsStore = useSettingsStore();
  const notifications = useUINotificationStore();

  const containers = ref<DockerContainer[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isDockerAvailable = ref(true);
  const expandedContainerIds = ref<Set<string>>(new Set());
  const initialLoadDone = ref(false);
  const requestInFlight = ref(false);
  const showSudoPasswordDialog = ref(false);
  const sudoPasswordError = ref('');
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  const refreshIntervalSeconds = computed(() =>
    settingsStore.getInteger('dockerStatusIntervalSeconds', DEFAULT_REFRESH_SECONDS, 1),
  );
  const dockerDefaultExpand = computed(() =>
    settingsStore.getBoolean('dockerDefaultExpand', false),
  );
  const dockerUseSudo = computed(() =>
    settingsStore.getBoolean('dockerUseSudo', false),
  );

  function clearCurrentSessionSudoPassword(): void {
    clearCachedSudoPassword(sessionId.value);
    sudoPasswordError.value = '';
  }

  function requestSudoPassword(message = '', force = true): void {
    if (!force && hasPromptedForSudo(sessionId.value)) {
      return;
    }
    if (force) {
      clearSudoPromptState(sessionId.value);
    }
    markSudoPrompted(sessionId.value);
    sudoPasswordError.value = message;
    showSudoPasswordDialog.value = true;
  }

  function closeSudoPasswordDialog(): void {
    showSudoPasswordDialog.value = false;
    sudoPasswordError.value = '';
  }

  async function execDockerCommand(command: string, timeoutMs: number): Promise<SshExecResult> {
    if (!sessionId.value) {
      throw new Error('SSH 连接未就绪');
    }

    if (!dockerUseSudo.value) {
      return execRemote(sessionId.value, command, timeoutMs);
    }

    const cachedPassword = getCachedSudoPassword(sessionId.value);
    if (cachedPassword) {
      return sshApi.executeCommandWithInput(
        sessionId.value,
        `sudo -S -p '' ${command}`,
        `${cachedPassword}\n`,
        timeoutMs,
        true,
      );
    }

    return execRemote(sessionId.value, `sudo -n ${command}`, timeoutMs);
  }

  async function validateAndStoreSudoPassword(password: string): Promise<boolean> {
    if (!sessionId.value) {
      sudoPasswordError.value = 'SSH 连接未就绪';
      return false;
    }

    sudoPasswordError.value = '';
    try {
      const result = await sshApi.executeCommandWithInput(
        sessionId.value,
        "sudo -S -p '' -v",
        `${password}\n`,
        COMMAND_TIMEOUT_MS,
        true,
      );
      if (result.exit_code !== 0) {
        throw new Error(result.stderr.trim() || 'sudo 密码验证失败');
      }
      setCachedSudoPassword(sessionId.value, password);
      showSudoPasswordDialog.value = false;
      sudoPasswordError.value = '';
      void requestDockerStatus(true);
      return true;
    } catch (err) {
      sudoPasswordError.value = err instanceof Error ? err.message : 'sudo 密码验证失败';
      return false;
    }
  }

  function resetState() {
    containers.value = [];
    isLoading.value = false;
    error.value = null;
    isDockerAvailable.value = true;
    expandedContainerIds.value = new Set();
    initialLoadDone.value = false;
  }

  function stopPolling() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  function cleanupExpansionState(nextContainers: DockerContainer[]) {
    const nextIds = new Set(nextContainers.map((container) => container.id));
    const nextExpanded = new Set<string>();
    expandedContainerIds.value.forEach((containerId) => {
      if (nextIds.has(containerId)) {
        nextExpanded.add(containerId);
      }
    });
    expandedContainerIds.value = nextExpanded;
  }

  function applyDefaultExpansion(nextContainers: DockerContainer[]) {
    if (initialLoadDone.value || !dockerDefaultExpand.value) {
      return;
    }
    expandedContainerIds.value = new Set(nextContainers.map((container) => container.id));
    initialLoadDone.value = true;
  }

  async function requestDockerStatus(force = false): Promise<void> {
    if (!sessionId.value || sshConnectionStatus.value !== 'connected') {
      stopPolling();
      resetState();
      error.value = sessionId.value ? 'SSH 连接未就绪' : null;
      isDockerAvailable.value = false;
      return;
    }

    if (requestInFlight.value && !force) {
      return;
    }

    requestInFlight.value = true;
    isLoading.value = true;
    error.value = null;

    try {
      const versionResult = await execDockerCommand(
        "docker version --format '{{.Server.Version}}'",
        STATUS_TIMEOUT_MS,
      );

      if (isDockerUnavailable(versionResult)) {
        isDockerAvailable.value = false;
        containers.value = [];
        expandedContainerIds.value = new Set();
        if (dockerUseSudo.value && isDockerPermissionIssue(versionResult)) {
          if (getCachedSudoPassword(sessionId.value)) {
            clearCurrentSessionSudoPassword();
            requestSudoPassword('请输入有效 sudo 密码以访问 Docker');
          } else {
            requestSudoPassword('请输入 sudo 密码以访问 Docker', false);
          }
          error.value = '需要 sudo 密码才能访问 Docker';
        }
        return;
      }

      isDockerAvailable.value = true;

      const psResult = await execDockerCommand(
        "docker ps -a --no-trunc --format '{{json .}}'",
        STATUS_TIMEOUT_MS,
      );

      if (isDockerUnavailable(psResult)) {
        isDockerAvailable.value = false;
        containers.value = [];
        expandedContainerIds.value = new Set();
        if (dockerUseSudo.value && isDockerPermissionIssue(psResult)) {
          if (getCachedSudoPassword(sessionId.value)) {
            clearCurrentSessionSudoPassword();
            requestSudoPassword('请输入有效 sudo 密码以访问 Docker');
          } else {
            requestSudoPassword('请输入 sudo 密码以访问 Docker', false);
          }
          error.value = '需要 sudo 密码才能访问 Docker';
        }
        return;
      }

      if (psResult.exit_code !== 0) {
        throw new Error(psResult.stderr.trim() || 'docker ps 执行失败');
      }

      const nextContainers = psResult.stdout
        .split('\n')
        .map((line: string) => line.trim())
        .filter(Boolean)
        .map(parseContainerLine)
        .filter((container): container is DockerContainer => container !== null);

      const runningIds = nextContainers
        .filter((container: DockerContainer) => container.State === 'running')
        .map((container: DockerContainer) => container.id);

      if (runningIds.length > 0) {
        const statsResult = await execDockerCommand(
          `docker stats ${runningIds.join(' ')} --no-stream --format '{{json .}}'`,
          STATUS_TIMEOUT_MS,
        );

        if (!isDockerUnavailable(statsResult) && statsResult.stdout.trim()) {
          const statsMap = parseStatsMap(statsResult.stdout);
          nextContainers.forEach((container: DockerContainer) => {
            const shortId = container.id.slice(0, 12);
            container.stats = statsMap.get(container.id) ?? statsMap.get(shortId) ?? null;
          });
        }
      }

      containers.value = nextContainers;
      cleanupExpansionState(nextContainers);
      applyDefaultExpansion(nextContainers);
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取 Docker 状态失败';
      isDockerAvailable.value = false;
      containers.value = [];
      expandedContainerIds.value = new Set();
    } finally {
      isLoading.value = false;
      requestInFlight.value = false;
    }
  }

  function startPolling() {
    stopPolling();
    if (!sessionId.value || sshConnectionStatus.value !== 'connected') {
      return;
    }

    void requestDockerStatus(true);
    refreshInterval = setInterval(() => {
      void requestDockerStatus();
    }, refreshIntervalSeconds.value * 1000);
  }

  async function sendDockerCommand(containerId: string, command: DockerCommand) {
    if (!sessionId.value || sshConnectionStatus.value !== 'connected') {
      notifications.addNotification('warning', 'SSH 连接未就绪，无法执行 Docker 命令');
      return;
    }

    if (!isDockerAvailable.value) {
      notifications.addNotification('warning', '远程 Docker 不可用');
      return;
    }

    const cleanId = sanitizeContainerId(containerId);
    if (!cleanId) {
      notifications.addNotification('error', '无效的容器 ID');
      return;
    }

    const commands: Record<DockerCommand, string> = {
      start: `docker start ${cleanId}`,
      stop: `docker stop ${cleanId}`,
      restart: `docker restart ${cleanId}`,
      remove: `docker rm -f ${cleanId}`,
    };

    try {
      const result = await execDockerCommand(commands[command], COMMAND_TIMEOUT_MS);
      if (result.exit_code !== 0) {
        if (dockerUseSudo.value && isDockerPermissionIssue(result)) {
          clearCurrentSessionSudoPassword();
          requestSudoPassword('请输入有效 sudo 密码后重试 Docker 操作');
        }
        throw new Error(result.stderr.trim() || `${command} 执行失败`);
      }
      void requestDockerStatus(true);
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : 'Docker 命令执行失败');
    }
  }

  function toggleExpand(containerId: string) {
    const next = new Set(expandedContainerIds.value);
    if (next.has(containerId)) {
      next.delete(containerId);
    } else {
      next.add(containerId);
    }
    expandedContainerIds.value = next;
  }

  watch([sessionId, sshConnectionStatus], () => {
    closeSudoPasswordDialog();
    if (!sessionId.value || sshConnectionStatus.value !== 'connected') {
      stopPolling();
      resetState();
      if (sessionId.value) {
        error.value = 'SSH 连接未就绪';
        isDockerAvailable.value = false;
      }
      return;
    }

    startPolling();
  }, { immediate: true });

  watch(refreshIntervalSeconds, () => {
    if (sessionId.value && sshConnectionStatus.value === 'connected') {
      startPolling();
    }
  });

  onMounted(() => {
    void settingsStore.loadAll().catch(() => undefined);
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    containers: readonly(containers),
    isLoading: readonly(isLoading),
    error: readonly(error),
    isDockerAvailable: readonly(isDockerAvailable),
    expandedContainerIds: readonly(expandedContainerIds),
    showSudoPasswordDialog: readonly(showSudoPasswordDialog),
    sudoPasswordError: readonly(sudoPasswordError),
    requestDockerStatus,
    sendDockerCommand,
    toggleExpand,
    validateAndStoreSudoPassword,
    requestSudoPassword,
    closeSudoPasswordDialog,
  };
}
