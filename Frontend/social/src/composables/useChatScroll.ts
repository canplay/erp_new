/**
 * @file useChatScroll.ts
 * @description 聊天滚动行为 — 自动滚动、加载历史
 */

import { ref, nextTick } from 'vue';
import { useMessageStore } from '@/stores/message';

export function useChatScroll(roomId: string) {
  const messageStore = useMessageStore();

  const bottomAnchorRef = ref<HTMLElement | null>(null);
  const autoScroll = ref(true);
  const isLoadingMore = ref(false);

  async function scrollToBottom() {
    await nextTick();
    if (bottomAnchorRef.value) {
      bottomAnchorRef.value.scrollIntoView({ behavior: 'smooth' });
    }
  }

  function onScroll(info: { verticalPosition: number; verticalSize: number; verticalContainerSize: number }) {
    const position = info.verticalPosition;
    if (position <= 100 && (messageStore.hasMoreHistory as Record<string, boolean | undefined>)[roomId]) {
      void loadHistory();
    }
    autoScroll.value = position >= info.verticalSize - info.verticalContainerSize - 200;
  }

  async function loadHistory() {
    if (isLoadingMore.value) return;
    isLoadingMore.value = true;
    try {
      const msgs = messageStore.getMessages(roomId);
      const oldest = msgs[0]?.eventId;
      await messageStore.loadMessages(roomId, 30, oldest);
    } finally {
      isLoadingMore.value = false;
    }
  }

  return { bottomAnchorRef, autoScroll, isLoadingMore, scrollToBottom, onScroll, loadHistory };
}
