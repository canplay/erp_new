/**
 * @file useChatMessageOps.ts
 * @description ChatPage 消息操作（编辑/删除/回复/反应）— 从 ChatPage.vue 拆分
 */

import { ref, type Ref } from 'vue';
import { useQuasar } from 'quasar';
import { useMessageStore } from '@/stores/message';

export function useChatMessageOps(roomId: string, inputText: Ref<string>) {
  const $q = useQuasar();
  const messageStore = useMessageStore();

  const editingEventId = ref<string | null>(null);
  const editingText = ref('');

  function showReactionPicker(eventId: string) {
    $q.dialog({
      title: '选择表情反应',
      message: '😀😂🤣😍🥰😎🤩👍👎❤️🔥💪🎉💯✅❌',
      ok: '发送',
      cancel: true,
      prompt: {
        model: '👍',
        type: 'text',
        maxlength: 10,
      },
    }).onOk((emoji: string) => {
      void messageStore.sendReaction(roomId, eventId, emoji.trim());
    });
  }

  function startEdit(msg: { eventId: string; content?: { body?: string } }) {
    editingEventId.value = msg.eventId;
    editingText.value = msg.content?.body || '';
    inputText.value = `/edit ${editingText.value}`;
  }

  function confirmDelete(eventId: string) {
    $q.dialog({
      title: '确认删除',
      message: '确定要删除这条消息吗？',
      cancel: true,
      persistent: true,
    }).onOk(() => {
      void messageStore.deleteMessage(roomId, eventId);
    });
  }

  function replyTo(msg: { content?: { body?: string } }) {
    inputText.value = `> ${msg.content?.body || ''}\n\n`;
  }

  return {
    editingEventId,
    editingText,
    showReactionPicker,
    startEdit,
    confirmDelete,
    replyTo,
  };
}
