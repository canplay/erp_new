<template>
  <div class="permission-config">
    <div class="text-subtitle2 q-mb-sm">{{ $t('permission.config') }}</div>
    <div class="row q-col-gutter-sm">
      <div class="col-6">
        <q-select
          v-model="readUsers"
          :options="userOptions"
          multiple
          use-chips
          dense
          outlined
          :label="$t('permission.read')"
          @update:model-value="(val) => $emit('update:read', val)"
        />
      </div>
      <div class="col-6">
        <q-select
          v-model="writeUsers"
          :options="userOptions"
          multiple
          use-chips
          dense
          outlined
          :label="$t('permission.write')"
          @update:model-value="(val) => $emit('update:write', val)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

interface Props {
  readUsers: string[]
  writeUsers: string[]
  userOptions: Array<{ label: string; value: string }>
}

const props = defineProps<Props>()

defineEmits<{
  (e: 'update:read', value: string[]): void
  (e: 'update:write', value: string[]): void
}>()
</script>

<style scoped>
.permission-config {
  padding: 16px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  background: #fafafa;
}

.body--dark .permission-config {
  background: #1e1e1e;
  border-color: #3d3d3d;
}
</style>
