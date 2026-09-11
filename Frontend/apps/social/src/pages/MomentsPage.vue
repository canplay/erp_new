<template>
  <q-page class="moments-page">
    <div class="q-pa-md">
      <div class="text-h5 q-mb-md text-weight-bold">朋友圈</div>

      <!-- 发朋友圈 -->
      <q-card class="q-mb-md" flat bordered>
        <q-card-section class="row items-center no-wrap">
          <q-avatar size="40px" color="primary" text-color="white" class="q-mr-sm">
            {{ displayNameInitial }}
          </q-avatar>
          <q-input
            v-model="newPostContent"
            placeholder="分享新鲜事..."
            class="col"
            dense
            outlined
            autogrow
            maxlength="2000"
            :disable="!auth.isLoggedIn"
          />
        </q-card-section>
        <q-card-actions align="right" class="q-pa-sm">
          <q-btn color="primary" label="发布" :disable="!newPostContent.trim() || !auth.isLoggedIn" @click="submitPost" />
        </q-card-actions>
      </q-card>

      <!-- 朋友圈列表 -->
      <div v-if="auth.isLoggedIn">
        <FriendCircle
          v-for="post in posts"
          :key="post.eventId"
          :post="post"
          @delete="deletePost"
        />
        <div v-if="posts.length === 0" class="text-center q-pa-xl text-grey-5">
          <q-icon name="photo_library" size="64px" />
          <div class="text-h6 q-mt-md">暂无动态</div>
          <div class="text-caption">分享你的新鲜事吧</div>
        </div>
      </div>

      <div v-else class="text-center q-pa-xl text-grey-5">
        <q-icon name="lock" size="64px" />
        <div class="text-h6 q-mt-md">请先登录</div>
        <q-btn color="primary" label="去登录" to="/login" />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { MatrixService } from '@/services';
import FriendCircle from '@/components/FriendCircle.vue';
import type { FriendCirclePost } from '@/components/FriendCircle.vue';

const ACCOUNT_DATA_KEY = 'social.moments';

const auth = useAuthStore();
const newPostContent = ref('');
const posts = ref<FriendCirclePost[]>([]);

const displayName = computed(() => {
  if (!auth.userId) return '?';
  return (auth.userId.split(':')[0] ?? '').replace('@', '');
});

const displayNameInitial = computed(() => displayName.value.charAt(0).toUpperCase() || '?');

// Load posts from server account_data on mount
onMounted(async () => {
  if (auth.isLoggedIn) {
    await loadPosts();
  }
});

// Reload when login state changes
watch(() => auth.isLoggedIn, async (val) => {
  if (val) await loadPosts();
  else posts.value = [];
});

async function loadPosts() {
  try {
    const ms = MatrixService.getInstance();
    const raw = await ms.getAccountDataFromServer(ACCOUNT_DATA_KEY);
    if (raw?.posts && Array.isArray(raw.posts)) {
      posts.value = raw.posts as FriendCirclePost[];
    }
  } catch {
    // Silent fallback: try local cache
    try {
      const cached = MatrixService.getInstance().getAccountData(ACCOUNT_DATA_KEY);
      if (cached?.posts) {
        posts.value = cached.posts as FriendCirclePost[];
      }
    } catch {
      // ignore
    }
  }
}

async function savePosts() {
  try {
    const ms = MatrixService.getInstance();
    await ms.setAccountData(ACCOUNT_DATA_KEY, { posts: posts.value });
  } catch (e: unknown) {
    console.error('Failed to persist moments:', e);
  }
}

async function submitPost() {
  if (!newPostContent.value.trim() || !auth.isLoggedIn) return;

  if (!auth.userId) return;

  const post: FriendCirclePost = {
    eventId: `moment_${Date.now()}`,
    roomId: '',
    sender: auth.userId,
    content: { body: newPostContent.value, msgtype: 'm.text' },
    type: 'm.room.message',
    timestamp: Date.now(),
  };

  posts.value.unshift(post);
  newPostContent.value = '';
  await savePosts();
}

async function deletePost(eventId: string) {
  posts.value = posts.value.filter((p) => p.eventId !== eventId);
  await savePosts();
}
</script>
