<template>
  <!-- 编辑器工具栏 -->
  <div class="editor-toolbar">
    <!-- 基础格式 -->
    <q-btn-group flat>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_bold"
        :color="isActive('bold') ? 'primary' : undefined"
        @click="execCommand('bold')"
      >
        <q-tooltip>{{ $t('editor.bold') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_italic"
        :color="isActive('italic') ? 'primary' : undefined"
        @click="execCommand('italic')"
      >
        <q-tooltip>{{ $t('editor.italic') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_underline"
        :color="isActive('underline') ? 'primary' : undefined"
        @click="execCommand('underline')"
      >
        <q-tooltip>{{ $t('editor.underline') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="strikethrough_s"
        :color="isActive('strikeThrough') ? 'primary' : undefined"
        @click="execCommand('strikeThrough')"
      >
        <q-tooltip>{{ $t('editor.strikethrough') }}</q-tooltip>
      </q-btn>
    </q-btn-group>

    <q-separator vertical class="q-mx-sm" />

    <!-- 标题和字号 -->
    <q-btn-group flat>
      <q-btn flat dense :size="size" icon="title">
        <q-tooltip>{{ $t('editor.heading') }}</q-tooltip>
        <q-menu anchor="bottom left" self="top left">
          <q-list dense style="min-width: 120px">
            <q-item
              v-for="level in [0, 1, 2, 3, 4, 5, 6]"
              :key="level"
              clickable
              v-close-popup
              @click="formatBlock(level === 0 ? 'p' : `h${level}`)"
            >
              <q-item-section>
                <span :style="{ fontSize: level === 0 ? '14px' : `${20 - level * 2}px`, fontWeight: level > 0 ? 'bold' : 'normal' }">
                  {{ level === 0 ? $t('editor.paragraph') : `${$t('editor.heading')} ${level}` }}
                </span>
              </q-item-section>
            </q-item>
          </q-list>
        </q-menu>
      </q-btn>
    </q-btn-group>

    <q-separator vertical class="q-mx-sm" />

    <!-- 列表 -->
    <q-btn-group flat>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_list_bulleted"
        :color="isActive('insertUnorderedList') ? 'primary' : undefined"
        @click="execCommand('insertUnorderedList')"
      >
        <q-tooltip>{{ $t('editor.unorderedList') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_list_numbered"
        :color="isActive('insertOrderedList') ? 'primary' : undefined"
        @click="execCommand('insertOrderedList')"
      >
        <q-tooltip>{{ $t('editor.orderedList') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_indent_decrease"
        @click="execCommand('outdent')"
      >
        <q-tooltip>{{ $t('editor.outdent') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_indent_increase"
        @click="execCommand('indent')"
      >
        <q-tooltip>{{ $t('editor.indent') }}</q-tooltip>
      </q-btn>
    </q-btn-group>

    <q-separator vertical class="q-mx-sm" />

    <!-- 对齐 -->
    <q-btn-group flat>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_align_left"
        :color="isActive('justifyLeft') ? 'primary' : undefined"
        @click="execCommand('justifyLeft')"
      >
        <q-tooltip>{{ $t('editor.alignLeft') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_align_center"
        :color="isActive('justifyCenter') ? 'primary' : undefined"
        @click="execCommand('justifyCenter')"
      >
        <q-tooltip>{{ $t('editor.alignCenter') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_align_right"
        :color="isActive('justifyRight') ? 'primary' : undefined"
        @click="execCommand('justifyRight')"
      >
        <q-tooltip>{{ $t('editor.alignRight') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="format_align_justify"
        :color="isActive('justifyFull') ? 'primary' : undefined"
        @click="execCommand('justifyFull')"
      >
        <q-tooltip>{{ $t('editor.alignJustify') }}</q-tooltip>
      </q-btn>
    </q-btn-group>

    <q-separator vertical class="q-mx-sm" />

    <!-- 链接和图片 -->
    <q-btn-group flat>
      <q-btn flat dense :size="size" icon="link" @click="showLinkDialog">
        <q-tooltip>{{ $t('editor.insertLink') }}</q-tooltip>
      </q-btn>
      <q-btn flat dense :size="size" icon="image" @click="triggerImageInput">
        <q-tooltip>{{ $t('editor.insertImage') }}</q-tooltip>
      </q-btn>
      <q-btn flat dense :size="size" icon="video_library" @click="showVideoDialog">
        <q-tooltip>{{ $t('editor.insertVideo') }}</q-tooltip>
      </q-btn>
      <q-btn flat dense :size="size" icon="code" @click="insertCodeBlock">
        <q-tooltip>{{ $t('editor.codeBlock') }}</q-tooltip>
      </q-btn>
      <q-btn flat dense :size="size" icon="format_quote" @click="insertBlockquote">
        <q-tooltip>{{ $t('editor.blockquote') }}</q-tooltip>
      </q-btn>
    </q-btn-group>

    <q-space />

    <!-- 撤销/重做 -->
    <q-btn-group flat>
      <q-btn
        flat
        dense
        :size="size"
        icon="undo"
        :disable="!canUndo"
        @click="execCommand('undo')"
      >
        <q-tooltip>{{ $t('editor.undo') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        dense
        :size="size"
        icon="redo"
        :disable="!canRedo"
        @click="execCommand('redo')"
      >
        <q-tooltip>{{ $t('editor.redo') }}</q-tooltip>
      </q-btn>
    </q-btn-group>
  </div>
</template>

<script setup lang="ts">
const {
  isActive,
  execCommand,
  formatBlock,
  showLinkDialog,
  insertLink,
  triggerImageInput,
  showVideoDialog,
  insertCodeBlock,
  insertBlockquote,
  canUndo,
  canRedo,
  size,
} = defineProps<{
  isActive: (command: string) => boolean
  execCommand: (command: string, value?: string) => void
  formatBlock: (tag: string) => void
  showLinkDialog: () => void
  insertLink: () => void
  triggerImageInput: () => void
  showVideoDialog: () => void
  insertCodeBlock: () => void
  insertBlockquote: () => void
  canUndo: boolean
  canRedo: boolean
  size: 'sm' | 'md' | 'lg'
}>()
</script>

<style scoped>
.editor-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  padding: 8px;
  background: #fafafa;
  border-bottom: 1px solid #e4e7ed;
  gap: 4px;
}

.editor-toolbar .q-btn-group {
  border-radius: 4px;
}

.editor-toolbar .q-btn:hover {
  background: #ecf5ff;
}

.body--dark .editor-toolbar {
  background: #252525;
  border-color: #3d3d3d;
}

.body--dark .editor-toolbar .q-btn {
  color: #b0b0b0;
}

.body--dark .editor-toolbar .q-btn:hover {
  background: rgba(64, 158, 255, 0.1);
}
</style>
