/**
 * @file useRoomList.ts
 * @description RoomListPage 房间列表/创建/过滤逻辑 — 从 RoomListPage.vue 拆分
 */

import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useAuthStore } from '@/stores/auth';
import { useRoomStore } from '@/stores/room';

export function useRoomList() {
  const $q = useQuasar();
  const router = useRouter();
  const auth = useAuthStore();
  const roomStore = useRoomStore();

  const searchQuery = ref('');
  const showCreateDialog = ref(false);
  const newRoom = ref({ name: '', topic: '', isPublic: true });

  // Lifecycle
  onMounted(() => {
    if (auth.isLoggedIn) {
      roomStore.loadRooms();
    }
  });

  // Computed
  const filteredRooms = computed(() => {
    const rooms = roomStore.sortedRooms;
    if (!searchQuery.value.trim()) return rooms;

    const q = searchQuery.value.toLowerCase();
    return rooms.filter(
      (r) =>
        r.name?.toLowerCase().includes(q) ||
        (r.canonicalAlias ?? '')?.toLowerCase().includes(q) ||
        r.roomId?.toLowerCase().includes(q),
    );
  });

  // Methods
  function getRoomInitial(room: { name?: string; roomId?: string }): string {
    if (room.name) return room.name.charAt(0).toUpperCase();
    return '#';
  }

  function formatTime(ts: number): string {
    const date = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;

    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
  }

  function enterRoom(roomId: string) {
    void router.push(`/chat/${roomId}`);
  }

  async function handleCreateRoom() {
    if (!newRoom.value.name.trim()) return;

    try {
      const roomId = await roomStore.createRoom(
        newRoom.value.name,
        newRoom.value.topic || undefined,
      );

      showCreateDialog.value = false;
      newRoom.value = { name: '', topic: '', isPublic: true };

      $q.notify({ type: 'positive', message: '房间创建成功' });

      void router.push(`/chat/${roomId}`);
    } catch (e: unknown) {
      $q.notify({
        type: 'negative',
        message: '创建失败: ' + (e instanceof Error ? e.message : '未知错误'),
      });
    }
  }

  return {
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
  };
}
