/**
 * @file usePublicRooms.ts
 * @description RoomListPage 公开房间发现逻辑 — 从 RoomListPage.vue 拆分
 */

import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useRoomStore } from '@/stores/room';
import { MatrixService } from '@/services';
import type { IPublicRoomsChunkRoom } from 'matrix-js-sdk';

export function usePublicRooms() {
  const $q = useQuasar();
  const router = useRouter();
  const roomStore = useRoomStore();

  const publicRooms = ref<IPublicRoomsChunkRoom[]>([]);
  const publicSearchTerm = ref('');
  const loadingPublic = ref(false);

  async function loadPublicRooms() {
    loadingPublic.value = true;
    try {
      const ms = MatrixService.getInstance();
      if (publicSearchTerm.value.trim()) {
        const res = await ms.searchPublicRooms(publicSearchTerm.value.trim());
        publicRooms.value = res.chunk;
      } else {
        const res = await ms.getPublicRooms(30);
        publicRooms.value = res.chunk;
      }
    } catch (e: unknown) {
      $q.notify({ type: 'negative', message: '获取公开房间失败: ' + (e instanceof Error ? e.message : '') });
    } finally {
      loadingPublic.value = false;
    }
  }

  async function joinPublicRoom(roomId: string) {
    try {
      await roomStore.joinRoom(roomId);
      $q.notify({ type: 'positive', message: '已加入房间' });
      void router.push(`/chat/${roomId}`);
    } catch (e: unknown) {
      $q.notify({ type: 'negative', message: '加入失败: ' + (e instanceof Error ? e.message : '未知错误') });
    }
  }

  return {
    publicRooms,
    publicSearchTerm,
    loadingPublic,
    loadPublicRooms,
    joinPublicRoom,
  };
}
