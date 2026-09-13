<template>
  <q-page class="q-pa-md file-manager">
    <!-- 页面标题和操作栏 -->
    <div class="row items-center q-mb-md">
      <div class="text-h5 text-weight-bold">{{ $t('file.title') }}</div>
      <q-space />
      <q-btn
        flat
        color="grey"
        :label="
          $t('file.storage') +
          ': ' +
          fileStore.formattedStorage.used +
          ' / ' +
          fileStore.formattedStorage.total
        "
      />
    </div>

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center q-col-gutter-md">
        <!-- 返回上一级 -->
        <div class="col-auto">
          <q-btn
            flat
            color="grey"
            icon="arrow_back"
            :disable="fileStore.breadcrumbs.length <= 1"
            @click="handleGoBack"
          />
        </div>

        <!-- 刷新 -->
        <div class="col-auto">
          <q-btn
            flat
            color="primary"
            icon="refresh"
            @click="handleRefresh"
          />
        </div>

        <!-- 新建文件夹 -->
        <div class="col-auto">
          <q-btn
            flat
            color="primary"
            icon="create_new_folder"
            :label="$t('file.newFolder')"
            @click="showNewFolderDialog = true"
          />
        </div>

        <!-- 上传文件 -->
        <div class="col-auto">
          <q-btn
            flat
            color="positive"
            icon="upload"
            :label="$t('file.upload')"
            @click="triggerUpload"
          />
          <input
            ref="fileInputRef"
            type="file"
            multiple
            style="display: none"
            @change="handleFileSelect"
          />
        </div>

        <q-space />

        <!-- 搜索 -->
        <div class="col-12 col-sm-4">
          <q-input
            v-model="searchKeyword"
            dense
            outlined
            :placeholder="$t('file.search')"
            @keyup.enter="handleSearch"
          >
            <template v-slot:prepend>
              <q-icon name="search" />
            </template>
            <template v-slot:append v-if="searchKeyword">
              <q-icon name="close" class="cursor-pointer" @click="handleClearSearch" />
            </template>
          </q-input>
        </div>
      </q-card-section>
    </q-card>

    <!-- 面包屑 -->
    <q-breadcrumbs class="q-mb-md" active-color="primary">
      <q-breadcrumbs-el
        v-for="crumb in fileStore.breadcrumbs"
        :key="crumb.id ?? 0"
        :label="crumb.name"
        @click="handleBreadcrumb(crumb)"
      />
    </q-breadcrumbs>

    <!-- 批量操作栏 -->
    <q-card v-if="fileStore.hasSelection" class="q-mb-md" bordered>
      <q-card-section class="row items-center q-col-gutter-md">
        <div class="col-auto">
          <span>{{ $t('batchActions.selected', { count: fileStore.selectionCount }) }}</span>
        </div>
        <q-space />
        <div class="col-auto">
          <q-btn
            flat
            color="primary"
            icon="download"
            :label="$t('file.download')"
            @click="handleBatchDownload"
          />
          <q-btn
            flat
            color="warning"
            icon="share"
            :label="$t('file.share')"
            @click="showShareDialog = true"
          />
          <q-btn
            flat
            color="negative"
            icon="delete"
            :label="$t('common.delete')"
            @click="handleBatchDelete"
          />
          <q-btn
            flat
            color="grey"
            icon="clear"
            :label="$t('batchActions.clearSelection')"
            @click="handleClearSelection"
          />
        </div>
      </q-card-section>
    </q-card>

    <!-- 文件列表 -->
    <q-card bordered>
      <q-card-section v-if="fileStore.loading">
        <div class="text-center q-pa-xl">
          <q-spinner-dots size="50px" color="primary" />
        </div>
      </q-card-section>

      <q-card-section v-else-if="fileStore.files.length === 0 && fileStore.folders.length === 0">
        <div class="text-center text-grey q-pa-xl">
          <q-icon name="folder_open" size="64px" class="q-mb-md" />
          <div>{{ $t('file.empty') }}</div>
        </div>
      </q-card-section>

      <q-card-section v-else>
        <!-- 表格视图 -->
        <q-table
          :rows="tableRows"
          :columns="columns"
          row-key="id"
          flat
          :loading="fileStore.loading"
          v-model:selected="selectedTableRows"
          selection="multiple"
          @row-click="handleRowClick"
        >
          <!-- 文件名列 -->
          <template v-slot:body-cell-name="props">
            <q-td :props="props">
              <div class="row items-center no-wrap">
                <q-icon
                  :name="getFileIcon(props.row)"
                  size="24px"
                  class="q-mr-sm"
                  :color="getFileIconColor(props.row)"
                />
                <span class="text-weight-medium">{{ props.row.name }}</span>
              </div>
            </q-td>
          </template>

          <!-- 大小列 -->
          <template v-slot:body-cell-size="props">
            <q-td :props="props">
              {{ props.row.isFolder ? '-' : fileStore.formatBytes(props.row.size) }}
            </q-td>
          </template>

          <!-- 操作列 -->
          <template v-slot:body-cell-actions="props">
            <q-td :props="props">
              <q-btn
                flat
                dense
                color="primary"
                :label="$t('common.view')"
                @click.stop="handleView(props.row)"
              />
              <q-btn
                v-if="!props.row.isFolder"
                flat
                dense
                color="positive"
                :label="$t('file.download')"
                @click.stop="handleDownload(props.row)"
              />
              <q-btn flat dense color="grey" icon="more_vert">
                <q-menu>
                  <q-list style="min-width: 120px">
                    <q-item clickable v-close-popup @click="handleRename(props.row)">
                      <q-item-section>{{ $t('file.rename') }}</q-item-section>
                    </q-item>
                    <q-item clickable v-close-popup @click="handleMove(props.row)">
                      <q-item-section>{{ $t('file.move') }}</q-item-section>
                    </q-item>
                    <q-item clickable v-close-popup @click="handleCopy(props.row)">
                      <q-item-section>{{ $t('file.copy') }}</q-item-section>
                    </q-item>
                    <q-separator />
                    <q-item
                      clickable
                      v-close-popup
                      @click="handleDelete(props.row)"
                      class="text-negative"
                    >
                      <q-item-section>{{ $t('common.delete') }}</q-item-section>
                    </q-item>
                  </q-list>
                </q-menu>
              </q-btn>
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 新建文件夹对话框 -->
    <q-dialog v-model="showNewFolderDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('file.newFolder') }}</div>
        </q-card-section>
        <q-card-section>
          <q-input
            v-model="newFolderName"
            :label="$t('file.folderName')"
            outlined
            autofocus
            @keyup.enter="handleCreateFolder"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showNewFolderDialog = false" />
          <q-btn color="primary" :label="$t('common.confirm')" @click="handleCreateFolder" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 重命名对话框 -->
    <q-dialog v-model="showRenameDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('file.rename') }}</div>
        </q-card-section>
        <q-card-section>
          <q-input
            v-model="renameValue"
            :label="$t('file.newName')"
            outlined
            autofocus
            @keyup.enter="handleDoRename"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showRenameDialog = false" />
          <q-btn color="primary" :label="$t('common.confirm')" @click="handleDoRename" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 移动对话框 -->
    <q-dialog v-model="showMoveDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('file.move') }}</div>
        </q-card-section>
        <q-card-section>
          <q-select
            v-model="targetFolderId"
            :options="folderOptions"
            :label="$t('file.targetFolder')"
            outlined
            clearable
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showMoveDialog = false" />
          <q-btn color="primary" :label="$t('common.confirm')" @click="handleDoMove" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 分享对话框 -->
    <q-dialog v-model="showShareDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('file.share') }}</div>
        </q-card-section>
        <q-card-section>
          <q-input
            v-model="sharePassword"
            :label="$t('file.sharePassword')"
            outlined
            type="password"
            :placeholder="$t('file.optional')"
          />
          <q-select
            v-model="shareExpiryDays"
            :options="expiryOptions"
            :label="$t('file.expiryDays')"
            outlined
            class="q-mt-md"
          />
        </q-card-section>
        <q-card-section v-if="shareUrl">
          <q-input v-model="shareUrl" readonly outlined :label="$t('file.shareUrl')">
            <template v-slot:append>
              <q-btn flat dense icon="content_copy" @click="copyShareUrl" />
            </template>
          </q-input>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn
            v-if="!shareUrl"
            flat
            :label="$t('common.cancel')"
            @click="showShareDialog = false"
          />
          <q-btn v-else flat :label="$t('common.close')" @click="showShareDialog = false" />
          <q-btn
            v-if="!shareUrl"
            color="primary"
            :label="$t('file.createShare')"
            @click="handleCreateShare"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 上传进度 -->
    <q-dialog v-model="showUploadProgress" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('file.uploading') }}</div>
        </q-card-section>
        <q-card-section>
          <q-linear-progress :value="uploadProgress / 100" color="primary" class="q-my-md" />
          <div class="text-center">{{ uploadProgress }}%</div>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useFileManager } from '@erp-new-frontend-monorepo/composables/src/useFileManager';;

