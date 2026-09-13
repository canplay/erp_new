<template>
  <q-dialog :model-value="modelValue" persistent @update:model-value="$emit('update:modelValue', $event)">
    <q-card style="min-width: 400px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ isEdit ? $t('dictionary.editItem') : $t('dictionary.addItem') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="$emit('update:modelValue', false)" />
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-form class="q-gutter-md">
          <q-input
            :model-value="form.label" :label="$t('dictionary.itemLabel')" outlined
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('dictionary.itemLabel') })]"
            @update:model-value="(val: unknown) => $emit('update:label', val as string)"
          />
          <q-input
            :model-value="form.value" :label="$t('dictionary.itemValue')" outlined
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('dictionary.itemValue') })]"
            @update:model-value="(val: unknown) => $emit('update:value', val as string)"
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
          <q-input
            :model-value="form.remark" :label="$t('dictionary.remark')" outlined type="textarea"
            @update:model-value="(val: unknown) => $emit('update:remark', val as string)"
          />
          <q-checkbox
            :model-value="form.is_default" :label="$t('dictionary.is_default')"
            @update:model-value="(val: unknown) => $emit('update:is_default', Boolean(val ?? false))"
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


interface DictItemForm {
  label: string;
  value: string;
  sort: number;
  status: number;
  remark: string;
  is_default: boolean;
}

defineProps<{
  modelValue: boolean;
  isEdit: boolean;
  form: DictItemForm;
}>();

defineEmits<{
  'update:modelValue': [value: boolean];
  'update:label': [value: string];
  'update:value': [value: string];
  'update:sort': [value: number];
  'update:status': [value: number];
  'update:remark': [value: string];
  'update:is_default': [value: boolean];
  save: [];
}>();

const { t } = useI18n();
const statusOptions = computed(() => [
  { label: t('dictionary.enabled'), value: 1 },
  { label: t('dictionary.disabled'), value: 0 },
]);
</script>
