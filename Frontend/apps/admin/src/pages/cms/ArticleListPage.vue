<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('cms.articleManagement') }}</div>

    <!-- 操作栏 -->
    <div class="row q-mb-md items-center">
      <div class="col">
        <q-input
          v-model="searchKeyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="max-width: 300px"
          @update:model-value="loadArticles"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </div>
      <div class="row q-gutter-sm">
        <q-btn color="positive" :icon="matAdd" :label="$t('cms.createArticle')" @click="createArticle" />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="loadArticles" />
      </div>
    </div>

    <!-- 文章列表 -->
    <q-card flat bordered>
      <q-table
        :rows="articleList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- 封面图 -->
        <template #body-cell-coverImage="{ row }">
          <q-td>
            <q-avatar v-if="row.coverImage" square size="40px">
              <img :src="row.coverImage" />
            </q-avatar>
            <q-icon v-else name="image" size="40px" color="grey-5" />
          </q-td>
        </template>

        <!-- 标题 -->
        <template #body-cell-title="{ row }">
          <q-td>
            <div class="text-weight-bold">{{ row.title }}</div>
            <div v-if="row.summary" class="text-caption text-grey ellipsis-2-lines" style="max-width: 300px">
              {{ row.summary }}
            </div>
          </q-td>
        </template>

        <!-- 分类 -->
        <template #body-cell-categoryName="{ row }">
          <q-td>
            <q-badge color="primary">{{ row.categoryName || '-' }}</q-badge>
          </q-td>
        </template>

        <!-- 作者 -->
        <template #body-cell-authorName="{ row }">
          <q-td>
            <div>{{ row.authorName }}</div>
          </q-td>
        </template>

        <!-- 标签 -->
        <template #body-cell-tags="{ row }">
          <q-td>
            <q-chip
              v-for="tag in (row.tags || []).slice(0, 2)"
              :key="tag"
              dense
              size="sm"
              color="info"
              text-color="white"
            >
              {{ tag }}
            </q-chip>
            <q-badge v-if="(row.tags || []).length > 2" color="grey" :label="`+${row.tags.length - 2}`" />
          </q-td>
        </template>

        <!-- 状态 -->
        <template #body-cell-status="{ row }">
          <q-td>
            <q-badge :color="getStatusColor(row.status)" :label="getStatusLabel(row.status)" />
          </q-td>
        </template>

        <!-- 数据统计 -->
        <template #body-cell-stats="{ row }">
          <q-td>
            <div class="row q-gutter-xs">
              <q-badge color="grey-7" :label="`阅 ${row.viewCount}`" />
              <q-badge color="positive" :label="`赞 ${row.likeCount}`" />
              <q-badge color="info" :label="`评 ${row.commentCount}`" />
            </div>
          </q-td>
        </template>

        <!-- 置顶/推荐 -->
        <template #body-cell-flags="{ row }">
          <q-td>
            <q-icon
              v-if="row.isTop"
              name="vertical_align_top"
              color="warning"
              size="20px"
            >
              <q-tooltip>{{ $t('cms.topped') }}</q-tooltip>
            </q-icon>
            <q-icon
              v-if="row.isFeatured"
              name="star"
              color="warning"
              size="20px"
              class="q-ml-sm"
            >
              <q-tooltip>{{ $t('cms.featured') }}</q-tooltip>
            </q-icon>
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matEdit" @click="editArticle(row)">
            <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense round :icon="matVisibility" @click="previewArticle(row)">
            <q-tooltip>{{ $t('common.preview') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.status === 'published'"
            flat
            dense
            round
            :icon="matPublic"
            color="positive"
            @click="unpublishArticle(row)"
          >
            <q-tooltip>{{ $t('cms.unpublish') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.status === 'draft' || row.status === 'rejected'"
            flat
            dense
            round
            :icon="matPublish"
            color="primary"
            @click="publishArticle(row)"
          >
            <q-tooltip>{{ $t('cms.publish') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense round :icon="matDelete" color="negative" @click="deleteArticle(row)">
            <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="article" size="48px" class="q-mb-sm" />
            <div>{{ $t('cms.noArticles') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file ArticleListPage.vue
 * @description CMS文章列表页面
 * @date 2026-04-06
 */

import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import {
  getArticleList,
  publishArticle as apiPublishArticle,
  unpublishArticle as apiUnpublishArticle,
  deleteArticle as apiDeleteArticle,
  type CmsArticle,
} from '@/api/cms';

// Material Icons
const { t } = useI18n();
const matAdd = 'add';
const matEdit = 'edit';
const matDelete = 'delete';
const matRefresh = 'refresh';
const matVisibility = 'visibility';
const matPublish = 'publish';
const matPublic = 'public';


const $q = useQuasar();
const router = useRouter();

// 状态
const loading = ref(false);
const searchKeyword = ref('');
const articleList = ref<CmsArticle[]>([]);
const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });

// 表格列定义
const columns = computed(() => [
  { name: 'coverImage', label: '', field: 'coverImage', align: 'center' as const, style: 'width: 60px' },
  { name: 'title', label: t('cms.title'), field: 'title', align: 'left' as const },
  { name: 'categoryName', label: t('cms.category'), field: 'categoryName', align: 'center' as const },
  { name: 'authorName', label: t('cms.author'), field: 'authorName', align: 'left' as const },
  { name: 'tags', label: t('cms.tags'), field: 'tags', align: 'left' as const },
  { name: 'status', label: t('cms.status'), field: 'status', align: 'center' as const },
  { name: 'stats', label: t('cms.stats'), field: 'stats', align: 'center' as const },
  { name: 'flags', label: '', field: 'flags', align: 'center' as const, style: 'width: 80px' },
  { name: 'publishedAt', label: t('cms.publishTime'), field: 'publishedAt', align: 'left' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

// 方法

/**
 * @brief 加载文章列表
 */
async function loadArticles() {
  loading.value = true;
  try {
    const response = await getArticleList({
      keyword: searchKeyword.value,
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    });
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: CmsArticle[]; data?: CmsArticle[]; total?: number };
    articleList.value = respData.list || respData.data || [];
    pagination.value.total = respData.total || 0;
  } catch (error) {
    logger.error('【加载文章列表失败】', error);
    $q.notify({
      type: 'negative',
      message: t('cms.loadFailed'),
    });
  } finally {
    loading.value = false;
  }
}

/**
 * @brief 分页请求
 */
function onRequest(props: { pagination: { page: number; rowsPerPage: number } }): void {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  void loadArticles();
}

/**
 * @brief 获取状态颜色
 */
function getStatusColor(status: string) {
  const colors: Record<string, string> = {
    draft: 'grey',
    pending: 'warning',
    published: 'positive',
    rejected: 'negative',
    archived: 'blue-grey',
  };
  return colors[status] || 'grey';
}

/**
 * @brief 获取状态标签
 */
function getStatusLabel(status: string) {
  const labels: Record<string, string> = {
    draft: t('cms.statusDraft'),
    pending: t('cms.statusPending'),
    published: t('cms.statusPublished'),
    rejected: t('cms.statusRejected'),
    archived: t('cms.statusArchived'),
  };
  return labels[status] || status;
}

/**
 * @brief 创建文章
 */
function createArticle(): void {
  void router.push('/cms/article/edit/new');
}

/**
 * @brief 编辑文章
 */
function editArticle(row: CmsArticle): void {
  void router.push(`/cms/article/edit/${row.id}`);
}

/**
 * @brief 预览文章
 */
function previewArticle(row: CmsArticle): void {
  void router.push(`/cms/article/preview/${row.id}`);
}

/**
 * @brief 发布文章
 */
async function publishArticle(row: CmsArticle) {
  try {
    await apiPublishArticle(row.id);
    $q.notify({
      type: 'positive',
      message: t('cms.publishSuccess'),
    });
    await loadArticles();
  } catch (error) {
    logger.error('【发布文章失败】', error);
    $q.notify({
      type: 'negative',
      message: t('cms.publishFailed'),
    });
  }
}

/**
 * @brief 下架文章
 */
async function unpublishArticle(row: CmsArticle) {
  try {
    await apiUnpublishArticle(row.id);
    $q.notify({
      type: 'positive',
      message: t('cms.unpublishSuccess'),
    });
    await loadArticles();
  } catch (error) {
    logger.error('【下架文章失败】', error);
    $q.notify({
      type: 'negative',
      message: t('cms.unpublishFailed'),
    });
  }
}

/**
 * @brief 删除文章
 */
async function deleteArticle(row: CmsArticle): Promise<void> {
  try {
    await new Promise<void>((resolve, reject) => {
      $q.dialog({
        title: t('common.confirmDelete'),
        message: t('cms.deleteConfirmMessage'),
        cancel: true,
        persistent: true,
        ok: {
          label: t('common.delete'),
          color: 'negative',
        },
      }).onOk(() => resolve()).onCancel(() => reject(new Error('cancelled')));
    });
    
    await apiDeleteArticle(row.id);
    $q.notify({
      type: 'positive',
      message: t('cms.deleteSuccess'),
    });
    await loadArticles();
  } catch (error) {
    if (error instanceof Error && error.message === 'cancelled') {
      return;
    }
    logger.error('【删除文章失败】', error);
    $q.notify({
      type: 'negative',
      message: t('cms.deleteFailed'),
    });
  }
}

// 生命周期
onMounted(() => {
  void loadArticles();
});
</script>