const {
  fileStore, searchKeyword, fileInputRef, selectedTableRows,
  showNewFolderDialog, newFolderName,
  showRenameDialog, renameValue,
  showMoveDialog, targetFolderId,
  showShareDialog, sharePassword, shareExpiryDays, shareUrl,
  showUploadProgress, uploadProgress,
  columns, tableRows, folderOptions, expiryOptions,
  getFileIcon, getFileIconColor,
  handleRowClick, handleCopy, handleFileSelect, handleCreateFolder,
  handleRename, handleDoRename, handleMove, handleDoMove,
  handleGoBack, handleRefresh, handleClearSelection,
  handleDelete, handleBatchDelete, handleView, handleDownload, handleBatchDownload,
  handleCreateShare, copyShareUrl,
} = useFileManager();

function triggerUpload() { fileInputRef.value?.click(); }
function handleSearch() { void fileStore.search(searchKeyword.value); }
function handleClearSearch() { searchKeyword.value = ''; void fileStore.clearSearch(); }
function handleBreadcrumb(crumb: { id?: number; name: string }) {
  const idx = fileStore.breadcrumbs.findIndex((b) => b.name === crumb.name);
  if (idx >= 0) {
    fileStore.breadcrumbs.splice(idx + 1);
    void fileStore.loadFiles(crumb.id);
  }
}

onMounted(() => {
  void fileStore.loadFiles();
  void fileStore.loadStorageStats();
});
</script>

<style scoped>
.file-manager {
  min-height: 100vh;
}
</style>
