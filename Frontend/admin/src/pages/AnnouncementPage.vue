<template>
  <q-page class="q-pa-md announcement-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('announcement.title') }}</div>

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
          <q-btn color="positive" icon="add" :label="$t('announcement.add')" @click="openCreateDialog" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 公告列表 -->
    <q-card bordered>
      <q-table :rows="announcements" :columns="columns" row-key="id" flat :loading="loading" :pagination="pagination" @request="onTableRequest">
        <!-- 标题列 -->
        <template v-slot:body-cell-title="props">
          <q-td :props="props">
            <div class="row items-center">
              <q-badge v-if="props.row.isPinned" color="warning" class="q-mr-sm" label="置顶" />
              <q-badge :color="getTypeColor(props.row.type)" :label="getTypeLabel(props.row.type)" class="q-mr-sm" />
              <span class="text-weight-medium">{{ props.row.title }}</span>
            </div>
          </q-td>
        </template>

        <!-- 有效期列 -->
        <template v-slot:body-cell-period="props">
          <q-td :props="props">
            <span v-if="props.row.start_time || props.row.end_time">
              {{ formatDate(props.row.start_time) }} - {{ formatDate(props.row.end_time) }}
            </span>
            <span v-else class="text-grey">{{ $t('announcement.permanent') }}</span>
          </q-td>
        </template>

        <!-- 状态列 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-badge :color="props.row.isActive ? 'positive' : 'grey'" :label="props.row.isActive ? '已发布' : '未发布'" />
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense color="primary" :label="$t('common.view')" @click="openViewDialog(props.row)" />
            <q-btn flat dense color="primary" :label="$t('common.edit')" @click="openEditDialog(props.row)" />
            <q-btn flat dense :color="props.row.isPinned ? 'grey' : 'warning'" :icon="props.row.isPinned ? 'push_pin' : 'outlined_flag_flag'" @click="handleTogglePin(props.row)" />
            <q-btn flat dense :color="props.row.isActive ? 'grey' : 'positive'" :icon="props.row.isActive ? 'pause' : 'play_arrow'" @click="handleToggleActive(props.row)" />
            <q-btn flat dense color="negative" icon="delete" @click="handleDelete(props.row)" />
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent :maximized="isMobile">
      <q-card :style="{ width: isMobile ? '100%' : '700px', maxWidth: '700px' }">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? $t('announcement.edit') : $t('announcement.add') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section style="max-height: 60vh" class="scroll">
          <q-form class="q-gutter-md">
            <q-input v-model="form.title" :label="$t('announcement.titleLabel')" outlined :rules="[(val) => !!val || $t('validation.required')]" />
            <q-select v-model="form.type" :options="typeOptions" :label="$t('common.type')" outlined emit-value map-options />
            <q-input v-model="form.content" :label="$t('announcement.content')" type="textarea" outlined rows="6" :rules="[(val) => !!val || $t('validation.required')]" />
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-input v-model="form.start_time" :label="$t('announcement.start_time')" outlined type="datetime-local" />
              </div>
              <div class="col-12 col-sm-6">
                <q-input v-model="form.end_time" :label="$t('announcement.end_time')" outlined type="datetime-local" />
              </div>
            </div>
            <div class="row items-center">
              <q-checkbox v-model="form.isPinned" :label="$t('announcement.pin')" />
              <q-checkbox v-model="form.isActive" :label="$t('announcement.publish')" />
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
          <div class="text-h6">{{ $t('common.view') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="q-mb-md">
            <div class="text-h6">{{ viewingAnnouncement?.title }}</div>
            <div class="row q-gutter-sm q-mt-sm">
              <q-badge v-if="viewingAnnouncement?.isPinned" color="warning" label="置顶" />
              <q-badge v-if="viewingAnnouncement?.isActive" color="positive" label="已发布" />
            </div>
          </div>
          <q-separator class="q-my-md" />
          <div class="announcement-content" v-html="sanitizeHTML(viewingAnnouncement?.content || '')"></div>
          <q-separator class="q-my-md" />
          <div class="text-caption text-grey">
            {{ $t('announcement.period') }}: {{ formatDate(viewingAnnouncement?.start_time) }} - {{ formatDate(viewingAnnouncement?.end_time) || $t('announcement.permanent') }}
          </div>
          <div class="text-caption text-grey">
            {{ $t('announcement.created_by') }}: {{ viewingAnnouncement?.created_by?.username }}
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
 * @file AnnouncementPage.vue
 * @description 系统公告管理页面
 * @date 2026-04-04
 */

import { ref, computed, onMounted, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { sanitizeHTML } from '@/utils/sanitize';
import {
  listAnnouncements,
  createAnnouncement,
  updateAnnouncement,
  deleteAnnouncement,
  toggleAnnouncementPin,
  toggleAnnouncementActive,
  type Announcement,
} from '@/api/notification';

const { t } = useI18n();
const $q = useQuasar();

// 状态
const loading = ref(false);
const announcements = ref<Announcement[]>([]);
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
const viewingAnnouncement = ref<Announcement | null>(null);

// 表单
const form = reactive({
  id: 0,
  title: '',
  content: '',
  type: 'normal',
  isPinned: false,
  isActive: false,
  start_time: '',
  end_time: '',
});

// 选项
const typeOptions = [
  { label: '普通', value: 'normal' },
  { label: '重要', value: 'important' },
  { label: '紧急', value: 'urgent' },
];

// 表格列
const columns = computed(() => [
  { name: 'title', label: t('announcement.titleLabel'), field: 'title', align: 'left' as const },
  { name: 'period', label: t('announcement.period'), field: 'period', align: 'center' as const },
  { name: 'status', label: t('announcement.status'), field: 'status', align: 'center' as const },
  { name: 'created_at', label: t('announcement.created_at'), field: 'created_at', align: 'left' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

const isMobile = computed(() => $q.screen.lt.md);

// 方法
function getTypeColor(type: string): string {
  const colors: Record<string, string> = { normal: 'blue', important: 'orange', urgent: 'negative' };
  return colors[type] || 'grey';
}

function getTypeLabel(type: string): string {
  const labels: Record<string, string> = { normal: '普通', important: '重要', urgent: '紧急' };
  return labels[type] || type;
}

function formatDate(date?: string): string {
  if (!date) return '-';
  return new Date(date).toLocaleString('zh-CN');
}

async function loadAnnouncements() {
  loading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };
    if (filters.keyword) params.keyword = filters.keyword;
    if (filters.type) params.type = filters.type;
    const response = await listAnnouncements(params);
    // 新 httpClient 响应格式：{ list, total, data, ... }
    const data = response as { list?: Announcement[]; data?: { list?: Announcement[]; total?: number }; total?: number };
    announcements.value = data.list || data.data?.list || [];
    pagination.value.rowsNumber = data.total || data.data?.total || 0;
  } catch (error) {
    console.error('【加载公告失败】', error);
  } finally {
    loading.value = false;
  }
}

function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadAnnouncements();
}

function handleSearch() {
  pagination.value.page = 1;
  void loadAnnouncements();
}

function openCreateDialog() {
  isEdit.value = false;
  Object.assign(form, { id: 0, title: '', content: '', type: 'normal', isPinned: false, isActive: false, start_time: '', end_time: '' });
  showDialog.value = true;
}

function openEditDialog(announcement: Announcement) {
  isEdit.value = true;
  Object.assign(form, {
    ...announcement,
    start_time: announcement.start_time ? announcement.start_time.slice(0, 16) : '',
    end_time: announcement.end_time ? announcement.end_time.slice(0, 16) : '',
  });
  showDialog.value = true;
}

function openViewDialog(announcement: Announcement) {
  viewingAnnouncement.value = announcement;
  showViewDialog.value = true;
}

async function handleSave() {
  try {
    const submitData: Record<string, unknown> = {
      title: form.title,
      content: form.content,
      type: form.type,
      isPinned: form.isPinned,
      isActive: form.isActive,
    };
    if (form.start_time) submitData.start_time = form.start_time;
    if (form.end_time) submitData.end_time = form.end_time;
    if (isEdit.value) {
      await updateAnnouncement(form.id, submitData);
    } else {
      await createAnnouncement(submitData as Parameters<typeof createAnnouncement>[0]);
    }
    showDialog.value = false;
    void loadAnnouncements();
    $q.notify({ type: 'positive', message: t('common.success') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

async function handleTogglePin(announcement: Announcement) {
  try {
    await toggleAnnouncementPin(announcement.id, !announcement.isPinned);
    void loadAnnouncements();
    $q.notify({ type: 'positive', message: t('common.success') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

async function handleToggleActive(announcement: Announcement) {
  try {
    await toggleAnnouncementActive(announcement.id, !announcement.isActive);
    void loadAnnouncements();
    $q.notify({ type: 'positive', message: t('common.success') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

function handleDelete(announcement: Announcement) {
  void $q.dialog({
    title: t('common.confirm'),
    message: t('announcement.deleteConfirm', { title: announcement.title }),
    cancel: true,
    persistent: true,
  }).onOk(() => {
    void (async () => {
      try {
        await deleteAnnouncement(announcement.id);
        await loadAnnouncements();
        $q.notify({ type: 'positive', message: t('common.success') });
      } catch {
        $q.notify({ type: 'negative', message: t('common.error') });
      }
    })();
  });
}

onMounted(() => {
  void loadAnnouncements();
});
</script>

<style scoped>
.announcement-content {
  line-height: 1.8;
  white-space: pre-wrap;
}
</style>
