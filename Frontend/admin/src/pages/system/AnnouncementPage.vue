/**
 * @file system/AnnouncementPage.vue
 * @description 公告管理页面 - 组合 AnnouncementList + AnnouncementEditor
 * @date 2026-04-04
 */

<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('announcement.title') }}</div>

    <AnnouncementList
      :loading="loading"
      :announcements="announcements"
      :filters="filters"
      :pagination="pagination"
      :status-options-for-search="statusOptionsForSearch"
      :type-labels="typeLabels"
      :keyword-placeholder="$t('announcement.searchPlaceholder')"
      :status-label="$t('announcement.status')"
      :date-range-label="$t('common.dateRange')"
      :add-label="$t('announcement.add')"
      :edit-label="$t('common.edit')"
      :delete-label="$t('common.delete')"
      :active-label="$t('announcement.active')"
      :inactive-label="$t('announcement.inactive')"
      @search="handleSearch"
      @reset="handleReset"
      @open-dialog="openDialog()"
      @edit="openDialog"
      @delete="handleDelete"
      @request="onTableRequest"
    />

    <AnnouncementEditor
      v-model="showDialog"
      :is-edit="isEdit"
      :form="form"
      :type-options="typeOptions"
      :edit-title="$t('announcement.edit')"
      :add-title="$t('announcement.add')"
      :title-label="$t('announcement.titleField')"
      :type-label="$t('announcement.typeLabel')"
      :priority-label="$t('announcement.priority')"
      :content-label="$t('announcement.content')"
      :content-placeholder="$t('announcement.contentPlaceholder')"
      :start-time-label="$t('announcement.start_time')"
      :end-time-label="$t('announcement.end_time')"
      :pinned-label="$t('announcement.pinned')"
      :active-label="$t('announcement.active')"
      :cancel-label="$t('common.cancel')"
      :save-label="$t('common.save')"
      @save="handleSave"
    />

    <ConfirmDialog
      ref="deleteDialogRef"
      :title="$t('announcement.deleteConfirmTitle')"
      :message="$t('announcement.deleteConfirmMessage')"
      icon="warning"
      confirm-color="negative"
      @confirm="doDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import {
  listAnnouncements,
  createAnnouncement,
  updateAnnouncement,
  deleteAnnouncement,
  type Announcement,
} from '@/api/system';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import AnnouncementList from './AnnouncementList.vue';
import AnnouncementEditor from './AnnouncementEditor.vue';
import type { AdvancedFilters } from '@/types/advanced-search';

const { t } = useI18n();
const $q = useQuasar();

const loading = ref(false);
const announcements = ref<Announcement[]>([]);
const showDialog = ref(false);
const isEdit = ref(false);
const currentAnnouncement = ref<Announcement | null>(null);
const deleteDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);
const pendingDelete = ref<Announcement | null>(null);

const pagination = ref({
  page: 1,
  rowsPerPage: 20,
  rowsNumber: 0,
});

const filters = ref<AdvancedFilters>({
  keyword: '',
  start_date: '',
  end_date: '',
  status: null,
});

const form = reactive({
  title: '',
  content: '',
  type: 'info',
  priority: 0,
  isPinned: false,
  isActive: true,
  start_time: '',
  end_time: '',
});

const statusOptionsForSearch = computed(() => [
  { label: t('announcement.statusOptions.all'), value: null },
  { label: t('announcement.active'), value: 'active' },
  { label: t('announcement.inactive'), value: 'inactive' },
]);

const typeOptions = computed(() => [
  { label: t('announcement.typeEnum.info'), value: 'info' },
  { label: t('announcement.typeEnum.warning'), value: 'warning' },
  { label: t('announcement.typeEnum.success'), value: 'success' },
  { label: t('announcement.typeEnum.error'), value: 'error' },
]);

const typeLabels = computed(() => ({
  info: t('announcement.typeEnum.info'),
  warning: t('announcement.typeEnum.warning'),
  success: t('announcement.typeEnum.success'),
  error: t('announcement.typeEnum.error'),
}));

async function loadAnnouncements() {
  loading.value = true;
  try {
    const response = await listAnnouncements({
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    });
    const respData = response as { data?: { list?: Announcement[]; total?: number } };
    if (respData.data) {
      announcements.value = respData.data.list || [];
      pagination.value.rowsNumber = respData.data.total || 0;
    }
  } catch (error) {
    console.error('【加载公告失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
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
}

function handleReset() {
  filters.value = {
    keyword: '',
    start_date: '',
    end_date: '',
    status: null,
  };
  pagination.value.page = 1;
}

function openDialog(announcement?: Announcement) {
  if (announcement) {
    isEdit.value = true;
    currentAnnouncement.value = announcement;
    Object.assign(form, {
      title: announcement.title,
      content: announcement.content,
      type: announcement.type,
      priority: announcement.priority,
      isPinned: announcement.isPinned,
      isActive: announcement.isActive,
      start_time: announcement.start_time || '',
      end_time: announcement.end_time || '',
    });
  } else {
    isEdit.value = false;
    currentAnnouncement.value = null;
    Object.assign(form, {
      title: '',
      content: '',
      type: 'info',
      priority: 0,
      isPinned: false,
      isActive: true,
      start_time: '',
      end_time: '',
    });
  }
  showDialog.value = true;
}

async function handleSave(formData: typeof form) {
  try {
    const data: {
      title: string;
      content: string;
      type: 'info' | 'warning' | 'success' | 'error';
      priority: number;
      isPinned: boolean;
      isActive: boolean;
      start_time?: string;
      end_time?: string;
    } = {
      title: formData.title,
      content: formData.content,
      type: formData.type as 'info' | 'warning' | 'success' | 'error',
      priority: formData.priority,
      isPinned: formData.isPinned,
      isActive: formData.isActive,
    };
    if (formData.start_time) data.start_time = formData.start_time;
    if (formData.end_time) data.end_time = formData.end_time;

    if (isEdit.value && currentAnnouncement.value) {
      await updateAnnouncement(currentAnnouncement.value.id, data);
    } else {
      await createAnnouncement(data);
    }

    $q.notify({ type: 'positive', message: t('common.success') });
    showDialog.value = false;
    void loadAnnouncements();
  } catch (error) {
    console.error('【保存公告失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

function handleDelete(announcement: Announcement) {
  pendingDelete.value = announcement;
  deleteDialogRef.value?.open();
}

async function doDelete() {
  if (!pendingDelete.value) return;
  try {
    await deleteAnnouncement(pendingDelete.value.id);
    $q.notify({ type: 'positive', message: t('common.success') });
    pendingDelete.value = null;
    void loadAnnouncements();
  } catch (error) {
    console.error('【删除公告失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

onMounted(() => {
  void loadAnnouncements();
});
</script>
