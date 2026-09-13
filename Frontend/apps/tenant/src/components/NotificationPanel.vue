/**
 * @file NotificationPanel.vue
 * @description 通知面板组件
 * @date 2026-04-03
 */

<template>
  <q-btn flat dense round icon="notifications" class="q-mr-sm" @click="openPanel">
    <q-badge v-if="unreadCount > 0" color="negative" floating>
      {{ unreadCount > 99 ? '99+' : unreadCount }}
    </q-badge>
    <q-tooltip>{{ $t('common.notifications') }}</q-tooltip>
  </q-btn>

  <!-- 通知面板 -->
  <q-dialog v-model="showPanel" position="right" full-height>
    <q-card class="notification-panel">
      <!-- 头部 -->
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6">{{ $t('notification.title') }}</div>
        <q-space />
        <q-btn
          v-if="notifications.length > 0"
          flat
          dense
          color="grey"
          :label="$t('notification.markAllRead')"
          size="sm"
          @click="markAllAsRead"
        />
        <q-btn flat round dense icon="close" @click="showPanel = false" />
      </q-card-section>

      <q-separator />

      <!-- 标签页 -->
      <q-tabs v-model="activeTab" dense align="justify" class="text-grey-7" active-color="primary" indicator-color="primary">
        <q-tab name="all" :label="$t('common.all')" />
        <q-tab name="unread" :label="$t('notification.unread')" />
        <q-tab name="system" :label="$t('notification.typeEnum.system')" />
      </q-tabs>

      <q-separator />

      <!-- 通知列表 -->
      <q-card-section class="notification-list">
        <q-scroll-area style="height: calc(100vh - 200px)">
          <!-- 空状态 -->
          <div v-if="filteredNotifications.length === 0" class="text-center q-pa-xl">
            <q-icon name="notifications_none" size="64px" color="grey-5" />
            <div class="text-grey-6 q-mt-md">{{ $t('notification.empty') }}</div>
          </div>

          <!-- 通知列表 -->
          <q-list v-else separator>
            <q-item
              v-for="notification in filteredNotifications"
              :key="notification.id"
              clickable
              :class="{ 'bg-blue-1': !notification.is_read }"
              @click="handleNotificationClick(notification)"
            >
              <!-- 图标 -->
              <q-item-section avatar>
                <q-avatar :color="getIconColor(notification.type)" text-color="white" size="40px">
                  <q-icon :name="getIconName(notification.type)" />
                </q-avatar>
              </q-item-section>

              <!-- 内容 -->
              <q-item-section>
                <q-item-label :class="{ 'text-bold': !notification.is_read }">
                  {{ notification.title }}
                </q-item-label>
                <q-item-label caption lines="2" class="q-mt-xs">
                  {{ notification.content }}
                </q-item-label>
                <div class="text-caption text-grey-6 q-mt-xs">
                  <q-icon name="schedule" size="12px" />
                  {{ formatTime(notification.createTime) }}
                </div>
              </q-item-section>

              <!-- 操作 -->
              <q-item-section side>
                <q-btn
                  v-if="!notification.is_read"
                  flat
                  dense
                  round
                  icon="done"
                  color="grey"
                  size="sm"
                  @click.stop="markAsRead(notification.id)"
                >
                  <q-tooltip>{{ $t('notification.markRead') }}</q-tooltip>
                </q-btn>
              </q-item-section>
            </q-item>
          </q-list>
        </q-scroll-area>
      </q-card-section>

      <!-- 底部操作 -->
      <q-separator />
      <q-card-actions align="center">
        <q-btn flat color="primary" :label="$t('notification.detail')" @click="goToNotificationPage" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
/**
 * @file NotificationPanel.vue
 * @description 通知面板组件
 * @date 2026-04-03
 */

import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const router = useRouter();

// ============ 类型定义 ============

/**
 * @brief 通知类型
 */
type NotificationType = 'info' | 'warning' | 'success' | 'error' | 'system';

/**
 * @brief 通知项接口
 */
interface Notification {
  id: string;
  title: string;
  content: string;
  type: NotificationType;
  is_read: boolean;
  createTime: string;
  link?: string;
}

// ============ 状态 ============
const showPanel = ref(false);
const activeTab = ref('all');

