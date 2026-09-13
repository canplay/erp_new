/**
 * 实时通知 Composable
 *
 * 提供 WebSocket 推送、通知中心、声音提醒等功能
 */

import { ref, readonly, onMounted, onUnmounted, computed } from 'vue'
import { useQuasar } from 'quasar'
import { logger } from '@/utils/logger'
import { useAuthStore } from '@/stores/auth'
import { getStorageItem } from '@/utils/storage'

export interface Notification {
  /** 通知 ID */
  id: string
  /** 通知类型 */
  type: 'info' | 'success' | 'warning' | 'error'
  /** 通知标题 */
  title: string
  /** 通知内容 */
  message?: string
  /** 通知来源 */
  source?: string
  /** 关联数据 */
  data?: unknown
  /** 是否已读 */
  read: boolean
  /** 创建时间 */
  created_at: number
  /** 过期时间（毫秒） */
  expireAt?: number
}

export interface NotificationOptions {
  /** 最大通知数 */
  maxNotifications?: number
  /** 默认过期时间（毫秒） */
  defaultExpireMs?: number
  /** 是否启用声音 */
  enableSound?: boolean
  /** WebSocket 地址 */
  wsUrl?: string
  /** 自动重连 */
  autoReconnect?: boolean
  /** 重连间隔（毫秒） */
  reconnectInterval?: number
}

const DEFAULT_OPTIONS: Required<NotificationOptions> = {
  maxNotifications: 100,
  defaultExpireMs: 60000, // 1 分钟
  enableSound: true,
  wsUrl: import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws/messages',
  autoReconnect: true,
  reconnectInterval: 5000
}

/**
 * 通知 Composable
 */
