<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">
      {{ isEdit ? $t('cms.editArticle') : $t('cms.createArticle') }}
    </div>

    <!-- 操作栏 -->
    <div class="row q-mb-md justify-end">
      <q-btn flat :label="$t('common.cancel')" @click="goBack" />
      <q-btn color="primary" :label="$t('cms.publish')" :loading="saving" @click="handlePublish" />
      <q-btn
        v-if="!isPublished"
        color="warning"
        :label="$t('cms.saveDraft')"
        flat
        @click="handleSaveDraft"
      />
      <q-btn color="secondary" :label="$t('cms.preview')" flat @click="previewArticle" />
    </div>

    <!-- 文章表单 -->
    <q-card flat bordered>
      <q-card-section>
        <q-form @submit="handleSubmit" class="q-gutter-md">
          <!-- 标题 -->
          <q-input
            v-model="formData.title"
            :label="$t('cms.title')"
            outlined
            dense
            :rules="[(val) => !!val || $t('common.required')]"
          />

          <!-- 分类和标签 -->
          <div class="row q-gutter-md">
            <q-select
              v-model="formData.categoryId"
              :options="categoryOptions"
              :label="$t('cms.category')"
              outlined
              dense
              emit-value
              map-options
              class="col"
              :rules="[(val) => !!val || $t('common.required')]"
            />
            <q-select
              v-model="formData.tags"
              :options="tagOptions"
              :label="$t('cms.tags')"
              outlined
              dense
              multiple
              use-chips
              class="col"
            />
          </div>

          <!-- 摘要 -->
          <q-input
            v-model="formData.summary"
            :label="$t('cms.summary')"
            outlined
            dense
            type="textarea"
            rows="3"
          />

          <!-- 内容编辑器 -->
          <div>
            <div class="text-subtitle2 q-mb-sm">{{ $t('cms.content') }}</div>
            <RichTextEditor
              v-model="formData.content"
              :height="'400px'"
              :placeholder="$t('cms.contentPlaceholder')"
              @change="handleContentChange"
            />
          </div>

          <!-- 封面图片 -->
          <div>
            <div class="text-subtitle2 q-mb-sm">{{ $t('cms.coverImage') }}</div>
            <FileUploader
              v-model="formData.coverImage"
              :upload-url="uploadUrl"
              :accept="'.jpg,.png,.gif,.webp'"
              @upload="(files: File[]) => onCoverUploaded(files, '')"
            />
          </div>

          <!-- 高级设置 -->
          <q-expansion-item
            :label="$t('cms.advancedSettings')"
            icon="settings"
            header-class="text-body2"
          >
            <div class="q-pa-md">
              <!-- SEO 设置 -->
              <div class="text-subtitle2 q-mb-sm">{{ $t('cms.seoSettings') }}</div>
              <q-input
                v-model="formData.seoTitle"
                :label="$t('cms.seoTitle')"
                outlined
                dense
                class="q-mb-sm"
              />
              <q-input
                v-model="formData.seoKeywords"
                :label="$t('cms.seoKeywords')"
                outlined
                dense
                class="q-mb-sm"
              />
              <q-input
                v-model="formData.seoDescription"
                :label="$t('cms.seoDescription')"
                outlined
                dense
                type="textarea"
                rows="2"
              />
            </div>
          </q-expansion-item>
        </q-form>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file ArticleEditPage.vue
 * @description 文章编辑页面
 * @date 2026-04-08
 */
