<template>
  <div ref="containerRef" class="virtual-list-container" @scroll="handleScroll">
    <!-- 占位元素，用于计算总高度 -->
    <div :style="{ height: totalHeight + 'px', position: 'relative' }">
      <!-- 可见区域的列表项 -->
      <div
        v-for="item in visibleItems"
        :key="item.index"
        class="virtual-list-item"
        :style="getItemStyle(item)"
      >
        <slot :item="item.data" :index="item.index" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * @file VirtualList.vue
 * @description 虚拟列表组件 - 用于优化大数据列表渲染性能
 * @date 2026-04-04
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

interface Props<T = unknown> {
  /** 列表数据 */
  items: T[];
  /** 列表项高度（固定高度模式） */
  itemHeight?: number;
  /** 缓冲区大小 */
  buffer?: number;
  /** 容器高度 */
  height?: string;
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
  itemHeight: 50,
  buffer: 5,
  height: '400px',
});

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerHeight = ref(400);

// 计算总高度
const totalHeight = computed(() => props.items.length * props.itemHeight);

// 计算可见项数量
const visibleCount = computed(() => Math.ceil(containerHeight.value / props.itemHeight));

// 计算可见项的起始和结束索引
const startIndex = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight) - props.buffer;
  return Math.max(0, start);
});

const endIndex = computed(() => {
  const end = startIndex.value + visibleCount.value + props.buffer * 2;
  return Math.min(props.items.length, end);
});

// 获取可见项
const visibleItems = computed(() => {
  const result: { index: number; data: unknown }[] = [];
  for (let i = startIndex.value; i < endIndex.value; i++) {
    result.push({ index: i, data: props.items[i] });
  }
  return result;
});

// 获取列表项样式
function getItemStyle(item: { index: number }) {
  return {
    position: 'absolute' as const,
    top: item.index * props.itemHeight + 'px',
    left: 0,
    right: 0,
    height: props.itemHeight + 'px',
  };
}

// 处理滚动
function handleScroll(e: Event) {
  const target = e.target as HTMLElement;
  scrollTop.value = target.scrollTop;
}

// 更新容器高度
function updateContainerHeight() {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight;
  }
}

// 监听窗口大小变化
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateContainerHeight();

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      updateContainerHeight();
    });
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

// 监听 items 变化，重置滚动位置
watch(() => props.items, () => {
  scrollTop.value = 0;
});
</script>

<style scoped>
.virtual-list-container {
  overflow-y: auto;
  position: relative;
}

.virtual-list-item {
  box-sizing: border-box;
}
</style>
