<template>
  <div class="skeleton-wrapper">
    <!-- 表格骨架屏 -->
    <div v-if="type === 'table'" class="skeleton-table">
      <!-- 表头 -->
      <div class="skeleton-header">
        <div
          v-for="i in columns"
          :key="i"
          class="skeleton-cell"
          :style="{ width: `${100 / columns}%` }"
        >
          <div class="skeleton-line short" />
        </div>
      </div>
      <!-- 表格行 -->
      <div v-for="row in rows" :key="row" class="skeleton-row">
        <div
          v-for="col in columns"
          :key="col"
          class="skeleton-cell"
          :style="{ width: `${100 / columns}%` }"
        >
          <div class="skeleton-line" :class="{ short: Math.random() > 0.6 }" />
        </div>
      </div>
    </div>

    <!-- 卡片骨架屏 -->
    <div v-else-if="type === 'card'" class="skeleton-cards">
      <div v-for="i in rows" :key="i" class="skeleton-card">
        <div class="skeleton-image" />
        <div class="skeleton-content">
          <div class="skeleton-line" style="width: 70%" />
          <div class="skeleton-line short" style="width: 40%" />
        </div>
      </div>
    </div>

    <!-- 表单骨架屏 -->
    <div v-else-if="type === 'form'" class="skeleton-form">
      <div v-for="i in rows" :key="i" class="skeleton-form-item">
        <div class="skeleton-label" />
        <div class="skeleton-input" />
      </div>
    </div>

    <!-- 列表骨架屏 -->
    <div v-else-if="type === 'list'" class="skeleton-list">
      <div v-for="i in rows" :key="i" class="skeleton-list-item">
        <div class="skeleton-avatar" />
        <div class="skeleton-list-content">
          <div class="skeleton-line" style="width: 50%" />
          <div class="skeleton-line short" style="width: 30%" />
        </div>
      </div>
    </div>

    <!-- 详情骨架屏 -->
    <div v-else-if="type === 'detail'" class="skeleton-detail">
      <div class="skeleton-title" />
      <div class="skeleton-line" />
      <div class="skeleton-line" />
      <div class="skeleton-line short" />
      <div class="skeleton-divider" />
      <div class="skeleton-line" />
      <div class="skeleton-line short" />
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * @brief 骨架屏组件
 * 支持多种类型：表格、卡片、表单、列表、详情
 */
defineProps({
  /** 骨架屏类型 */
  type: {
    type: String as () => 'table' | 'card' | 'form' | 'list' | 'detail',
    default: 'table',
  },
  /** 行数 */
  rows: {
    type: Number,
    default: 5,
  },
  /** 列数（仅 table 类型） */
  columns: {
    type: Number,
    default: 4,
  },
});
</script>

<style scoped>
.skeleton-wrapper {
  width: 100%;
}

/* 骨架屏动画 */
.skeleton-line,
.skeleton-image,
.skeleton-avatar,
.skeleton-label,
.skeleton-input,
.skeleton-title,
.skeleton-divider {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: skeleton-loading 1.5s ease-in-out infinite;
  border-radius: 4px;
}

@keyframes skeleton-loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* 表格骨架屏 */
.skeleton-table {
  border: 1px solid #eee;
  border-radius: 8px;
  overflow: hidden;
}

.skeleton-header {
  display: flex;
  background: #fafafa;
  border-bottom: 1px solid #eee;
}

.skeleton-row {
  display: flex;
  border-bottom: 1px solid #f5f5f5;
}

.skeleton-row:last-child {
  border-bottom: none;
}

.skeleton-cell {
  padding: 16px;
  display: flex;
  align-items: center;
}

.skeleton-line {
  height: 14px;
  width: 100%;
}

.skeleton-line.short {
  width: 60%;
}

/* 卡片骨架屏 */
.skeleton-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.skeleton-card {
  border: 1px solid #eee;
  border-radius: 8px;
  overflow: hidden;
}

.skeleton-image {
  height: 160px;
  width: 100%;
  border-radius: 0;
}

.skeleton-content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 表单骨架屏 */
.skeleton-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.skeleton-form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-label {
  height: 12px;
  width: 80px;
}

.skeleton-input {
  height: 40px;
  width: 100%;
  border-radius: 4px;
}

/* 列表骨架屏 */
.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton-list-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px;
  border: 1px solid #eee;
  border-radius: 8px;
}

.skeleton-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  flex-shrink: 0;
}

.skeleton-list-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 详情骨架屏 */
.skeleton-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 24px;
  border: 1px solid #eee;
  border-radius: 8px;
}

.skeleton-title {
  height: 28px;
  width: 40%;
  margin-bottom: 8px;
}

.skeleton-divider {
  height: 1px;
  width: 100%;
  margin: 8px 0;
}
</style>
