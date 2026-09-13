<template>
  <div>
    <div class="row items-center q-mb-md">
      <div class="text-subtitle1">{{ i18nT('raw.s2f0afe') }}</div>
      <span class="text-caption text-grey-7 q-ml-sm">{{
        i18nT('raw.s508a02', { count: categoryTotal })
      }}</span>
      <q-space />
      <q-btn color="primary" icon="add" :label="i18nT('raw.sfc65af')" @click="openCategoryCreate" />
    </div>

    <q-banner v-if="categoryError" type="negative" rounded class="q-mb-md">
      {{ categoryError }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.sdedda3')" @click="categoryError = ''" />
      </template>
    </q-banner>

    <PageTable
      v-if="categoryLoading || categoryRows.length > 0"
      :dense="$q.screen.lt.md"
      :rows="categoryRows"
      :columns="categoryColumns"
      :loading="categoryLoading"
      :total="categoryTotal"
      flat
      bordered
      @request="onCategoryRequest"
    >
      <template #body-cell-description="props">
        <q-td class="ellipsis" style="max-width: 320px">
          <span :title="props.row.description || ''">{{ props.row.description || '—' }}</span>
        </q-td>
      </template>
      <template #body-cell-parent="props">
        <q-td>{{ parentName(props.row) }}</q-td>
      </template>
      <template #body-cell-createdAtUtc="props">
        <q-td>{{ formatDate(props.row.createdAtUtc) }}</q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td>
          <q-btn
            flat
            dense
            round
            icon="edit"
            size="sm"
            @click="openCategoryEdit(props.row)"
            :title="i18nT('raw.s207008')"
          />
          <q-btn
            flat
            dense
            round
            icon="delete"
            size="sm"
            text-color="negative"
            @click="
              categoryToDelete = props.row;
              categoryDeleteDialog = true;
            "
            :title="i18nT('raw.sbd7449')"
          />
        </q-td>
      </template>
    </PageTable>
    <EmptyState
      v-else
      :title="i18nT('raw.s1a2d31')"
      :hint="i18nT('raw.s40d2f7')"
      icon="account_tree"
    />

    <!-- 新建/编辑分类 -->
    <q-dialog v-model="categoryDialog">
      <q-card style="min-width: 480px; max-width: 90vw">
        <q-card-section class="text-h6">{{
          editingCategory ? i18nT('raw.s3c0951') : i18nT('raw.sfc65af')
        }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="categoryForm.name"
            :label="i18nT('raw.sf43de3')"
            filled
            autofocus
            maxlength="128"
            counter
            :rules="[(v) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
          <q-select
            v-model="categoryForm.parentId"
            :options="categoryParentOptions"
            option-value="id"
            option-label="name"
            :label="i18nT('raw.s77f18a')"
            filled
            clearable
            emit-value
            map-options
            class="q-mt-md"
          >
            <template #option="scope">
              <q-item v-bind="scope.itemProps">
                <q-item-section>
                  <span :style="{ paddingLeft: `${scope.opt.depth * 16}px` }">
                    {{ scope.opt.depth > 0 ? '↳ ' : i18nT('raw.s0612b9') }}{{ scope.opt.name }}
                  </span>
                </q-item-section>
              </q-item>
            </template>
          </q-select>
          <q-input
            v-model="categoryForm.description"
            :label="i18nT('raw.s9e58f1')"
            filled
            type="textarea"
            class="q-mt-md"
            maxlength="1024"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sfe9512')"
            color="primary"
            :loading="categorySaving"
            :disable="!categoryForm.name.trim()"
            @click="saveCategory"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 删除分类 -->
    <q-dialog v-model="categoryDeleteDialog">
      <q-card style="min-width: 420px">
        <q-card-section class="row items-center">
          <q-icon name="warning" color="negative" size="28px" class="q-mr-sm" />
          <div class="text-h6">{{ i18nT('raw.scaa8d6') }}</div>
        </q-card-section>
        <q-card-section class="q-pt-none">
          {{ i18nT('raw.sc04a2c', { name: categoryToDelete?.name }) }}
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sbd7449')"
            color="negative"
            :loading="categoryDeleting"
            @click="confirmDeleteCategory"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { computed, onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { catalogApi } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';

interface CategoryItem {
  id: string;
  name: string;
  slug?: string;
  description?: string | null;
  parentCategoryId?: string | null;
  createdAtUtc?: string;
}

interface CategoryTreeNode {
  id: string;
  name: string;
  slug?: string;
  description?: string | null;
  children?: CategoryTreeNode[];
}

interface FlatNode {
  id: string;
  name: string;
  depth: number;
  ancestorIds: string[];
}

const $q = useQuasar();

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

function flattenTree(nodes: CategoryTreeNode[], depth = 0, ancestors: string[] = []): FlatNode[] {
  const out: FlatNode[] = [];
  for (const n of nodes) {
    out.push({ id: n.id, name: n.name, depth, ancestorIds: ancestors });
    if (n.children?.length) out.push(...flattenTree(n.children, depth + 1, [...ancestors, n.id]));
  }
  return out;
}

const categoryRows = ref<CategoryItem[]>([]);
const categoryTreeRows = ref<CategoryTreeNode[]>([]);
const categoryLoading = ref(false);
const categoryError = ref('');
const categoryTotal = ref(0);
const categoryPagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

const categoryColumns = [
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  {
    name: 'description',
    label: i18nT('raw.s9e58f1'),
    field: 'description',
    align: 'left' as const,
  },
  { name: 'parent', label: i18nT('raw.sc8b724'), field: 'parent', align: 'left' as const },
  {
    name: 'createdAtUtc',
    label: i18nT('raw.saf9956'),
    field: 'createdAtUtc',
    align: 'left' as const,
  },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions', align: 'right' as const },
];

async function loadCategories(p?: { page: number; rowsPerPage: number }) {
  const pg = p ?? categoryPagination.value;
  categoryLoading.value = true;
  try {
    const [res, tree] = await Promise.all([
      catalogApi.categories({ page: pg.page, pageSize: pg.rowsPerPage }) as Promise<{
        total: number;
        items: CategoryItem[];
      }>,
      catalogApi.categoryTree() as Promise<CategoryTreeNode[]>,
    ]);
    categoryRows.value = res.items ?? [];
    categoryTotal.value = res.total ?? categoryRows.value.length;
    categoryPagination.value = {
      page: pg.page,
      rowsPerPage: pg.rowsPerPage,
      rowsNumber: res.total ?? categoryRows.value.length,
    };
    categoryTreeRows.value = Array.isArray(tree) ? tree : [];
  } catch (e) {
    categoryError.value = e instanceof Error ? e.message : i18nT('raw.s01d7a9');
  } finally {
    categoryLoading.value = false;
  }
}

function onCategoryRequest(payload: { page: number; rowsPerPage: number }) {
  void loadCategories(payload);
}

const categoryNameById = computed(() => {
  const map = new Map<string, string>();
  const walk = (nodes: CategoryTreeNode[]) => {
    for (const n of nodes) {
      map.set(n.id, n.name);
      if (n.children?.length) walk(n.children);
    }
  };
  walk(categoryTreeRows.value);
  return map;
});

function parentName(c: CategoryItem) {
  if (!c.parentCategoryId) return '—';
  return categoryNameById.value.get(c.parentCategoryId) ?? '—';
}

const categoryDialog = ref(false);
const categorySaving = ref(false);
const editingCategory = ref<CategoryItem | null>(null);
const categoryForm = ref<{ name: string; description: string; parentId: string }>({
  name: '',
  description: '',
  parentId: '',
});

const categoryParentOptions = computed(() => {
  const flat = flattenTree(categoryTreeRows.value);
  const editingId = editingCategory.value?.id;
  if (!editingId) return flat;
  return flat.filter((n) => n.id !== editingId && !n.ancestorIds.includes(editingId));
});

function openCategoryCreate() {
  editingCategory.value = null;
  categoryForm.value = { name: '', description: '', parentId: '' };
  categoryDialog.value = true;
}

function openCategoryEdit(c: CategoryItem) {
  editingCategory.value = c;
  categoryForm.value = {
    name: c.name ?? '',
    description: c.description ?? '',
    parentId: c.parentCategoryId ?? '',
  };
  categoryDialog.value = true;
}

async function saveCategory() {
  if (!categoryForm.value.name.trim()) return;
  categorySaving.value = true;
  try {
    const payload: { name: string; description?: string; parentId?: string } = {
      name: categoryForm.value.name.trim(),
    };
    if (categoryForm.value.description.trim())
      payload.description = categoryForm.value.description.trim();
    if (categoryForm.value.parentId) payload.parentId = categoryForm.value.parentId;
    if (editingCategory.value) {
      await catalogApi.updateCategory(editingCategory.value.id, payload);
      toast('positive', i18nT('raw.s72f83f'));
    } else {
      await catalogApi.createCategory(payload);
      toast('positive', i18nT('raw.s20c454'));
    }
    categoryDialog.value = false;
    await loadCategories();
  } catch (e) {
    categoryError.value =
      e instanceof Error
        ? e.message
        : editingCategory.value
          ? i18nT('raw.se410d8')
          : i18nT('raw.sb62cb3');
  } finally {
    categorySaving.value = false;
  }
}

const categoryDeleteDialog = ref(false);
const categoryDeleting = ref(false);
const categoryToDelete = ref<CategoryItem | null>(null);

async function confirmDeleteCategory() {
  if (!categoryToDelete.value) return;
  categoryDeleting.value = true;
  try {
    await catalogApi.deleteCategory(categoryToDelete.value.id);
    toast('positive', i18nT('raw.s51a8b9'));
    categoryDeleteDialog.value = false;
    await loadCategories();
  } catch (e) {
    categoryError.value = e instanceof Error ? e.message : i18nT('raw.s2324d0');
  } finally {
    categoryDeleting.value = false;
  }
}

onMounted(() => {
  void loadCategories();
});
</script>
