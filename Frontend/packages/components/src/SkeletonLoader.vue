<template>
  <div class="skeleton-wrapper" :class="{ 'skeleton-wrapper--animate': animate }">
    <!-- 表格骨架屏 -->
    <div v-if="type === 'table'" class="skeleton-table">
      <!-- 表头 -->
      <div v-if="showHeader" class="skeleton-table__header">
        <div
          v-for="col in columns"
          :key="col"
          class="skeleton-cell"
          :style="{ width: colWidth || 'auto', flex: colWidth ? 'none' : '1' }"
        >
          <div class="skeleton-line skeleton-line--header"></div>
        </div>
      </div>

      <!-- 表格行 -->
      <div
        v-for="row in rows"
        :key="row"
        class="skeleton-table__row"
        :style="{ animationDelay: `${row * 0.1}s` }"
      >
        <div
          v-for="(col, colIndex) in columns"
          :key="colIndex"
          class="skeleton-cell"
        >
          <!-- 复选框列 -->
          <div v-if="showCheckbox && colIndex === 0" class="skeleton-checkbox"></div>

          <!-- 内容 -->
          <div
            class="skeleton-line"
            :class="{ 'skeleton-line--circle': colIndex === 0 && showAvatar }"
            :style="{ width: getWidth(colIndex) }"
          ></div>
        </div>
      </div>
    </div>

    <!-- 卡片骨架屏 -->
    <div v-else-if="type === 'card'" class="skeleton-card">
      <div
        v-for="i in rows"
        :key="i"
        class="skeleton-card__item"
        :style="{ animationDelay: `${i * 0.15}s` }"
      >
        <div class="skeleton-card__image skeleton-line"></div>
        <div class="skeleton-card__content">
          <div class="skeleton-line skeleton-line--title"></div>
          <div class="skeleton-line skeleton-line--text"></div>
          <div class="skeleton-line skeleton-line--text skeleton-line--short"></div>
        </div>
      </div>
    </div>

    <!-- 表单骨架屏 -->
    <div v-else-if="type === 'form'" class="skeleton-form">
      <div
        v-for="i in rows"
        :key="i"
        class="skeleton-form__item"
        :style="{ animationDelay: `${i * 0.1}s` }"
      >
        <div v-if="showLabel" class="skeleton-line skeleton-line--label"></div>
        <div class="skeleton-line skeleton-line--input"></div>
      </div>
    </div>

    <!-- 列表骨架屏 -->
    <div v-else-if="type === 'list'" class="skeleton-list">
      <div
        v-for="i in rows"
        :key="i"
        class="skeleton-list__item"
        :style="{ animationDelay: `${i * 0.1}s` }"
      >
        <div class="skeleton-list__avatar skeleton-line skeleton-line--circle"></div>
        <div class="skeleton-list__content">
          <div class="skeleton-line skeleton-line--title"></div>
          <div class="skeleton-line skeleton-line--subtitle"></div>
        </div>
      </div>
    </div>

    <!-- 自定义内容插槽 -->
    <slot v-else />
  </div>
</template>

<script setup lang="ts">
/**
 * @file SkeletonLoader.vue
 * @description 骨架屏加载组件（已合并 SkeletonTable 部分功能）
 * @date 2026-04-04
 */

interface Props {
  /** 骨架屏类型: table | card | form | list | custom */
  type?: 'table' | 'card' | 'form' | 'list' | 'custom'
  /** 行数 */
  rows?: number
  /** 列数 (表格) */
  columns?: number
  /** 是否显示表头 */
  showHeader?: boolean
  /** 是否显示复选框 */
  showCheckbox?: boolean
  /** 是否显示头像 */
  showAvatar?: boolean
  /** 是否显示标签 */
  showLabel?: boolean
  /** 是否显示动画 */
  animate?: boolean
  /** 固定列宽 */
  colWidth?: string
  /** 自定义宽度数组 (用于生成随机宽度) */
  customWidths?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  type: 'table',
  rows: 5,
  columns: 4,
  showHeader: true,
  showCheckbox: false,
  showAvatar: true,
  showLabel: true,
  animate: true,
  customWidths: () => []
})

/**
 * @brief 获取列宽度
 */
function getWidth(colIndex: number): string {
  if (props.customWidths[colIndex]) {
    return props.customWidths[colIndex]
  }

  // 根据列索引生成合理的宽度
  const widths: string[] = ['40px', '80px', '60%', '30%', '50px', '100px', '70%']
  return widths[colIndex % widths.length] ?? '100%'
}
</script>

<style scoped>
.skeleton-wrapper {
  width: 100%;
}

.skeleton-wrapper--animate {
  --skeleton-duration: 1.5s;
}

/* 通用骨架线 */
.skeleton-line {
  height: 14px;
  background: linear-gradient(
    90deg,
    #f0f0f0 25%,
    #e0e0e0 50%,
    #f0f0f0 75%
  );
  background-size: 200% 100%;
  border-radius: 4px;
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}

@keyframes skeleton-shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* 表格骨架屏 */
.skeleton-table {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.skeleton-table__header {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  background: #fafafa;
  border-bottom: 1px solid #f0f0f0;
}

.skeleton-table__header .skeleton-line--header {
  height: 12px;
  width: 80%;
}

.skeleton-table__row {
  display: flex;
  gap: 8px;
  padding: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.skeleton-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skeleton-checkbox {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  background: #f0f0f0;
}

/* 卡片骨架屏 */
.skeleton-card {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.skeleton-card__item {
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  overflow: hidden;
}

.skeleton-card__image {
  height: 160px;
  background: #f0f0f0;
}

.skeleton-card__content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-line--title {
  height: 18px;
  width: 70%;
}

.skeleton-line--text {
  height: 14px;
}

.skeleton-line--short {
  width: 50%;
}

/* 表单骨架屏 */
.skeleton-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton-form__item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-line--label {
  width: 80px;
  height: 12px;
}

.skeleton-line--input {
  height: 40px;
  border-radius: 8px;
}

/* 列表骨架屏 */
.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-list__item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
}

.skeleton-list__avatar {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
}

.skeleton-list__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-line--subtitle {
  height: 12px;
  width: 40%;
}

/* 圆形骨架线 */
.skeleton-line--circle {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* 暗色主题 */
.body--dark .skeleton-line {
  background: linear-gradient(
    90deg,
    #2d2d2d 25%,
    #3d3d3d 50%,
    #2d2d2d 75%
  );
  background-size: 200% 100%;
}

.body--dark .skeleton-table__header {
  background: #252525;
  border-color: #3d3d3d;
}

.body--dark .skeleton-table__row {
  border-color: #3d3d3d;
}

.body--dark .skeleton-card__item {
  border-color: #3d3d3d;
}

.body--dark .skeleton-card__image {
  background: #2d2d2d;
}

.body--dark .skeleton-form__item {
  border-color: #3d3d3d;
}

.body--dark .skeleton-list__item {
  border-color: #3d3d3d;
}

.body--dark .skeleton-checkbox {
  background: #2d2d2d;
}
</style>