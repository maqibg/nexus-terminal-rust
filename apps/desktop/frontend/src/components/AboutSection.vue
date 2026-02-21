<template>
  <section class="about-section">
    <h3 class="section-title">关于</h3>
    <div class="info-row"><span class="label">应用名称</span><span>Nexus Terminal</span></div>
    <div class="info-row"><span class="label">版本</span><span>{{ version }}</span></div>
    <div class="info-row"><span class="label">框架</span><span>Tauri 2 + Vue 3</span></div>
    <div class="info-row"><span class="label">许可证</span><span>GPL-3.0</span></div>
    <div class="info-row"><span class="label">上游项目</span><a class="link" href="https://github.com/Heavrnl/nexus-terminal" target="_blank">nexus-terminal</a></div>
  </section>
</template>

<script setup lang="ts">
type AppVersionHost = {
  __APP_VERSION__?: unknown;
};

const version = (() => {
  const maybeVersion = (globalThis as AppVersionHost).__APP_VERSION__;
  if (typeof maybeVersion === 'string' && maybeVersion.trim()) {
    return maybeVersion;
  }
  return '0.1.0';
})();
</script>

<style scoped>
.about-section { display: flex; flex-direction: column; gap: 8px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.info-row { display: flex; gap: 12px; font-size: 13px; color: var(--text); }
.info-row .label { color: var(--text-sub); min-width: 80px; }
.info-row .link { color: var(--accent, #4a9eff); text-decoration: none; }
.info-row .link:hover { text-decoration: underline; }
</style>
