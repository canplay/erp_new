<template>
  <q-page class="contact-page">
    <div v-if="!auth.isLoggedIn" class="flex flex-center column" style="height: 60vh">
      <q-icon name="contacts" size="64px" color="grey-4" />
      <div class="text-h6 text-grey-6 q-mt-md">请先登录</div>
      <q-btn color="primary" label="去登录" to="/login" class="q-mt-sm" />
    </div>

    <template v-else>
      <div class="row items-center q-pa-md q-pb-sm">
        <div class="text-h5 text-weight-bold">通讯录</div>
        <q-space />
        <q-btn flat round icon="refresh" @click="contactStore.loadContacts()">
          <q-tooltip>刷新</q-tooltip>
        </q-btn>
      </div>

      <!-- 搜索用户 -->
      <div class="q-px-md q-pb-md">
        <q-input
          v-model="searchQuery"
          placeholder="搜索 Matrix 用户..."
          outlined
          dense
          clearable
          debounce="400"
          @update:model-value="onSearch as any"
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
          <template v-slot:append>
            <q-spinner v-if="contactStore.isSearching" size="xs" color="primary" />
          </template>
        </q-input>
      </div>

      <!-- 搜索结果 -->
      <template v-if="searchQuery.trim()">
        <div class="q-px-md q-py-xs text-caption text-grey-6">
          搜索结果 ({{ contactStore.searchResults.length }})
        </div>
        <q-list separator v-if="contactStore.searchResults.length > 0">
          <q-item
            v-for="user in contactStore.searchResults"
            :key="user.userId"
            clickable
            v-ripple
            @click="startDirectChat(user)"
          >
            <q-item-section avatar>
              <q-avatar color="primary" text-color="white" size="40px">
                {{ getInitial(user) }}
              </q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label>{{ user.displayName || user.userId }}</q-item-label>
              <q-item-label caption>{{ user.userId }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn flat round icon="chat" size="sm" color="primary" />
            </q-item-section>
          </q-item>
        </q-list>
        <div v-else-if="!contactStore.isSearching" class="text-center q-pa-lg text-grey-5">
          未找到匹配的用户
        </div>
      </template>

      <!-- 联系人列表 -->
      <template v-else>
        <div class="q-px-md q-py-xs text-caption text-grey-6">
          我的联系人 ({{ contactStore.contacts.length }})
        </div>

        <div v-if="contactStore.isLoading && contactStore.contacts.length === 0" class="text-center q-pa-lg">
          <q-spinner-dots color="primary" size="40px" />
        </div>

        <div v-else-if="contactStore.contacts.length === 0" class="text-center q-pa-xl text-grey-5">
          <q-icon name="person_add" size="64px" />
          <div class="text-h6 q-mt-md">暂无联系人</div>
          <div class="text-caption q-mb-md">搜索并添加 Matrix 用户开始聊天</div>
          <q-btn color="primary" outline icon="search" label="搜索用户" />
        </div>

        <q-list separator v-else>
          <ContactCard
            v-for="contact in contactStore.contacts"
            :key="contact.userId"
            :contact="contact"
            @chat="startDirectChat"
          />
        </q-list>
      </template>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useContactStore } from '@/stores/contact';
import { useRoomStore } from '@/stores/room';
import { useQuasar } from 'quasar';
import ContactCard from '@erp-new-frontend-monorepo/components/src/ContactCard.vue';
import type { UserInfo } from '@/services';
import type { Contact } from '@/stores/contact';

const $q = useQuasar();
const router = useRouter();
const auth = useAuthStore();
const contactStore = useContactStore();
const roomStore = useRoomStore();

const searchQuery = ref('');

onMounted(() => {
  if (auth.isLoggedIn) {
    void contactStore.loadContacts();
  }
});

function getInitial(user: UserInfo): string {
  return (user.displayName || user.userId).charAt(0).toUpperCase();
}

function onSearch(value: string) {
  if (value.trim()) {
    void contactStore.searchUsers(value);
  } else {
    contactStore.clearSearch();
  }
}

async function startDirectChat(user: UserInfo | Contact) {
  try {
    const contact = contactStore.contacts.find((c) => c.userId === user.userId);
    if (contact?.directRoomId) {
      void router.push(`/chat/${contact.directRoomId}`);
      return;
    }

    const roomId = await roomStore.createDirectRoom(user.userId);
    $q.notify({ type: 'positive', message: `已创建与 ${user.displayName || user.userId} 的聊天` });
    void router.push(`/chat/${roomId}`);
  } catch (e: unknown) {
    $q.notify({
      type: 'negative',
      message: '创建聊天失败: ' + (e instanceof Error ? e.message : '未知错误'),
    });
  }
}
</script>

<style scoped>
.contact-page {
  background: #fafafa;
  min-height: 100vh;
}
</style>
