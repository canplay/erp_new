<template>
  <q-table
    :rows="rows"
    :columns="columns"
    :loading="loading"
    row-key="id"
    :pagination="{ rowsPerPage: 10 }"
  >
    <template #body-cell-actions="{ row }">
      <q-td auto-width>
        <q-btn flat dense icon="edit" color="primary" @click="$emit('edit', row)" />
        <q-btn flat dense icon="delete" color="negative" @click="$emit('delete', row)" />
      </q-td>
    </template>
  </q-table>
</template>

<script setup lang="ts">
defineOptions({ name: 'AnnouncementTable' });

import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

interface AnnouncementRow {
  id: string | number;
  title: string;
  content: string;
  status: string;
  created_at: string;
}

const props = defineProps<{
  rows: AnnouncementRow[];
  loading?: boolean;
}>();

defineEmits<{
  (e: 'edit', row: AnnouncementRow): void;
  (e: 'delete', row: AnnouncementRow): void;
}>();

const { t: $t } = useI18n();

const columns = computed(() => [
  { name: 'title', label: $t('announcement.titleField', '标题'), field: 'title', align: 'left', sortable: true },
  { name: 'status', label: $t('announcement.status', '状态'), field: 'status', align: 'center' },
  { name: 'created_at', label: $t('common.createdAt', '创建时间'), field: 'created_at', align: 'center' },
  { name: 'actions', label: $t('common.actions', '操作'), align: 'center' },
]);
</script>
