<template>
  <q-dialog v-model="showDialog" persistent>
    <q-card style="min-width: 450px">
      <q-card-section class="row items-center">
        <div class="text-h6">
          {{ isEdit ? $t('role.editRole') : $t('role.createRole') }}
        </div>
        <q-space />
        <q-btn flat round icon="close" v-close-popup />
      </q-card-section>

      <q-separator />

      <q-card-section>
        <q-form class="q-gutter-md">
          <!-- 角色名称 -->
          <q-input
            v-model="form.name"
            :label="$t('role.role_name')"
            outlined
            :disable="isEdit"
            :rules="[
              (val) => !!val || $t('validation.required', { field: $t('role.role_name') }),
              (val) => /^[a-z_]+$/.test(val) || $t('role.role_name_format_hint')
            ]"
          >
            <template #hint>
              <span class="text-grey-6">{{ $t('role.role_name_hint') }}</span>
            </template>
          </q-input>

          <!-- 角色描述 -->
          <q-input
            v-model="form.description"
            :label="$t('role.description')"
            outlined
            type="textarea"
            rows="3"
          />

          <!-- 角色类型 -->
          <q-select
            v-model="form.type"
            :options="typeOptions"
            :label="$t('role.roleType')"
            outlined
            emit-value
            map-options
            :disable="isEdit"
          />

          <!-- 角色状态 -->
          <q-select
            v-if="isEdit"
            v-model="form.status"
            :options="statusOptions"
            :label="$t('common.status')"
            outlined
            emit-value
            map-options
          />

          <!-- 预定义角色提示 -->
          <q-banner v-if="isEdit && role?.is_predefined" class="bg-info text-white" rounded>
            <template #avatar>
              <q-icon name="info" />
            </template>
            {{ $t('role.predefinedRoleHint') }}
          </q-banner>
        </q-form>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" color="grey" v-close-popup />
        <q-btn
          color="primary"
          :label="$t('common.save')"
          :loading="loading"
          @click="handleSave"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
/**
 * @file RoleEditDialog.vue
 * @description 角色编辑弹窗
 * @date 2026-04-06
 */

import { ref, computed, watch, reactive, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { createAdminRole as createRole, updateAdminRole as updateRole } from '@/api';
import type { RoleDetail } from '@/types/role';
import { RoleType, RoleStatus } from '@/types/role';
import { useDictionaryStore } from '@/stores/system';


const { t } = useI18n();
const $q = useQuasar();
const dictionaryStore = useDictionaryStore();

// Props
interface Props {
  role?: RoleDetail | null;
  modelValue?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  role: null,
  modelValue: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'success': [];
}>();

// 状态
const showDialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const isEdit = computed(() => !!props.role);
const loading = ref(false);

const form = reactive({
  name: '',
  description: '',
  type: RoleType.USER,
  status: RoleStatus.ENABLED,
});

// 类型选项（从数据字典动态获取）
const typeOptions = ref<{ label: string; value: string }[]>([
  { label: t('user.normalUser'), value: RoleType.USER },
  { label: t('user.vip'), value: RoleType.VIP },
  { label: t('user.admin'), value: RoleType.ADMIN },
]);

// 状态选项
const statusOptions = computed(() => [
  { label: t('dictionary.enabled'), value: RoleStatus.ENABLED },
  { label: t('dictionary.disabled'), value: RoleStatus.DISABLED },
]);

/**
 * @brief 加载角色类型选项（从数据字典）
 */
async function loadRoleTypeOptions() {
  try {
    const types = await dictionaryStore.fetchRoleTypes();
    if (types && types.length > 0) {
      typeOptions.value = types;
    }
  } catch (error) {
    console.error('【加载角色类型选项失败】', error);
  }
}

// 生命周期
onMounted(() => {
  void loadRoleTypeOptions();
});

// 监听弹窗打开，初始化表单
watch(showDialog, (value) => {
  if (value) {
    if (props.role) {
      form.name = props.role.name;
      form.description = props.role.description || '';
      form.type = props.role.type || RoleType.USER;
      form.status = props.role.status || RoleStatus.ENABLED;
    } else {
      form.name = '';
      form.description = '';
      form.type = RoleType.USER;
      form.status = RoleStatus.ENABLED;
    }
  }
});

// 保存
async function handleSave() {
  loading.value = true;
  try {
    if (isEdit.value) {
      await updateRole(form.name, {
        description: form.description,
        status: form.status,
      });
    } else {
      await createRole({
        name: form.name,
        description: form.description,
        type: form.type,
      });
    }
    $q.notify({
      type: 'positive',
      message: t('common.success'),
    });
    emit('success');
    showDialog.value = false;
  } catch (error) {
    console.error('【保存角色失败】', error);
    $q.notify({
      type: 'negative',
      message: t('common.error'),
    });
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.body--dark .q-card {
  background: #1e1e1e;
}
</style>
