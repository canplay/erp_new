/**
 * @file ExportDialog/ExportPreview.vue
 * @description 导出预览组件 - 文件信息、预览数据、额外参数
 * @date 2026-08-22
 */

<template>
  <div class="export-preview">
    <!-- 文件信息 -->
    <q-card-section v-if="fileInfo">
      <q-list>
        <q-item>
          <q-item-section avatar>
            <q-icon name="insert-drive-file" />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ fileInfo.name }}</q-item-label>
            <q-item-label caption>
              {{ formatFileSize(fileInfo.size) }}
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-list>
    </q-card-section>

    <!-- 文件验证结果 -->
    <q-card-section v-if="fileToUpload">
      <q-list bordered>
        <q-item>
          <q-item-section avatar>
            <q-icon name="description" />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ fileToUpload.name }}</q-item-label>
            <q-item-label caption>
              {{ formatFileSize(fileToUpload.size) }}
            </q-item-label>
          </q-item-section>
          <q-item-section side>
            <q-btn round flat dense icon="close" @click="$emit('clearFile')" />
          </q-item-section>
        </q-item>
      </q-list>

      <!-- 额外参数 -->
      <div v-if="showExtraParams" class="q-mt-md">
        <q-input
          v-model="localDescription"
          outlined
          :label="$t('common.fileDescription')"
          type="textarea"
          autogrow
        />
      </div>
    </q-card-section>

    <!-- 批量导入预览 -->
    <q-card-section v-if="previewData.length > 0">
      <div class="text-subtitle2 q-mb-sm">导入预览（前 5 条）</div>
      <q-table
        :rows="previewData"
        :columns="previewColumns"
        flat
        dense
        :rows-per-page="5"
        :pagination="{ rowsPerPage: 5 }"
      />
      <div class="text-caption text-grey q-mt-sm">共 {{ totalRows }} 条数据</div>
    </q-card-section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
interface UploadFile {
  file: File;
  name: string;
  size: number;
  type: string;
}

interface Props {
  fileInfo?: { name: string; size: number } | undefined;
  fileToUpload?: UploadFile | null;
  showExtraParams?: boolean;
  previewData?: Record<string, unknown>[];
  totalRows?: number;
  previewColumns?: Array<{ name: string; label: string; field: string }>;
}

const props = withDefaults(defineProps<Props>(), {
  fileInfo: () => ({ name: '', size: 0 }),
  fileToUpload: null,
  showExtraParams: false,
  previewData: () => [],
  totalRows: 0,
  previewColumns: () => [],
});

const emit = defineEmits<{
  clearFile: [];
}>();

const localDescription = ref<string>('');

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
</script>
