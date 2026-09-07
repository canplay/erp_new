<template>
  <q-dialog :model-value="modelValue" persistent @update:model-value="$emit('update:modelValue', $event)">
    <q-card style="min-width: 450px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ isEdit ? $t('dictionary.editType') : $t('dictionary.addType') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="$emit('update:modelValue', false)" />
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-form class="q-gutter-md">
          <q-input
            :model-value="form.code" :label="$t('dictionary.typeCode')" outlined :disable="isEdit"
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('dictionary.typeCode') })]"
            @update:model-value="(val: unknown) => $emit('update:code', val as string)"
          />
          <q-input
            :model-value="form.name" :label="$t('dictionary.typeName')" outlined
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('dictionary.typeName') })]"
            @update:model-value="(val: unknown) => $emit('update:name', val as string)"
          />
          <q-input
            :model-value="form.description" :label="$t('dictionary.description')" outlined type="textarea"
            @update:model-value="(val: unknown) => $emit('update:description', val as string)"
          />
          <q-input
            :model-value="form.sort" :label="$t('dictionary.sort')" outlined type="number"
            @update:model-value="(val: unknown) => $emit('update:sort', Number(val ?? 0))"
          />
          <q-select
            :model-value="form.status" :options="statusOptions" :label="$t('dictionary.status')"
            outlined emit-value map-options
            @update:model-value="(val: unknown) => $emit('update:status', Number(val ?? 1))"
          />
        </q-form>
      </q-card-section>
      <q-separator />
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" @click="$emit('update:modelValue', false)" />
        <q-btn color="primary" :label="$t('common.save')" @click="$emit('save')" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';


interface DictTypeForm {
  code: string;
  name: string;
  description: string;
  sort: number;
  status: number;
}

defineProps<{
  modelValue: boolean;
  isEdit: boolean;
  form: DictTypeForm;
}>();

defineEmits<{
  'update:modelValue': [value: boolean];
  'update:code': [value: string];
  'update:name': [value: string];
  'update:description': [value: string];
  'update:sort': [value: number];
  'update:status': [value: number];
  save: [];
}>();

const { t } = useI18n();
const statusOptions = computed(() => [
  { label: t('dictionary.enabled'), value: 1 },
  { label: t('dictionary.disabled'), value: 0 },
]);
</script>
