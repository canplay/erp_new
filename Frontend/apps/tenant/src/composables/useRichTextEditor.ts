/**
 * @file useRichTextEditor.ts
 * @description 富文本编辑器核心逻辑 composable
 * @date 2026-05-06
 */

import { ref, computed, watch, onMounted, onBeforeUnmount, type Ref } from 'vue'
import { useQuasar } from 'quasar'
import { sanitizeHTML, isSafeUrl } from '@/utils/sanitize'

export function useRichTextEditor(
  props: {
    modelValue: string
    placeholder: string
    readonly: boolean
    height: string
    size: 'sm' | 'md' | 'lg'
    allowFullscreen: boolean
    toolbar: string[]
  },
  emit: (event: 'update:modelValue' | 'change' | 'focus' | 'blur', value?: string  ) => void,
  editorRef: Ref<HTMLDivElement | null>,
  imageInputRef: Ref<HTMLInputElement | null>,
  fullscreenEditorRef: Ref<HTMLDivElement | null>,
) {
  const $q = useQuasar()

  // Refs
  const showLink = ref(false)
  const showVideo = ref(false)
  const linkUrl = ref('')
  const linkText = ref('')
  const videoUrl = ref('')
  const isFullscreen = ref(false)
  const canUndo = ref(false)
  const canRedo = ref(false)

  // Computed
  const wordCount = computed(() => {
    const text = editorRef.value?.innerText || ''
    return text.trim().split(/\s+/).filter(w => w.length > 0).length
  })

  const sanitizedValue = computed(() => {
    return sanitizeHTML(props.modelValue || '')
  })

  // HTML escape utility
  function escapeHtml(input: string): string {
    return input
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;')
  }

  /**
   * @brief 检查命令是否激活
   */
  function isActive(command: string): boolean {
    return document.queryCommandState(command)
  }

  /**
   * @brief 执行命令
   */
  function execCommand(command: string, value?: string): void {
    if (document.activeElement !== editorRef.value) {
      editorRef.value?.focus()
    }

    try {
      document.execCommand(command, false, value || undefined)
      updateUndoRedoState()
      handleInput()
    } catch (error) {
      console.error('【富文本编辑器命令执行失败】', command, error)
    }
  }

  /**
   * @brief 格式化块级元素
   */
  function formatBlock(tag: string): void {
    execCommand('formatBlock', tag)
  }

  /**
   * @brief 显示链接对话框
   */
  function showLinkDialog(): void {
    const selection = window.getSelection()
    linkText.value = selection?.toString() || ''
    linkUrl.value = ''

    const parentLink = selection?.anchorNode?.parentElement?.closest('a')
    if (parentLink) {
      linkUrl.value = parentLink.href
      linkText.value = parentLink.textContent || ''
    }

    showLink.value = true
  }

  /**
   * @brief 插入链接
   * 审计修复 (C2): URL 必须通过 http/https 白名单校验, 文本做 HTML 转义, 防 javascript: 注入
   */
  function insertLink(): void {
    if (linkUrl.value) {
      if (!isSafeUrl(linkUrl.value)) {
        $q.notify({ type: 'negative', message: '链接地址不合法, 仅支持 http/https' })
        return
      }
      const safeUrl = linkUrl.value.replace(/["'<>]/g, '')
      const safeText = escapeHtml(linkText.value || linkUrl.value)
      execCommand('insertHTML', `<a href="${safeUrl}" target="_blank" rel="noopener noreferrer">${safeText}</a>`)
    }
    showLink.value = false
    linkUrl.value = ''
    linkText.value = ''
  }

  /**
   * @brief 触发图片上传
   */
  function triggerImageInput(): void {
    imageInputRef.value?.click()
  }

  /**
   * @brief 处理图片选择
   */
  async function handleImageSelect(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]

    if (!file) return

    try {
      const base64 = await fileToBase64(file)
      execCommand('insertHTML', `<img src="${base64}" alt="${file.name}" style="max-width: 100%;" />`)
    } catch (error) {
      console.error('【图片上传失败】', error)
    }

    input.value = ''
  }

  /**
   * @brief 文件转 base64
   */
  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(reader.result as string)
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
  }

  /**
   * @brief 显示视频对话框
   */
  function showVideoDialog(): void {
    videoUrl.value = ''
    showVideo.value = true
  }

  /**
   * @brief 插入视频
   * 审计修复 (C2): 仅允许 youtube/vimeo 白名单来源, URL 非法则中止, 防任意 iframe 注入
   */
  function insertVideo(): void {
    if (videoUrl.value) {
      if (!isSafeUrl(videoUrl.value)) {
        $q.notify({ type: 'negative', message: '视频地址不合法, 仅支持 http/https' })
        return
      }
      let embedUrl = ''

      if (videoUrl.value.includes('youtube.com/watch')) {
        const videoId = new URLSearchParams(videoUrl.value.split('?')[1]).get('v')
        if (videoId) embedUrl = `https://www.youtube.com/embed/${escapeHtml(videoId)}`
      } else if (videoUrl.value.includes('youtu.be/')) {
        const videoId = videoUrl.value.split('youtu.be/')[1]?.split(/[?#]/)[0]
        if (videoId) embedUrl = `https://www.youtube.com/embed/${escapeHtml(videoId)}`
      } else if (videoUrl.value.includes('vimeo.com/')) {
        const videoId = videoUrl.value.split('vimeo.com/')[1]?.split('?')[0]
        if (videoId) embedUrl = `https://player.vimeo.com/video/${escapeHtml(videoId)}`
      }

      if (!embedUrl) {
        $q.notify({ type: 'negative', message: '仅支持 YouTube / Vimeo 视频链接' })
        return
      }

      const videoHtml = `
        <div class="video-container" style="margin: 16px 0;">
          <iframe
            src="${embedUrl}"
            frameborder="0"
            allowfullscreen
            referrerpolicy="no-referrer"
            style="width: 100%; height: 315px;"
          ></iframe>
        </div>
      `
      execCommand('insertHTML', videoHtml)
    }
    showVideo.value = false
  }

  /**
   * @brief 插入代码块
   * 审计修复 (C2): 选中文本先 HTML 转义
   */
  function insertCodeBlock(): void {
    const selection = window.getSelection()
    const text = selection?.toString() || ''

    const codeHtml = `<pre class="code-block" style="background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto;"><code>${escapeHtml(text || '// 代码')}</code></pre>`
    execCommand('insertHTML', codeHtml)
  }

  /**
   * @brief 插入引用块
   * 审计修复 (C2): 选中文本先 HTML 转义
   */
  function insertBlockquote(): void {
    const selection = window.getSelection()
    const text = selection?.toString() || ''

    const quoteHtml = `<blockquote style="border-left: 4px solid #ccc; padding-left: 16px; margin: 16px 0; color: #666;">${escapeHtml(text || '引用内容')}</blockquote>`
    execCommand('insertHTML', quoteHtml)
  }

  /**
   * @brief 处理输入
   */
  function handleInput(): void {
    const content = editorRef.value?.innerHTML || ''
    emit('update:modelValue', content)
    emit('change', content)
  }

  /**
   * @brief 处理按键
   */
  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Tab') {
      event.preventDefault()
      execCommand('insertHTML', '&nbsp;&nbsp;&nbsp;&nbsp;')
    }

    if (event.ctrlKey && event.key === 's') {
      event.preventDefault()
    }
  }

  /**
   * @brief 处理粘贴
   */
  function handlePaste(event: ClipboardEvent): void {
    const items = event.clipboardData?.items

    if (items) {
      for (const item of items) {
        if (item.type.startsWith('image/')) {
          event.preventDefault()
          const file = item.getAsFile()
          if (file) {
            fileToBase64(file).then(base64 => {
              execCommand('insertHTML', `<img src="${base64}" alt="pasted image" />`)
            }).catch((error) => {
              console.warn('【粘贴图片失败，已忽略】', error);
            })
          }
          return
        }

        if (item.type === 'text/plain') {
          // 允许默认粘贴行为
        }
      }
    }

    event.preventDefault()
    const text = event.clipboardData?.getData('text/plain')
    if (text) {
      execCommand('insertText', text)
    }
  }

  /**
   * @brief 更新撤销/重做状态
   */
  function updateUndoRedoState(): void {
    canUndo.value = document.queryCommandEnabled('undo')
    canRedo.value = document.queryCommandEnabled('redo')
  }

  /**
   * @brief 获取纯文本内容
   */
  function getText(): string {
    return editorRef.value?.innerText || ''
  }

  /**
   * @brief 清空内容
   */
  function clear(): void {
    if (editorRef.value) {
      editorRef.value.innerHTML = ''
      emit('update:modelValue', '')
      emit('change', '')
    }
  }

  /**
   * @brief 设置内容
   * 审计修复 (C2): 内容统一经过 sanitizeHTML 清洗后写入, 防止存储内容中的 XSS
   */
  function setContent(html: string): void {
    if (editorRef.value) {
      editorRef.value.innerHTML = sanitizeHTML(html)
    }
  }

  /**
   * @brief 聚焦编辑器
   */
  function focus(): void {
    editorRef.value?.focus()
  }

  // 监听 modelValue 变化
  watch(() => props.modelValue, (newValue) => {
    if (editorRef.value && editorRef.value.innerHTML !== newValue) {
      editorRef.value.innerHTML = sanitizeHTML(newValue || '')
    }
  })

  // 组件挂载
  onMounted(() => {
    if (props.modelValue && editorRef.value) {
      editorRef.value.innerHTML = sanitizeHTML(props.modelValue)
    }

    if (props.placeholder && editorRef.value) {
      editorRef.value.setAttribute('data-placeholder', props.placeholder)
    }

    document.addEventListener('selectionchange', updateUndoRedoState)
  })

  // 组件卸载
  onBeforeUnmount(() => {
    document.removeEventListener('selectionchange', updateUndoRedoState)
  })

  return {
    // State
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

    // Editor functions
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
    updateUndoRedoState,

    // Public API
    getText,
    clear,
    setContent,
    focus,
  }
}
