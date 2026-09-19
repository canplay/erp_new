<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md"><div class="text-h5">智能洗文</div><q-space /><q-btn color="primary" icon="add" label="新建任务" @click="showForm = true" /></div>
    <q-table :rows="tasks" :columns="cols" row-key="id" flat bordered :loading="loading">
      <template v-slot:body-cell-status="{ row }"><td><q-badge :color="row.status === 'completed' ? 'green' : row.status === 'failed' ? 'red' : 'grey'">{{ row.status }}</q-badge></td></template>
    </q-table>
    <q-dialog v-model="showForm">
      <q-card style="min-width:450px">
        <q-card-section><div class="text-h6">新建洗文任务</div></q-card-section>
        <q-card-section>
          <q-select v-model="form.content_id" :options="contents" option-label="title" option-value="id" label="内容" outlined use-input @filter="filterContent" />
          <q-select v-model="form.llm_provider_id" :options="providers" option-label="provider_name" option-value="id" label="LLM" outlined class="q-mt-sm" />
        </q-card-section>
        <q-card-actions align="right"><q-btn flat label="取消" v-close-popup /><q-btn color="primary" label="提交" @click="save" /></q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { rewriteApi } from '@/api/rewrite';
import { contentApi } from '@/api/contents';
import { llmApi, type LLMProvider } from '@/api/llm';
const cols = [
  { name: 'id', label: 'ID', field: 'id', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'left' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'left' as const },
];
const tasks = ref<Array<Record<string, unknown>>>([]);
const contents = ref<Array<Record<string, unknown>>>([]);
const allContents = ref<Array<Record<string, unknown>>>([]);
const providers = ref<LLMProvider[]>([]);
const loading = ref(false);
const showForm = ref(false);
const form = ref({ content_id: '', llm_provider_id: '', target_count: 2 });
function filterContent(val: string, update: (fn: () => void) => void) { update(() => { contents.value = allContents.value.filter((c) => String(c.title).includes(val)); }); }
async function save() { await rewriteApi.createTask(form.value); showForm.value = false; void load(); }
async function load() { loading.value = true; try { tasks.value = await rewriteApi.listTasks(); } catch { void 0; } loading.value = false; }
onMounted(async () => { void load(); try { allContents.value = (await contentApi.list()) as Array<Record<string, unknown>>; contents.value = [...allContents.value]; providers.value = await llmApi.list(); } catch { void 0; } });
</script>
