<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">发布任务</div>
    <q-table :rows="tasks" :columns="cols" row-key="id" flat bordered :loading="loading">
      <template v-slot:body-cell-status="{ row }"><td><q-badge :color="row.status === 'published' ? 'green' : row.status === 'failed' ? 'red' : 'grey'">{{ row.status }}</q-badge></td></template>
    </q-table>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { publishApi } from '@/api/publish';
const cols = [
  { name: 'id', label: 'ID', field: 'id', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'left' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'left' as const },
];
const tasks = ref<Array<Record<string, unknown>>>([]);
const loading = ref(false);
onMounted(async () => { loading.value = true; try { tasks.value = await publishApi.listTasks(); } catch { void 0; } loading.value = false; });
</script>
