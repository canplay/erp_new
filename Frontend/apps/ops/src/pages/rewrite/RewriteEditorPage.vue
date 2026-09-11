<template>
  <q-page class="q-pa-md">
    <q-btn flat icon="arrow_back" label="返回任务列表" @click="$router.push('/rewrite')" class="q-mb-md" />
    <div v-if="task">
      <div class="text-h5 q-mb-md">洗文结果: {{ String(task.id ?? '').substring(0, 8) }}...</div>

      <div class="row q-col-gutter-md">
        <div class="col-12 col-md-4">
          <q-card bordered>
            <q-card-section><div class="text-subtitle1 text-weight-bold">原文</div></q-card-section>
            <q-separator />
            <q-card-section style="max-height:500px; overflow-y:auto; white-space:pre-wrap" class="bg-grey-1">
              {{ originalBody || '暂无原文' }}
            </q-card-section>
          </q-card>
        </div>
        <div class="col-12 col-md-8">
          <div class="row q-col-gutter-md">
            <div v-for="(v, i) in versions" :key="v.id" class="col-12 col-md-6">
              <q-card bordered>
                <q-card-section class="row items-center">
                  <div class="text-subtitle1 text-weight-bold">版本 {{ i + 1 }}</div>
                  <q-space />
                  <q-badge :color="v.status === 'confirmed' ? 'green' : 'orange'">{{ v.status }}</q-badge>
                </q-card-section>
                <q-separator />
                <q-card-section style="max-height:450px; overflow-y:auto; white-space:pre-wrap">
                  <div class="text-weight-bold">{{ v.rewritten_title || '无标题' }}</div>
                  <hr class="q-my-sm" />
                  {{ v.rewritten_body || '无内容' }}
                </q-card-section>
                <q-separator />
                <q-card-actions>
                  <q-btn v-if="v.status !== 'confirmed'" flat color="green" icon="check" label="确认选用" @click="confirm(v.id)" />
                  <q-btn v-else flat color="grey" icon="check" label="已选用" disable />
                </q-card-actions>
              </q-card>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="text-center q-py-xl"><q-spinner size="40px" /><div class="q-mt-sm text-grey">加载洗文结果...</div></div>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { rewriteApi } from '@/api/rewrite';
import { contentApi } from '@/api/contents';

interface RewriteVersion {
  id: string;
  status: string;
  rewritten_title: string;
  rewritten_body: string;
}

const route = useRoute();
const params = route.params as Record<string, string>;
const task = ref<Record<string, unknown> | null>(null);
const versions = ref<RewriteVersion[]>([]);
const originalBody = ref('');

async function confirm(versionId: string) {
  try {
    await rewriteApi.updateVersion(versionId, { status: 'confirmed' });
    versions.value = versions.value.map(v => v.id === versionId ? { ...v, status: 'confirmed' as const } : v);
  } catch { void 0; }
}

onMounted(async () => {
  try {
    const taskId = String(params.taskId ?? '');
    task.value = await rewriteApi.getTask(taskId);
    if (task.value?.content_id) {
      const content = await contentApi.get(task.value.content_id as string);
      originalBody.value = content.body ?? '';
    }
  } catch { void 0; }
});
</script>
