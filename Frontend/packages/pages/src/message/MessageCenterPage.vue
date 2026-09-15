<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('message.title') }}</div>

    <!-- 消息标签页 -->
    <q-tabs
      v-model="activeTab"
      class="text-grey-7"
      active-color="primary"
      indicator-color="primary"
      align="left"
    >
      <q-tab name="inbox" :label="$t('message.inbox')" />
      <q-tab name="outbox" :label="$t('message.outbox')" />
      <q-tab name="announcements" :label="$t('message.announcements')" />
    </q-tabs>

    <q-separator />

    <!-- 操作栏 -->
    <div class="row q-my-md items-center">
      <div class="col">
        <q-input
          v-model="searchKeyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="max-width: 300px"
          @update:model-value="loadMessages"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </div>
      <div class="row q-gutter-sm">
        <q-btn
          v-if="activeTab === 'outbox'"
          color="primary"
          :icon="matAdd"
          :label="$t('message.sendMessage')"
          @click="openSendDialog"
        />
        <q-btn
          flat
          :icon="matRefresh"
          :label="$t('common.refresh')"
          @click="loadMessages"
        />
      </div>
    </div>

    <!-- 消息列表 -->
    <q-card flat bordered>
      <q-table
        :rows="messageList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- 选择框 -->
        <template #header-selection="scope">
          <q-checkbox v-model="scope.selected" />
        </template>

        <template #body-selection="scope">
          <q-checkbox v-model="scope.selected" />
        </template>

        <!-- 类型 -->
        <template #body-cell-type="{ row }">
          <q-td>
            <q-icon
              :name="getMessageTypeIcon(row.type)"
              :color="getMessageTypeColor(row.type)"
              size="20px"
            />
            {{ getMessageTypeLabel(row.type) }}
          </q-td>
        </template>

        <!-- 优先级 -->
        <template #body-cell-priority="{ row }">
          <q-td>
            <q-badge
              v-if="row.priority > 0"
              :color="getPriorityColor(row.priority)"
              :label="getPriorityLabel(row.priority)"
            />
            <span v-else class="text-grey-5">-</span>
          </q-td>
        </template>

        <!-- 是否已读 -->
        <template #body-cell-is_read="{ row }">
          <q-td>
            <q-badge
              :color="row.is_read ? 'grey' : 'primary'"
              :label="row.is_read ? $t('message.read') : $t('message.unread')"
            />
          </q-td>
        </template>

        <!-- 星标 -->
        <template #body-cell-is_starred="{ row }">
          <q-td>
            <q-icon
              :name="row.is_starred ? 'star' : 'star_border'"
              :color="row.is_starred ? 'amber' : 'grey'"
              size="20px"
              class="cursor-pointer"
              @click="toggleStar(row)"
            />
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-td>
            <q-btn flat dense :icon="matVisibility" color="primary" @click="viewMessage(row)" />
            <q-btn flat dense :icon="matDelete" color="negative" @click="doDeleteMessage(row)" />
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 发送消息对话框 -->
    <q-dialog v-model="sendDialogVisible" persistent>
      <q-card style="min-width: 500px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('message.sendMessage') }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section>
          <q-select
            v-model="sendForm.type"
            :options="messageTypeOptions"
            :label="$t('message.type')"
            emit-value
            map-options
          />
          <q-select
            v-model="sendForm.targetType"
            :options="targetTypeOptions"
            :label="$t('message.target')"
            emit-value
            map-options
            class="q-mt-sm"
          />
          <q-input
            v-model="sendForm.title"
            :label="$t('message.title')"
            class="q-mt-sm"
          />
          <q-input
            v-model="sendForm.content"
            :label="$t('message.content')"
            type="textarea"
            class="q-mt-sm"
          />
          <q-select
            v-model="sendForm.priority"
            :options="priorityOptions"
            :label="$t('message.priority')"
            emit-value
            map-options
            class="q-mt-sm"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn
            color="primary"
            :label="$t('message.send')"
            :loading="sending"
            @click="handleSend"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 消息详情对话框 -->
    <q-dialog v-model="detailDialogVisible">
      <q-card style="min-width: 600px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ currentMessage?.title }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section>
          <div class="q-mb-sm">
            <q-badge :color="currentMessage?.is_read ? 'grey' : 'primary'">
              {{ currentMessage?.is_read ? $t('message.read') : $t('message.unread') }}
            </q-badge>
            <q-badge class="q-ml-sm" color="blue">
              {{ currentMessage?.sender_name }}
            </q-badge>
            <q-badge class="q-ml-sm" color="grey">
              {{ currentMessage?.created_at }}
            </q-badge>
          </div>
          <div v-html="sanitizeHTML(currentMessage?.content || '')"></div>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useMessageCenter } from '@erp-new-frontend-monorepo/composables/src/useMessageCenter';
import { sanitizeHTML } from '@/utils/sanitize';

const {
  activeTab,
  loading,
  sending,
  searchKeyword,
  messageList,
  sendDialogVisible,
  detailDialogVisible,
  currentMessage,
  sendForm,
  messageTypeOptions,
  targetTypeOptions,
  priorityOptions,
  columns,
  matAdd,
  matRefresh,
  matVisibility,
  matDelete,
  loadMessages,
  onRequest,
  getMessageTypeIcon,
  getMessageTypeColor,
  getMessageTypeLabel,
  getPriorityColor,
  getPriorityLabel,
  openSendDialog,
  handleSend,
  viewMessage,
  toggleStar,
  doDeleteMessage,
  initData,
} = useMessageCenter();

void initData();
</script>
