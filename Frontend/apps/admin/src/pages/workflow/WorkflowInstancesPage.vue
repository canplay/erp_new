<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">流程实例</div>
    <q-card>
      <q-card-section>
        <q-table
          :rows="instances"
          :columns="columns"
          :loading="loading"
          row-key="id"
          :pagination="pagination"
        >
          <template #body-cell-status="props">
            <q-td :props="props">
              <q-badge
                :color="getStatusColor(props.row.status)"
                :label="getStatusLabel(props.row.status)"
              />
            </q-td>
          </template>
          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense color="primary" icon="visibility" @click="handleView(props.row)">
                <q-tooltip>查看</q-tooltip>
              </q-btn>
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { listInstances, type WorkflowInstance } from '@/api/workflow';

/**
 * @brief 工作流实例列表页面
 */
const router = useRouter();
const $q = useQuasar();

const loading = ref(false);
const instances = ref<WorkflowInstance[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 20 });

const columns = [
  { name: 'id', label: '实例ID', field: 'id', align: 'left' as const },
  { name: 'workflowId', label: '工作流', field: 'workflowId', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'startedBy', label: '发起人', field: 'startedBy', align: 'left' as const },
  { name: 'startedAt', label: '开始时间', field: 'startedAt', align: 'center' as const },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
];

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    pending: 'grey',
    running: 'primary',
    completed: 'positive',
    cancelled: 'negative',
  };
  return colors[status] || 'grey';
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    pending: '待启动',
    running: '运行中',
    completed: '已完成',
    cancelled: '已取消',
  };
  return labels[status] || status;
}

function handleView(row: WorkflowInstance) {
  void router.push(`/workflow/instances/${row.id}`);
}

onMounted(async () => {
  loading.value = true;
  try {
    const response = await listInstances('', {});
    instances.value = response.data?.data?.list || []; // 使用 list 而不是 items
  } catch {
    $q.notify({ type: 'negative', message: '加载失败' });
  } finally {
    loading.value = false;
  }
});
</script>