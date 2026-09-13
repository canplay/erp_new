<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5">抓取源管理</div>
      <q-space />
      <q-btn color="primary" icon="add" label="新建抓取源" @click="showForm = true" />
    </div>
    <q-table :rows="sources" :columns="columns" row-key="id" flat bordered :loading="loading">
      <template v-slot:body-cell-platform="{ row }">
        <td><q-badge :color="platformColor(row.platform)">{{ row.platform }}</q-badge></td>
      </template>
      <template v-slot:body-cell-is_active="{ row }">
        <td><q-toggle v-model="row.is_active" @update:model-value="toggleSource(row)" /></td>
      </template>
      <template v-slot:body-cell-actions="{ row }">
        <td class="text-center">
          <q-btn dense flat icon="play_arrow" color="green" @click="triggerCrawl(row)" :loading="row.triggering" />
          <q-btn dense flat icon="delete" color="negative" @click="confirmDelete(row)" />
        </td>
      </template>
    </q-table>
    <q-dialog v-model="showForm">
      <q-card style="min-width:500px">
        <q-card-section><div class="text-h6">新建抓取源</div></q-card-section>
        <q-card-section>
          <q-input v-model="form.source_name" label="名称" outlined />
          <q-select v-model="form.platform" :options="platforms" label="平台" outlined class="q-mt-sm" />
          <q-input v-model="form.keyword" label="关键词" outlined class="q-mt-sm" />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="取消" v-close-popup />
          <q-btn color="primary" label="保存" @click="saveSource" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { crawlApi } from '@/api/crawl';

const $q = useQuasar();

interface CrawlSource {
  id: string;
  platform: string;
  source_name: string;
  is_active: boolean;
  crawl_interval?: number;
  last_crawled_at?: string;
  triggering?: boolean;
}

const columns = [
  { name: 'source_name', label: '名称', field: 'source_name', align: 'left' as const },
  { name: 'platform', label: '平台', field: 'platform', align: 'left' as const },
  { name: 'crawl_interval', label: '间隔', field: 'crawl_interval', align: 'left' as const },
  { name: 'is_active', label: '启用', field: 'is_active', align: 'center' as const },
  { name: 'last_crawled_at', label: '上次抓取', field: 'last_crawled_at', align: 'left' as const },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
];
const platforms = ['bilibili', 'weibo', 'xiaohongshu', 'douyin', 'wechat_mp', 'custom'];
const sources = ref<CrawlSource[]>([]);
const loading = ref(false);
const showForm = ref(false);
const form = ref({ platform: 'bilibili', source_name: '', keyword: '', crawl_interval: 3600 });

function platformColor(p: string) {
  const map: Record<string, string> = { weibo: 'red', bilibili: 'blue', xiaohongshu: 'orange', douyin: 'purple', wechat_mp: 'green', custom: 'grey' };
  return map[p] || 'grey';
}
async function load() { loading.value = true; try { sources.value = await crawlApi.listSources(); } catch { void 0; } loading.value = false; }
async function saveSource() {
  await crawlApi.createSource({ platform: form.value.platform, source_name: form.value.source_name, source_config: { keyword: form.value.keyword }, crawl_interval: form.value.crawl_interval });
  showForm.value = false; form.value = { platform: 'bilibili', source_name: '', keyword: '', crawl_interval: 3600 }; void load();
}
async function triggerCrawl(row: CrawlSource) { row.triggering = true; try { await crawlApi.trigger(row.id); } catch { void 0; } row.triggering = false; void load(); }
async function toggleSource(row: CrawlSource) { await crawlApi.updateSource(row.id, { is_active: row.is_active }); }
function confirmDelete(row: CrawlSource) {
  $q.dialog({
    title: '确认删除',
    message: `确定删除抓取源「${row.source_name}」？`,
    cancel: true,
    persistent: true,
  }).onOk(() => {
    void (async () => {
      try { await crawlApi.deleteSource(row.id); $q.notify({ type: 'positive', message: '已删除' }); void load(); }
      catch { $q.notify({ type: 'negative', message: '删除失败' }); }
    })();
  });
}
onMounted(() => void load());
</script>
