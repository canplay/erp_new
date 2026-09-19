<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('cms.categoryManagement') }}</div>

    <!-- 操作栏 -->
    <div class="row q-mb-md justify-between">
      <div></div>
      <q-btn color="positive" :icon="matAdd" :label="$t('common.add')" @click="openCreateDialog()" />
    </div>

    <!-- 分类列表 -->
    <q-card flat bordered>
      <q-card-section>
        <q-table
          :rows="categoryList"
          :columns="columns"
          row-key="id"
          :loading="loading"
          :pagination="{ rowsPerPage: 20 }"
        >
          <!-- 加载状态 -->
          <template v-slot:loading>
            <q-inner-loading showing color="primary" />
          </template>

          <!-- 空状态 -->
          <template v-slot:no-data>
            <div class="full-width row flex-center q-pa-lg text-grey">
              {{ $t('cms.noCategories') }}
            </div>
          </template>

          <!-- 操作列 -->
          <template v-slot:body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense size="sm" icon="edit" color="primary" @click="openEditDialog(props.row)">
                <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
              </q-btn>
              <q-btn flat dense size="sm" icon="add" color="positive" @click="openCreateDialog(props.row.id)">
                <q-tooltip>{{ $t('cms.addSubCategory') }}</q-tooltip>
              </q-btn>
              <q-btn flat dense size="sm" icon="delete" color="negative" @click="confirmDelete(props.row)">
                <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
              </q-btn>
            </q-td>
          </template>

          <!-- 状态列 -->
          <template v-slot:body-cell-status="props">
            <q-td :props="props">
              <q-badge :color="props.value === 1 ? 'positive' : 'grey'" :label="props.value === 1 ? $t('common.enabled') : $t('common.disabled')" />
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 创建/编辑分类弹窗 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 500px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ isEdit ? $t('cms.editCategory') : $t('cms.createCategory') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showDialog = false" />
        </q-card-section>

        <q-separator />

        <q-card-section>
          <q-form class="q-gutter-md" @submit.prevent="handleSubmit">
            <q-input
              v-model="formData.name"
              :label="$t('cms.categoryName')"
              outlined
              :rules="[(val) => !!val || $t('common.required')]"
            />
            <q-input
              v-model="formData.slug"
              :label="$t('cms.slug')"
              outlined
              hint="URL 别名"
              :rules="[(val) => !!val || $t('common.required')]"
            />
            <q-input
              v-model="formData.description"
              :label="$t('cms.description')"
              outlined
              type="textarea"
              rows="2"
            />
            <q-input
              v-model.number="formData.sort_order"
              :label="$t('cms.sort_order')"
              outlined
              type="number"
            />
            <q-select
              v-model="formData.status"
              :options="statusOptions"
              :label="$t('cms.status')"
              outlined
              emit-value
              map-options
            />
            <q-toggle v-model="formData.allowAttachment" :label="$t('cms.allowAttachment')" />
          </q-form>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDialog = false" />
          <q-btn color="primary" :label="$t('common.save')" type="submit" @click="handleSubmit" :loading="submitting" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 删除确认弹窗 -->
    <q-dialog v-model="showDeleteDialog">
      <q-card>
        <q-card-section class="row items-center">
          <q-icon name="warning" color="negative" size="24px" class="q-mr-sm" />
          <span>{{ $t('common.confirm') }}</span>
        </q-card-section>
        <q-card-section>
          {{ $t('cms.deleteCategoryConfirmMessage') }}
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDeleteDialog = false" />
          <q-btn color="negative" :label="$t('common.delete')" @click="handleDelete" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  getCategoryList,
  createCategory,
  updateCategory,
  deleteCategory,
  type CmsCategory,
  type CategoryCreateParams,
  type CategoryUpdateParams,
} from '@/api/cms';

const { t } = useI18n();
const $q = useQuasar();

// Material icons
const matAdd = 'add';

// 状态
const loading = ref(false);
const submitting = ref(false);
const categoryList = ref<CmsCategory[]>([]);
const showDialog = ref(false);
const showDeleteDialog = ref(false);
const isEdit = ref(false);
const currentEditId = ref<number | null>(null);
const pendingDeleteCategory = ref<CmsCategory | null>(null);

// 表单数据
const formData = reactive({
  name: '',
  slug: '',
  description: '',
  sort_order: 0,
  status: 1,
  allowAttachment: false,
  parent_id: undefined as number | undefined,
});

