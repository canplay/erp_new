<template>
  <q-page class="room-list-page">
    <!-- 未登录 -->
    <div v-if="!auth.isLoggedIn" class="flex flex-center column" style="height: 60vh">
      <q-icon name="chat" size="64px" color="grey-4" />
      <div class="text-h6 text-grey-6 q-mt-md">请先登录查看聊天室</div>
      <q-btn color="primary" label="去登录" to="/login" class="q-mt-sm" />
    </div>

    <template v-else>
      <!-- Tabs: 我的房间 / 发现公开房间 -->
      <q-tabs v-model="tab" class="q-mx-md" dense>
        <q-tab name="mine" icon="chat" label="我的房间" />
        <q-tab name="public" icon="explore" label="发现" />
      </q-tabs>
      <q-separator />

      <!-- 我的房间 -->
      <template v-if="tabMineVisible">
      <!-- 标题栏 -->
      <div class="row items-center justify-between q-pa-md q-pb-sm">
        <div class="text-h5 text-weight-bold">聊天室</div>
        <q-btn
          round
          color="primary"
          icon="add"
          size="md"
          @click="showCreateDialog = true"
        >
          <q-tooltip>创建房间</q-tooltip>
        </q-btn>
      </div>

      <!-- 搜索框 -->
      <div class="q-px-md q-pb-sm">
        <q-input
          v-model="searchQuery"
          placeholder="搜索房间..."
          outlined
          dense
          clearable
          debounce="300"
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </div>

      <!-- 房间列表 -->
      <q-scroll-area style="height: calc(100vh - 200px)">
        <!-- 加载状态 -->
        <div v-if="roomStore.isLoading && roomStore.rooms.length === 0" class="q-pa-lg text-center">
          <q-spinner-dots color="primary" size="40px" />
          <div class="text-grey-6 q-mt-sm">加载中...</div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="filteredRooms.length === 0 && !roomStore.isLoading" class="q-pa-xl text-center text-grey-5">
          <q-icon name="forum" size="64px" />
          <div class="text-h6 q-mt-md" v-if="searchQuery">未找到匹配的房间</div>
          <div class="text-h6 q-mt-md" v-else>暂无房间</div>
          <div class="text-caption q-mb-md" v-if="!searchQuery">创建第一个聊天室开始交流</div>
          <q-btn
            v-if="!searchQuery"
            color="primary"
            outline
            icon="add"
            label="创建房间"
            @click="showCreateDialog = true"
          />
        </div>

        <!-- 房间列表 -->
        <q-list v-else separator>
          <q-item
            v-for="room in filteredRooms"
            :key="room.roomId"
            clickable
            v-ripple
            class="room-item"
            @click="enterRoom(room.roomId)"
          >
            <q-item-section avatar>
              <q-avatar :color="room.isDirect ? 'teal' : 'primary'" text-color="white" size="40px">
                {{ getRoomInitial(room) }}
              </q-avatar>
            </q-item-section>

            <q-item-section>
              <q-item-label class="text-weight-medium">{{ room.name || '未命名房间' }}</q-item-label>
              <q-item-label caption lines="1">
                <q-icon
                  v-if="room.isDirect"
                  name="lock"
                  size="xs"
                  class="q-mr-xs"
                />
                <q-icon
                  v-else
                  name="groups"
                  size="xs"
                  class="q-mr-xs"
                />
                {{ room.memberCount }} 成员
                <template v-if="room.canonicalAlias">
                  · {{ room.canonicalAlias }}
                </template>
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <div class="text-caption text-grey-5">
                {{ room.lastEvent ? formatTime((room.lastEvent as any).getTs()) : '' }}
              </div>
            </q-item-section>
          </q-item>
        </q-list>

        <!-- 加载更多 -->
        <div v-if="roomStore.isLoading && roomStore.rooms.length > 0" class="text-center q-pa-md">
          <q-spinner-dots color="primary" size="24px" />
        </div>
      </q-scroll-area>

      <!-- 创建房间对话框 -->
      <q-dialog v-model="showCreateDialog" persistent>
        <q-card style="min-width: 400px">
          <q-card-section>
            <div class="text-h6">创建房间</div>
          </q-card-section>

          <q-card-section class="q-pt-none">
            <q-form @submit.prevent="handleCreateRoom" class="q-gutter-sm">
              <q-input
                v-model="newRoom.name"
                label="房间名称"
                outlined
                dense
                autofocus
                :rules="[val => !!val || '请输入房间名称']"
              />
              <q-input
                v-model="newRoom.topic"
                label="房间主题（可选）"
                outlined
                dense
              />
              <div class="row items-center">
                <q-checkbox v-model="newRoom.isPublic" label="公开房间" />
              </div>
            </q-form>
          </q-card-section>

          <q-card-actions align="right" class="text-primary">
            <q-btn flat label="取消" v-close-popup />
            <q-btn
              flat
              label="创建"
              color="primary"
              :loading="roomStore.isLoading"
              @click="handleCreateRoom"
            />
          </q-card-actions>
        </q-card>
      </q-dialog>
      </template>

      <!-- 公开房间 -->
      <div v-show="tabPublicVisible">
        <div class="q-pa-md">
          <div class="text-h5 text-weight-bold q-mb-md">发现公开房间</div>
          <q-input v-model="publicSearchTerm" placeholder="搜索公开房间..." outlined dense clearable debounce="400" @update:model-value="loadPublicRooms">
            <template v-slot:prepend><q-icon name="search" /></template>
            <template v-slot:append><q-spinner v-if="loadingPublic" size="xs" color="primary" /></template>
          </q-input>
        </div>
        <q-list v-if="publicRooms.length > 0" separator>
          <q-item v-for="r in publicRooms" :key="r.room_id" clickable v-ripple @click="joinPublicRoom(r.room_id)">
            <q-item-section avatar>
              <q-avatar color="primary" text-color="white" size="40px">{{ (r.name || r.room_id).charAt(0).toUpperCase() }}</q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label class="text-weight-medium">{{ r.name || '未命名' }}</q-item-label>
              <q-item-label caption>{{ r.room_id }} · {{ r.num_joined_members }} 成员</q-item-label>
            </q-item-section>
            <q-item-section side><q-btn flat round icon="login" size="sm" color="primary" /></q-item-section>
          </q-item>
        </q-list>
        <div v-else-if="!loadingPublic" class="text-center q-pa-xl text-grey-5">
          <q-icon name="explore" size="48px" /><div class="q-mt-sm">搜索 Matrix 公开房间</div>
        </div>
      </div>
      </template>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoomList } from '@erp-new-frontend-monorepo/composables/src/useRoomList';;
import { usePublicRooms } from '@erp-new-frontend-monorepo/composables/src/usePublicRooms';;

const {
  auth,
  roomStore,
  searchQuery,
  showCreateDialog,
  newRoom,
  filteredRooms,
  getRoomInitial,
  formatTime,
  enterRoom,
  handleCreateRoom,
} = useRoomList();

const {
  publicRooms,
  publicSearchTerm,
  loadingPublic,
  loadPublicRooms,
  joinPublicRoom,
} = usePublicRooms();

// State
const tab = ref<string>('mine');
const tabPublicVisible = computed(() => tab.value === 'public');
const tabMineVisible = computed(() => tab.value === 'mine');
</script>

<style scoped>
.room-list-page {
  background: #fafafa;
  min-height: 100vh;
}

.room-item {
  border-radius: 0;
  transition: background 0.15s;
}
.room-item:hover {
  background: #f0f0f0;
}

/* 移动端适配 */
@media (max-width: 600px) {
  .room-list-page {
    min-height: calc(100vh - 56px);
  }
  .room-item .q-item__section--avatar {
    min-width: 40px;
  }
}
</style>
