<template>
  <div class="permission-validity-config">
    <!-- 永久有效开关 -->
    <q-toggle
      v-model="validity.is_permanent"
      :label="$t('permission.permanent')"
      color="primary"
    />

    <q-separator class="q-my-md" />

    <!-- 有效期配置 -->
    <div v-if="!validity.is_permanent" class="row q-col-gutter-md">
      <!-- 生效时间 -->
      <div class="col-12 col-sm-6">
        <q-input
          v-model="validity.valid_from"
          :label="$t('permission.valid_from')"
          outlined
          dense
          type="datetime-local"
          :rules="[
            (val) => !!val || $t('validation.required', { field: $t('permission.valid_from') })
          ]"
        >
          <template #prepend>
            <q-icon name="event" class="cursor-pointer">
              <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                <q-date v-model="validity.valid_from" mask="YYYY-MM-DD HH:mm">
                  <div class="row items-center justify-end">
                    <q-btn v-close-popup label="Close" color="primary" flat />
                  </div>
                </q-date>
              </q-popup-proxy>
            </q-icon>
          </template>
        </q-input>
      </div>

      <!-- 失效时间 -->
      <div class="col-12 col-sm-6">
        <q-input
          v-model="validity.valid_until"
          :label="$t('permission.valid_until')"
          outlined
          dense
          type="datetime-local"
          :rules="[
            (val) => !!val || $t('validation.required', { field: $t('permission.valid_until') }),
            (val) => !validity.valid_from || val > validity.valid_from || $t('permission.validUntilHint')
          ]"
        >
          <template #prepend>
            <q-icon name="event" class="cursor-pointer">
              <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                <q-date v-model="validity.valid_until" mask="YYYY-MM-DD HH:mm">
                  <div class="row items-center justify-end">
                    <q-btn v-close-popup label="Close" color="primary" flat />
                  </div>
                </q-date>
              </q-popup-proxy>
            </q-icon>
          </template>
        </q-input>
      </div>

      <!-- 提前提醒 -->
      <div class="col-12">
        <q-select
          v-model="validity.reminder_days"
          :options="reminderOptions"
          :label="$t('permission.reminder_days')"
          outlined
          dense
          emit-value
          map-options
          clearable
        >
          <template #hint>
            <span class="text-grey-6">{{ $t('permission.reminderHint') }}</span>
          </template>
        </q-select>
      </div>
    </div>

    <!-- 剩余有效期提示 -->
    <q-banner
      v-if="!validity.is_permanent && remainingDays !== null"
      :class="remainingDays < 7 ? 'bg-warning' : 'bg-info'"
      class="text-white q-mt-md"
      rounded
    >
      <template #avatar>
        <q-icon name="schedule" />
      </template>
      {{ $t('permission.remainingDays', { days: remainingDays }) }}
    </q-banner>
  </div>
</template>

<script setup lang="ts">
/**
 * @file PermissionValidityConfig.vue
 * @description 权限有效期配置组件
 * @date 2026-04-06
 */

import { computed, reactive, watch } from 'vue';
import type { PermissionValidity } from '@/types/permission';

interface Props {
  modelValue?: PermissionValidity;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => ({
    is_permanent: true,
  }),
});

const emit = defineEmits<{
  'update:modelValue': [value: PermissionValidity];
}>();

// 内部状态
const validity = reactive<PermissionValidity>({
  is_permanent: props.modelValue?.is_permanent ?? true,
  ...(props.modelValue?.valid_from !== undefined ? { valid_from: props.modelValue.valid_from } : {}),
  ...(props.modelValue?.valid_until !== undefined ? { valid_until: props.modelValue.valid_until } : {}),
  ...(props.modelValue?.reminder_days !== undefined ? { reminder_days: props.modelValue.reminder_days } : {}),
});

// 监听变化，同步到父组件
watch(
  validity,
  (newVal) => {
    emit('update:modelValue', { ...newVal });
  },
  { deep: true }
);

// 监听外部变化
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      Object.assign(validity, newVal);
    }
  },
  { deep: true }
);

// 提醒天数选项
const reminderOptions = [
  { label: '1 天', value: 1 },
  { label: '3 天', value: 3 },
  { label: '7 天', value: 7 },
  { label: '14 天', value: 14 },
  { label: '30 天', value: 30 },
];

// 计算剩余天数
const remainingDays = computed(() => {
  if (validity.is_permanent || !validity.valid_until) return null;

  const now = new Date();
  const until = new Date(validity.valid_until);
  const diff = until.getTime() - now.getTime();

  if (diff <= 0) return 0;

  return Math.ceil(diff / (1000 * 60 * 60 * 24));
});
</script>

<style scoped>
.permission-validity-config {
  padding: 16px;
}
</style>
