<template>
  <q-dialog v-model="showDialog">
    <q-card style="min-width: 400px">
      <q-card-section>
        <div class="text-h6">{{ $t('table.selectColumns') }}</div>
      </q-card-section>

      <q-separator />

      <q-card-section>
        <q-list dense>
          <q-item v-for="col in columns" :key="col.name" clickable @click="toggleColumn(col.name)">
            <q-item-section avatar>
              <q-checkbox
                :model-value="selectedColumns.includes(col.name)"
                @update:model-value="toggleColumn(col.name)"
              />
            </q-item-section>
            <q-item-section>{{ col.label }}</q-item-section>
          </q-item>
        </q-list>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat color="warning" :label="$t('common.reset')" @click="resetToDefault" />
        <q-space />
        <q-btn flat :label="$t('common.cancel')" v-close-popup />
        <q-btn color="primary" :label="$t('common.confirm')" @click="confirmSelection" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t: $t } = useI18n();

interface Column {
  name: string;
  label: string;
}

interface Props {
  columns: Column[];
  modelValue: string[];
  defaultColumns?: string[];
}

const props = withDefaults(defineProps<Props>(), {
  defaultColumns: () => [],
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void;
}>();

const showDialog = ref(false);
const selectedColumns = ref<string[]>([...props.modelValue]);

watch(
  () => props.modelValue,
  (newVal) => {
    selectedColumns.value = [...newVal];
  }
);

/**
 * @brief 切换列选择
 */
function toggleColumn(name: string) {
  const index = selectedColumns.value.indexOf(name);
  if (index === -1) {
    selectedColumns.value.push(name);
  } else if (selectedColumns.value.length > 1) {
    // 至少保留一列
    selectedColumns.value.splice(index, 1);
  }
}

/**
 * @brief 重置为默认列
 */
function resetToDefault() {
  if (props.defaultColumns.length > 0) {
    selectedColumns.value = [...props.defaultColumns];
  } else {
    // 如果没有指定默认列，则选择所有列
    selectedColumns.value = props.columns.map((col) => col.name);
  }
}

/**
 * @brief 确认选择
 */
function confirmSelection() {
  emit('update:modelValue', [...selectedColumns.value]);
  showDialog.value = false;
}

/**
 * @brief 打开弹窗
 */
function open() {
  showDialog.value = true;
}

defineExpose({ open });
</script>
