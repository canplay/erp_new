/**
 * @file useWebSocket.ts
 * @description WebSocket 消息通知管理 Composable
 * @date 2026-08-15
 */

import { ref, onUnmounted } from 'vue';
import { useQuasar } from 'quasar';
import { useAuthStore } from '@/stores/auth';
import { logger } from '@/utils/logger';
import { useNotificationStore } from '@/stores/auth';
import type { WebSocketMessage } from '@/types/notification';

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'reconnecting';

export interface WebSocketConfig {
  url: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
  heartbeatTimeout?: number;
}

export type MessageHandler = (message: WebSocketMessage) => void;

export interface ConnectionInfo {
  status: WebSocketStatus;
  reconnectAttempts: number;
  lastConnectedTime: number | null;
  lastHeartbeatTime: number | null;
}

export function useWebSocket() {
  const $q = useQuasar();
  const authStore = useAuthStore();
  const notificationStore = useNotificationStore();

  let socket: WebSocket | null = null;
  let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  let heartbeatTimeoutTimer: ReturnType<typeof setTimeout> | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  const status = ref<WebSocketStatus>('disconnected');
  const lastMessage = ref<WebSocketMessage | null>(null);
  const messageHandlers = ref<MessageHandler[]>([]);
  const connectionInfo = ref<ConnectionInfo>({
    status: 'disconnected',
    reconnectAttempts: 0,
    lastConnectedTime: null,
    lastHeartbeatTime: null,
  });

  let currentUrl = '';

  const defaultConfig: Required<Omit<WebSocketConfig, 'url'>> = {
    reconnectInterval: 3000,
    maxReconnectAttempts: 10,
    heartbeatInterval: 30000,
    heartbeatTimeout: 10000,
  };

  function stopHeartbeatTimeout() {
    if (heartbeatTimeoutTimer) { clearTimeout(heartbeatTimeoutTimer); heartbeatTimeoutTimer = null; }
  }

  function stopHeartbeat() {
    if (heartbeatTimer) { clearInterval(heartbeatTimer); heartbeatTimer = null; }
    stopHeartbeatTimeout();
  }

  function handleHeartbeatResponse() {
    stopHeartbeatTimeout();
    connectionInfo.value.lastHeartbeatTime = Date.now();
  }

  function startHeartbeat(interval: number) {
    stopHeartbeat();
    heartbeatTimer = setInterval(() => {
      if (status.value === 'connected' && socket?.readyState === WebSocket.OPEN) {
        send({ type: 'ping', timestamp: Date.now() });
        startHeartbeatTimeout();
      }
    }, interval);
  }

  function startHeartbeatTimeout() {
    stopHeartbeatTimeout();
    heartbeatTimeoutTimer = setTimeout(() => {
      logger.warn('【WebSocket】心跳超时');
      handleConnectionLoss();
    }, defaultConfig.heartbeatTimeout);
  }

  function scheduleReconnect(config: Required<Omit<WebSocketConfig, 'url'>> & { url: string }) {
    if (connectionInfo.value.reconnectAttempts >= config.maxReconnectAttempts) {
      logger.error(`【WebSocket】已达到最大重连次数 (${config.maxReconnectAttempts})`);
      status.value = 'disconnected';
      $q.notify({ type: 'negative', message: '网络连接失败，请检查网络后刷新页面', timeout: 0 });
      return;
    }
    if (!authStore.isLoggedIn) return;
    if (socket?.readyState === WebSocket.OPEN) return;

    status.value = 'reconnecting';
    const delay = Math.min(
      config.reconnectInterval * Math.pow(2, connectionInfo.value.reconnectAttempts),
      config.reconnectInterval * 10
    );

    reconnectTimer = setTimeout(() => {
      if (authStore.isLoggedIn) {
        connectionInfo.value.reconnectAttempts++;
        connect({ url: currentUrl || config.url });
      }
    }, delay);
  }

  function stopReconnect() {
    if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
  }

  function handleConnectionLoss() {
    stopHeartbeat();
    stopReconnect();
    if (socket) { socket.close(4000, 'Connection lost'); socket = null; }
    status.value = 'disconnected';
    $q.notify({ type: 'warning', message: '网络连接断开，正在尝试重新连接...', timeout: 3000 });
    const config = { ...defaultConfig, url: currentUrl };
    scheduleReconnect(config);
  }

  function connect(config: WebSocketConfig) {
    const { url, heartbeatInterval } = {
      ...defaultConfig, ...config,
    };
    currentUrl = url;

    if (!authStore.isLoggedIn) return;
    if (socket && status.value === 'connected') disconnect();
    if (status.value === 'reconnecting') return;

    status.value = 'connecting';
    socket = new WebSocket(url);

    socket.onopen = () => {
      status.value = 'connected';
      connectionInfo.value.status = 'connected';
      connectionInfo.value.lastConnectedTime = Date.now();
      connectionInfo.value.reconnectAttempts = 0;
      startHeartbeat(heartbeatInterval);
      if (authStore.token) send({ action: 'auth', token: authStore.token });
    };

    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WebSocketMessage;
        handleMessage(message);
      } catch (error) {
        logger.error('【WebSocket】消息解析失败', error);
      }
    };

    socket.onerror = () => {
      logger.error('【WebSocket】连接错误');
      status.value = 'error';
      connectionInfo.value.status = 'error';
    };

    socket.onclose = (event) => {
      stopHeartbeat();
      const wasCleanClose = event.code === 1000 || event.code === 1001;
      status.value = wasCleanClose ? 'disconnected' : 'disconnected';
      if (!wasCleanClose) handleConnectionLoss();
    };
  }

  function disconnect() {
    stopHeartbeat();
    stopReconnect();
    connectionInfo.value.reconnectAttempts = 0;
    if (socket) { socket.close(1000, '主动断开'); socket = null; }
    status.value = 'disconnected';
    connectionInfo.value.status = 'disconnected';
  }

  function send(data: unknown): boolean {
    if (socket && status.value === 'connected' && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(data));
      return true;
    }
    return false;
  }

  function handleMessage(message: WebSocketMessage) {
    lastMessage.value = message;
    if ((message as { type?: string }).type === 'pong') {
      handleHeartbeatResponse();
      return;
    }
    if (message.type === 'notification') {
      notificationStore.addNotification(message.data);
      $q.notify({
        type: message.data.priority === 'urgent' || message.data.priority === 'high' ? 'warning' : 'info',
        message: message.data.title,
        caption: message.data.content,
        timeout: message.data.priority === 'urgent' ? 0 : 5000,
      });
    }
    messageHandlers.value.forEach((handler) => {
      try { handler(message); } catch (error) { logger.error('【WebSocket】消息处理失败', error); }
    });
  }

  function onMessage(handler: MessageHandler) {
    messageHandlers.value.push(handler);
    return () => {
      const index = messageHandlers.value.indexOf(handler);
      if (index > -1) messageHandlers.value.splice(index, 1);
    };
  }

  function clearMessageHandlers() { messageHandlers.value = []; }

  function resetConnection() {
    disconnect();
    connectionInfo.value.reconnectAttempts = 0;
    if (authStore.isLoggedIn && currentUrl) connect({ url: currentUrl });
  }

  function getConnectionInfo(): ConnectionInfo { return { ...connectionInfo.value }; }

  onUnmounted(() => { disconnect(); });

  return {
    status, lastMessage, connectionInfo, connect, disconnect, send, onMessage,
    clearMessageHandlers, resetConnection, getConnectionInfo,
  };
}
