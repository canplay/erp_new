/**
 * @file StatsCharts/ChartCard.vue
 * @description 图表卡片容器子组件
 */

<template>
  <q-card flat bordered :class="cardClass">
    <q-card-section>
      <div class="text-subtitle1 q-mb-md">
        {{ title }}
        <q-badge
          v-if="badgeCount !== undefined && badgeCount > 0"
          :color="badgeColor"
          :label="badgeCount"
          class="q-ml-sm"
        />
      </div>
      <slot></slot>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  title: string;
  badgeCount?: number;
  badgeColor?: string;
  extraClass?: string;
}

const props = withDefaults(defineProps<Props>(), {
  badgeColor: 'primary',
  extraClass: '',
});

const cardClass = computed(() => props.extraClass);
</script>
