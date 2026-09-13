<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5">内容库</div>
      <q-space />
      <q-btn color="primary" icon="add" label="手动创建" @click="showCreate = true" />
    </div>
    <q-table :rows="contentStore.contents" :columns="cols" row-key="id" flat bordered :loading="contentStore.loading">
      <template v-slot:body-cell-status="{ row }"><td><q-badge :color="row.status === 'draft' ? 'grey' : 'green'">{{ row.status }}</q-badge></td></template>
      <template v-slot:body-cell-title="{ row }"><td class="cursor-pointer text-primary" @click="$router.push('/contents/' + row.id)">{{ (row.title || '').substring(0, 50) }}</td></template>
    </q-table>
    <q-dialog v-model="showCreate" maximized>
      <q-card>
        <q-card-section><div class="text-h6">手动创建内容</div></q-card-section>
        <q-card-section>
          <q-input v-model="form.title" label="标题" outlined />
          <q-input v-model="form.body" label="正文" outlined type="textarea" class="q-mt-sm" autogrow />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="取消" v-close-popup />
          <q-btn color="primary" label="保存" @click="saveContent" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useContentStore } from '@/stores/content';
import { contentApi } from '@/api/contents';
const cols = [
  { name: 'title', label: '标题', field: 'title', align: 'left' as const },
  { name: 'content_type', label: '类型', field: 'content_type', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'left' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'left' as const },
];
const $q = useQuasar();
const contentStore = useContentStore();
const showCreate = ref(false);
const form = ref({ title: '', content_type: 'article', body: '' });
async function saveContent() {
  try {
    await contentStore.fetchList(); // 先刷新列表
    await contentApi.create(form.value);
    showCreate.value = false;
    form.value = { title: '', content_type: 'article', body: '' };
    $q.notify({ type: 'positive', message: '内容已创建' });
    void contentStore.fetchList();
  } catch {
    $q.notify({ type: 'negative', message: '创建失败' });
  }
}
onMounted(() => void contentStore.fetchList());
</script>
