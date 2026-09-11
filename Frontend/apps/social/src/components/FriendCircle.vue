<template>
  <q-card class="friend-circle-card q-mb-md" flat bordered>
    <!-- 头部：用户信息和时间 -->
    <q-card-section class="q-pb-xs row items-center no-wrap">
      <q-avatar :color="avatarColor" text-color="white" size="40px" class="q-mr-sm">
        {{ userInitial }}
      </q-avatar>
      <div class="col">
        <div class="text-weight-medium">{{ userName }}</div>
        <div class="text-caption text-grey-6">{{ formattedTime }}</div>
      </div>
      <q-btn flat round dense icon="more_vert" size="sm" color="grey-6">
        <q-menu auto-close>
          <q-list dense>
            <q-item v-if="isOwner" clickable @click="$emit('delete', post.eventId)">
              <q-item-section side><q-icon name="delete" color="red" size="xs" /></q-item-section>
              <q-item-section class="text-red">删除</q-item-section>
            </q-item>
          </q-list>
        </q-menu>
      </q-btn>
    </q-card-section>

    <!-- 文字内容 -->
    <q-card-section class="q-py-sm" v-if="postContent">
      <div class="text-body2 text-pre-wrap">{{ postContent }}</div>
    </q-card-section>

    <!-- 图片 -->
    <q-card-section v-if="imageUrl" class="q-pt-none q-pb-sm">
      <q-img
        :src="imageUrl"
        class="rounded-borders"
        style="max-height: 300px"
        @click="previewImage"
      />
    </q-card-section>

    <!-- 互动栏 -->
    <q-card-actions class="q-px-md q-py-sm" align="around">
      <q-btn
        flat
        dense
        :icon="liked ? 'favorite' : 'favorite_border'"
        :color="liked ? 'red' : 'grey-7'"
        :label="likeCount > 0 ? String(likeCount) : ''"
        @click="toggleLike"
      />
      <q-btn
        flat
        dense
        icon="chat_bubble_outline"
        color="grey-7"
        :label="commentCount > 0 ? String(commentCount) : ''"
        @click="toggleComment"
      />
      <q-btn flat dense icon="share" color="grey-7" @click="sharePost" />
    </q-card-actions>

    <!-- 评论区域 -->
    <q-slide-transition>
      <div v-if="showComments" class="bg-grey-1 q-pa-sm">
        <q-list dense separator v-if="comments.length > 0">
          <q-item v-for="(comment, idx) in comments" :key="idx" dense class="q-py-xs">
            <q-item-section avatar class="q-pr-xs">
              <q-avatar size="24px" color="primary" text-color="white">
                {{ getInitial(comment.sender) }}
              </q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label class="text-caption text-weight-medium">
                {{ getDisplayName(comment.sender) }}
              </q-item-label>
              <q-item-label class="text-caption">{{ comment.body }}</q-item-label>
            </q-item-section>
          </q-item>
        </q-list>

        <div class="row q-mt-sm q-gutter-xs">
          <q-input
            v-model="newComment"
            placeholder="写评论..."
            dense
            outlined
            class="col"
            maxlength="500"
            @keydown.enter.prevent="submitComment"
          />
          <q-btn flat round icon="send" color="primary" size="sm" @click="submitComment" />
        </div>
      </div>
    </q-slide-transition>

    <!-- 图片预览 -->
    <q-dialog v-model="showPreview">
      <q-img :src="imageUrl" style="max-width: 90vw; max-height: 90vh" />
    </q-dialog>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';

export interface FriendCirclePost {
  eventId: string
  roomId: string
  sender: string
  content: {
    body?: string
    msgtype?: string
    url?: string
    info?: { mimetype?: string }
    ['m.relates_to']?: Record<string, unknown>
  }
  type: string
  timestamp: number
  likes?: string[]
  comments?: Array<{ sender: string; body: string; timestamp: number }>
}

const props = defineProps<{
  post: FriendCirclePost
}>();

defineEmits<{
  delete: [eventId: string]
}>();

const auth = useAuthStore();

// State
const liked = ref(false);
const showComments = ref(false);
const showPreview = ref(false);
const newComment = ref('');
const comments = ref<Array<{ sender: string; body: string; timestamp: number }>>(
  props.post.comments || [],
);

// Computed
const isOwner = computed(() => props.post.sender === auth.userId);
const postContent = computed(() => props.post.content?.body || '');
const imageUrl = computed(() => {
  if (props.post.content?.url) {
    return mxcToHttp(props.post.content.url);
  }
  return '';
});
const likeCount = computed(() => props.post.likes?.length || 0);
const commentCount = computed(() => comments.value.length);

const userName = computed(() => getDisplayName(props.post?.sender ?? ''));
const userInitial = computed(() => (userName.value || '?').charAt(0).toUpperCase());
const avatarColor = computed(() => {
  const colors = ['primary', 'teal', 'orange', 'purple', 'cyan', 'pink'];
  const sender = String(props.post?.sender ?? '');
  const hash = sender.split('').reduce((a, c) => a + c.charCodeAt(0), 0);
  return colors[hash % colors.length];
});

const formattedTime = computed(() => {
  const date = new Date(props.post.timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60000) return '刚刚';
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
  if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`;

  return date.toLocaleDateString('zh-CN', {
    year: 'numeric', month: 'short', day: 'numeric',
  });
});

// Methods
function getDisplayName(userId: string): string {
  if (!userId) return '未知用户';
  return (userId.split(':')[0] ?? '').replace('@', '');
}

function getInitial(userId: string): string {
  const name = getDisplayName(userId ?? '');
  return name.charAt(0).toUpperCase() || '?';
}

function mxcToHttp(mxcUrl: string): string {
  if (!mxcUrl) return '';
  const parts = mxcUrl.replace('mxc://', '').split('/');
  if (parts.length !== 2) return mxcUrl;
  const base = import.meta.env.VITE_TUWUNEL_URL || 'http://localhost:8008';
  return `${base}/_matrix/media/v3/download/${parts[0]}/${parts[1]}`;
}

function previewImage() {
  showPreview.value = true;
}

function toggleLike() {
  if (!auth.isLoggedIn) return;
  liked.value = !liked.value;
  // TODO: 使用 Matrix 自定义事件发送点赞
}

function toggleComment() {
  showComments.value = !showComments.value;
}

function submitComment() {
  const text = newComment.value.trim();
  if (!text || !auth.isLoggedIn) return;

  comments.value.push({
    sender: auth.userId!,
    body: text,
    timestamp: Date.now(),
  });
  newComment.value = '';
  // TODO: 使用 Matrix 关系事件发送评论
}

function sharePost() {
  // 复制链接到剪贴板
  navigator.clipboard.writeText(window.location.href).catch(() => {});
}
</script>

<style scoped>
.friend-circle-card {
  border-radius: 12px;
}

.text-pre-wrap {
  white-space: pre-wrap;
  word-break: break-word;
}

.rounded-borders {
  border-radius: 8px;
}
</style>
