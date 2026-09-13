<template>
  <q-dialog v-model="showDialog" persistent>
    <q-card style="min-width: 600px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('role.roleUsers') }} - {{ role_name }}</div>
        <q-space />
        <q-btn flat round icon="close" v-close-popup />
      </q-card-section>

      <q-separator />

      <!-- 用户列表 -->
      <q-card-section>
        <q-list v-if="users.length > 0" separator>
          <q-item v-for="user in users" :key="user.id">
            <q-item-section avatar>
              <q-avatar size="36px" color="primary" text-color="white">
                {{ user.username?.substring(0, 1).toUpperCase() || 'U' }}
              </q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label>{{ user.username }}</q-item-label>
              <q-item-label caption>
                {{ user.email || '-' }}
                <span v-if="user.department"> · {{ user.department }}</span>
                <span v-if="user.position"> · {{ user.position }}</span>
              </q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn
                flat
                dense
                color="primary"
                :label="$t('common.view')"
                @click="viewUser(user.id)"
              />
            </q-item-section>
          </q-item>
        </q-list>

        <!-- 空状态 -->
        <div v-else class="text-center text-grey q-pa-md">
          <q-icon name="people_outline" size="48px" class="q-mb-sm" />
          <div>{{ $t('role.noUsersInRole') }}</div>
        </div>
      </q-card-section>

      <!-- 分页 -->
      <q-card-section v-if="total > page_size" class="q-pt-none">
        <q-pagination
          v-model="currentPage"
          :max="Math.ceil(total / page_size)"
          :max-pages="6"
          boundary-numbers
          @update:model-value="loadUsers"
        />
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.close')" color="grey" v-close-popup />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
/**
 * @file RoleUsersDialog.vue
 * @description 查看角色用户弹窗
 * @date 2026-04-06
 */

import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { getAdminRoleUsers as getRoleUsers } from '@/api';
import type { RoleUserSummary } from '@/types/role';

const { t: $t } = useI18n();

// Props
interface Props {
  role_name?: string;
  modelValue?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  role_name: '',
  modelValue: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'view-user': [user_id: number];
}>();

// 状态
const showDialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const users = ref<RoleUserSummary[]>([]);
const total = ref(0);
const currentPage = ref(1);
const page_size = 10;

// 加载用户
async function loadUsers() {
  if (!props.role_name) return;

  try {
    const response = await getRoleUsers(props.role_name);
    const data = response.data?.data as { records?: Array<{ id: number; username: string }>; total?: number } | undefined;
    // 分页响应中的记录数组
    if (data?.records) {
      users.value = data.records;
      total.value = data.total || data.records.length || 0;
    }
  } catch (error) {
    console.error('【加载角色用户失败】', error);
    users.value = [];
  }
}

// 监听弹窗打开
watch(showDialog, (value) => {
  if (value) {
    currentPage.value = 1;
    void loadUsers();
  }
});

// 查看用户
function viewUser(user_id: number) {
  emit('view-user', user_id);
}
</script>

<style scoped>
.body--dark .q-card {
  background: #1e1e1e;
}
</style>
