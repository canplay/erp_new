// @ts-nocheck
<template>
  <div class="field-mask-config">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle q-mb-sm">
          <q-icon name="mask" class="q-mr-sm" color="primary" />
          {{ t('permission.fieldMask') || '字段脱敏规则' }}
        </div>
        <div class="text-caption text-grey q-mb-md">
          配置敏感字段的脱敏规则，保护用户隐私数据
        </div>

        <!-- 脱敏说明 -->
        <q-banner class="q-mb-md" rounded banner-class="bg-orange-1">
          <template v-slot:avatar>
            <q-icon name="security" color="orange" />
          </template>
          <div class="text-body2">
            <strong>脱敏规则说明：</strong>
            <ul class="q-my-sm" style="padding-left: 20px">
              <li><strong>手机号</strong> - 显示前3位和后4位，如 138****5678</li>
              <li><strong>身份证</strong> - 显示前6位和后4位，如 110101***********1234</li>
              <li><strong>邮箱</strong> - 中间部分脱敏，如 a***@example.com</li>
              <li><strong>银行卡</strong> - 只显示后4位，如 **** **** **** 1234</li>
              <li><strong>姓名</strong> - 隐藏中间字符，如 张*三</li>
            </ul>
          </div>
        </q-banner>

        <FieldMaskSelectors
          v-model:selected-entity-type="selectedEntityType"
          v-model:selected-role="selectedRole"
          v-model:search-query="searchQuery"
          :entity-type-options="entityTypeOptions"
          :role-options="roleOptions"
        />

        <FieldMaskTable
          :field-masks="(filteredFieldMasks as any)"
          :columns="columns"
          :mask-strategy-options="maskStrategyOptions"
          @strategy-change="(rule: any) => handleStrategyChange(rule)"
          @rule-change="(rule: any) => handleRuleChange(rule)"
          @preview="(rule: any) => showPreview(rule as any)"
        />

        <!-- 批量操作 -->
        <div class="row q-mt-md items-center">
          <q-select
            v-model="batchStrategy"
            :options="maskStrategyOptions"
            outlined
            dense
            label="批量设置脱敏策略"
            emit-value
            map-options
            style="width: 200px"
          />
          <q-btn
            color="primary"
            label="应用到所选"
            class="q-ml-sm"
            :disable="selectedFields.length === 0"
            @click="applyBatchStrategy"
          />
          <q-space />
          <q-btn
            color="positive"
            label="保存所有修改"
            icon="save"
            @click="saveAllRules"
          />
        </div>
      </q-card-section>
    </q-card>

    <FieldMaskPreviewDialog
      v-model="showPreviewDialog"
      :preview-rule="(previewRule as any)" :strategy="(maskStrategyOptions[0] as any)"
      :masked-value="maskedPreview"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import type { QTableProps } from 'quasar';
import type { FieldMaskRule } from '@/types/fieldMask';
import { MASK_STRATEGY_OPTIONS } from '@/types/fieldMask';
import { useFieldMask } from '@/composables/useFieldMask';
import FieldMaskSelectors from '../FieldMaskSelectors.vue';
import FieldMaskTable from '../FieldMaskTable.vue';
import FieldMaskPreviewDialog from '../FieldMaskPreviewDialog.vue';
import type { LocalFieldMaskItem } from '../FieldMaskTable.vue';

const { t } = useI18n();

const props = defineProps<{
  initialRules?: FieldMaskRule[];
  role_name?: string;
}>();

const emit = defineEmits<{
  saved: [rules: FieldMaskRule[]];
}>();

const {
  selectedEntityType,
  selectedRole,
  searchQuery,
  fieldMasks,
  batchStrategy,
  selectedFields,
  previewRule,
  maskedPreview,
  showPreviewDialog,
  entityTypeOptions,
  roleOptions,
  filteredFieldMasks,
  initFieldMasks,
  handleStrategyChange,
  handleRuleChange,
  showPreview,
  applyBatchStrategy,
  saveAllRules,
  setOnSaved,
} = useFieldMask();

const columns: QTableProps['columns'] = [
  { name: 'field', label: '字段', field: 'field', align: 'left', sortable: true },
  { name: 'strategy', label: '脱敏策略', field: 'strategy', align: 'center' },
  { name: 'customConfig', label: '自定义配置', field: 'customConfig', align: 'center' },
  { name: 'sensitive', label: '敏感度', field: 'sensitive', align: 'center' },
  { name: 'enabled', label: '启用', field: 'enabled', align: 'center' },
  { name: 'preview', label: '预览', field: 'preview', align: 'center' },
];

const maskStrategyOptions = MASK_STRATEGY_OPTIONS;

setOnSaved((rules) => {
  emit('saved', rules);
});

onMounted(() => {
  initFieldMasks(props.initialRules);
});
</script>

<style scoped>
.field-mask-config {
  width: 100%;
}

.mask-table {
  border-radius: 8px;
}

.body--dark .mask-table {
  background: #1e1e1e;
}
</style>
