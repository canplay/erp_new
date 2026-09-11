/**
 * @file BatchActions.vue
 * @description 表格批量操作组件
 * @date 2026-04-03
 */

<template>
  <div v-if="selected.length > 0" class="batch-actions">
    <div class="batch-info">
      <q-icon name="check_circle" color="primary" size="20px" />
      <span class="q-ml-sm">{{ $t('batchActions.selected', { count: selected.length }) }}</span>
    </div>

    <div class="batch-buttons">
      <!-- 批量启用 -->
      <q-btn
        v-if="showEnable"
        flat
        dense
        color="positive"
        :label="$t('batchActions.enable')"
        icon="check_circle"
        @click="handleBatchEnable"
      />

      <!-- 批量禁用 -->
      <q-btn
        v-if="showDisable"
        flat
        dense
        color="warning"
        :label="$t('batchActions.disable')"
        icon="block"
        @click="handleBatchDisable"
      />

      <!-- 批量删除 -->
      <q-btn
        v-if="showDelete"
        flat
        dense
        color="negative"
        :label="$t('batchActions.delete')"
        icon="delete"
        @click="handleBatchDelete"
      />

      <!-- 导出 -->
      <q-btn
        v-if="showExport"
        flat
        dense
        color="info"
        :label="$t('batchActions.export')"
        icon="download"
        :loading="exporting"
        @click="handleExport"
      />

      <!-- 自定义操作插槽 -->
      <slot name="actions" :selected="selected" />
    </div>

    <!-- 清除选择 -->
    <q-btn
      flat
      dense
      color="grey"
      :label="$t('batchActions.clearSelection')"
      icon="close"
      class="q-ml-md"
      @click="$emit('update:selected', [])"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * @file BatchActions.vue
 * @description 表格批量操作组件
 * @date 2026-04-03
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const $q = useQuasar();

// ============ Props ============

const props = withDefaults(
  defineProps<{
    /** 已选择的项 */
    selected: unknown[];
    /** 是否显示启用按钮 */
    showEnable?: boolean;
    /** 是否显示禁用按钮 */
    showDisable?: boolean;
    /** 是否显示删除按钮 */
    showDelete?: boolean;
    /** 是否显示导出按钮 */
    showExport?: boolean;
    /** 删除确认消息 */
    deleteConfirmMessage?: string;
  }>(),
  {
    selected: () => [],
    showEnable: false,
    showDisable: false,
    showDelete: true,
    showExport: true,
    deleteConfirmMessage: '确定要删除选中的 {count} 项吗？此操作不可恢复。',
  }
);

// ============ Emits ============

const emit = defineEmits<{
  'update:selected': [value: unknown[]];
  'batch-enable': [items: unknown[]];
  'batch-disable': [items: unknown[]];
  'batch-delete': [items: unknown[]];
  'export': [items: unknown[]];
}>();

// ============ 状态 ============

const exporting = ref(false);

// ============ 方法 ============

/**
 * @brief 处理批量启用
 */
function handleBatchEnable() {
  $q.dialog({
    title: t('batchActions.enable'),
    message: t('batchActions.enableConfirm', { count: props.selected.length }),
    ok: {
      label: t('common.confirm'),
      color: 'positive',
    },
    cancel: {
      label: t('common.cancel'),
      flat: true,
    },
  }).onOk(() => {
    emit('batch-enable', props.selected);
  });
}

/**
 * @brief 处理批量禁用
 */
function handleBatchDisable() {
  $q.dialog({
    title: t('batchActions.disable'),
    message: t('batchActions.disableConfirm', { count: props.selected.length }),
    ok: {
      label: t('common.confirm'),
      color: 'warning',
    },
    cancel: {
      label: t('common.cancel'),
      flat: true,
    },
  }).onOk(() => {
    emit('batch-disable', props.selected);
  });
}

/**
 * @brief 处理批量删除
 */
function handleBatchDelete() {
  const message = props.deleteConfirmMessage.replace('{count}', String(props.selected.length));

  $q.dialog({
    title: t('batchActions.delete'),
    message,
    ok: {
      label: t('common.confirm'),
      color: 'negative',
    },
    cancel: {
      label: t('common.cancel'),
      flat: true,
    },
  }).onOk(() => {
    emit('batch-delete', props.selected);
  });
}

/**
 * @brief 处理导出
 */
function handleExport() {
  exporting.value = true;

  try {
    emit('export', props.selected);
  } finally {
    setTimeout(() => {
      exporting.value = false;
    }, 1000);
  }
}
</script>

<style scoped>
.batch-actions {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  background: rgba(25, 118, 210, 0.08);
  border-radius: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 12px;
}

.batch-info {
  display: flex;
  align-items: center;
  font-size: 14px;
  color: #333;
  font-weight: 500;
}

.batch-buttons {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 16px;
}

/* 暗色主题适配 */
.body--dark .batch-actions {
  background: rgba(25, 118, 210, 0.15);
}

.body--dark .batch-info {
  color: #ffffff;
}
</style>
