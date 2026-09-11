import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { MatrixService } from '@/services'
import type { RoomMessage } from '@/services'

export interface SendMessageOptions {
  /** 富文本 HTML 内容（可选） */
  htmlBody?: string
  /** 回复的消息 ID */
  replyTo?: string
  /** 线程 ID */
  threadId?: string
}

// ─── Store ──────────────────────────────────────────────────────

export const useMessageStore = defineStore('message', () => {
  // State
  const messagesByRoom = ref<Record<string, RoomMessage[]>>({})
  const isLoading = ref(false)
  const isSending = ref(false)
  const error = ref<string | null>(null)
  const hasMoreHistory = ref<Record<string, boolean>>({})

  // Getters
  function getMessages(roomId: string): RoomMessage[] {
    return messagesByRoom.value[roomId] ?? []
  }

  const sortedMessages = computed(() => {
    const result: Record<string, RoomMessage[]> = {}
    for (const [roomId, msgs] of Object.entries(messagesByRoom.value)) {
      result[roomId] = [...msgs].sort((a, b) => a.timestamp - b.timestamp)
    }
    return result
  })

  // Actions
  function onRoomMessage(roomId: string, message: RoomMessage): void {
    if (!messagesByRoom.value[roomId]) {
      messagesByRoom.value[roomId] = []
    }
    const exists = messagesByRoom.value[roomId].some(
      (m) => m.eventId === message.eventId,
    )
    if (!exists) {
      messagesByRoom.value[roomId].push(message)
    }
  }

  async function loadMessages(
    roomId: string,
    limit = 30,
    from?: string,
  ): Promise<RoomMessage[]> {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const res = await ms.getMessages(roomId, from, limit)

      if (!messagesByRoom.value[roomId]) {
        messagesByRoom.value[roomId] = []
      }

      const existingIds = new Set(
        messagesByRoom.value[roomId].map((m) => m.eventId),
      )
      const newMessages = res.chunk.filter(
        (m) => !existingIds.has(m.eventId),
      )

      messagesByRoom.value[roomId] = [...messagesByRoom.value[roomId], ...newMessages]

      hasMoreHistory.value[roomId] = !!res.end
      return newMessages
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '加载消息失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function sendMessage(
    roomId: string,
    body: string,
    options?: SendMessageOptions,
  ): Promise<string> {
    isSending.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      let content: Record<string, unknown> = {
        body,
        msgtype: 'm.text',
      }

      if (options?.htmlBody) {
        content = {
          ...content,
          format: 'org.matrix.custom.html',
          formatted_body: options.htmlBody,
        }
      }

      if (options?.replyTo) {
        content = {
          ...content,
          'm.relates_to': {
            'm.in_reply_to': {
              event_id: options.replyTo,
            },
          },
        }
      }

      const eventId = await ms.sendMessage(roomId, content, options?.threadId)

      // emit optimistic event
      const optimisticMessage: RoomMessage = {
        eventId,
        roomId,
        sender: ms.getUserId() ?? '',
        content: content,
        type: 'm.room.message',
        timestamp: Date.now(),
        status: 'sent',
      }
      onRoomMessage(roomId, optimisticMessage)

      return eventId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '发送消息失败'
      throw e
    } finally {
      isSending.value = false
    }
  }

  async function sendImage(
    roomId: string,
    imageUrl: string,
    caption?: string,
  ): Promise<string> {
    isSending.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const content = {
        body: caption ?? '图片',
        msgtype: 'm.image',
        url: imageUrl,
        info: {
          mimetype: 'image/png',
        },
      }
      return await ms.sendMessage(roomId, content)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '发送图片失败'
      throw e
    } finally {
      isSending.value = false
    }
  }

  async function sendFile(
    roomId: string,
    fileUrl: string,
    fileName: string,
    mimeType: string,
  ): Promise<string> {
    isSending.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const content = {
        body: fileName,
        msgtype: 'm.file',
        url: fileUrl,
        info: {
          mimetype: mimeType,
        },
      }
      return await ms.sendMessage(roomId, content)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '发送文件失败'
      throw e
    } finally {
      isSending.value = false
    }
  }

  async function sendTyping(roomId: string, isTyping: boolean): Promise<void> {
    try {
      const ms = MatrixService.getInstance()
      await ms.sendTyping(roomId, isTyping)
    } catch {
      // 忽略输入状态错误
    }
  }

  async function sendReadReceipt(roomId: string, eventId: string): Promise<void> {
    try {
      const ms = MatrixService.getInstance()
      await ms.sendReadReceipt(roomId, eventId)
    } catch {
      // 忽略回执错误
    }
  }

  /** 发送 emoji reaction */
  async function sendReaction(roomId: string, eventId: string, emoji: string): Promise<void> {
    try {
      const ms = MatrixService.getInstance()
      await ms.sendReaction(roomId, eventId, emoji)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '发送 reaction 失败'
    }
  }

  /** 编辑消息 */
  async function editMessage(roomId: string, eventId: string, newBody: string): Promise<void> {
    isSending.value = true
    try {
      const ms = MatrixService.getInstance()
      await ms.editMessage(roomId, eventId, newBody)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '编辑消息失败'
    } finally {
      isSending.value = false
    }
  }

  /** 删除/撤回消息 */
  async function deleteMessage(roomId: string, eventId: string, reason?: string): Promise<void> {
    try {
      const ms = MatrixService.getInstance()
      await ms.redactEvent(roomId, eventId, reason)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '删除消息失败'
    }
  }

  function clearMessages(roomId: string): void {
    delete messagesByRoom.value[roomId]
    delete hasMoreHistory.value[roomId]
  }

  function clearAll(): void {
    messagesByRoom.value = {}
    hasMoreHistory.value = {}
    isLoading.value = false
    isSending.value = false
    error.value = null
  }

  function clearError(): void {
    error.value = null
  }

  return {
    // State
    messagesByRoom,
    isLoading,
    isSending,
    error,
    hasMoreHistory,
    // Getters
    getMessages,
    sortedMessages,
    // Actions
    onRoomMessage,
    loadMessages,
    sendMessage,
    sendImage,
    sendFile,
    sendTyping,
    sendReadReceipt,
    sendReaction,
    editMessage,
    deleteMessage,
    clearMessages,
    clearAll,
    clearError,
  }
})
