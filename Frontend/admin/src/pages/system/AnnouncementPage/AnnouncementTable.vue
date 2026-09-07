<template>
  <div class="announcement-table">
    <q-table
      :rows="announcements"
      :columns="columns"
      row-key="id"
      :loading="loading"
      virtual-scroll
      virtual-scroll-item-size="48"
      flat
      bordered
    >
      <template v-slot:body-cell-actions="props">
        <q-td :props="props">
          <q-btn
            flat
            dense
            icon="edit"
            color="primary"
            @click="$emit('edit', props.row)"
          />
          <q-btn
            flat
            dense
            icon="delete"
            color="negative"
            @click="$emit('delete', props.row)"
          />
        </q-td>
      </template>
    </q-table>
  </div>
</template>

<script setup lang="ts">
interface Announcement {
  id: string | number
  title: string
  content: string
  status: string
  created_at: string
}

interface Props {
  announcements: Announcement[]
  loading: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  (e: 'edit', row: Announcement): void
  (e: 'delete', row: Announcement): void
}>()

const columns: any[] = [
  { name: 'title', label: '标题', field: 'title', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'right' },
  { name: 'actions', label: '操作', align: 'center' }
]
</script>

<style scoped>
.announcement-table {
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  overflow: hidden;
}
</style>