// 表格列定义
const columns = computed(() => [
  { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
  { name: 'name', label: t('cms.categoryName'), field: 'name', align: 'left' as const },
  { name: 'slug', label: t('cms.slug'), field: 'slug', align: 'left' as const },
  { name: 'description', label: t('cms.description'), field: 'description', align: 'left' as const },
  { name: 'sort_order', label: t('cms.sort_order'), field: 'sort_order', align: 'center' as const },
  { name: 'status', label: t('cms.status'), field: 'status', align: 'center' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
]);

// 状态选项
const statusOptions = [
  { label: t('common.enabled'), value: 1 },
  { label: t('common.disabled'), value: 0 },
];

/**
 * @brief 加载分类列表 - 已实现实际 API 调用
 */
async function loadCategories(): Promise<void> {
  loading.value = true;
  try {
    const response = await getCategoryList();
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: CmsCategory[]; data?: CmsCategory[] };
    categoryList.value = respData.list || respData.data || [];
    logger.info('【CMS分类】加载成功', { count: categoryList.value.length });
  } catch (error) {
    logger.error('【CMS分类】加载失败', error);
    $q.notify({ type: 'negative', message: t('common.error') });
    // 使用空数组作为降级处理
    categoryList.value = [];
  } finally {
    loading.value = false;
  }
}

/**
 * @brief 打开创建弹窗
 */
function openCreateDialog(parent_id?: number) {
  isEdit.value = false;
  currentEditId.value = null;
  Object.assign(formData, {
    name: '',
    slug: '',
    description: '',
    sort_order: 0,
    status: 1,
    allowAttachment: false,
    parent_id: parent_id,
  });
  showDialog.value = true;
}

/**
 * @brief 打开编辑弹窗
 */
function openEditDialog(category: CmsCategory) {
  isEdit.value = true;
  currentEditId.value = category.id;
  Object.assign(formData, {
    name: category.name,
    slug: category.slug || '',
    description: category.description || '',
    sort_order: category.sort_order || 0,
    status: category.status,
    allowAttachment: category.allowAttachment || false,
    parent_id: category.parent_id,
  });
  showDialog.value = true;
}

/**
 * @brief 提交表单 - 已实现实际 API 调用
 */
async function handleSubmit(): Promise<void> {
  if (!formData.name || !formData.slug) {
    $q.notify({ type: 'warning', message: t('common.required') });
    return;
  }

  submitting.value = true;
  try {
    if (isEdit.value && currentEditId.value) {
      // 更新分类 API
      const params: Record<string, unknown> = {
        id: currentEditId.value,
        name: formData.name,
        slug: formData.slug,
        description: formData.description,
        sort_order: formData.sort_order,
        status: formData.status,
        allowAttachment: formData.allowAttachment,
      };
      if (formData.parent_id !== undefined) params.parent_id = formData.parent_id;
      await updateCategory(currentEditId.value, params as CategoryUpdateParams);
      logger.info('【CMS分类】更新成功', { id: currentEditId.value });
    } else {
      // 创建分类 API
      const params: Record<string, unknown> = {
        name: formData.name,
        slug: formData.slug,
        description: formData.description,
        sort_order: formData.sort_order,
        status: formData.status,
        allowAttachment: formData.allowAttachment,
      };
      if (formData.parent_id !== undefined) params.parent_id = formData.parent_id;
      await createCategory(params as CategoryCreateParams);
      logger.info('【CMS分类】创建成功', { name: formData.name });
    }
    $q.notify({ type: 'positive', message: t('common.success') });
    showDialog.value = false;
    await loadCategories();
  } catch (error) {
    logger.error('【CMS分类】提交失败', error);
    $q.notify({ type: 'negative', message: t('common.error') });
    throw error;
  } finally {
    submitting.value = false;
  }
}

/**
 * @brief 确认删除
 */
function confirmDelete(category: CmsCategory) {
  pendingDeleteCategory.value = category;
  showDeleteDialog.value = true;
}

/**
 * @brief 执行删除 - 已实现实际 API 调用
 */
async function handleDelete(): Promise<void> {
  if (!pendingDeleteCategory.value) return;

  try {
    // 删除分类 API
    await deleteCategory(pendingDeleteCategory.value.id);
    logger.info('【CMS分类】删除成功', { id: pendingDeleteCategory.value.id });
    $q.notify({ type: 'positive', message: t('common.success') });
    pendingDeleteCategory.value = null;
    showDeleteDialog.value = false;
    await loadCategories();
  } catch (error) {
    logger.error('【CMS分类】删除失败', error);
    $q.notify({ type: 'negative', message: t('common.error') });
    throw error;
  }
}

// 生命周期
onMounted(() => {
  void loadCategories();
});
</script>

<style scoped>
</style>