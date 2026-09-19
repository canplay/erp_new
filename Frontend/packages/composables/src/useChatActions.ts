/**
 * @file useChatActions.ts
 * @description ChatPage 辅助操作（文件上传/离开房间/邀请/表情/搜索）— 从 ChatPage.vue 拆分
 */

import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useRoomStore } from '@/stores/room';
import { useMessageStore } from '@/stores/message';
import { MatrixService } from '@/services';

export function useChatActions(roomId: string) {
  const $q = useQuasar();
  const router = useRouter();
  const roomStore = useRoomStore();
  const messageStore = useMessageStore();

  // ── Emoji ──
  const emojis = ['😀','😂','🤣','😍','🥰','😎','🤩','😡','😱','😴',
    '👍','👎','👊','✊','🤝','🙏','💪','🔥','⭐','❤️',
    '🎉','🎊','🎈','💯','✅','❌','❗','❓','💡','📌'];
  const showEmojiPicker = ref(false);

  function openEmojiPicker() {
    showEmojiPicker.value = true;
  }

  function insertEmoji(emoji: string, inputText: { value: string }) {
    inputText.value += emoji;
    showEmojiPicker.value = false;
  }

  // ── File Upload ──
  function openFilePicker() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*,video/*,audio/*,.pdf,.doc,.docx,.txt';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;

      try {
        const ms = MatrixService.getInstance();
        const result = await ms.uploadContent(file);
        const mxcUrl = result.contentUri;

        if (file.type.startsWith('image/')) {
          await messageStore.sendImage(roomId, mxcUrl, file.name);
        } else {
          await messageStore.sendFile(roomId, mxcUrl, file.name, file.type);
        }
      } catch (e: unknown) {
        $q.notify({ type: 'negative', message: '上传失败: ' + (e instanceof Error ? e.message : '未知错误') });
      }
    };
    input.click();
  }

  // ── Voice Recording ──
  function startVoiceRecording() {
    $q.notify({ type: 'info', message: '语音录制功能即将推出', icon: 'mic' });
  }

  // ── Leave Room ──
  function handleLeaveRoom() {
    $q.dialog({
      title: '离开房间',
      message: `确定要离开这个房间吗？`,
      cancel: true,
      persistent: true,
    }).onOk(() => {
      roomStore.leaveRoom(roomId).then(() => {
        void router.push('/rooms');
        $q.notify({ type: 'positive', message: '已离开房间' });
      }).catch((e: unknown) => {
        $q.notify({ type: 'negative', message: '操作失败: ' + (e instanceof Error ? e.message : '未知错误') });
      });
    });
  }

  // ── Invite ──
  const inviteUserId = ref('');
  const showInviteDialog = ref(false);
  const isInviting = ref(false);

  async function handleInvite() {
    if (!inviteUserId.value.trim()) return;
    isInviting.value = true;
    try {
      const ms = MatrixService.getInstance();
      await ms.inviteUser(roomId, inviteUserId.value.trim());
      $q.notify({ type: 'positive', message: `已邀请 ${inviteUserId.value}` });
      inviteUserId.value = '';
      showInviteDialog.value = false;
    } catch (e: unknown) {
      $q.notify({ type: 'negative', message: '邀请失败: ' + (e instanceof Error ? e.message : '未知错误') });
    } finally {
      isInviting.value = false;
    }
  }

  // ── Search ──
  const showSearchDialog = ref(false);
  const searchQuery = ref('');

  interface SearchResultItem {
    eventId: string;
    sender: string;
    content?: { body?: string };
  }
  const searchResults = ref<SearchResultItem[]>([]);
  const isSearching = ref(false);

  async function handleSearch() {
    const q = searchQuery.value.trim();
    if (!q || isSearching.value) return;
    isSearching.value = true;
    try {
      const ms = MatrixService.getInstance();
      const results = await ms.searchMessages(roomId, q);
      searchResults.value = results as SearchResultItem[];
    } catch (e: unknown) {
      $q.notify({ type: 'negative', message: '搜索失败: ' + (e instanceof Error ? e.message : '未知错误') });
    } finally {
      isSearching.value = false;
    }
  }

  return {
    emojis,
    showEmojiPicker,
    openEmojiPicker,
    insertEmoji,
    openFilePicker,
    startVoiceRecording,
    handleLeaveRoom,
    inviteUserId,
    showInviteDialog,
    isInviting,
    handleInvite,
    showSearchDialog,
    searchQuery,
    searchResults,
    isSearching,
    handleSearch,
  };
}
