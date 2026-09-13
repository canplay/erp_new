<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('notification.title') }}</div>

    <!-- 批量操作栏 -->
    <BatchActions
      v-model:selected="selectedNotifications"
      :show-enable="false"
      :show-disable="false"
      :show-delete="true"
      :show-export="true"
      @batch-delete="handleBatchDelete"
      @export="handleBatchExport"
    />

    <!-- 高级搜索（合并版） -->
    <AdvancedSearch
      v-model="filters"
      :field-options="[]"
      :show-type="true"
      :show-status="true"
      :show-date-range="true"
      :show-date-shortcuts="true"
      :keyword-placeholder="$t('notification.searchPlaceholder')"
      :type-label="$t('notification.typeLabel')"
      :status-label="$t('notification.status')"
      :date-range-label="$t('common.dateRange')"
      :type-options="typeOptionsForSearch"
      keyword-class="col-12 col-sm-6"
      @search="handleSearch"
      @reset="handleReset"
    />

    <!-- 工具栏（简化版） -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-end">
        <q-btn
          flat
          color="primary"
          :label="$t('notification.markAllRead')"
          icon="done_all"
          :disable="notificationStore.isAllRead"
          @click="handleMarkAllRead"
        />
      </q-card-section>
    </q-card>

    <!-- 通知列表 -->
    <q-card bordered>
      <q-table
        v-model:selected="selectedNotifications"
        :rows="notificationStore.notifications"
        :columns="columns"
        row-key="id"
        :loading="notificationStore.isLoading"
        :pagination="pagination"
        selection="multiple"
        @request="onTableRequest"
      >
        <!-- 类型列 -->
        <template v-slot:body-cell-type="props">
          <q-td :props="props">
            <q-chip
              :color="getTypeColor(props.row.type)"
              text-color="white"
              dense
              icon-size="12px"
            >
              {{ getTypeLabel(props.row.type) }}
            </q-chip>
          </q-td>
        </template>

        <!-- 优先级列 -->
        <template v-slot:body-cell-priority="props">
          <q-td :props="props">
            <q-badge
              :color="getPriorityColor(props.row.priority)"
              :label="$t(`notification.priority.${props.row.priority}`)"
            />
          </q-td>
        </template>

        <!-- 标题列（未读加粗） -->
        <template v-slot:body-cell-title="props">
          <q-td :props="props">
            <div class="row items-center no-wrap">
              <q-icon
                v-if="!props.row.is_read"
                name="fiber_manual_record"
                color="primary"
                size="8px"
                class="q-mr-sm"
              />
              <span :class="{ 'text-weight-bold': !props.row.is_read }">
                {{ props.row.title }}
              </span>
            </div>
          </q-td>
        </template>

        <!-- 状态列 -->
        <template v-slot:body-cell-is_read="props">
          <q-td :props="props">
            <q-badge
              :color="props.row.is_read ? 'grey' : 'primary'"
              :label="props.row.is_read ? $t('notification.read') : $t('notification.unread')"
            />
          </q-td>
        </template>

        <!-- 时间列 -->
        <template v-slot:body-cell-created_at="props">
          <q-td :props="props">
            {{ formatTime(props.row.created_at) }}
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn
              v-if="!props.row.is_read"
              flat
              dense
              color="positive"
              :label="$t('notification.markRead')"
              @click="handleMarkRead(props.row)"
            />
            <q-btn
              flat
              dense
              color="primary"
              :label="$t('common.view')"
              @click="viewNotification(props.row)"
            />
            <q-btn
              flat
              dense
              color="negative"
              :label="$t('common.delete')"
              @click="handleDelete(props.row)"
            />
          </q-td>
        </template>

        <!-- 加载状态 -->
        <template v-slot:loading>
          <q-spinner-dots />
        </template>

        <!-- 空状态 -->
        <template v-slot:no-data>
          <div class="text-center q-pa-xl text-grey-6">
            <q-icon name="notifications_none" size="64px" />
            <div class="q-mt-md">{{ $t('notification.empty') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 通知详情弹窗 -->
    <q-dialog v-model="showDetailDialog">
      <q-card style="min-width: 600px; max-width: 700px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('notification.detail') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showDetailDialog = false" />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="currentNotification">
          <!-- 类型和优先级 -->
          <div class="row q-col-gutter-md q-mb-md">
            <div class="col-auto">
              <q-chip
                :color="getTypeColor(currentNotification.type)"
                text-color="white"
                dense
              >
                {{ getTypeLabel(currentNotification.type) }}
              </q-chip>
            </div>
            <div class="col-auto">
              <q-badge
                :color="getPriorityColor(currentNotification.priority)"
                :label="$t(`notification.priority.${currentNotification.priority}`)"
              />
            </div>
          </div>

          <!-- 标题 -->
          <div class="text-h6 q-mb-md">{{ currentNotification.title }}</div>

          <!-- 内容 -->
          <div class="q-mb-md" style="white-space: pre-wrap">{{ currentNotification.content }}</div>

          <!-- 元信息 -->
          <div class="row q-col-gutter-sm text-caption text-grey-7">
            <div class="col-6">
              <q-icon name="schedule" class="q-mr-xs" />
              {{ $t('notification.created_at') }}：{{ formatTime(currentNotification.created_at) }}
            </div>
            <div class="col-6" v-if="currentNotification.read_at">
              <q-icon name="done_all" class="q-mr-xs" />
              {{ $t('notification.read_at') }}：{{ formatTime(currentNotification.read_at) }}
            </div>
          </div>

          <!-- 操作按钮 -->
          <q-card-actions align="right" class="q-mt-md">
            <q-btn
              v-if="!currentNotification.is_read"
              flat
              color="positive"
              :label="$t('notification.markRead')"
              @click="handleMarkRead(currentNotification)"
            />
            <q-btn flat color="primary" :label="$t('common.close')" @click="showDetailDialog = false" />
          </q-card-actions>
        </q-card-section>
      </q-card>
    </q-dialog>

    <!-- 删除确认 -->
    <ConfirmDialog
      ref="deleteDialogRef"
      :title="$t('common.confirmDelete')"
      :message="$t('notification.confirmDeleteMessage')"
      :confirm-text="$t('common.delete')"
      confirm-color="negative"
      @confirm="doDeleteNotification"
    />
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNotificationList } from '@erp-new-frontend-monorepo/composables/src/useNotificationList';;
import BatchActions from '@erp-new-frontend-monorepo/components/src/BatchActions.vue';
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import AdvancedSearch from '@erp-new-frontend-monorepo/components/src/AdvancedSearch/Main.vue';

useI18n();

const {
  selectedNotifications,
  filters,
  showDetailDialog,
  currentNotification,
  deleteDialogRef,
  pagination,
  typeOptionsForSearch,
  columns,
  notificationStore,
  onTableRequest,
  handleSearch,
  handleReset,
  getTypeLabel,
  getTypeColor,
  getPriorityColor,
  formatTime,
  viewNotification,
  handleMarkRead,
  handleMarkAllRead,
  handleDelete,
  doDeleteNotification,
  handleBatchDelete,
  handleBatchExport,
  initData,
} = useNotificationList();

onMounted(() => {
  void initData();
});
</script>
