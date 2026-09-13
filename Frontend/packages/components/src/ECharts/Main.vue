<template>
  <div class="e-charts">
    <div class="e-charts__header">
      <div class="text-h6">{{ title }}</div>
      <div class="row q-gutter-sm">
        <q-btn
          v-for="option in chartOptions"
          :key="option.value"
          :label="option.label"
          :color="selectedOption === option.value ? 'primary' : 'grey-5'"
          flat
          dense
          @click="selectedOption = option.value"
        />
      </div>
    </div>

    <div ref="chartRef" class="e-charts__container" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useECharts } from './useECharts'
import { chartOptions } from './chartOptions'

interface Props {
  title?: string
  options?: Record<string, unknown>
  height?: string | number
}

const props = withDefaults(defineProps<Props>(), {
  title: '图表',
  options: () => ({}),
  height: '400px'
})

const emit = defineEmits<{
  (e: 'render', chart: unknown): void
}>()

const chartRef = ref<HTMLDivElement | null>(null)
const selectedOption = ref('default')

const { initChart, destroyChart, setOptions } = useECharts({
  containerRef: chartRef,
  height: props.height,
  onRender: emit
})

onMounted(() => {
  initChart()
  setOptions(props.options)
})

onBeforeUnmount(() => {
  destroyChart()
})
</script>

<style scoped>
.e-charts {
  padding: 16px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.e-charts__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.e-charts__container {
  width: 100%;
  height: v-bind('props.height');
  min-height: v-bind('props.height');
}

.body--dark .e-charts {
  background: #1e1e1e;
  box-shadow: 0 2px 4px rgba(255, 255, 255, 0.1);
}
</style>
