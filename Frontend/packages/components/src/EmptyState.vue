<template>
  <div class="empty-state" :class="{ 'empty-state--centered': centered }">
    <!-- 图标区域 -->
    <div class="empty-state__icon" :style="{ color: iconColor || 'currentColor' }">
      <q-icon :name="icon || 'inbox'" :size="iconSize || '64px'" />
    </div>

    <!-- 标题 -->
    <div class="empty-state__title">{{ title || $t('empty.noData') }}</div>

    <!-- 描述 -->
    <div v-if="description" class="empty-state__description">
      {{ description }}
    </div>

    <!-- 默认插槽 - 自定义内容 -->
    <slot>
      <!-- 默认操作按钮 -->
      <div v-if="showAction" class="empty-state__actions">
        <slot name="action">
          <q-btn
            v-if="actionText"
            color="primary"
            :label="actionText"
            @click="handleAction"
          />
          <q-btn
            v-if="secondaryActionText"
            flat
            color="primary"
            :label="secondaryActionText"
            @click="handleSecondaryAction"
          />
        </slot>
      </div>
    </slot>

    <!-- 额外信息 -->
    <div v-if="extra" class="empty-state__extra">
      <slot name="extra" />
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * @file EmptyState.vue
 * @description 空状态组件 - 无数据时的友好提示
 * @date 2026-05-06
 */

import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

interface Props {
  /** 空状态图标 */
  icon?: string
  /** 图标大小 */
  iconSize?: string
  /** 图标颜色 */
  iconColor?: string
  /** 标题文本 */
  title?: string
  /** 描述文本 */
  description?: string
  /** 是否居中显示 */
  centered?: boolean
  /** 是否显示操作按钮 */
  showAction?: boolean
  /** 主操作按钮文本 */
  actionText?: string
  /** 次要操作按钮文本 */
  secondaryActionText?: string
  /** 额外内容 */
  extra?: boolean
}

withDefaults(defineProps<Props>(), {
  icon: 'inbox',
  iconSize: '64px',
  iconColor: '',
  title: '',
  description: '',
  centered: true,
  showAction: false,
  actionText: '',
  secondaryActionText: '',
  extra: false
})

// Emits
const emit = defineEmits<{
  (e: 'action'): void
  (e: 'secondaryAction'): void
}>()

/**
 * @brief 处理主操作点击
 */
function handleAction(): void {
  emit('action')
}

/**
 * @brief 处理次要操作点击
 */
function handleSecondaryAction(): void {
  emit('secondaryAction')
}
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 40px 20px;
  text-align: left;
}

.empty-state--centered {
  align-items: center;
  text-align: center;
}

/* 图标 */
.empty-state__icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state__icon .q-icon {
  font-size: inherit;
}

/* 标题 */
.empty-state__title {
  font-size: 16px;
  font-weight: 500;
  color: #303133;
  margin-bottom: 8px;
}

/* 描述 */
.empty-state__description {
  font-size: 14px;
  color: #909399;
  margin-bottom: 16px;
  max-width: 400px;
  line-height: 1.6;
}

/* 操作按钮 */
.empty-state__actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

/* 额外信息 */
.empty-state__extra {
  margin-top: 16px;
  color: #c0c4cc;
  font-size: 12px;
}

/* 暗色主题 */
.body--dark .empty-state__title {
  color: #b0b0b0;
}

.body--dark .empty-state__description {
  color: #757575;
}

.body--dark .empty-state__icon {
  opacity: 0.4;
}

.body--dark .empty-state__extra {
  color: #606266;
}

/* 响应式 */
@media (max-width: 768px) {
  .empty-state {
    padding: 32px 16px;
  }

  .empty-state__title {
    font-size: 15px;
  }

  .empty-state__description {
    font-size: 13px;
  }
}

/* 动画 */
.empty-state__icon {
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-8px);
  }
}

/* 不同类型图标样式 */
.empty-state--type-inbox .empty-state__icon {
  color: #909399;
}

.empty-state--type-search .empty-state__icon {
  color: #c0c4cc;
}

.empty-state--type-error .empty-state__icon {
  color: #f56c6c;
}

.empty-state--type-success .empty-state__icon {
  color: #67c23a;
}
</style>