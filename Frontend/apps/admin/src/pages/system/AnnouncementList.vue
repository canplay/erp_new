// @ts-nocheck
/**
 * @file system/AnnouncementList.vue
 * @description 公告列表 - 搜索、工具栏、表格
 * @date 2026-08-22
 */

<template>
  <div class="announcement-list">
    <!-- 高级搜索 -->
    <AdvancedSearch
      v-model="filters"
      :show-keyword="true"
      :show-status="true"
      :show-date-range="true"
      :show-date-shortcuts="true"
      :keyword-placeholder="keywordPlaceholder"
      :status-label="statusLabel"
      :date-range-label="dateRangeLabel"
      :status-options="statusOptionsForSearch"
      :field-options="[]"
      keyword-class="col-12 col-sm-6"
      :on-search="(() => {}) as any"
      :on-reset="(() => {}) as any"
      @search="(() => {}) as any"
    />

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-end">
        <q-btn color="positive" icon="add" :label="addLabel" @click="$emit('open-dialog')" />
      </q-card-section>
    </q-card>

    <!-- 公告列表 -->
    <q-card bordered>
      <q-table
        :rows="filteredAnnouncements"
        :columns="(columns as any)"
        row-key="id"
        :loading="loading"
        :pagination="pagination"
        @request="(onTableRequest as any)"
      >
        <template v-slot:body-cell-type="props">
          <q-td :props="props">
            <q-chip
              :color="getTypeColor(props.row.type)"
              text-color="white"
              dense
              size="sm"
            >
              {{ typeLabels[props.row.type] || props.row.type }}
            </q-chip>
          </q-td>
        </template>

        <template v-slot:body-cell-title="props">
          <q-td :props="props">
            <div class="row items-center no-wrap">
              <q-icon
                v-if="props.row.isPinned"
                name="push_pin"
                color="warning"
                size="16px"
                class="q-mr-sm"
              />
              <span class="text-weight-medium">{{ props.row.title }}</span>
            </div>
          </q-td>
        </template>

        <template v-slot:body-cell-isActive="props">
          <q-td :props="props">
            <q-badge
              :color="props.row.isActive ? 'positive' : 'grey'"
              :label="props.row.isActive ? activeLabel : inactiveLabel"
            />
          </q-td>
        </template>

        <template v-slot:body-cell-start_time="props">
          <q-td :props="props">
            {{ formatTime(props.row.start_time) }}
          </q-td>
        </template>

        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense color="primary" :label="editLabel" @click="$emit('edit', props.row)" />
            <q-btn flat dense color="negative" :label="deleteLabel" @click="$emit('delete', props.row)" />
          </q-td>
        </template>

        <template v-slot:loading>
          <q-spinner-dots />
        </template>
      </q-table>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Announcement } from '@/api/system';
import type { AdvancedFilters } from '@/types/advanced-search';
import AdvancedSearch from '@/components/AdvancedSearch/Main.vue';

interface Props {
  loading: boolean;
  announcements: Announcement[];
  filters: AdvancedFilters;
  pagination: { page: number; rowsPerPage: number; rowsNumber: number };
  statusOptionsForSearch: { label: string; value: string | number | null }[];
  typeLabels: Record<string, string>;
  keywordPlaceholder?: string;
  statusLabel?: string;
  dateRangeLabel?: string;
  addLabel?: string;
  editLabel?: string;
  deleteLabel?: string;
  activeLabel?: string;
  inactiveLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  keywordPlaceholder: '',
  statusLabel: '',
  dateRangeLabel: '',
  addLabel: '添加',
  editLabel: '编辑',
  deleteLabel: '删除',
  activeLabel: '激活',
  inactiveLabel: '未激活',
});

const emit = defineEmits<{
  'search': [];
  'reset': [];
  'open-dialog': [];
  'edit': [announcement: Announcement];
  'delete': [announcement: Announcement];
  'request': [props: { pagination: { page: number; rowsPerPage: number } }];
}>();

const filteredAnnouncements = computed(() => {
  let items = props.announcements;

  if (props.filters.keyword) {
    const kw = props.filters.keyword.toLowerCase();
    items = items.filter((a) => a.title.toLowerCase().includes(kw));
  }

  if (props.filters.status !== null && props.filters.status !== '') {
    const statusVal = props.filters.status;
    const isActive = String(statusVal) === '1' || String(statusVal) === 'true' || statusVal === 1;
    items = items.filter((a) => a.isActive === isActive);
  }

  if (props.filters.start_date) {
    const start_date = new Date(props.filters.start_date);
    items = items.filter((a) => {
      if (!a.start_time) return false;
      return new Date(a.start_time) >= start_date;
    });
  }

  if (props.filters.end_date) {
    const end_date = new Date(props.filters.end_date);
    items = items.filter((a) => {
      if (!a.end_time) return true;
      return new Date(a.end_time) <= end_date;
    });
  }

  return items;
});

function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    info: 'blue',
    warning: 'orange',
    success: 'green',
    error: 'red',
  };
  return colors[type] || 'grey';
}

function formatTime(timeStr?: string): string {
  if (!timeStr) return '-';
  return new Date(timeStr).toLocaleString('zh-CN');
}
</script>
