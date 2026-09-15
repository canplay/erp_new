<template>
  <q-page class="chat-page">
    <!-- 加载状态 -->
    <div v-if="!auth.isLoggedIn" class="flex flex-center column" style="height: 100%">
      <q-icon name="lock" size="64px" color="grey-4" />
      <div class="text-h6 text-grey-6 q-mt-md">请先登录</div>
      <q-btn color="primary" label="去登录" to="/login" class="q-mt-sm" />
    </div>

    <!-- 聊天界面 -->
    <template v-else>
      <!-- 顶部导航 -->
      <q-header elevated class="bg-primary text-white">
        <q-toolbar>
          <q-btn flat round dense icon="arrow_back" @click="goBack" />
          <q-avatar size="32px" class="q-mr-sm">
            <img v-if="room?.avatarUrl" :src="mxcToHttp(room.avatarUrl)" />
            <span v-else class="text-white">{{ roomInitial }}</span>
          </q-avatar>
          <q-toolbar-title class="text-body1">
            <div class="text-weight-medium">{{ displayRoomName }}</div>
            <div class="text-caption text-white text-opacity-70">
              {{ room?.memberCount ?? 0 }} 成员
              <template v-if="typingUsers.length > 0">
                · <q-spinner-dots size="xs" /> {{ typingUsers[0] }} 正在输入...
              </template>
            </div>
          </q-toolbar-title>
          <q-btn flat round dense icon="more_vert">
            <q-menu auto-close>
              <q-list dense style="min-width: 160px">
                <q-item clickable @click="showRoomInfo = true">
                  <q-item-section side><q-icon name="info" size="xs" /></q-item-section>
                  <q-item-section>房间信息</q-item-section>
                </q-item>
                <q-item clickable @click="handleLeaveRoom">
                  <q-item-section side><q-icon name="exit_to_app" size="xs" color="red" /></q-item-section>
                  <q-item-section class="text-red">离开房间</q-item-section>
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>
        </q-toolbar>
      </q-header>

      <!-- 消息区 -->
      <q-scroll-area
        ref="scrollAreaRef"
        class="chat-messages"
        :thumb-style="thumbStyle"
        @scroll="onScroll"
      >
        <div ref="scrollAnchorRef" class="q-pa-sm">
          <!-- 加载更多 -->
          <div v-if="hasMoreMessages" class="text-center q-pa-sm">
            <q-btn
              flat
              dense
              size="sm"
              color="primary"
              label="加载更多消息"
              :loading="messageStore.isLoading"
              @click="loadHistory"
            />
          </div>

          <!-- 消息列表 -->
          <div
            v-for="msg in messages"
            :key="msg.eventId"
            class="q-my-xs"
          >
            <QuasarChatMessage
              :message="msg"
              :is-sender="msg.sender === auth.userId"
              :sender-name="getDisplayName(msg.sender)"
              :sender-avatar="getAvatarUrl(msg.sender)"
            />
            <!-- 消息操作行 -->
            <div class="row justify-center q-gutter-xs q-mt-xs" style="opacity:0.6">
              <q-btn dense flat round icon="add_reaction" size="xs" color="grey-6"
                @click="showReactionPicker(msg.eventId)" />
              <q-btn v-if="msg.sender === auth.userId" dense flat round icon="edit" size="xs" color="grey-6"
                @click="startEdit(msg)" />
              <q-btn v-if="msg.sender === auth.userId" dense flat round icon="delete" size="xs" color="grey-6"
                @click="confirmDelete(msg.eventId)" />
              <q-btn dense flat round icon="reply" size="xs" color="grey-6"
                @click="replyTo(msg)" />
            </div>
          </div>

          <!-- 空状态 -->
          <div
            v-if="messages.length === 0 && !messageStore.isLoading"
            class="text-center q-pa-xl text-grey-5"
          >
            <q-icon name="chat_bubble_outline" size="48px" />
            <div class="q-mt-sm">暂无消息</div>
            <div class="text-caption">发送第一条消息开始聊天</div>
          </div>

          <!-- 加载中 -->
          <div v-if="messageStore.isLoading && messages.length === 0" class="text-center q-pa-lg">
            <q-spinner-dots color="primary" size="40px" />
          </div>

          <!-- 底部锚点 -->
          <div ref="bottomAnchorRef" />
        </div>

      <!-- 滚动到底部按钮（用户滚动到上方时显示） -->
      <q-btn
        v-if="!autoScroll && messages.length > 0"
        fab
        icon="keyboard_arrow_down"
        color="primary"
        size="sm"
        class="scroll-bottom-btn"
        @click="scrollToBottom"
      >
        <q-tooltip>滚动到底部</q-tooltip>
      </q-btn>
      </q-scroll-area>

      <!-- 底部输入区 -->
      <q-footer class="bg-white">
        <q-toolbar class="q-pa-xs">
          <q-btn flat round icon="attach_file" size="sm" color="grey-7" @click="openFilePicker" />
          <q-btn flat round icon="mic" size="sm" color="grey-7" @click="startVoiceRecording" />
          <q-input
            v-model="inputText"
            placeholder="输入消息..."
            outlined
            dense
            class="col q-mx-xs"
            maxlength="10000"
            autogrow
            @keydown.enter.prevent="sendMessage"
          >
            <template v-slot:append>
              <q-btn
                flat
                round
                :icon="inputText.trim() ? 'send' : 'emoji_emotions'"
                :color="inputText.trim() ? 'primary' : 'grey-5'"
                :disable="messageStore.isSending"
                @click="inputText.trim() ? sendMessage() : openEmojiPicker()"
              />
            </template>
          </q-input>
        </q-toolbar>
      </q-footer>

      <!-- 表情面板 -->
      <q-dialog v-model="showEmojiPicker" position="bottom">
        <q-card style="width: 100%; max-width: 400px">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-weight-bold">选择表情</div>
            <q-space />
            <q-btn flat round dense icon="close" v-close-popup />
          </q-card-section>
          <q-card-section class="q-pt-md">
            <div class="row q-gutter-xs">
              <span
                v-for="emoji in emojis"
                :key="emoji"
                class="emoji-item cursor-pointer"
                @click="insertEmoji(emoji)"
              >{{ emoji }}</span>
            </div>
          </q-card-section>
        </q-card>
      </q-dialog>

      <!-- 房间信息对话框 -->
      <q-dialog v-model="showRoomInfo">
        <q-card style="min-width: 350px">
          <q-card-section>
            <div class="text-h6">{{ displayRoomName }}</div>
          </q-card-section>
          <q-card-section class="q-pt-none">
            <q-item>
              <q-item-section>
                <q-item-label caption>房间 ID</q-item-label>
                <q-item-label class="text-caption text-grey">{{ props.roomId }}</q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>成员数</q-item-label>
                <q-item-label>{{ room?.memberCount ?? '?' }}</q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-btn flat icon="person_add" label="邀请成员" color="primary" @click="showInviteDialog = true" />
              </q-item-section>
            </q-item>
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="关闭" color="primary" v-close-popup />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <!-- 邀请成员对话框 -->
      <q-dialog v-model="showInviteDialog" persistent>
        <q-card style="min-width: 350px">
          <q-card-section>
            <div class="text-h6">邀请成员</div>
          </q-card-section>
          <q-card-section class="q-pt-none">
            <q-input v-model="inviteUserId" label="用户 Matrix ID" placeholder="@username:server.tld" outlined dense autofocus />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="取消" color="grey" v-close-popup />
            <q-btn flat label="邀请" color="primary" :loading="isInviting" @click="handleInvite" />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <!-- 消息搜索对话框 -->
      <q-dialog v-model="showSearchDialog">
        <q-card style="min-width: 400px; max-width: 90vw">
          <q-card-section>
            <div class="text-h6">搜索消息</div>
          </q-card-section>
          <q-card-section class="q-pt-none">
            <q-input v-model="searchQuery" placeholder="输入搜索关键词..." outlined dense autofocus @keydown.enter="handleSearch" />
          </q-card-section>
          <q-card-section v-if="searchResults.length > 0" style="max-height: 300px" class="scroll">
            <q-item v-for="r in searchResults" :key="r.eventId" clickable dense>
              <q-item-section>
                <q-item-label caption>{{ r.sender }}</q-item-label>
                <q-item-label>{{ r.content?.body }}</q-item-label>
              </q-item-section>
            </q-item>
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="关闭" color="primary" v-close-popup />
          </q-card-actions>
        </q-card>
      </q-dialog>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useRoomStore } from '@/stores/room';
