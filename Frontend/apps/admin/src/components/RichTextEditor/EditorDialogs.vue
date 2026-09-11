<template>
  <!-- 链接对话框 -->
  <q-dialog :model-value="showLink" @update:model-value="(val) => $emit('update:showLink', val)" persistent>
    <q-card style="min-width: 400px">
      <q-card-section>
        <div class="text-h6">{{ $t('editor.insertLink') }}</div>
      </q-card-section>
      <q-card-section>
        <q-input
          :model-value="linkUrl"
          @update:model-value="(val) => $emit('update:linkUrl', val)"
          :label="$t('editor.url')"
          outlined
          dense
          class="q-mb-md"
        />
        <q-input
          :model-value="linkText"
          @update:model-value="(val) => $emit('update:linkText', val)"
          :label="$t('editor.linkText')"
          outlined
          dense
        />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" @click="$emit('closeLink')" />
        <q-btn color="primary" :label="$t('common.confirm')" @click="insertLink" />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 视频对话框 -->
  <q-dialog :model-value="showVideo" @update:model-value="(val) => $emit('update:showVideo', val)" persistent>
    <q-card style="min-width: 400px">
      <q-card-section>
        <div class="text-h6">{{ $t('editor.insertVideo') }}</div>
      </q-card-section>
      <q-card-section>
        <q-input
          :model-value="videoUrl"
          @update:model-value="(val) => $emit('update:videoUrl', val)"
          :label="$t('editor.videoUrl')"
          outlined
          dense
          hint="YouTube, Vimeo or URL"
        />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" @click="$emit('closeVideo')" />
        <q-btn color="primary" :label="$t('common.confirm')" @click="insertVideo" />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 全屏模式 -->
  <q-dialog :model-value="isFullscreen" @update:model-value="(val) => $emit('update:isFullscreen', val)" maximized>
    <div class="fullscreen-editor">
      <div class="fullscreen-toolbar">
        <q-btn flat icon="close" @click="$emit('closeFullscreen')">
          {{ $t('editor.exitFullscreen') }}
        </q-btn>
      </div>
      <div
        ref="fullscreenEditorRef"
        class="editor-content"
        contenteditable="true"
        :style="{ minHeight: 'calc(100vh - 60px)' }"
        v-html="sanitizedValue"
        @input="handleInput"
      />
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
const {
  showLink,
  showVideo,
  linkUrl,
  linkText,
  videoUrl,
  isFullscreen,
  sanitizedValue,
  insertLink,
  insertVideo,
  handleInput,
} = defineProps<{
  showLink: boolean
  showVideo: boolean
  linkUrl: string
  linkText: string
  videoUrl: string
  isFullscreen: boolean
  sanitizedValue: string
  insertLink: () => void
  insertVideo: () => void
  handleInput: () => void
  closeLink: () => void
  closeVideo: () => void
  closeFullscreen: () => void
}>()
</script>

<style scoped>
.fullscreen-editor {
  background: white;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.fullscreen-toolbar {
  padding: 8px 16px;
  background: #fafafa;
  border-bottom: 1px solid #e4e7ed;
}

.fullscreen-editor .editor-content {
  flex: 1;
  padding: 24px;
}

.body--dark .fullscreen-editor {
  background: #1e1e1e;
}

.body--dark .fullscreen-toolbar {
  background: #252525;
  border-color: #3d3d3d;
}
</style>
