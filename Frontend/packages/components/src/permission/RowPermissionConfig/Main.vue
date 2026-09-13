
<template>
  <div class="row-permission-config">
    <div class="text-h6 q-mb-md">{{ $t('permission.rowLevel') }}</div>

    <!-- 数据源选择 -->
    <div class="q-mb-md">
      <q-select
        v-model="currentSource"
        :options="sourceOptions"
        emit-value
        map-options
        dense
        outlined
        :label="$t('permission.dataSource')"
        class="full-width"
        @update:model-value="onSourceChange"
      />
    </div>

    <!-- 规则列表 -->
    <div v-if="currentRules.length > 0" class="rules-list">
      <div v-for="(ruleItem, index) in currentRules" :key="index" class="rule-item">
        <div class="rule-header">
          <span class="text-subtitle2">{{ $t('permission.condition') }} {{ index + 1 }}</span>
          <div class="row q-gutter-sm">
            <q-btn
              flat
              dense
              icon="edit"
              color="primary"
              @click="editRule(index)"
            />
            <q-btn
              v-if="currentRules.length > 1"
              flat
              dense
              icon="delete"
              color="negative"
              @click="removeRule(index)"
            />
          </div>
        </div>

        <!-- 规则条件 -->
        <div class="rule-condition">
          <div class="row q-col-gutter-sm">
            <div class="col-4">
              <q-select
                v-model="ruleItem.conditionField"
                :options="conditionOptions"
                emit-value
                map-options
                dense
                outlined
                :label="$t('permission.field')"
              />
            </div>
            <div class="col-4">
              <q-select
                v-model="ruleItem.conditionOperator"
                :options="operatorOptions"
                emit-value
                map-options
                dense
                outlined
                :label="$t('permission.operator')"
              />
            </div>
            <div class="col-4">
              <q-input
                v-model="ruleItem.conditionValue"
                dense
                outlined
                :label="$t('permission.value')"
              />
            </div>
          </div>
        </div>
      </div>

      <q-btn
        flat
        dense
        icon="add"
        :label="$t('permission.addCondition')"
        color="primary"
        class="q-mt-sm"
        @click="addRule"
      />
    </div>

    <!-- 权限配置 -->
    <div v-if="currentRules.length > 0" class="permission-config q-mt-md">
      <div class="text-subtitle2 q-mb-sm">{{ $t('permission.config') }}</div>
      <div class="row q-col-gutter-sm">
        <div class="col-6">
          <q-select
            v-model="currentPermissions.read"
            :options="userOptions"
            multiple
            use-chips
            dense
            outlined
            :label="$t('permission.read')"
          />
        </div>
        <div class="col-6">
          <q-select
            v-model="currentPermissions.write"
            :options="userOptions"
            multiple
            use-chips
            dense
            outlined
            :label="$t('permission.write')"
          />
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="row q-mt-md q-gutter-sm">
      <q-btn
        color="primary"
        :label="$t('permission.save')"
        @click="saveConfig"
      />
      <q-btn
        flat
        color="grey"
        :label="$t('permission.reset')"
        @click="resetConfig"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRowPermission } from './useRowPermission'

const { t: $t } = useI18n()

interface Rule {
  conditionField: string
  conditionOperator: string
  conditionValue: string
}

interface Permissions {
  read: string[]
  write: string[]
}

interface Props {
  dataSources?: Array<{ label: string; value: string }>;
  sourceOptions: Array<{ label: string; value: string }>;
  conditionOptions: Array<{ label: string; value: string }>;
  operatorOptions: Array<{ label: string; value: string }>;
  userOptions: Array<{ label: string; value: string }>;
  initialConfig?: Record<string, unknown>;
}

const props = withDefaults(defineProps<Props>(), {
  sourceOptions: () => [],
  conditionOptions: () => [],
  operatorOptions: () => [],
  userOptions: () => [],
  initialConfig: () => ({})
})

const emit = defineEmits<{
  (e: 'save', config: Record<string, unknown>): void
  (e: 'reset'): void
}>()

const {
  currentSource,
  currentRules,
  currentPermissions,
  addRule,
  removeRule,
  editRule,
  onSourceChange,
  saveConfig,
  resetConfig
} = useRowPermission({
  dataSources: props.dataSources as Array<{ path?: string; title?: string; icon?: string; children?: Array<unknown>; permissions?: string[] }>,
  conditionOptions: props.conditionOptions,
  operatorOptions: props.operatorOptions,
  userOptions: props.userOptions,
  initialConfig: props.initialConfig
})

watch([() => currentPermissions.value.read, () => currentPermissions.value.write], () => {
  saveConfig()
}, { deep: true })
</script>

<style scoped>
.row-permission-config {
  max-width: 800px;
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rule-item {
  padding: 16px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  background: white;
}

.rule-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.rule-condition {
  margin-top: 8px;
}

.permission-config {
  padding: 16px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  background: #fafafa;
}

.body--dark .rule-item {
  background: #1e1e1e;
  border-color: #3d3d3d;
}

.body--dark .permission-config {
  background: #1e1e1e;
  border-color: #3d3d3d;
}
</style>
