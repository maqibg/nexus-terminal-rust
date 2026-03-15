<template>
  <div class="page tools-page">
    <div class="page-header">
      <h2>工具箱</h2>
      <div class="page-subtitle">本地开发工具集合（JSON / 编码解码 / Hash / JWT / Regex / Diff / UUID / Cron / QR 等）</div>
    </div>

    <div class="tools-layout">
      <aside class="tools-sidebar">
        <input v-model="searchTerm" class="search-input" placeholder="搜索工具..." />
        <button
          v-for="tool in filteredTools"
          :key="tool.id"
          type="button"
          class="tool-item"
          :class="{ active: tool.id === activeToolId }"
          @click="activeToolId = tool.id"
        >
          <i :class="tool.icon" class="tool-icon" aria-hidden="true"></i>
          <div class="tool-meta">
            <div class="tool-name">{{ tool.name }}</div>
            <div class="tool-desc">{{ tool.description }}</div>
          </div>
        </button>
      </aside>

      <section class="tools-content">
        <div class="tool-header">
          <div>
            <div class="tool-title">{{ activeTool.name }}</div>
            <div class="tool-desc-main">{{ activeTool.description }}</div>
          </div>
          <button type="button" class="btn btn-ghost" @click="clearActiveTool">清空</button>
        </div>

        <component :is="activeTool.component" :reset-seq="resetSeq" />
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue';

import './tools/tools.css';

const JsonTool = defineAsyncComponent(() => import('./tools/JsonTool.vue'));
const Base64Tool = defineAsyncComponent(() => import('./tools/Base64Tool.vue'));
const UrlTool = defineAsyncComponent(() => import('./tools/UrlTool.vue'));
const HtmlEntitiesTool = defineAsyncComponent(() => import('./tools/HtmlEntitiesTool.vue'));
const UnicodeTool = defineAsyncComponent(() => import('./tools/UnicodeTool.vue'));
const HashTool = defineAsyncComponent(() => import('./tools/HashTool.vue'));
const JwtTool = defineAsyncComponent(() => import('./tools/JwtTool.vue'));
const RegexTool = defineAsyncComponent(() => import('./tools/RegexTool.vue'));
const DiffTool = defineAsyncComponent(() => import('./tools/DiffTool.vue'));
const UuidTool = defineAsyncComponent(() => import('./tools/UuidTool.vue'));
const CronTool = defineAsyncComponent(() => import('./tools/CronTool.vue'));
const QrCodeTool = defineAsyncComponent(() => import('./tools/QrCodeTool.vue'));
const TimestampTool = defineAsyncComponent(() => import('./tools/TimestampTool.vue'));
const PasswordTool = defineAsyncComponent(() => import('./tools/PasswordTool.vue'));
const ColorTool = defineAsyncComponent(() => import('./tools/ColorTool.vue'));
const CaseTool = defineAsyncComponent(() => import('./tools/CaseTool.vue'));
const LinesTool = defineAsyncComponent(() => import('./tools/LinesTool.vue'));

type ToolDescriptor = {
  id: string;
  name: string;
  description: string;
  icon: string;
  component: unknown;
};

const tools = [
  { id: 'json', name: 'JSON 格式化', description: '格式化/压缩 JSON', icon: 'fas fa-code', component: JsonTool },
  { id: 'base64', name: 'Base64', description: 'UTF-8 安全编码/解码', icon: 'fas fa-code', component: Base64Tool },
  { id: 'url', name: 'URL 编解码', description: 'encodeURIComponent / decode', icon: 'fas fa-link', component: UrlTool },
  { id: 'htmlEntities', name: 'HTML 实体', description: '转义/反转义', icon: 'fas fa-file-code', component: HtmlEntitiesTool },
  { id: 'unicode', name: 'Unicode 转义', description: String.raw`\\uXXXX / \\u{...}`, icon: 'fas fa-language', component: UnicodeTool },
  { id: 'hash', name: 'Hash', description: 'SHA-1/SHA-256/SHA-384/SHA-512', icon: 'fas fa-hashtag', component: HashTool },
  { id: 'jwt', name: 'JWT', description: '解析 Header/Payload（不验签）', icon: 'fas fa-key', component: JwtTool },
  { id: 'regex', name: 'Regex', description: '匹配与替换预览', icon: 'fas fa-asterisk', component: RegexTool },
  { id: 'diff', name: 'Diff', description: '统一 Diff / 行级对比', icon: 'fas fa-not-equal', component: DiffTool },
  { id: 'uuid', name: 'UUID', description: '批量生成 v4 UUID', icon: 'fas fa-fingerprint', component: UuidTool },
  { id: 'cron', name: 'Cron', description: '预览未来触发时间', icon: 'fas fa-clock', component: CronTool },
  { id: 'qrcode', name: 'QR 码', description: '生成二维码 PNG', icon: 'fas fa-qrcode', component: QrCodeTool },
  { id: 'timestamp', name: '时间戳', description: 'Unix <-> 日期', icon: 'fas fa-calendar-alt', component: TimestampTool },
  { id: 'password', name: '密码生成', description: '可选字符集 + CSPRNG', icon: 'fas fa-shield-alt', component: PasswordTool },
  { id: 'color', name: '颜色转换', description: 'HEX <-> RGB', icon: 'fas fa-palette', component: ColorTool },
  { id: 'case', name: '命名转换', description: 'camel/snake/kebab 等', icon: 'fas fa-font', component: CaseTool },
  { id: 'lines', name: '行处理', description: '去重/排序/Trim', icon: 'fas fa-list', component: LinesTool },
] as const satisfies readonly ToolDescriptor[];

type ToolId = (typeof tools)[number]['id'];

const searchTerm = ref('');
const activeToolId = ref<ToolId>('json');
const resetSeq = ref(0);

const filteredTools = computed(() => {
  const term = searchTerm.value.trim().toLowerCase();
  if (!term) {
    return tools;
  }
  return tools.filter(tool => tool.name.toLowerCase().includes(term) || tool.description.toLowerCase().includes(term));
});

const activeTool = computed(() => tools.find(t => t.id === activeToolId.value) ?? tools[0]);

function clearActiveTool() {
  resetSeq.value += 1;
}
</script>
