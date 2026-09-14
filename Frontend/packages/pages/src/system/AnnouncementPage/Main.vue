<template>
  <div class="announcement-page">
    <q-page padding>
      <div class="row items-center justify-between q-mb-md">
        <div class="text-h5">{{ $t('announcement.title', '公告管理') }}</div>
        <q-btn color="primary" icon="add" :label="$t('common.add', '新增')" @click="openCreate" />
      </div>
      <AnnouncementTable
        :rows="announcements"
        :loading="loading"
        @edit="openEdit"
        @delete="onDelete"
      />
      <AnnouncementFormDialog
        v-model="dialogVisible"
        :edit-data="editingData"
        @submit="onSubmit"
      />
    </q-page>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'AnnouncementMain' });

import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import AnnouncementTable from './AnnouncementTable.vue';
import AnnouncementFormDialog from './AnnouncementFormDialog.vue';
import { useAnnouncement } from './useAnnouncement';

const $q = useQuasar();
const { t: $t } = useI18n();
const { announcements, loading, loadAnnouncements, deleteAnnouncement } = useAnnouncement();

const dialogVisible = ref(false);
const editingData = ref<any>(null);

onMounted(() => {
  loadAnnouncements();
});

function openCreate() {
  editingData.value = null;
  dialogVisible.value = true;
}

function openEdit(row: any) {
  editingData.value = row;
  dialogVisible.value = true;
}

function onDelete(row: any) {
  $q.dialog({
    title: $t('common.confirm', '确认'),
    message: $t('common.deleteConfirm', '确定删除？'),
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    await deleteAnnouncement(row.id);
    await loadAnnouncements();
  });
}

async function onSubmit() {
  dialogVisible.value = false;
  await loadAnnouncements();
}
</script>
