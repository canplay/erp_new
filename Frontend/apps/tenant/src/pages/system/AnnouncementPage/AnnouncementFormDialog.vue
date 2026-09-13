<template>
  <q-dialog v-model="model" persistent>
    <q-card class="announcement-form-dialog">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6">{{ isEdit ? $t('announcement.edit') : $t('announcement.add') }}</div>
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-card-section>
        <q-form @submit="onSubmit" class="q-gutter-y-md">
          <q-input
            v-model="form.title"
            :label="$t('announcement.title')"
            :rules="[val => !!val || $t('announcement.titleRequired')]"
            dense
            outlined
          />
          <q-input
            v-model="form.content"
            :label="$t('announcement.content')"
            type="textarea"
            :rules="[val => !!val || $t('announcement.contentRequired')]"
            dense
            outlined
          />
          <q-select
            v-model="form.status"
            :options="statusOptions"
            :label="$t('announcement.status')"
            emit-value
            map-options
            dense
            outlined
          />
        </q-form>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.cancel')" v-close-popup />
        <q-btn color="primary" :label="$t('common.save')" @click="onSubmit" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

interface Props {
  modelValue: boolean
  announcement: unknown
}

const props = withDefaults(defineProps<Props>(), {
  announcement: () => ({})
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save'): void
}>()

const { t: $t } = useI18n()

const model = ref(false)
const form = ref({ title: '', content: '', status: 'active' })

const statusOptions = [
  { label: '启用', value: 'active' },
  { label: '禁用', value: 'inactive' }
]

const isEdit = ref(false)

watch(() => props.modelValue, (val) => {
  model.value = val
  if (val && props.announcement) {
    form.value = { title: '', content: '', status: 'active' }
    isEdit.value = true
  } else {
    form.value = { title: '', content: '', status: 'active' }
    isEdit.value = false
  }
})

watch(model, (val) => {
  emit('update:modelValue', val)
})

function onSubmit(): void {
  emit('save')
  model.value = false
}
</script>

<style scoped>
.announcement-form-dialog {
  width: 500px;
}
</style>
