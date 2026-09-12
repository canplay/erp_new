
<template>
  <div class="rich-text-editor">
    <EditorToolbar
      :is-active="isActive"
      :exec-command="execCommand"
      :format-block="formatBlock"
      :show-link-dialog="showLinkDialog"
      :insert-link="insertLink"
      :trigger-image-input="triggerImageInput"
      :show-video-dialog="showVideoDialog"
      :insert-code-block="insertCodeBlock"
      :insert-blockquote="insertBlockquote"
      :can-undo="canUndo"
      :can-redo="canRedo"
      :size="size"
    />

    <!-- 隐藏的图片输入 -->
    <input
      ref="imageInputRef"
      type="file"
      accept="image/*"
      style="display: none"
      @change="handleImageSelect"
    />

    <!-- 编辑器内容区 -->
    <div
      ref="editorRef"
      class="editor-content"
      :contenteditable="!readonly"
      :style="{ minHeight: height, backgroundColor: readonly ? '#f5f5f5' : undefined }"
      @input="handleInput"
      @keydown="handleKeydown"
      @paste="handlePaste"
      v-html="sanitizedValue"
    />

    <EditorDialogs
      :show-link="showLink"
      :show-video="showVideo"
      :link-url="linkUrl"
      :link-text="linkText"
      :video-url="videoUrl"
      :is-fullscreen="isFullscreen"
      :sanitized-value="sanitizedValue"
      :insert-link="insertLink"
      :insert-video="insertVideo"
      :handle-input="handleInput"
      @close-link="showLink = false"
      @close-video="showVideo = false"
      @close-fullscreen="isFullscreen = false"
    />

    <!-- 底部工具栏 -->
    <div class="editor-footer">
      <span class="text-caption text-grey-6">
        {{ wordCount }} {{ $t('editor.words') }}
      </span>
      <q-space />
      <q-btn
        v-if="allowFullscreen"
        flat
        dense
        size="sm"
        icon="fullscreen"
        :label="$t('editor.fullscreen')"
        @click="isFullscreen = true"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import EditorToolbar from './RichTextEditor/EditorToolbar.vue'
import EditorDialogs from './RichTextEditor/EditorDialogs.vue'
import { useRichTextEditor } from '@/composables/useRichTextEditor'

const { t: $t } = useI18n()

interface Props {
  modelValue?: string
  placeholder?: string
  readonly?: boolean
  height?: string
  size?: 'sm' | 'md' | 'lg'
  allowFullscreen?: boolean
  toolbar?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  placeholder: '',
  readonly: false,
  height: '300px',
  size: 'sm',
  allowFullscreen: true,
  toolbar: () => [
    'bold', 'italic', 'underline', 'strikeThrough',
    'heading', 'list', 'align', 'link', 'image', 'video', 'code'
  ]
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string, event?: Event): void
  (e: 'change', value: string): void
  (e: 'focus'): void
  (e: 'blur'): void
}>()

const editorRef = ref<HTMLDivElement | null>(null)
const imageInputRef = ref<HTMLInputElement | null>(null as HTMLInputElement | null)
const fullscreenEditorRef = ref<HTMLDivElement | null>(null)

const {
  showLink,
  showVideo,
  linkUrl,
  linkText,
  videoUrl,
  isFullscreen,
  canUndo,
  canRedo,
  wordCount,
  sanitizedValue,
  isActive,
  execCommand,
  formatBlock,
  showLinkDialog,
  insertLink,
  triggerImageInput,
  handleImageSelect,
  showVideoDialog,
  insertVideo,
  insertCodeBlock,
  insertBlockquote,
  handleInput,
  handleKeydown,
  handlePaste,
  getText,
  clear,
  setContent,
  focus,
} = useRichTextEditor(props, emit, editorRef, imageInputRef, fullscreenEditorRef)

defineExpose({
  getText,
  clear,
  setContent,
  focus,
  execCommand,
})
</script>

<style scoped>
.rich-text-editor {
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  overflow: hidden;
  background: white;
}

.rich-text-editor:focus-within {
  border-color: #409eff;
}

.editor-content {
  padding: 16px;
  outline: none;
  overflow-y: auto;
  line-height: 1.8;
}

.editor-content:empty:before {
  content: attr(data-placeholder);
  color: #aaa;
  pointer-events: none;
}

.editor-content :deep(h1),
.editor-content :deep(h2),
.editor-content :deep(h3),
.editor-content :deep(h4),
.editor-content :deep(h5),
.editor-content :deep(h6) {
  margin: 16px 0 8px 0;
  font-weight: 600;
}

.editor-content :deep(p) {
  margin: 8px 0;
}

.editor-content :deep(ul),
.editor-content :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.editor-content :deep(a) {
  color: #409eff;
  text-decoration: none;
}

.editor-content :deep(a:hover) {
  text-decoration: underline;
}

.editor-content :deep(img) {
  max-width: 100%;
  height: auto;
  margin: 8px 0;
  border-radius: 4px;
}

.editor-content :deep(pre) {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
  font-family: 'Consolas', 'Monaco', monospace;
}

.editor-content :deep(code) {
  font-family: 'Consolas', 'Monaco', monospace;
  background: #f0f0f0;
  padding: 2px 4px;
  border-radius: 2px;
  font-size: 0.9em;
}

.editor-content :deep(blockquote) {
  border-left: 4px solid #ccc;
  padding-left: 16px;
  margin: 16px 0;
  color: #666;
}

.editor-content :deep(.video-container) {
  margin: 16px 0;
}

.editor-content :deep(.video-container iframe) {
  max-width: 100%;
}

.editor-footer {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  background: #fafafa;
  border-top: 1px solid #e4e7ed;
  font-size: 12px;
}

.body--dark .rich-text-editor {
  background: #1e1e1e;
  border-color: #3d3d3d;
}

.body--dark .rich-text-editor:focus-within {
  border-color: #409eff;
}

.body--dark .editor-content {
  background: #1e1e1e;
  color: #b0b0b0;
}

.body--dark .editor-content:empty:before {
  color: #666;
}

.body--dark .editor-footer {
  background: #252525;
  border-color: #3d3d3d;
}

.body--dark .editor-content :deep(pre),
.body--dark .editor-content :deep(code) {
  background: #2d2d2d;
  color: #b0b0b0;
}

.body--dark .editor-content :deep(blockquote) {
  border-color: #555;
  color: #999;
}

@media (max-width: 768px) {
  .editor-toolbar {
    padding: 4px;
  }

  .editor-toolbar .q-btn :deep(.q-icon) {
    font-size: 18px;
  }
}
</style>
