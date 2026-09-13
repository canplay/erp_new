<template>
  <q-page class="q-pa-md">
    <!-- 加载状态 -->
    <div v-if="loading" class="row justify-center items-center q-pa-xl">
      <q-spinner-dots size="50px" color="primary" />
    </div>

    <!-- 用户详情 -->
    <div v-else-if="user">
      <!-- 页面标题和操作按钮 -->
      <div class="row items-center q-mb-md">
        <q-btn flat color="grey" icon="arrow_back" :label="$t('common.back')" @click="goBack" />
        <q-space />
        <q-btn color="primary" icon="edit" :label="$t('common.edit')" @click="showEditDialog = true" />
      </div>

      <!-- 基本信息卡片 -->
      <q-card class="q-mb-md" bordered>
        <q-card-section>
          <div class="text-h6">{{ $t('user.basicInfo') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="row q-col-gutter-md">
            <!-- 头像 -->
            <div class="col-12 col-md-3 flex flex-center">
              <q-avatar size="120px" color="primary" text-color="white" class="q-mb-md">
                <img v-if="user.avatar" :src="user.avatar" />
                <span v-else class="text-h4">{{ user.username?.substring(0, 1).toUpperCase() }}</span>
              </q-avatar>
              <div class="text-center">
                <q-chip
                  :color="getStatusColor(user.status)"
                  text-color="white"
                  dense
                  :icon="getStatusIcon(user.status)"
                >
                  {{ getStatusLabel(user.status) }}
                </q-chip>
              </div>
            </div>

            <!-- 基本信息 -->
            <div class="col-12 col-md-9">
              <q-list dense>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.username') }}</q-item-label>
                    <q-item-label class="text-body1">{{ user.username }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.nickname') }}</q-item-label>
                    <q-item-label class="text-body1">{{ user.nickname || '-' }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.email') }}</q-item-label>
                    <q-item-label class="text-body1">{{ user.email || '-' }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.phone') }}</q-item-label>
                    <q-item-label class="text-body1">{{ user.phone || '-' }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.role') }}</q-item-label>
                    <q-chip :color="getRoleChipColor(user.role)" text-color="white" dense size="sm">
                      {{ getRoleLabel(user.role) }}
                    </q-chip>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.gender') }}</q-item-label>
                    <q-item-label class="text-body1">{{ getGenderLabel(user.gender) }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.created_at') }}</q-item-label>
                    <q-item-label class="text-body1">{{ formatDateTime(user.created_at) }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>
                    <q-item-label caption>{{ $t('user.last_login_at') }}</q-item-label>
                    <q-item-label class="text-body1">{{ formatDateTime(user.last_login_at) }}</q-item-label>
                  </q-item-section>
                </q-item>
              </q-list>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- 操作卡片 -->
      <q-card class="q-mb-md" bordered>
        <q-card-section>
          <div class="text-h6">{{ $t('user.actions') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="q-gutter-sm">
            <q-btn
              v-if="user.status === 1"
              color="warning"
              icon="block"
              :label="$t('user.disable')"
              @click="toggleUserStatus"
            />
            <q-btn
              v-else
              color="positive"
              icon="check_circle"
              :label="$t('user.enable')"
              @click="toggleUserStatus"
            />
            <q-btn
              color="secondary"
              icon="lock_reset"
              :label="$t('user.resetPassword')"
              @click="showResetPasswordDialog = true"
            />
            <q-btn
              color="negative"
              icon="delete"
              :label="$t('user.delete')"
              @click="deleteDialogRef?.open()"
            />
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 未找到用户 -->
    <div v-else class="text-center q-pa-xl">
      <q-icon name="person_off" size="64px" color="grey" />
      <div class="text-h6 text-grey q-mt-md">{{ $t('user.notFound') }}</div>
    </div>

    <!-- 编辑对话框 -->
    <q-dialog v-model="showEditDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('user.editUser') }}</div>
        </q-card-section>

        <q-card-section>
          <q-input
            v-model="editForm.username"
            :label="$t('user.username')"
            outlined
            dense
            disable
            class="q-mb-md"
          />
          <q-input
            v-model="editForm.nickname"
            :label="$t('user.nickname')"
            outlined
            dense
            class="q-mb-md"
          />
          <q-input
            v-model="editForm.email"
            :label="$t('user.email')"
            outlined
            dense
            class="q-mb-md"
          />
          <q-input
            v-model="editForm.phone"
            :label="$t('user.phone')"
            outlined
            dense
            class="q-mb-md"
          />
          <q-select
            v-model="editForm.role"
            :options="roleOptions"
            emit-value
            map-options
            :label="$t('user.role')"
            outlined
            dense
            class="q-mb-md"
          />
          <q-select
            v-model="editForm.status"
            :options="statusOptions"
            emit-value
            map-options
            :label="$t('user.status')"
            outlined
            dense
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn flat color="primary" :label="$t('common.save')" @click="saveUser" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 重置密码对话框 -->
    <q-dialog v-model="showResetPasswordDialog" persistent>
      <q-card style="min-width: 350px">
        <q-card-section>
          <div class="text-h6">{{ $t('user.resetPassword') }}</div>
        </q-card-section>

        <q-card-section>
          <q-input
            v-model="new_password"
            :label="$t('user.new_password')"
            outlined
            dense
            type="password"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn flat color="primary" :label="$t('common.confirm')" @click="resetPassword" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 确认删除对话框 -->
    <ConfirmDialog ref="deleteDialogRef" :title="$t('user.confirmDelete')" :message="$t('user.confirmDeleteDesc')" @confirm="doDeleteUser" />
  </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useUserDetail } from '@erp-new-frontend-monorepo/composables/src/useUserDetail';;
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';

const deleteDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);

const {
  loading,
  user,
  showEditDialog,
  showResetPasswordDialog,
  editForm,
  new_password,
  roleOptions,
  statusOptions,
  getStatusLabel,
  getStatusColor,
  getStatusIcon,
  getRoleLabel,
  getRoleChipColor,
  getGenderLabel,
  formatDateTime,
  goBack,
  saveUser,
  toggleUserStatus,
  resetPassword,
  doDeleteUser,
} = useUserDetail();
</script>
