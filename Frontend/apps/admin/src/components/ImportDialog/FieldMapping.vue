/**
 * @file FieldMapping.vue
 * @description 字段映射配置 - Excel列到目标字段的映射
 * @date 2026-04-04
 */

<template>
  <div class="q-pa-md">
    <div class="text-subtitle1 q-mb-md">{{ $t('importMod.mappingTitle') }}</div>
    <q-table
      :rows="mappings"
      :columns="columns"
      row-key="excelColumn"
      flat
      bordered
    >
      <template v-slot:body-cell-required="props">
        <q-td :props="props">
          <q-toggle v-model="props.row.required" dense />
        </q-td>
      </template>
      <template v-slot:body-cell-targetField="props">
        <q-td :props="props">
          <q-select
            v-model="props.row.targetField"
            :options="targetFields"
            dense
            outlined
            :disable="!props.row.excelColumn"
            style="min-width: 150px"
          />
        </q-td>
      </template>
      <template v-slot:body-cell-dataType="props">
        <q-td :props="props">
          <q-select
            v-model="props.row.dataType"
            :options="dataTypeOptions"
            dense
            outlined
            style="min-width: 100px"
          />
        </q-td>
      </template>
    </q-table>
    <q-btn
      color="primary"
      :label="$t('importMod.applyMapping')"
      class="q-mt-md"
      @click="$emit('apply')"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

interface MappingRow {
  excelColumn: string;
  targetField: string;
  required: boolean;
  dataType: string;
}

interface Props {
  mappings: MappingRow[];
  targetFields: string[];
  dataTypeOptions: string[];
}

defineProps<Props>();

const _emit = defineEmits<{
  (e: 'apply'): void;
}>();

const { t } = useI18n();

const columns = [
  { name: 'excelColumn', label: t('importMod.excelColumn'), field: 'excelColumn', align: 'left' as const },
  { name: 'targetField', label: t('importMod.targetField'), field: 'targetField', align: 'left' as const },
  { name: 'required', label: t('importMod.required'), field: 'required', align: 'center' as const },
  { name: 'dataType', label: t('importMod.dataType'), field: 'dataType', align: 'center' as const },
];
</script>
