/**
 * @file useFeedbackList.ts
 * @description 反馈列表页面业务逻辑 composable
 * @date 2026-04-06
 */

import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import { useFeedbackStore } from '@/stores/feedback';
import type { Feedback } from '@/api/feedback';

// Material Icons
const matRefresh = 'refresh';
const matVisibility = 'visibility';
const matCheck = 'check';
const matPlayArrow = 'play_arrow';
const matClose = 'close';

export function useFeedbackList() {
  const { t: $t } = useI18n();
  const $q = useQuasar();
  const feedbackStore = useFeedbackStore();

  // 状态
  const loading = computed(() => feedbackStore.isLoading);
  const submitting = ref(false);
  const detailDialogVisible = ref(false);
  const currentFeedback = ref<Feedback | null>(null);
  const replyContent = ref('');
  const feedbackList = computed(() => feedbackStore.feedbackList);
  const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });

  // 筛选条件
  const filters = ref({
    type: undefined as string | undefined,
    status: undefined as string | undefined,
    keyword: '',
  });

  // 选项配置
  const typeOptions = [
    { label: '建议', value: 'suggestion' },
    { label: '缺陷', value: 'bug' },
    { label: '投诉', value: 'complaint' },
    { label: '其他', value: 'other' },
  ];

  const statusOptions = [
    { label: '待处理', value: 'pending' },
    { label: '处理中', value: 'processing' },
    { label: '已解决', value: 'resolved' },
    { label: '已拒绝', value: 'rejected' },
    { label: '已关闭', value: 'closed' },
  ];

  // 表格列定义
  const columns = computed(() => [
    { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
    { name: 'type', label: $t('feedback.type'), field: 'type', align: 'center' as const },
    { name: 'title', label: $t('feedback.title'), field: 'title', align: 'left' as const },
    { name: 'userName', label: $t('feedback.submitter'), field: 'userName', align: 'left' as const },
    { name: 'status', label: $t('feedback.status'), field: 'status', align: 'center' as const },
    { name: 'rating', label: $t('feedback.rating'), field: 'rating', align: 'center' as const },
    { name: 'created_at', label: $t('feedback.submitTime'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ---- 方法 ----

  async function loadFeedback(): Promise<void> {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };
    if (filters.value.type) params.type = filters.value.type;
    if (filters.value.status) params.status = filters.value.status;
    if (filters.value.keyword) params.keyword = filters.value.keyword;

    try {
      await feedbackStore.fetchFeedbackList(params);
      pagination.value.total = feedbackStore.pagination.total;
    } catch (error) {
      logger.error('【加载反馈列表失败】', error);
      $q.notify({ type: 'negative', message: $t('feedback.loadFailed') });
      throw error;
    }
  }

  interface TableRequestProps {
    pagination: {
      page: number;
      rowsPerPage: number;
      sortBy?: string;
      descending?: boolean;
    };
  }

  function onRequest(props: TableRequestProps): void {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadFeedback();
  }

  function resetFilters(): void {
    filters.value = { type: undefined, status: undefined, keyword: '' };
    void loadFeedback();
  }

  function getTypeColor(type: string) {
    const colors: Record<string, string> = {
      suggestion: 'blue',
      bug: 'negative',
      complaint: 'orange',
      other: 'grey',
    };
    return colors[type] || 'grey';
  }

  function getTypeLabel(type: string) {
    const labels: Record<string, string> = {
      suggestion: $t('feedback.typeSuggestion'),
      bug: $t('feedback.typeBug'),
      complaint: $t('feedback.typeComplaint'),
      other: $t('feedback.typeOther'),
    };
    return labels[type] || type;
  }

  function getStatusColor(status: string) {
    const colors: Record<string, string> = {
      pending: 'warning',
      processing: 'info',
      resolved: 'positive',
      rejected: 'negative',
      closed: 'grey',
    };
    return colors[status] || 'grey';
  }

  function getStatusLabel(status: string) {
    const labels: Record<string, string> = {
      pending: $t('feedback.statusPending'),
      processing: $t('feedback.statusProcessing'),
      resolved: $t('feedback.statusResolved'),
      rejected: $t('feedback.statusRejected'),
      closed: $t('feedback.statusClosed'),
    };
    return labels[status] || status;
  }

  function viewDetail(row: Feedback) {
    currentFeedback.value = row;
    replyContent.value = '';
    detailDialogVisible.value = true;
  }

  async function handleFeedback(row: Feedback, status: string): Promise<void> {
    try {
      await feedbackStore.processFeedback(row.id, {
        status: status as Feedback['status'],
        handlerReply: '',
      });
      $q.notify({ type: 'positive', message: $t('feedback.handleSuccess') });
      await loadFeedback();
    } catch (error) {
      logger.error('【处理反馈失败】', error);
      $q.notify({ type: 'negative', message: $t('feedback.handleFailed') });
    }
  }

  async function submitReply(): Promise<void> {
    if (!currentFeedback.value || !replyContent.value.trim()) {
      $q.notify({ type: 'warning', message: $t('feedback.replyRequired') });
      return;
    }

    submitting.value = true;
    try {
      await feedbackStore.fetchFeedbackDetail(currentFeedback.value.id);
      $q.notify({ type: 'positive', message: $t('feedback.replySuccess') });
      detailDialogVisible.value = false;
      await loadFeedback();
    } catch (error) {
      logger.error('【提交回复失败】', error);
      $q.notify({ type: 'negative', message: $t('feedback.replyFailed') });
      throw error;
    } finally {
      submitting.value = false;
    }
  }

  return {
    loading,
    submitting,
    detailDialogVisible,
    currentFeedback,
    replyContent,
    feedbackList,
    filters,
    typeOptions,
    statusOptions,
    columns,
    loadFeedback,
    onRequest,
    resetFilters,
    getTypeColor,
    getTypeLabel,
    getStatusColor,
    getStatusLabel,
    viewDetail,
    handleFeedback,
    submitReply,
    matRefresh,
    matVisibility,
    matCheck,
    matPlayArrow,
    matClose,
  };
}
