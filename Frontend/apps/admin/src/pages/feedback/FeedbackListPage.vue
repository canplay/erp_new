<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('feedback.title') }}</div>

    <!-- 筛选栏 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section class="row q-gutter-md">
        <q-select
          v-model="filters.type"
          :options="typeOptions"
          :label="$t('feedback.type')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadFeedback"
        />
        <q-select
          v-model="filters.status"
          :options="statusOptions"
          :label="$t('feedback.status')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadFeedback"
        />
        <q-input
          v-model="filters.keyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="min-width: 200px"
          @update:model-value="loadFeedback"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
        <q-space />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="resetFilters" />
      </q-card-section>
    </q-card>

    <!-- 反馈列表 -->
    <q-card flat bordered>
      <q-table
        :rows="feedbackList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- 类型 -->
        <template #body-cell-type="{ row }">
          <q-td>
            <q-badge :color="getTypeColor(row.type)" :label="getTypeLabel(row.type)" />
          </q-td>
        </template>

        <!-- 状态 -->
        <template #body-cell-status="{ row }">
          <q-td>
            <q-badge :color="getStatusColor(row.status)" :label="getStatusLabel(row.status)" />
          </q-td>
        </template>

        <!-- 评分 -->
        <template #body-cell-rating="{ row }">
          <q-td>
            <q-rating
              v-if="row.rating"
              :model-value="row.rating"
              size="1em"
              color="warning"
              readonly
            />
            <span v-else class="text-grey-5">-</span>
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matVisibility" @click="viewDetail(row)">
            <q-tooltip>{{ $t('common.view') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.status === 'pending' || row.status === 'processing'"
            flat
            dense
            round
            :icon="matCheck"
            color="positive"
            @click="handleFeedback(row, 'resolved')"
          >
            <q-tooltip>{{ $t('feedback.resolve') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.status === 'pending'"
            flat
            dense
            round
            :icon="matPlayArrow"
            color="info"
            @click="handleFeedback(row, 'processing')"
          >
            <q-tooltip>{{ $t('feedback.startProcess') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.status === 'pending' || row.status === 'processing'"
            flat
            dense
            round
            :icon="matClose"
            color="negative"
            @click="handleFeedback(row, 'rejected')"
          >
            <q-tooltip>{{ $t('feedback.reject') }}</q-tooltip>
          </q-btn>
        </template>
      </q-table>
    </q-card>

    <!-- 详情弹窗 -->
    <q-dialog v-model="detailDialogVisible" persistent>
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6">{{ $t('feedback.detail') }}</div>
        </q-card-section>

        <q-card-section v-if="currentFeedback" class="q-pt-none">
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.type') }}：</strong>
            <q-badge :color="getTypeColor(currentFeedback.type)" :label="getTypeLabel(currentFeedback.type)" />
          </div>
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.title') }}：</strong>
            {{ currentFeedback.title }}
          </div>
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.content') }}：</strong>
            <div class="q-mt-sm q-pa-sm bg-grey-1 rounded-borders">
              {{ currentFeedback.content }}
            </div>
          </div>
          <div v-if="currentFeedback.attachments?.length" class="q-mb-sm">
            <strong>{{ $t('feedback.images') }}：</strong>
            <div class="q-mt-sm row q-gutter-sm">
              <q-img
                v-for="(attachment, index) in currentFeedback.attachments"
                :key="index"
                :src="attachment.url"
                style="width: 100px; height: 100px"
                fit="cover"
                class="rounded-borders"
              />
            </div>
          </div>
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.rating') }}：</strong>
            <q-rating
              v-if="currentFeedback.rating"
              :model-value="currentFeedback.rating"
              size="1em"
              color="warning"
              readonly
            />
            <span v-else class="text-grey-5">-</span>
          </div>
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.submitter') }}：</strong>
            {{ currentFeedback.userName }}
          </div>
          <div class="q-mb-sm">
            <strong>{{ $t('feedback.submitTime') }}：</strong>
            {{ currentFeedback.created_at }}
          </div>
        </q-card-section>

        <q-card-section>
          <q-input
            v-model="replyContent"
            type="textarea"
            :label="$t('feedback.reply')"
            outlined
            autogrow
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="detailDialogVisible = false" />
          <q-btn
            color="primary"
            :label="$t('feedback.submitReply')"
            :loading="submitting"
            @click="submitReply"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useFeedbackList } from '@/composables/useFeedbackList';

const {
  loading,
  submitting,
  detailDialogVisible,
  currentFeedback,
  replyContent,
  feedbackList,
  filters,
  typeOptions,
  statusOptions,
  columns,
  loadFeedback,
  onRequest,
  resetFilters,
  getTypeColor,
  getTypeLabel,
  getStatusColor,
  getStatusLabel,
  viewDetail,
  handleFeedback,
  submitReply,
  matRefresh,
  matVisibility,
  matCheck,
  matPlayArrow,
  matClose,
} = useFeedbackList();

onMounted(() => {
  void loadFeedback();
});
</script>
