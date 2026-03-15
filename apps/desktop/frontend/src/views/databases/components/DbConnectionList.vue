<template>
  <div class="db-conn-list">
    <div class="section-title row">
      <span>{{ title }}</span>
      <div class="db-conn-actions">
        <button type="button" class="btn btn-primary btn-sm" @click="emit('create')">{{ createLabel }}</button>
        <button type="button" class="btn btn-ghost btn-sm" @click="emit('refresh')">{{ refreshLabel }}</button>
      </div>
    </div>

    <div class="db-conn-search">
      <input v-model="search" class="field-input" type="text" :placeholder="searchPlaceholder" />
    </div>

    <div v-if="filtered.length === 0" class="empty">{{ emptyText }}</div>
    <div v-else class="conn-list">
      <div v-for="item in filtered" :key="item.id" class="conn-item-row">
        <button
          type="button"
          class="conn-item conn-main"
          :class="{ active: item.id === activeId }"
          :title="item.subtitle || item.name"
          @click="emit('select', item.id)"
        >
          <div class="conn-name">{{ item.name }}</div>
          <div class="conn-path">{{ item.subtitle }}</div>
        </button>
        <button
          v-if="showEdit"
          type="button"
          class="btn btn-ghost btn-sm conn-action"
          title="编辑连接"
          @click.stop="emit('edit', item.id)"
        >
          编辑
        </button>
        <button
          v-if="showDelete"
          type="button"
          class="btn btn-ghost btn-sm conn-action"
          title="删除连接"
          @click.stop="emit('delete', item.id)"
        >
          删除
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';

export type DbConnListItem = {
  id: string;
  name: string;
  subtitle: string;
};

const props = withDefaults(
  defineProps<{
    title: string;
    items: DbConnListItem[];
    activeId: string | null;
    emptyText?: string;
    searchPlaceholder?: string;
    createLabel?: string;
    refreshLabel?: string;
    showEdit?: boolean;
    showDelete?: boolean;
  }>(),
  {
    emptyText: '暂无连接',
    searchPlaceholder: '搜索连接...',
    createLabel: '新建',
    refreshLabel: '刷新',
    showEdit: true,
    showDelete: true,
  },
);

const emit = defineEmits<{
  select: [id: string];
  create: [];
  refresh: [];
  edit: [id: string];
  delete: [id: string];
}>();

const search = ref('');

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) {
    return props.items;
  }
  return props.items.filter((item) => {
    const name = item.name.toLowerCase();
    const subtitle = item.subtitle.toLowerCase();
    return name.includes(q) || subtitle.includes(q);
  });
});
</script>
