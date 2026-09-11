<template>
  <q-page class="q-pa-md notification-template-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('notificationTemplate.title') }}</div>

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center q-col-gutter-md">
        <div class="col-12 col-sm-6">
          <q-input v-model="filters.keyword" dense outlined :placeholder="$t('common.keyword')" @keyup.enter="handleSearch">
            <template v-slot:prepend>
              <q-icon name="search" />
            </template>
          </q-input>
        </div>
        <div class="col-12 col-sm-3">
          <q-select v-model="filters.type" :options="typeOptions" :label="$t('common.type')" dense outlined clearable emit-value map-options />
        </div>
        <q-space />
        <div class="col-auto">
          <q-btn color="positive" icon="add" :label="$t('notificationTemplate.add')" @click="openCreateDialog" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 模板列表 -->
    <q-card bordered>
      <q-table :rows="templates" :columns="columns" row-key="id" flat :loading="loading" :pagination="pagination" @request="onTableRequest">
        <!-- 名称列 -->
        <template v-slot:body-cell-name="props">
          <q-td :props="props">
            <div class="text-weight-medium">{{ props.row.name }}</div>
            <div class="text-caption text-grey">{{ props.row.code }}</div>
          </q-td>
        </template>

        <!-- 类型列 -->
        <template v-slot:body-cell-type="props">
          <q-td :props="props">
            <q-badge :color="getTypeColor(props.row.type)" :label="getTypeLabel(props.row.type)" />
          </q-td>
        </template>

        <!-- 渠道列 -->
        <template v-slot:body-cell-channels="props">
          <q-td :props="props">
            <q-chip v-for="channel in props.row.channels" :key="channel" dense size="sm" :color="getChannelColor(channel)" text-color="white">
              {{ getChannelLabel(channel) }}
            </q-chip>
          </q-td>
        </template>

        <!-- 启用状态列 -->
        <template v-slot:body-cell-enabled="props">
          <q-td :props="props">
            <q-toggle v-model="props.row.enabled" dense @update:model-value="(val) => handleToggle(props.row, val)" />
          </q-td>
        </template>

        <!-- 创建时间列 -->
        <template v-slot:body-cell-created_at="props">
          <q-td :props="props">
            {{ props.value ? formatTs(props.value) : '-' }}
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense color="primary" :label="$t('common.view')" @click="openViewDialog(props.row)" />
            <q-btn flat dense color="primary" :label="$t('common.edit')" @click="openEditDialog(props.row)" />
            <q-btn flat dense color="positive" :label="$t('notificationTemplate.test')" @click="handleTest(props.row)" />
            <q-btn flat dense color="negative" :label="$t('common.delete')" @click="handleDelete(props.row)" />
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent :maximized="isMobile">
      <q-card :style="{ width: isMobile ? '100%' : '600px', maxWidth: '600px' }">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? $t('notificationTemplate.edit') : $t('notificationTemplate.add') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section style="max-height: 60vh" class="scroll">
          <q-form class="q-gutter-md">
            <q-input v-model="form.name" :label="$t('notificationTemplate.name')" outlined :rules="[(val) => !!val || $t('validation.required', { field: $t('notificationTemplate.name') })]" />
            <q-input v-model="form.code" :label="$t('notificationTemplate.code')" outlined :disable="isEdit" :rules="[(val) => !!val || $t('validation.required', { field: $t('notificationTemplate.code') })]" />
            <q-input v-model="form.title" :label="$t('notificationTemplate.titleTemplate')" outlined :rules="[(val) => !!val || $t('validation.required', { field: $t('notificationTemplate.titleTemplate') })]" />
            <q-input v-model="form.content" :label="$t('notificationTemplate.contentTemplate')" type="textarea" outlined rows="4" :rules="[(val) => !!val || $t('validation.required', { field: $t('notificationTemplate.contentTemplate') })]" />
            <q-select v-model="form.type" :options="typeOptions" :label="$t('common.type')" outlined emit-value map-options />
            <q-select v-model="form.channels" :options="channelOptions" :label="$t('notificationTemplate.channels')" outlined multiple emit-value map-options />
            <div class="text-caption text-grey">
              {{ $t('notificationTemplate.variableHint') }}
            </div>
          </q-form>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDialog = false" />
          <q-btn color="primary" :label="$t('common.save')" @click="handleSave" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 查看对话框 -->
    <q-dialog v-model="showViewDialog">
      <q-card style="min-width: 600px">
        <q-card-section>
          <div class="text-h6">{{ $t('notificationTemplate.preview') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="q-mb-md">
            <div class="text-caption text-grey">{{ $t('notificationTemplate.name') }}</div>
            <div>{{ viewingTemplate?.name }}</div>
          </div>
          <div class="q-mb-md">
            <div class="text-caption text-grey">{{ $t('notificationTemplate.code') }}</div>
            <div>{{ viewingTemplate?.code }}</div>
          </div>
          <div class="q-mb-md">
            <div class="text-caption text-grey">{{ $t('notificationTemplate.titleTemplate') }}</div>
            <div>{{ viewingTemplate?.title }}</div>
          </div>
          <div class="q-mb-md">
            <div class="text-caption text-grey">{{ $t('notificationTemplate.contentTemplate') }}</div>
            <div>{{ viewingTemplate?.content }}</div>
          </div>
          <div class="q-mb-md">
            <div class="text-caption text-grey">{{ $t('notificationTemplate.variables') }}</div>
            <div>{{ viewingTemplate?.variables?.join(', ') || '-' }}</div>
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" color="primary" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file NotificationTemplatePage.vue
 * @description 通知模板管理页面
 * @date 2026-04-04
 */

import { ref, computed, onMounted, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { formatDate } from '@/utils/format';

/** 后端返回 epoch 秒，格式化显示 */
function formatTs(value: number | string): string {
  const num = typeof value === 'string' ? Number(value) : value;
  if (!num) return '-';
  return formatDate(num < 1e12 ? num * 1000 : num);
}
import {
  listNotificationTemplates,
  createNotificationTemplate,
  updateNotificationTemplate,
  deleteNotificationTemplate,
  toggleNotificationTemplate,
  sendTestNotification,
  type NotificationTemplate,
} from '@/api/notification';

const { t } = useI18n();
const $q = useQuasar();

// 状态
const loading = ref(false);
const templates = ref<NotificationTemplate[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 10, rowsNumber: 0 });

// 筛选
const filters = reactive({
  keyword: '',
  type: null,
});

// 对话框状态
const showDialog = ref(false);
const showViewDialog = ref(false);
const isEdit = ref(false);
const viewingTemplate = ref<NotificationTemplate | null>(null);

// 表单
const form = reactive({
  id: 0,
  name: '',
  code: '',
  title: '',
  content: '',
  type: 'system',
  channels: ['in_app'] as string[],
});

// 选项
const typeOptions = [
  { label: '系统通知', value: 'system' },
  { label: '操作通知', value: 'operation' },
  { label: '审批提醒', value: 'approval' },
  { label: '自定义', value: 'custom' },
];

const channelOptions = [
  { label: '站内通知', value: 'in_app' },
  { label: '邮件', value: 'email' },
  { label: '短信', value: 'sms' },
];

// 表格列
const columns = computed(() => [
  { name: 'name', label: t('notificationTemplate.name'), field: 'name', align: 'left' as const },
  { name: 'type', label: t('common.type'), field: 'type', align: 'center' as const },
  { name: 'channels', label: t('notificationTemplate.channels'), field: 'channels', align: 'left' as const },
  { name: 'enabled', label: t('notificationTemplate.enabled'), field: 'enabled', align: 'center' as const },
  { name: 'created_at', label: t('notificationTemplate.created_at'), field: 'created_at', align: 'left' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

const isMobile = computed(() => $q.screen.lt.md);

// 方法
function getTypeColor(type: string): string {
  const colors: Record<string, string> = { system: 'blue', operation: 'green', approval: 'orange', custom: 'purple' };
  return colors[type] || 'grey';
}

function getTypeLabel(type: string): string {
  // 兼容两种契约：前端语义值 (system/operation/approval/custom) 与后端存储值 (notification/email/push)
  const labels: Record<string, string> = {
    system: '系统', operation: '操作', approval: '审批', custom: '自定义',
    notification: '系统通知', email: '邮件', push: '推送', sms: '短信',
  };
  return labels[type] || type;
}

function getChannelColor(channel: string): string {
  const colors: Record<string, string> = { in_app: 'blue', email: 'green', sms: 'orange', push: 'purple' };
  return colors[channel] || 'grey';
}

function getChannelLabel(channel: string): string {
  const labels: Record<string, string> = { in_app: '站内', email: '邮件', sms: '短信', push: '推送' };
  return labels[channel] || channel;
}

async function loadTemplates() {
  loading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };
    if (filters.keyword) params.keyword = filters.keyword;
    if (filters.type) params.type = filters.type;
    const response = await listNotificationTemplates(params);
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: NotificationTemplate[]; total?: number; data?: { list?: NotificationTemplate[]; total?: number } };
    const data = respData.list ? { list: respData.list, total: respData.total || 0 } : respData.data || {};
    templates.value = data.list || [];
    pagination.value.rowsNumber = data.total || 0;
  } catch (error) {
    console.error('【加载模板失败】', error);
  } finally {
    loading.value = false;
  }
}

function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadTemplates();
}

function handleSearch() {
  pagination.value.page = 1;
  void loadTemplates();
}

function openCreateDialog() {
  isEdit.value = false;
  Object.assign(form, { id: 0, name: '', code: '', title: '', content: '', type: 'system', channels: ['in_app'] });
  showDialog.value = true;
}

function openEditDialog(template: NotificationTemplate) {
  isEdit.value = true;
  Object.assign(form, template);
  showDialog.value = true;
}

function openViewDialog(template: NotificationTemplate) {
  viewingTemplate.value = template;
  showViewDialog.value = true;
}

async function handleSave() {
  try {
    if (isEdit.value) {
      await updateNotificationTemplate(form.id, form as Partial<NotificationTemplate>);
    } else {
      await createNotificationTemplate(form as Partial<NotificationTemplate>);
    }
    showDialog.value = false;
    void loadTemplates();
    $q.notify({ type: 'positive', message: t('common.success') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

async function handleToggle(template: NotificationTemplate, enabled: boolean) {
  try {
    await toggleNotificationTemplate(template.id, enabled);
    $q.notify({ type: 'positive', message: t('common.success') });
  } catch {
    template.enabled = !enabled;
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

async function handleTest(template: NotificationTemplate) {
  try {
    await sendTestNotification(template.id);
    $q.notify({ type: 'positive', message: t('notificationTemplate.testSent') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

function handleDelete(template: NotificationTemplate) {
  $q.dialog({
    title: t('common.confirm'),
    message: t('notificationTemplate.deleteConfirm', { name: template.name }),
    cancel: true,
    persistent: true,
  }).onOk(() => {
    void (async () => {
      try {
        await deleteNotificationTemplate(template.id);
        void loadTemplates();
        $q.notify({ type: 'positive', message: t('common.success') });
      } catch {
        $q.notify({ type: 'negative', message: t('common.error') });
      }
    })();
  });
}

onMounted(() => {
  void loadTemplates();
});
</script>
