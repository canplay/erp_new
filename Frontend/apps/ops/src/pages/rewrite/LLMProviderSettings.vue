<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md"><div class="text-h5">LLM 提供商</div><q-space /><q-btn color="primary" icon="add" label="添加" @click="showForm = true" /></div>
    <q-table :rows="llmStore.providers" :columns="cols" row-key="id" flat bordered :loading="llmStore.loading">
      <template v-slot:body-cell-is_active="{ row }"><td><q-badge :color="row.is_active ? 'green' : 'grey'">{{ row.is_active ? '启用' : '停用' }}</q-badge></td></template>
    </q-table>
    <q-dialog v-model="showForm">
      <q-card style="min-width:500px">
        <q-card-section><div class="text-h6">添加 LLM 提供商</div></q-card-section>
        <q-card-section>
          <q-input v-model="form.provider_name" label="名称" outlined placeholder="如 OpenAI" />
          <q-input v-model="form.api_endpoint" label="API 端点" outlined class="q-mt-sm" placeholder="https://api.openai.com/v1" />
          <q-input v-model="form.api_key" label="API Key" outlined class="q-mt-sm" type="password" />
          <q-input v-model="form.model_name" label="模型" outlined class="q-mt-sm" placeholder="gpt-4o" />
        </q-card-section>
        <q-card-actions align="right"><q-btn flat label="取消" v-close-popup /><q-btn color="primary" label="保存" @click="save" /></q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useLlmStore } from '@/stores/llm';
const cols = [
  { name: 'provider_name', label: '名称', field: 'provider_name', align: 'left' as const },
  { name: 'api_endpoint', label: '端点', field: 'api_endpoint', align: 'left' as const },
  { name: 'model_name', label: '模型', field: 'model_name', align: 'left' as const },
  { name: 'is_active', label: '状态', field: 'is_active', align: 'left' as const },
];
const $q = useQuasar();
const llmStore = useLlmStore();
const showForm = ref(false);
const form = ref({ provider_name: '', api_endpoint: '', api_key: '', model_name: '' });
async function save() {
  try {
    await llmStore.addProvider(form.value);
    showForm.value = false;
    form.value = { provider_name: '', api_endpoint: '', api_key: '', model_name: '' };
    $q.notify({ type: 'positive', message: 'LLM 提供商已添加' });
  } catch {
    $q.notify({ type: 'negative', message: '添加失败' });
  }
}
onMounted(() => void llmStore.fetchProviders());
</script>
