/**
 * @file useChatEvents.ts
 * @description ChatPage Matrix 事件处理 — 从 ChatPage.vue 拆分
 */

import { ref } from 'vue';
import { RoomEvent, RoomMemberEvent, type MatrixEvent, type Room, type RoomMember } from 'matrix-js-sdk';
import { useAuthStore } from '@/stores/auth';
import { useMessageStore } from '@/stores/message';
import { MatrixService } from '@/services';
import type { RoomMessage } from '@/services';

export function useChatEvents(roomId: string) {
  const auth = useAuthStore();
  const messageStore = useMessageStore();
  const typingUsers = ref<string[]>([]);

  function onRoomTimeline(event: MatrixEvent, room: Room | undefined) {
    if (!room || room.roomId !== roomId) return;
    if (event.getType() !== 'm.room.message') return;

    const msg: RoomMessage = {
      eventId: event.getId()!,
      roomId: event.getRoomId()!,
      sender: event.getSender()!,
      content: event.getContent<Record<string, unknown>>(),
      type: event.getType(),
      timestamp: event.getTs(),
    };
    messageStore.onRoomMessage(roomId, msg);
  }

  function onTyping(_event: MatrixEvent, member: RoomMember) {
    if (member.roomId !== roomId) return;
    if (member.userId === auth.userId) return;
    typingUsers.value = member.typing ? [member.name || member.userId] : [];
  }

  function bindEvents() {
    if (!auth.isLoggedIn) return;
    const client = MatrixService.getInstance().getClient();
    client.on(RoomEvent.Timeline, onRoomTimeline);
    client.on(RoomMemberEvent.Typing, onTyping);
  }

  function unbindEvents() {
    if (!auth.isLoggedIn) return;
    const client = MatrixService.getInstance().getClient();
    client.removeListener(RoomEvent.Timeline, onRoomTimeline);
    client.removeListener(RoomMemberEvent.Typing, onTyping);
  }

  return {
    typingUsers,
    bindEvents,
    unbindEvents,
  };
}
