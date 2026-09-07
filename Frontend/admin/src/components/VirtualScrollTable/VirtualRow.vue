<template>
  <div class="virtual-row-container">
    <div
      v-for="row in $props.visibleRows"
      :key="rowIndex++"
      class="virtual-row"
      :style="{ height: rowHeight + 'px' }"
      @click="$emit('rowClick', row)"
    >
      <slot name="render-row" :row="row"  :index="rowIndex" />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  visibleRows: unknown[];
  rowIndex: number;
  rowHeight: number;
}

const props = withDefaults(defineProps<Props>(), {
  visibleRows: () => [],
  rowIndex: 0,
  rowHeight: 48,
});

const _emit = defineEmits<{
  (e: 'rowClick', row: unknown): void
}>()
</script>

<style scoped>
.virtual-row-container {
  position: relative;
  height: 100%;
  overflow-y: auto;
}

.virtual-row {
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  cursor: pointer;
  transition: background-color 0.2s;
}

.virtual-row:hover {
  background: #f5f5f5;
}
</style>
