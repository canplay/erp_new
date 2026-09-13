/**
 * @file useMessageCenter.ts
 * @description 消息中心页面业务逻辑 composable
 * @description 提取自 MessageCenterPage.vue，参照 useDashboard.ts 模式
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  getMessageList,
  getInboxMessages,
  getOutboxMessages,
  getAnnouncements,
  sendMessage as apiSendMessage,
  markAsRead,
  markAsStarred,
  deleteMessage as apiDeleteMessage,
  type Message,
  type MessageType,
  type MessageCreateParams,
} from '@/api/message';
// notification API 预留，后续通知功能使用

// Material Icons (字符串常量，供模板 :icon 绑定)
const matAdd = 'add';
const matRefresh = 'refresh';
const matVisibility = 'visibility';
const matDelete = 'delete';

export function useMessageCenter() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // ─── 状态 ───────────────────────────────────────
  const loading = ref(false);
  const sending = ref(false);
  const activeTab = ref<'inbox' | 'outbox' | 'announcements'>('inbox');
  const searchKeyword = ref('');
  const messageList = ref<Message[]>([]);

  // 发送对话框
  const sendDialogVisible = ref(false);
  const sendForm = ref<MessageCreateParams>({
    type: 'user',
    title: '',
    content: '',
    priority: 0,
    targetType: 'user',
    target_ids: [],
  });

  // 详情对话框
  const detailDialogVisible = ref(false);
  const currentMessage = ref<Message | null>(null);

  // 分页
  const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });

  // ─── 下拉选项 ──────────────────────────────────
  const messageTypeOptions = computed(() => [
    { label: $t('message.type.all'), value: '' },
    { label: $t('message.type.system'), value: 'system' },
    { label: $t('message.type.user'), value: 'user' },
    { label: $t('message.type.announcement'), value: 'announcement' },
  ]);

  const targetTypeOptions = computed(() => [
    { label: $t('message.target.all'), value: 'all' },
    { label: $t('message.target.dept'), value: 'dept' },
    { label: $t('message.target.role'), value: 'role' },
    { label: $t('message.target.user'), value: 'user' },
  ]);

  const priorityOptions = computed(() => [
    { label: $t('message.priority.normal'), value: 0 },
    { label: $t('message.priority.important'), value: 1 },
    { label: $t('message.priority.urgent'), value: 2 },
  ]);

  // ─── 表格列定义 ────────────────────────────────
  const columns = computed(() => [
    { name: 'type', label: $t('message.type'), field: 'type', align: 'center' as const },
    { name: 'title', label: $t('message.title'), field: 'title', align: 'left' as const },
    { name: 'priority', label: $t('message.priority'), field: 'priority', align: 'center' as const },
    { name: 'sender_name', label: $t('message.sender'), field: 'sender_name', align: 'left' as const },
    { name: 'is_read', label: $t('message.status'), field: 'is_read', align: 'center' as const },
    { name: 'is_starred', label: $t('message.starred'), field: 'is_starred', align: 'center' as const },
    { name: 'created_at', label: $t('message.time'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ─── 辅助函数 ──────────────────────────────────

  /** 获取消息类型图标 */
  function getMessageTypeIcon(type: MessageType): string {
    const icons: Record<MessageType, string> = {
      system: 'settings',
      user: 'person',
      announcement: 'campaign',
    };
    return icons[type] || 'mail';
  }

  /** 获取消息类型颜色 */
  function getMessageTypeColor(type: MessageType): string {
    const colors: Record<MessageType, string> = {
      system: 'blue',
      user: 'green',
      announcement: 'orange',
    };
    return colors[type] || 'grey';
  }

  /** 获取消息类型标签 */
  function getMessageTypeLabel(type: MessageType): string {
    const labels: Record<MessageType, string> = {
      system: $t('message.type.system'),
      user: $t('message.type.user'),
      announcement: $t('message.type.announcement'),
    };
    return labels[type] || type;
  }

  /** 获取优先级颜色 */
  function getPriorityColor(priority: number): string {
    if (priority >= 2) return 'red';
    if (priority >= 1) return 'orange';
    return 'grey';
  }

  /** 获取优先级标签 */
  function getPriorityLabel(priority: number): string {
    if (priority >= 2) return $t('message.priority.urgent');
    if (priority >= 1) return $t('message.priority.important');
    return $t('message.priority.normal');
  }

  // ─── 数据加载 ──────────────────────────────────

  /** 加载消息列表（根据当前 tab） */
  async function loadMessages() {
    loading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
      };
      if (searchKeyword.value) {
        params.keyword = searchKeyword.value;
      }

      let response;
      switch (activeTab.value) {
        case 'inbox':
          response = await getInboxMessages(params);
          break;
        case 'outbox':
          response = await getOutboxMessages(params);
          break;
        case 'announcements':
          response = await getAnnouncements(params);
          break;
        default:
          response = await getMessageList(params);
      }

      // 兼容多种响应格式
      const respData = response as {
        list?: Message[];
        data?: { list?: Message[]; total?: number };
        total?: number;
      };
      const list = respData.list ?? respData.data?.list ?? [];
      const total = respData.total ?? respData.data?.total ?? 0;
      messageList.value = list;
      pagination.value.total = total;
    } catch (error) {
      logger.error('【加载消息列表失败】', error);
      messageList.value = [];
    } finally {
      loading.value = false;
    }
  }

  /** 表格分页请求回调 */
  function onRequest(props: { pagination: { page: number; rowsPerPage: number } }): void {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadMessages();
  }

  // ─── 操作函数 ──────────────────────────────────

  /** 打开发送对话框 */
  function openSendDialog(): void {
    sendForm.value = {
      type: activeTab.value === 'announcements' ? 'announcement' : 'user',
      title: '',
      content: '',
      priority: 0,
      targetType: 'user',
      target_ids: [],
    };
    sendDialogVisible.value = true;
  }

  /** 发送消息 */
  async function handleSend(): Promise<void> {
    if (!sendForm.value.title.trim() || !sendForm.value.content.trim()) {
      $q.notify({ type: 'warning', message: $t('message.fillRequired') });
      return;
    }
    sending.value = true;
    try {
      await apiSendMessage(sendForm.value);
      $q.notify({ type: 'positive', message: $t('message.sendSuccess') });
      sendDialogVisible.value = false;
      await loadMessages();
    } catch (error) {
      logger.error('【发送消息失败】', error);
      $q.notify({ type: 'negative', message: $t('message.sendFailed') });
    } finally {
      sending.value = false;
    }
  }

  /** 查看消息详情 */
  function viewMessage(message: Message): void {
    currentMessage.value = message;
    detailDialogVisible.value = true;
    // 标记已读
    if (!message.is_read) {
      void markAsRead(message.id).then(() => {
        message.is_read = true;
      }).catch((error: unknown) => {
        logger.error('【标记已读失败】', error);
      });
    }
  }

  /** 切换星标 */
  async function toggleStar(message: Message): Promise<void> {
    try {
      const newStarred = !message.is_starred;
      await markAsStarred(message.id, newStarred);
      message.is_starred = newStarred;
      $q.notify({
        type: 'info',
        message: newStarred ? $t('message.starred') : $t('message.unstarred'),
      });
    } catch (error) {
      logger.error('【切换星标失败】', error);
    }
  }

  /** 删除消息 */
  function doDeleteMessage(message: Message): void {
    $q.dialog({
      title: $t('message.deleteConfirm'),
      message: $t('message.deleteMessage'),
      cancel: $t('common.cancel'),
      ok: $t('common.confirm'),
      persistent: true,
    }).onOk(() => {
      void (async () => {
        try {
          await apiDeleteMessage(message.id);
          messageList.value = messageList.value.filter((m) => m.id !== message.id);
          $q.notify({ type: 'info', message: $t('message.deleted') });
        } catch (error) {
          logger.error('【删除消息失败】', error);
          $q.notify({ type: 'negative', message: $t('common.error') });
        }
      })();
    });
  }

  // ─── 初始化 ────────────────────────────────────
  async function initData() {
    await loadMessages();
  }

  // ─── 返回 ──────────────────────────────────────
  return {
    // 状态
    activeTab,
    loading,
    sending,
    searchKeyword,
    messageList,
    sendDialogVisible,
    detailDialogVisible,
    currentMessage,
    sendForm,

    // 选项
    messageTypeOptions,
    targetTypeOptions,
    priorityOptions,

    // 表格
    columns,

    // 图标常量
    matAdd,
    matRefresh,
    matVisibility,
    matDelete,

    // 方法
    loadMessages,
    onRequest,
    getMessageTypeIcon,
    getMessageTypeColor,
    getMessageTypeLabel,
    getPriorityColor,
    getPriorityLabel,
    openSendDialog,
    handleSend,
    viewMessage,
    toggleStar,
    doDeleteMessage,
    initData,
  };
}
