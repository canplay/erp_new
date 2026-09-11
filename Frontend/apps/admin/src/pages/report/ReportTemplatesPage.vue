<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5">报表模板</div>
      <q-space />
      <q-btn color="primary" icon="refresh" flat round dense @click="reload" :loading="loading" />
    </div>

    <!-- 空态提示 -->
    <div v-if="!loading && templates.length === 0" class="column items-center q-pa-xl text-grey-6">
      <q-icon name="folder_open" size="56px" />
      <div class="text-subtitle1 q-mt-md">暂无报表模板</div>
      <div class="text-caption q-mt-xs">模板由后端初始化提供</div>
    </div>

    <!-- 加载骨架 -->
    <div v-if="loading" class="row q-col-gutter-md">
      <div v-for="i in 4" :key="i" class="col-3">
        <q-card flat bordered>
          <q-card-section class="text-center">
            <q-skeleton type="circle" size="48px" class="q-mx-auto" />
          </q-card-section>
          <q-card-section>
            <q-skeleton type="text" class="q-mb-sm" />
            <q-skeleton type="text" width="80%" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div v-if="!loading" class="row q-col-gutter-md">
      <div v-for="template in templates" :key="template.id" class="col-3">
        <q-card class="template-card" clickable v-ripple @click="handleUse(template)">
          <q-card-section class="text-center">
            <q-icon :name="getIcon(template.reportType)" size="xl" color="primary" />
          </q-card-section>
          <q-card-section>
            <div class="text-subtitle1 ellipsis">{{ template.name }}</div>
            <div class="text-caption text-grey ellipsis-2-lines">{{ template.description || '无描述' }}</div>
          </q-card-section>
          <q-card-actions align="center">
            <q-btn flat color="primary" :label="$t('common.use')" @click.stop="handleUse(template)" />
          </q-card-actions>
        </q-card>
      </div>
    </div>
    <q-dialog v-model="showDialog">
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">创建报表</div>
        </q-card-section>
        <q-card-section>
          <q-input v-model="reportName" :label="$t('common.reportTitle')" outlined />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup :disable="creating" />
          <q-btn color="primary" :label="$t('common.create')" @click="confirmCreate" :loading="creating" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { listTemplates, createFromTemplate, type ReportTemplate } from '@/api/report';

/**
 * @brief 报表模板页面
 */
const router = useRouter();
const $q = useQuasar();

const templates = ref<ReportTemplate[]>([]);
const loading = ref(false);
const showDialog = ref(false);
const reportName = ref('');
const creating = ref(false);
const selectedTemplate = ref<ReportTemplate | null>(null);

function getIcon(type: string): string {
  const icons: Record<string, string> = {
    table: 'table_chart',
    chart: 'bar_chart',
    dashboard: 'dashboard',
  };
  return icons[type] || 'assessment';
}

function handleUse(template: ReportTemplate) {
  selectedTemplate.value = template;
  reportName.value = template.name;
  showDialog.value = true;
}

async function confirmCreate() {
  if (!selectedTemplate.value) return;
  creating.value = true;
  try {
    const response = await createFromTemplate(selectedTemplate.value.id, reportName.value);
    $q.notify({ type: 'positive', message: '创建成功' });
    showDialog.value = false;
    // alova transformResponse 已解包，业务体 {id} 在 response.data（兼容两层）
    const respData = response as { data?: { id?: string; data?: { id?: string } } };
    const reportId = respData.data?.id || respData.data?.data?.id;
    if (reportId) {
      await router.push(`/reports/${reportId}/edit`);
    }
  } catch {
    $q.notify({ type: 'negative', message: '创建失败' });
  } finally {
    creating.value = false;
  }
}

async function reload() {
  loading.value = true;
  try {
    const response = await listTemplates();
    // alova transformResponse 已把业务体放 response.data；兼容 {list,total} 与裸数组
    const respData = response as { data?: { list?: ReportTemplate[] } | ReportTemplate[]; list?: ReportTemplate[] };
    const dataObj = respData.data;
    const list = Array.isArray(dataObj) ? dataObj : (dataObj?.list || respData.list || []);
    templates.value = list;
  } catch {
    $q.notify({ type: 'negative', message: '加载失败' });
  } finally {
    loading.value = false;
  }
}

onMounted(reload);
</script>

<style scoped>
.template-card:hover {
  border: 2px solid var(--q-primary);
}
</style>