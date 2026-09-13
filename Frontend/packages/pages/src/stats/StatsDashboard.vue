<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">数据统计</div>

    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-md-3" v-for="card in statCards" :key="card.title">
        <q-card bordered>
          <q-card-section class="text-center">
            <div class="text-grey text-subtitle2">{{ card.title }}</div>
            <div class="text-h3 text-weight-bold text-primary">{{ card.value }}</div>
            <div class="text-caption text-grey">{{ card.subtitle }}</div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-8">
        <q-card bordered>
          <q-card-section><div class="text-h6">粉丝增长趋势</div></q-card-section>
          <q-card-section style="min-height:300px">
            <ECharts :config="trendChartConfig" />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-4">
        <q-card bordered>
          <q-card-section><div class="text-h6">平台分布</div></q-card-section>
          <q-card-section style="min-height:300px">
            <ECharts :config="pieChartConfig" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div class="row q-col-gutter-md q-mt-md">
      <div class="col-12">
        <q-card bordered>
          <q-card-section><div class="text-h6">热门内容排行</div></q-card-section>
          <q-table :rows="hotContents" :columns="hotColumns" row-key="id" flat hide-bottom>
            <template v-slot:no-data><div class="text-grey text-center q-py-md">暂无数据</div></template>
          </q-table>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import ECharts from '@erp-new-frontend-monorepo/components/src/ECharts.vue';
import type { EChartsConfig } from '@erp-new-frontend-monorepo/components/src/ECharts.vue';

const trendChartConfig = computed<EChartsConfig>(() => ({
  type: 'line',
  height: '280px',
  showLegend: true,
  showTooltip: true,
  data: Array.from({ length: 12 }, (_, i) => ({
    label: `${i + 1}月`,
    value: Math.floor(Math.random() * 100 + 50),
  })),
}));

const pieChartConfig = computed<EChartsConfig>(() => ({
  type: 'pie',
  height: '280px',
  showLegend: true,
  showTooltip: true,
  data: [
    { label: '微博', value: 35 },
    { label: '公众号', value: 25 },
    { label: 'B站', value: 20 },
    { label: '小红书', value: 15 },
    { label: '抖音', value: 5 },
  ],
}));

const statCards = ref([
  { title: '总关注', value: '0', subtitle: '全部平台合计' },
  { title: '本周发文', value: '0', subtitle: '本周已发布' },
  { title: '总阅读', value: '0', subtitle: '累计阅读量' },
  { title: '互动率', value: '0%', subtitle: '点赞+评论/阅读' },
]);

const hotColumns = [
  { name: 'rank', label: '#', field: 'rank', align: 'left' as const },
  { name: 'title', label: '标题', field: 'title', align: 'left' as const },
  { name: 'views', label: '阅读', field: 'views', align: 'right' as const },
  { name: 'likes', label: '点赞', field: 'likes', align: 'right' as const },
  { name: 'comments', label: '评论', field: 'comments', align: 'right' as const },
];
const hotContents = ref<Array<Record<string, unknown>>>([]);
</script>
