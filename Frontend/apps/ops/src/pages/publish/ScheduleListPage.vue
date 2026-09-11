<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md"><div class="text-h5">定时计划</div><q-space /><q-btn color="primary" icon="add" label="新建" @click="showForm = true" /></div>
    <q-table :rows="schedules" :columns="cols" row-key="id" flat bordered :loading="loading">
      <template v-slot:body-cell-status="{ row }"><td><q-badge :color="row.status === 'active' ? 'green' : 'grey'">{{ row.status }}</q-badge></td></template>
    </q-table>
    <q-dialog v-model="showForm">
      <q-card style="min-width:400px">
        <q-card-section><div class="text-h6">新建定时计划</div></q-card-section>
        <q-card-section>
          <q-input v-model="form.cron_expression" label="Cron 表达式" outlined placeholder="0 8 * * 1" />
        </q-card-section>
        <q-card-actions align="right"><q-btn flat label="取消" v-close-popup /><q-btn color="primary" label="保存" @click="save" /></q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { publishApi } from '@/api/publish';
const cols = [
  { name: 'cron_expression', label: 'Cron', field: 'cron_expression', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'left' as const },
  { name: 'next_run_at', label: '下次执行', field: 'next_run_at', align: 'left' as const },
];
const schedules = ref<Array<Record<string, unknown>>>([]);
const loading = ref(false);
const showForm = ref(false);
const form = ref({ cron_expression: '' });
async function save() { await publishApi.createSchedule(form.value); showForm.value = false; void load(); }
async function load() { loading.value = true; try { schedules.value = await publishApi.listSchedules(); } catch { void 0; } loading.value = false; }
onMounted(() => void load());
</script>
