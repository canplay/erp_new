/**
 * @file useChatSend.ts
 * @description 聊天消息发送逻辑 — 输入框状态、发送消息、表情
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useMessageStore } from '@/stores/message';

export function useChatSend(roomId: string) {
  const $q = useQuasar();
  const messageStore = useMessageStore();

  const inputText = ref('');
  const showEmojiPicker = ref(false);

  async function sendMessage() {
    const text = inputText.value.trim();
    if (!text) return;
    try {
      await messageStore.sendMessage(roomId, text);
      inputText.value = '';
      showEmojiPicker.value = false;
    } catch (e) {
      $q.notify({ type: 'negative', message: e instanceof Error ? e.message : '发送失败' });
    }
  }

  function sendImage(file: File) {
    try {
      const reader = new FileReader();
      reader.onload = () => {
        const base64 = reader.result as string;
        void messageStore.sendMessage(roomId, `[图片] ${base64}`);
      };
      reader.readAsDataURL(file);
    } catch (e) {
      $q.notify({ type: 'negative', message: e instanceof Error ? e.message : '图片发送失败' });
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void sendMessage();
    }
  }

  function onEmojiClick(emoji: string) {
    inputText.value += emoji;
  }

  return { inputText, showEmojiPicker, sendMessage, sendImage, handleKeydown, onEmojiClick };
}
