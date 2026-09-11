<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">抓取历史</div>
    <q-table :rows="tasks" :columns="columns" row-key="id" flat bordered :loading="loading">
      <template v-slot:body-cell-status="{ row }">
        <td><q-badge :color="statusColor(row.status)">{{ row.status }}</q-badge></td>
      </template>
      <template v-slot:body-cell-items="{ row }">
        <td>{{ row.items_found || 0 }} (新: {{ row.items_new || 0 }})</td>
      </template>
      <template v-slot:body-cell-error_message="{ row }">
        <td v-if="row.error_message" class="text-negative">{{ row.error_message.substring(0, 80) }}</td>
        <td v-else class="text-grey">-</td>
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { crawlApi } from '@/api/crawl';
import { useRoute } from 'vue-router';

const route = useRoute();
const columns = [
  { name: 'status', label: '状态', field: 'status', align: 'left' as const },
  { name: 'items', label: '结果', field: 'items', align: 'left' as const },
  { name: 'error_message', label: '错误', field: 'error_message', align: 'left' as const },
  { name: 'started_at', label: '开始', field: 'started_at', align: 'left' as const },
  { name: 'completed_at', label: '完成', field: 'completed_at', align: 'left' as const },
];
const tasks = ref<Array<Record<string, unknown>>>([]);
const loading = ref(false);

function statusColor(s: string) {
  return ({ running: 'blue', completed: 'green', failed: 'red', pending: 'grey' } as Record<string, string>)[s] || 'grey';
}

onMounted(async () => {
  loading.value = true;
  const sourceId = route.query.source_id as string;
  try { tasks.value = sourceId ? await crawlApi.history(sourceId) : []; } catch { void 0; }
  loading.value = false;
});
</script>
