<template>
  <div class="chat-message-wrapper" :class="{ 'is-sender': isSender }">
    <div class="message-row" :class="{ 'flex-reverse': isSender }">
      <!-- 头像（对方消息显示） -->
      <q-avatar v-if="!isSender" size="36px" class="avatar">
        <img v-if="senderAvatar" :src="senderAvatar" />
        <span v-else class="text-white bg-primary rounded-borders full-width full-height flex flex-center">
          {{ displayNameInitial }}
        </span>
      </q-avatar>

      <!-- 消息气泡 -->
      <div class="message-bubble" :class="{ 'bubble-sender': isSender, 'bubble-receiver': !isSender }">
        <!-- 发送者名称（对方消息显示） -->
        <div v-if="!isSender" class="sender-name text-caption text-grey-6 q-mb-xs">
          {{ senderName || '未知用户' }}
        </div>

        <!-- 文本消息 -->
        <div v-if="msgType === 'm.text' || msgType === 'm.notice'" class="message-text text-body2">
          {{ messageBody }}
        </div>

        <!-- 富文本消息 -->
        <div v-else-if="msgType === 'm.emote'" class="message-text text-body2">
          * {{ senderName }} {{ messageBody }}
        </div>

        <!-- 图片消息 -->
        <div v-else-if="msgType === 'm.image'" class="image-message">
          <q-img
            :src="mxcToHttp(msgContent().url || '')"
            class="message-image"
            style="max-width: 240px; max-height: 240px"
            @click="previewImage(mxcToHttp(msgContent().url || ''))"
          />
          <div v-if="messageBody" class="text-caption q-mt-xs">{{ messageBody }}</div>
        </div>

        <!-- 语音消息 -->
        <div v-else-if="msgType === 'm.audio' || (msgContent().url && msgContent().info?.mimetype?.startsWith('audio/'))">
          <VoiceMessage :content="message.content" :is-sender="isSender" />
        </div>

        <!-- 文件消息 -->
        <div v-else-if="msgType === 'm.file'" class="file-message row items-center q-gutter-sm">
          <q-icon name="insert_drive_file" size="32px" :color="isSender ? 'white' : 'primary'" />
          <div>
            <div class="text-body2">{{ messageBody || '文件' }}</div>
            <div class="text-caption" :class="isSender ? 'text-white text-opacity-70' : 'text-grey-6'">
              {{ formatFileSize(msgContent().info?.size) }}
            </div>
          </div>
        </div>

        <!-- 视频消息 -->
        <div v-else-if="msgType === 'm.video'" class="video-message">
          <video
            :src="mxcToHttp(msgContent().url || '')"
            controls
            style="max-width: 240px; max-height: 200px; border-radius: 8px;"
          />
        </div>

        <!-- 未知消息类型 -->
        <div v-else class="message-text text-body2 text-grey">
          <q-icon name="help_outline" size="xs" class="q-mr-xs" />
          暂不支持的消息类型
        </div>

        <!-- 消息时间和状态 -->
        <div class="message-footer row items-center no-wrap q-mt-xs" :class="{ 'justify-end': isSender }">
          <span class="text-caption" :class="isSender ? 'text-white text-opacity-60' : 'text-grey-6'">
            {{ formattedTime }}
          </span>
          <span v-if="isSender && message.status" class="text-caption q-ml-xs" :class="statusClass">
            <q-icon :name="statusIcon" size="xs" />
          </span>
        </div>
      </div>

      <!-- 头像（自己的消息显示） -->
      <q-avatar v-if="isSender" size="36px" class="avatar">
        <img v-if="senderAvatar" :src="senderAvatar" />
        <span v-else class="text-white bg-primary rounded-borders full-width full-height flex flex-center">
          {{ displayNameInitial }}
        </span>
      </q-avatar>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useQuasar } from 'quasar';
import VoiceMessage from './VoiceMessage.vue';
import type { RoomMessage } from '@/services';

const props = defineProps<{
  message: RoomMessage
  isSender?: boolean
  senderAvatar?: string
  senderName?: string
}>();

const $q = useQuasar();

// Helper: cast content to typed accessor
function msgContent() {
  return props.message.content as {
    body?: string
    msgtype?: string
    url?: string
    info?: { mimetype?: string; size?: number }
  }
}

// Computed
const msgType = computed(() => {
  return msgContent().msgtype || props.message.type;
});

const messageBody = computed(() => {
  return msgContent().body || '';
});

const displayNameInitial = computed(() => {
  const name = props.senderName || '?';
  return name.charAt(0).toUpperCase();
});

const formattedTime = computed(() => {
  const date = new Date(props.message.timestamp);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  if (isToday) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
});

const statusClass = computed(() => {
  switch (props.message.status) {
    case 'sending': return 'text-grey-5';
    case 'sent': return 'text-grey-5';
    case 'delivered': return 'text-blue-3';
    case 'read': return 'text-blue-3';
    default: return 'text-grey-5';
  }
});

const statusIcon = computed(() => {
  switch (props.message.status) {
    case 'sending': return 'schedule';
    case 'sent': return 'check';
    case 'delivered': return 'done_all';
    case 'read': return 'done_all';
    default: return 'check';
  }
});

// Methods
function mxcToHttp(mxcUrl: string): string {
  if (!mxcUrl || !mxcUrl.startsWith('mxc://')) return mxcUrl;
  const parts = mxcUrl.replace('mxc://', '').split('/');
  if (parts.length !== 2) return mxcUrl;
  const base = import.meta.env.VITE_TUWUNEL_URL || 'http://localhost:8008';
  return `${base}/_matrix/media/v3/download/${parts[0]}/${parts[1]}`;
}

function formatFileSize(bytes?: number): string {
  if (!bytes) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

function previewImage(url: string) {
  $q.dialog({
    title: '',
    message: `<div class="text-center"><img src="${url}" style="max-width: 100%; max-height: 70vh; border-radius: 8px;" /></div>`,
    html: true,
    ok: '关闭',
    persistent: true,
  });
}
</script>

<style scoped>
.chat-message-wrapper {
  margin: 4px 8px;
}

.message-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.flex-reverse {
  flex-direction: row-reverse;
}

.avatar {
  flex-shrink: 0;
  margin-top: 4px;
}

.message-bubble {
  max-width: 70%;
  padding: 8px 12px;
  border-radius: 16px;
  word-break: break-word;
}

.bubble-receiver {
  background: #ffffff;
  border-top-left-radius: 4px;
}

.bubble-sender {
  background: var(--q-primary, #1976d2);
  color: white;
  border-top-right-radius: 4px;
}

.message-text {
  white-space: pre-wrap;
}

.message-image {
  border-radius: 8px;
  cursor: pointer;
}

.message-image:hover {
  opacity: 0.9;
}
</style>
