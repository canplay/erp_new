<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">{{ $t('workflow.instanceDetail') }}</div>
    <q-card>
      <q-card-section>
        <q-markup-table>
          <tbody>
            <tr>
              <td>{{ $t('workflow.instanceId') }}</td>
              <td>{{ instance?.id }}</td>
            </tr>
            <tr>
              <td>{{ $t('workflow.workflowId') }}</td>
              <td>{{ instance?.workflowId }}</td>
            </tr>
            <tr>
              <td>{{ $t('workflow.statusLabel') }}</td>
              <td>
                <q-badge
                  :color="getStatusColor(instance?.status || '')"
                  :label="getStatusLabel(instance?.status || '')"
                />
              </td>
            </tr>
            <tr>
              <td>{{ $t('workflow.initiator') }}</td>
              <td>{{ instance?.startedBy }}</td>
            </tr>
            <tr>
              <td>{{ $t('workflow.startTime') }}</td>
              <td>{{ instance?.startedAt }}</td>
            </tr>
            <tr>
              <td>{{ $t('workflow.completionTime') }}</td>
              <td>{{ instance?.completed_at || '-' }}</td>
            </tr>
          </tbody>
        </q-markup-table>
      </q-card-section>

      <q-separator />

      <q-card-section>
        <div class="text-subtitle1 q-mb-md">{{ $t('workflow.taskRecords') }}</div>
        <q-table
          :rows="tasks"
          :columns="taskColumns"
          flat
          bordered
          row-key="id"
        />
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.back')" @click="router.back()" />
      </q-card-actions>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { getInstance, listTasks, type WorkflowInstance, type TaskRecord } from '@/api/workflow';

/**
 * @brief 工作流实例详情页面
 */
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const $q = useQuasar();

const instance = ref<WorkflowInstance | null>(null);
const tasks = ref<TaskRecord[]>([]);

const taskColumns = [
  { name: 'nodeName', label: t('workflow.nodeName'), field: 'nodeName', align: 'left' as const },
  { name: 'assignee', label: t('workflow.assignee'), field: 'assignee', align: 'left' as const },
  { name: 'status', label: t('workflow.statusLabel'), field: 'status', align: 'center' as const },
  { name: 'startedAt', label: t('workflow.startTime'), field: 'startedAt', align: 'center' as const },
  { name: 'completed_at', label: t('workflow.completionTime'), field: 'completed_at', align: 'center' as const },
];

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    pending: 'grey',
    running: 'primary',
    completed: 'positive',
    rejected: 'negative',
  };
  return colors[status] || 'grey';
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    pending: t('workflow.statusPending'),
    running: t('workflow.statusRunning'),
    completed: t('workflow.statusCompleted'),
    rejected: t('workflow.statusRejected'),
  };
  return labels[status] || status;
}

onMounted(async () => {
  const instanceId = route.params.id as string;
  
  try {
    const [instanceRes, tasksRes] = await Promise.all([
      getInstance(instanceId),
      listTasks(instanceId),
    ]);
    instance.value = instanceRes.data?.data ?? null;
    tasks.value = tasksRes.data?.data ?? [];
  } catch {
    $q.notify({ type: 'negative', message: t('workflow.loadFailed') });
  }
});
</script>