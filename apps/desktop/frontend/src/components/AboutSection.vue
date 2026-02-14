<template>
  <section class="about-section">
    <h3 class="section-title">关于</h3>
    <div class="info-row"><span class="label">应用名称</span><span>Nexus Terminal</span></div>
    <div class="info-row"><span class="label">版本</span><span>{{ version }}</span></div>
    <div class="info-row"><span class="label">框架</span><span>Tauri 2 + Vue 3</span></div>
    <div class="info-row"><span class="label">许可证</span><span>MIT</span></div>
  </section>
</template>

<script setup lang="ts">
const version = __APP_VERSION__ ?? '0.1.0';
declare const __APP_VERSION__: string | undefined;
</script>

<style scoped>
.about-section { display: flex; flex-direction: column; gap: 8px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.info-row { display: flex; gap: 12px; font-size: 13px; color: var(--text); }
.info-row .label { color: var(--text-sub); min-width: 80px; }
</style>