import { useMessageStore } from '@/stores/message';
import QuasarChatMessage from '@erp-new-frontend-monorepo/components/src/QuasarChatMessage.vue';
import { useChatSend } from '@erp-new-frontend-monorepo/composables/src/useChatSend';
import { useChatScroll } from '@erp-new-frontend-monorepo/composables/src/useChatScroll';
import { useChatEvents } from '@erp-new-frontend-monorepo/composables/src/useChatEvents';
import { useChatMessageOps } from '@erp-new-frontend-monorepo/composables/src/useChatMessageOps';
import { useChatActions } from '@erp-new-frontend-monorepo/composables/src/useChatActions';

const router = useRouter();
const auth = useAuthStore();
const roomStore = useRoomStore();
const messageStore = useMessageStore();

// Props
const props = defineProps<{ roomId: string }>();

// Composables
const { inputText, showEmojiPicker, sendMessage } = useChatSend(props.roomId);
const { bottomAnchorRef, autoScroll, scrollToBottom, onScroll, loadHistory } = useChatScroll(props.roomId);
const { typingUsers, bindEvents, unbindEvents } = useChatEvents(props.roomId);
const { showReactionPicker, startEdit, confirmDelete, replyTo } = useChatMessageOps(props.roomId, inputText);
const { openFilePicker, startVoiceRecording, handleLeaveRoom,
  inviteUserId, showInviteDialog, isInviting, handleInvite,
  showSearchDialog, searchQuery, searchResults, handleSearch,
  emojis } = useChatActions(props.roomId);