// 模拟通知数据（实际应从 API 获取）
// 注意：通知内容使用 i18n key，实际项目中应从后端 API 获取并直接显示
const notifications = ref<Notification[]>([
  {
    id: '1',
    title: t('notification.systemUpdate'),
    content: t('notification.systemUpdate'),
    type: 'system',
    is_read: false,
    createTime: new Date().toISOString(),
    link: '/notice/1',
  },
  {
    id: '2',
    title: t('notification.newUser'),
    content: t('notification.newUser'),
    type: 'info',
    is_read: false,
    createTime: new Date(Date.now() - 3600000).toISOString(),
  },
  {
    id: '3',
    title: t('notification.loginAlert'),
    content: t('notification.loginAlert'),
    type: 'warning',
    is_read: true,
    createTime: new Date(Date.now() - 86400000).toISOString(),
  },
  {
    id: '4',
    title: t('notification.backupComplete'),
    content: t('notification.backupComplete'),
    type: 'success',
    is_read: true,
    createTime: new Date(Date.now() - 172800000).toISOString(),
  },
]);

// ============ 计算属性 ============

/**
 * @brief 未读通知数量
 */
const unreadCount = computed(() => notifications.value.filter((n) => !n.is_read).length);

/**
 * @brief 根据标签页筛选通知
 */
const filteredNotifications = computed(() => {
  switch (activeTab.value) {
    case 'unread':
      return notifications.value.filter((n) => !n.is_read);
    case 'system':
      return notifications.value.filter((n) => n.type === 'system');
    default:
      return notifications.value;
  }
});

// ============ 方法 ============

/**
 * @brief 打开通知面板
 */
function openPanel() {
  showPanel.value = true;
}

/**
 * @brief 根据类型获取图标名称
 */
function getIconName(type: NotificationType): string {
  const iconMap: Record<NotificationType, string> = {
    info: 'info',
    warning: 'warning',
    success: 'check_circle',
    error: 'error',
    system: 'computer',
  };
  return iconMap[type];
}

/**
 * @brief 根据类型获取图标颜色
 */
function getIconColor(type: NotificationType): string {
  const colorMap: Record<NotificationType, string> = {
    info: 'info',
    warning: 'warning',
    success: 'positive',
    error: 'negative',
    system: 'primary',
  };
  return colorMap[type];
}

/**
 * @brief 格式化时间
 */
function formatTime(timeStr: string): string {
  const date = new Date(timeStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // 1分钟内
  if (diff < 60000) {
    return t('notification.justNow') || '刚刚';
  }
  // 1小时内
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)} ${t('notification.minutesAgo') || '分钟前'}`;
  }
  // 24小时内
  if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)} ${t('notification.hoursAgo') || '小时前'}`;
  }
  // 7天内
  if (diff < 604800000) {
    return `${Math.floor(diff / 86400000)} ${t('notification.daysAgo') || '天前'}`;
  }
  // 超过7天显示日期
  return date.toLocaleDateString('zh-CN');
}

/**
 * @brief 标记单个通知为已读
 */
function markAsRead(id: string) {
  const notification = notifications.value.find((n) => n.id === id);
  if (notification) {
    notification.is_read = true;
  }
}

/**
 * @brief 标记全部通知为已读
 */
function markAllAsRead() {
  notifications.value.forEach((n) => {
    n.is_read = true;
  });
}

/**
 * @brief 处理通知点击
 */
function handleNotificationClick(notification: Notification) {
  // 标记为已读
  if (!notification.is_read) {
    markAsRead(notification.id);
  }

  // 如果有链接则跳转
  if (notification.link) {
    void router.push(notification.link);
    showPanel.value = false;
  }
}

/**
 * @brief 跳转到通知页面
 */
function goToNotificationPage() {
  showPanel.value = false;
  void router.push('/notifications');
}
</script>

<style scoped>
.notification-panel {
  width: 380px;
  max-width: 100vw;
  height: 100vh;
  max-height: 100vh;
}

.notification-list {
  padding: 0;
}

/* 暗色主题适配 */
.body--dark .notification-panel {
  background: #1e1e1e;
}

.body--dark .q-item {
  background: #1e1e1e;
}

.body--dark .q-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.body--dark .bg-blue-1 {
  background: rgba(25, 118, 210, 0.1) !important;
}

.body--dark .q-tabs {
  background: #1e1e1e;
}

.body--dark .q-tab {
  color: #b0b0b0;
}

.body--dark .q-tab--active {
  color: #1976d2;
}

.body--dark .q-separator {
  color: #2d2d2d;
}
</style>
