// @ts-nocheck
<template>
  <div class="announcement-page">
    <div class="text-h5 q-mb-md">{{ $t('announcement.title') }}</div>

    <div class="row q-mb-md q-gutter-sm">
      <q-btn color="primary" :label="$t('announcement.add')" @click="openDialog" />
      <q-input
        v-model="searchQuery"
        :label="$t('common.search')"
        dense
        outlined
        class="col"
      />
    </div>

    <AnnouncementTable
      :announcements="filteredAnnouncements"
      :loading="loading"
      @edit="openDialog"
      @delete="handleDelete"
    />

    <AnnouncementFormDialog
      v-model="showDialog"
      :announcement="currentAnnouncement"
      @save="handleSave"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import AnnouncementTable from './AnnouncementTable.vue'
import AnnouncementFormDialog from './AnnouncementFormDialog.vue'
import { useAnnouncement } from './useAnnouncement'

const { t: $t } = useI18n()

const searchQuery = ref('')
const showDialog = ref(false)
const currentAnnouncement = ref<unknown>(null)

const {
  announcements,
  loading,
  openDialog,
  handleSave,
  handleDelete
} = useAnnouncement()

const filteredAnnouncements = computed(() => {
  if (!searchQuery.value) return announcements.value
  const query = searchQuery.value.toLowerCase()
  return announcements.value.filter(ann =>
    ann.title?.toLowerCase().includes(query) ||
    ann.content?.toLowerCase().includes(query)
  )
})
</script>

<style scoped>
.announcement-page {
  padding: 24px;
}
</style>
