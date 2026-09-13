<template>
  <q-dialog v-model="model" persistent>
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">{{ $t('common.saveSearchCondition') }}</div>
      </q-card-section>
      <q-card-section>
        <q-input v-model="savedSearchName" outlined :label="$t('common.searchName')" :rules="[val => !!val || '请输入搜索名称']" />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" v-close-popup />
        <q-btn color="primary" :label="$t('common.save')" @click="$emit('save', savedSearchName)" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', name: string): void
}>()

const model = ref(false)
const savedSearchName = ref('')

watch(() => props.modelValue, (val) => { model.value = val })
watch(model, (val) => { emit('update:modelValue', val) })
</script>