// State
const scrollAreaRef = ref<{ scroll: (options: Record<string, unknown>) => void } | null>(null);
const scrollAnchorRef = ref<HTMLElement | null>(null);
const showRoomInfo = ref(false);

// Computed
const room = computed(() => roomStore.currentRoom);
const displayRoomName = computed(() => {
  if (room.value?.name) return room.value.name;
  return props.roomId || '聊天室';
});
const roomInitial = computed(() => displayRoomName.value.charAt(0).toUpperCase());

const messages = computed(() => messageStore.getMessages(props.roomId));
const hasMoreMessages = computed(() => !!(messageStore.hasMoreHistory as Record<string, boolean | undefined>)[props.roomId]);
const thumbStyle = { opacity: '0.3' as string };

// --- Lifecycle ---
onMounted(async () => {
  roomStore.setCurrentRoom(props.roomId);

  if (auth.isLoggedIn) {
    try {
      await messageStore.loadMessages(props.roomId, 30);
    } catch {
      // 静默
    }

    bindEvents();
    void scrollToBottom();
  }
});

onUnmounted(() => {
  roomStore.setCurrentRoom(null);
  unbindEvents();
});

// Watch for new messages to auto-scroll
watch(
  () => messages.value.length,
  () => {
    if (autoScroll.value) {
      void nextTick(scrollToBottom);
    }
  },
);

// --- Event Handlers (via useChatEvents) ---
// onRoomTimeline, onTyping → provided by useChatEvents

// --- Navigation ---
function goBack() {
  void router.push('/rooms');
}

// --- File / Voice / Video (via useChatActions) ---
// openFilePicker, startVoiceRecording → provided by useChatActions

// --- Room Actions (via useChatActions) ---
// handleLeaveRoom, handleInvite → provided by useChatActions

// --- Emoji (via useChatActions) ---
// emojis → provided by useChatActions

// --- Message Ops (via useChatMessageOps) ---
// showReactionPicker, startEdit, confirmDelete, replyTo → provided by useChatMessageOps

// --- Search (via useChatActions) ---
// searchQuery, showSearchDialog, searchResults, isSearching, handleSearch → provided by useChatActions

// --- User Info ---
function getDisplayName(userId: string): string {
  if (!userId) return '未知用户';
  return (userId.split(':')[0] ?? '').replace('@', '');
}

function getAvatarUrl(userId: string): string {
  return userId ? '' : '';
}

function mxcToHttp(mxcUrl: string): string {
  if (!mxcUrl) return '';
  const parts = mxcUrl.replace('mxc://', '').split('/');
  if (parts.length !== 2) return mxcUrl;
  const base = import.meta.env.VITE_TUWUNEL_URL || 'http://localhost:8008';
  return `${base}/_matrix/media/v3/download/${parts[0]}/${parts[1]}`;
}

// ── Message Operations (via useChatMessageOps) ──
// showReactionPicker, startEdit, confirmDelete, replyTo, editingEventId, editingText → provided by useChatMessageOps

// ── 邀请成员 (via useChatActions) ──
// inviteUserId, showInviteDialog, isInviting, handleInvite → provided by useChatActions

// ── 消息搜索 (via useChatActions) ──
// showSearchDialog, searchQuery, searchResults, isSearching, handleSearch → provided by useChatActions

// ── 打开表情选择器 (via useChatSend) ──
function openEmojiPicker() {
  showEmojiPicker.value = true
}

function insertEmoji(emoji: string) {
  inputText.value += emoji
  showEmojiPicker.value = false
}
</script>

<style scoped>
.chat-page {
  height: calc(100vh - 50px);
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
}

.emoji-item {
  font-size: 24px;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.2s;
}
.emoji-item:hover {
  background: #e0e0e0;
}

.scroll-bottom-btn {
  position: absolute;
  bottom: 80px;
  right: 20px;
  z-index: 100;
}

/* 移动端适配 */
@media (max-width: 600px) {
  .chat-page {
    height: calc(100vh - 56px);
  }
  .emoji-item {
    font-size: 28px;
    padding: 6px;
  }
}
</style>