import { ref, reactive, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import FileUploader from '@erp-new-frontend-monorepo/components/src/FileUploader.vue';
import RichTextEditor from '@erp-new-frontend-monorepo/components/src/RichTextEditor.vue';
import { getArticleDetail, createArticle, updateArticle } from '@/api/cms';
import type { ArticleUpdateParams } from '@/api/cms';

const { t } = useI18n();
const $q = useQuasar();

const route = useRoute();
const router = useRouter();

// 上传地址
const uploadUrl = '/files/upload';

// 状态
const saving = ref(false);
const articleId = computed(() => (route.params.id ? Number(route.params.id) : null));
const isEdit = computed(() => !!articleId.value);

// 文章表单数据
const formData = reactive({
  title: '',
  categoryId: null as number | null,
  tags: [] as string[],
  summary: '',
  content: '',
  coverImage: '',
  seoTitle: '',
  seoKeywords: '',
  seoDescription: '',
  status: 'draft',
});

// 是否已发布
const isPublished = computed(() => formData.status === 'published');

// 分类选项
const categoryOptions = ref([
  { label: '新闻', value: 1 },
  { label: '公告', value: 2 },
  { label: '作品', value: 3 },
]);

// 标签选项
const tagOptions = ['新闻', '公告', '技巧', '作品', '活动'];

/**
 * @brief 处理表单提交
 */
async function handleSubmit() {
  await saveArticle(false);
}

/**
 * @brief 发布文章
 */
async function handlePublish() {
  formData.status = 'published';
  await saveArticle(true);
}

/**
 * @brief 保存草稿
 */
async function handleSaveDraft() {
  formData.status = 'draft';
  await saveArticle(false);
}

/**
 * @brief 保存文章
 */
async function saveArticle(isPublish: boolean): Promise<void> {
  saving.value = true;
  try {
    // 准备文章数据
    const articleData = {
      title: formData.title,
      summary: formData.summary,
      content: formData.content,
      categoryId: formData.categoryId ?? 0,
      tags: formData.tags,
      coverImage: formData.coverImage,
      isDraft: !isPublish,
      seoTitle: formData.seoTitle,
      seoKeywords: formData.seoKeywords,
      seoDescription: formData.seoDescription,
    };

    // 调用 CMS API 保存文章
    if (isEdit.value && articleId.value) {
      await updateArticle(articleId.value, articleData as ArticleUpdateParams);
    } else {
      await createArticle(articleData);
    }

    $q.notify({
      type: 'positive',
      message: isPublish ? t('cms.publishSuccess') : t('cms.saveSuccess'),
    });
    await router.push('/cms/articles');
  } catch (error) {
    logger.error('【保存文章失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
    throw error;
  } finally {
    saving.value = false;
  }
}

/**
 * @brief 封面上传成功回调
 */
function onCoverUploaded(files: File[], url: string): void {
  formData.coverImage = url;
}

/**
 * @brief 预览文章
 */
function previewArticle() {
  window.open(`/cms/articles/${articleId.value}/preview`, '_blank');
}

/**
 * @brief 返回列表
 */
function goBack() {
  void router.push('/cms/articles');
}

// 生命周期
onMounted(async () => {
  // 加载文章详情或初始化
  if (articleId.value) {
    try {
      const response = await getArticleDetail(articleId.value);
      const respData = response as { data?: { data?: Record<string, unknown> } };
      const article = respData.data?.data;

      if (article) {
        // 填充表单数据
        formData.title = String((article.title as string) || '');
        formData.summary = String((article.summary as string) || '');
        formData.content = String((article.content as string) || '');
        formData.categoryId = article.categoryId as number | null;
        formData.tags = (article.tags as string[]) || [];
        formData.coverImage = String((article.coverImage as string) || '');
        formData.seoTitle = String((article.seoTitle as string) || '');
        formData.seoKeywords = String((article.seoKeywords as string) || '');
        formData.seoDescription = String((article.seoDescription as string) || '');
        formData.status = String((article.status as string) || 'draft');
      }
    } catch (error) {
      logger.error('【加载文章详情失败】', error);
      $q.notify({ type: 'negative', message: t('common.error') });
    }
  }
});

/**
 * @brief 处理内容变化
 */
function handleContentChange(content: string): void {
  formData.content = content;
}
</script>

<style scoped>
.code-block {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
}

.body--dark .code-block {
  background: #2d2d2d;
}
</style>
