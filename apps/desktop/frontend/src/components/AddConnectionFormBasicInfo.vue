<script setup lang="ts">
import { Teleport, nextTick, ref } from 'vue';

defineProps<{
  formData: {
    name: string;
    type: 'SSH' | 'RDP' | 'VNC';
    host: string;
    port: number;
  };
}>();

const showHostTooltip = ref(false);
const hostTooltipStyle = ref<Record<string, string>>({});
const hostIconRef = ref<HTMLElement | null>(null);
const hostTooltipContentRef = ref<HTMLElement | null>(null);

const handleHostIconMouseEnter = async () => {
  showHostTooltip.value = true;
  await nextTick();

  if (!hostIconRef.value || !hostTooltipContentRef.value) {
    return;
  }

  const iconRect = hostIconRef.value.getBoundingClientRect();
  const tooltipRect = hostTooltipContentRef.value.getBoundingClientRect();

  let top = iconRect.top - tooltipRect.height - 8;
  let left = iconRect.left + (iconRect.width / 2) - (tooltipRect.width / 2);

  if (top < 0) {
    top = iconRect.bottom + 8;
  }
  if (left < 0) {
    left = 0;
  }
  if (left + tooltipRect.width > window.innerWidth) {
    left = window.innerWidth - tooltipRect.width;
  }

  hostTooltipStyle.value = {
    position: 'fixed',
    top: `${top}px`,
    left: `${left}px`,
  };
};

const handleHostIconMouseLeave = () => {
  showHostTooltip.value = false;
};
</script>

<template>
  <Teleport to="body">
    <div
      v-if="showHostTooltip"
      ref="hostTooltipContentRef"
      class="host-tooltip"
      :style="hostTooltipStyle"
      role="tooltip"
    >
      支持 IP 范围，例如 `192.168.1.10~192.168.1.15`（仅添加模式生效）
    </div>
  </Teleport>

  <div class="section-card">
    <h4 class="section-title">基本信息</h4>

    <div class="field-block">
      <label for="conn-name" class="field-label">名称（可选）</label>
      <input id="conn-name" v-model="formData.name" type="text" class="field-input" />
    </div>

    <div class="field-block">
      <label class="field-label">连接类型</label>
      <div class="segment-group">
        <button
          type="button"
          class="segment-btn"
          :class="{ active: formData.type === 'SSH' }"
          @click="formData.type = 'SSH'"
        >
          SSH
        </button>
        <button
          type="button"
          class="segment-btn"
          :class="{ active: formData.type === 'RDP' }"
          @click="formData.type = 'RDP'"
        >
          RDP
        </button>
        <button
          type="button"
          class="segment-btn"
          :class="{ active: formData.type === 'VNC' }"
          @click="formData.type = 'VNC'"
        >
          VNC
        </button>
      </div>
    </div>

    <div class="field-grid">
      <div class="field-block host-field">
        <label for="conn-host" class="field-label host-label">
          <span>主机 / IP</span>
          <span class="info-icon-wrap" @mouseenter="handleHostIconMouseEnter" @mouseleave="handleHostIconMouseLeave">
            <i ref="hostIconRef" class="fas fa-info-circle info-icon"></i>
          </span>
        </label>
        <input id="conn-host" v-model="formData.host" type="text" class="field-input" required />
      </div>

      <div class="field-block">
        <label for="conn-port" class="field-label">端口</label>
        <input id="conn-port" v-model.number="formData.port" type="number" min="1" max="65535" class="field-input" required />
      </div>
    </div>
  </div>
</template>

<style scoped>
.section-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 45%, transparent);
}

.section-title {
  margin: 0;
  padding-bottom: 8px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  font-size: calc(16px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text);
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
  color: var(--text-sub);
}

.field-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
}

.field-input:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.segment-group {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.segment-btn {
  height: 34px;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
}

.segment-btn:last-child {
  border-right: none;
}

.segment-btn.active {
  background: var(--blue);
  color: var(--button-text-color);
}

.field-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 12px;
}

.host-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.info-icon {
  color: var(--text-sub);
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: help;
}

.host-tooltip {
  max-width: 260px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--ui-menu-bg);
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
  line-height: 1.5;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  z-index: 9999;
}

@media (max-width: 820px) {
  .field-grid {
    grid-template-columns: 1fr;
  }
}
</style>