export function useNotification(options: NotificationOptions = {}) {
  const opts = { ...DEFAULT_OPTIONS, ...options }
  const $q = useQuasar()

  // 通知列表
  const notifications = ref<Notification[]>([])
  
  // WebSocket 连接状态
  const wsStatus = ref<'connected' | 'disconnected' | 'connecting' | 'error'>('disconnected')
  
  // WebSocket 实例
  let ws: WebSocket | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null

  // 计算未读数量
  const unreadCount = computed(() => {
    return notifications.value.filter(n => !n.read).length
  })

  // 计算未读重要性通知
  const importantUnread = computed(() => {
    return notifications.value.filter(n => !n.read && (n.type === 'error' || n.type === 'warning'))
  })

  /**
   * 添加通知
   */
  function addNotification(notification: Omit<Notification, 'id' | 'read' | 'created_at'>) {
    const newNotification: Notification = {
      ...notification,
      id: crypto.randomUUID(),
      read: false,
      created_at: Date.now(),
      expireAt: notification.expireAt ?? Date.now() + opts.defaultExpireMs
    }

    // 添加到列表头部
    notifications.value.unshift(newNotification)

    // 清理过期通知
    cleanupExpired()

    // 限制最大数量
    while (notifications.value.length > opts.maxNotifications) {
      notifications.value.pop()
    }

    // 显示通知
    showNotification(newNotification)

    // 播放声音
    if (opts.enableSound) {
      playSound()
    }

    return newNotification.id
  }

  /**
   * 显示 Quasar 通知
   */
  function showNotification(notification: Notification) {
    const colorMap: Record<string, string> = {
      info: 'info',
      success: 'positive',
      warning: 'warning',
      error: 'negative'
    }

    const iconMap: Record<string, string> = {
      info: 'info',
      success: 'check_circle',
      warning: 'warning',
      error: 'error'
    }

    $q.notify({
      type: colorMap[notification.type] || 'info',
      icon: iconMap[notification.type] || 'notifications',
      message: notification.title,
      caption: notification.message ?? '',
      position: 'top-right',
      timeout: 5000,
      actions: [
        { icon: 'close', color: 'white', round: true, dense: true }
      ]
    })
  }

  /**
   * 播放声音
   */
  function playSound() {
    // 创建音频元素
    try {
      const audio = new Audio()
      // 使用内置的提示音或自定义音效
      audio.volume = 0.3
      // 这里可以使用 base64 编码的音频数据或外部 URL
      // audio.src = '/sounds/notification.mp3'
      // audio.play()
    } catch {
      // 忽略音频播放错误
    }
  }

  /**
   * 标记已读
   */
  function markAsRead(id: string) {
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.read = true
    }
  }

  /**
   * 标记全部已读
   */
  function markAllAsRead() {
    notifications.value.forEach(n => {
      n.read = true
    })
  }

  /**
   * 删除通知
   */
  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }

  /**
   * 清空所有通知
   */
  function clearAll() {
    notifications.value = []
  }

  /**
   * 清空已读通知
   */
  function clearRead() {
    notifications.value = notifications.value.filter(n => !n.read)
  }

  /**
   * 清理过期通知
   */
  function cleanupExpired() {
    const now = Date.now()
    notifications.value = notifications.value.filter(n => {
      return !n.expireAt || n.expireAt > now
    })
  }

  /**
   * 连接 WebSocket
   */
  function connect() {
    if (ws && ws.readyState === WebSocket.OPEN) {
      return
    }

    wsStatus.value = 'connecting'

    try {
      // 修复 (2026-08-07): 浏览器 WS 握手无法带 Authorization 头,
      // token 通过 URL 查询参数传递(后端 ws handler 验证), 同时保留
      // onopen 后的 {action:'auth'} 消息兼容
      const authStoreForUrl = useAuthStore()
      const tokenForUrl = authStoreForUrl.token || getStorageItem<string>('admin_token', '')
      const wsUrlWithToken = tokenForUrl && !opts.wsUrl.includes('token=')
        ? `${opts.wsUrl}?token=${encodeURIComponent(tokenForUrl)}`
        : opts.wsUrl
      ws = new WebSocket(wsUrlWithToken)

      ws.onopen = () => {
        wsStatus.value = 'connected'
        logger.info('[WebSocket] Connected')

        // 认证（如果需要）
        // 修复 (fix-plan-20260806 P16): 原 localStorage.getItem('token') 读错 key，
        // 实际 access token 存储在 'admin_token' (auth.ts STORAGE_KEYS.TOKEN)，
        // 导致 WS 认证消息永远不带 token、服务端拒绝连接。
        const authStore = useAuthStore()
        const token = authStore.token || getStorageItem<string>('admin_token', '')
        if (token) {
          ws?.send(JSON.stringify({ action: 'auth', token }))
        }
      }

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          handleWsMessage(data)
        } catch (e) {
          logger.error('[WebSocket] Failed to parse message:', e)
        }
      }

      ws.onclose = () => {
        wsStatus.value = 'disconnected'
        logger.info('[WebSocket] Disconnected')
        
        // 自动重连
        if (opts.autoReconnect) {
          scheduleReconnect()
        }
      }

      ws.onerror = (error) => {
        wsStatus.value = 'error'
        logger.error('[WebSocket] Error:', error)
      }
    } catch (e) {
      wsStatus.value = 'error'
      logger.error('[WebSocket] Failed to connect:', e)
    }
  }

  /**
   * WebSocket 消息结构
   */
  interface WsMessage {
    type: string
    title?: string
    message?: string
    source?: string
    data?: unknown
    notificationType?: string
  }

  /**
   * 处理 WebSocket 消息
   */
  function handleWsMessage(data: WsMessage) {
    switch (data.type) {
      case 'notification':
        addNotification({
          type: (data.notificationType || 'info') as 'info' | 'success' | 'warning' | 'error',
          title: data.title || '新通知',
          ...(data.message !== undefined ? { message: data.message } : {}),
          ...(data.source !== undefined ? { source: data.source } : {}),
          ...(data.data !== undefined ? { data: data.data } : {})
        })
        break

      case 'system':
        addNotification({
          type: 'info',
          title: '系统消息',
          ...(data.message !== undefined ? { message: data.message } : {})
        })
        break

      case 'alert':
        addNotification({
          type: 'error',
          title: '告警',
          ...(data.message !== undefined ? { message: data.message } : {}),
          ...(data.data !== undefined ? { data: data.data } : {})
        })
        break

      default:
        logger.info('[WebSocket] Unknown message type:', data.type)
    }
  }

  /**
   * 发送消息
   */
  function send(data: Record<string, unknown>) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(data))
    }
  }

  /**
   * 断开连接
   */
  function disconnect() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }

    if (ws) {
      ws.close()
      ws = null
    }

    wsStatus.value = 'disconnected'
  }

  /**
   * 计划重连
   */
  function scheduleReconnect() {
    if (reconnectTimer) {
      return
    }

    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      connect()
    }, opts.reconnectInterval)
  }

  // 生命周期
  onMounted(() => {
    // 定时清理过期通知
    setInterval(cleanupExpired, 10000)
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    // 状态
    notifications: readonly(notifications),
    wsStatus: readonly(wsStatus),
    unreadCount,
    importantUnread,

    // 方法
    addNotification,
    markAsRead,
    markAllAsRead,
    removeNotification,
    clearAll,
    clearRead,

    // WebSocket
    connect,
    disconnect,
    send
  }
}

/**
 * 创建通知快捷方法
 */
export function createNotification() {
  const { addNotification } = useNotification()
  return addNotification
}

// 快捷方法
export const notifySuccess = (title: string, message?: string) => {
  const add = createNotification()
  add({ type: 'success', title, ...(message !== undefined ? { message } : {}) })
}

export const notifyError = (title: string, message?: string) => {
  const add = createNotification()
  add({ type: 'error', title, ...(message !== undefined ? { message } : {}) })
}

export const notifyWarning = (title: string, message?: string) => {
  const add = createNotification()
  add({ type: 'warning', title, ...(message !== undefined ? { message } : {}) })
}

export const notifyInfo = (title: string, message?: string) => {
  const add = createNotification()
  add({ type: 'info', title, ...(message !== undefined ? { message } : {}) })
}