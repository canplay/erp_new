<template>
  <q-page class="q-pa-md">
    <div class="row q-col-gutter-md">
      <!-- 工具栏 -->
      <div class="col-12">
        <q-card>
          <q-card-section class="row q-gutter-md items-center">
            <q-input
              v-model="searchKeyword"
              dense
              outlined
              :placeholder="$t('common.searchPlaceholder')"
              style="width: 300px"
              @keyup.enter="handleSearch"
            >
              <template #prepend>
                <q-icon name="search" />
              </template>
            </q-input>

            <q-select
              v-model="filterStatus"
              :options="statusOptions"
              dense
              outlined
              clearable
              emit-value
              map-options
              style="width: 150px"
              :label="$t('common.status')"
            />

            <q-space />

            <q-btn
              color="positive"
              icon="add"
              :label="$t('common.newWorkflow')"
              @click="handleCreate"
            />
          </q-card-section>
        </q-card>
      </div>

      <!-- 工作流列表 -->
      <div class="col-12">
        <q-table
          :rows="workflowList"
          :columns="columns"
          :loading="loading"
          :pagination="pagination"
          row-key="id"
          @request="onTableRequest"
        >
          <template #body-cell-status="props">
            <q-td :props="props">
              <q-badge
                :color="getStatusColor(props.row.status)"
                :label="getStatusLabel(props.row.status)"
              />
            </q-td>
          </template>

          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn-group flat>
                <q-btn
                  flat
                  dense
                  color="primary"
                  icon="visibility"
                  @click="handleView(props.row)"
                >
                  <q-tooltip>查看</q-tooltip>
                </q-btn>
                <q-btn
                  flat
                  dense
                  color="primary"
                  icon="edit"
                  @click="handleEdit(props.row)"
                >
                  <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
                </q-btn>
                <q-btn
                  flat
                  dense
                  color="negative"
                  icon="delete"
                  @click="handleDelete(props.row)"
                >
                  <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
                </q-btn>
              </q-btn-group>
            </q-td>
          </template>

          <template #no-data>
            <EmptyState
              icon="account_tree"
              :message="$t('common.noWorkflowDefinition')"
              :action-text="$t('common.createWorkflow')"
              @action="handleCreate"
            />
          </template>
        </q-table>
      </div>
    </div>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 500px; max-width: 800px">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? '编辑工作流' : '新建工作流' }}</div>
        </q-card-section>

        <q-card-section>
          <q-form @submit="handleSubmit">
            <q-input
              v-model="formData.name"
              :label="$t('common.workflowName')"
              outlined
              :rules="[val => !!val || '请输入名称']"
            />

            <q-input
              v-model="formData.description"
              :label="$t('common.description')"
              outlined
              type="textarea"
              rows="3"
              class="q-mt-md"
            />

            <div v-if="isEdit" class="q-mt-md">
              <q-select
                v-model="formData.status"
                :options="statusOptions"
                emit-value
                map-options
                :label="$t('common.status')"
                outlined
              />
            </div>
          </q-form>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.save')" type="submit" @click="handleSubmit" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 确认删除对话框 -->
    <ConfirmDialog
      v-model="showDeleteDialog"
      :title="$t('common.confirmDelete')"
      message="确定要删除该工作流吗？此操作不可撤销。"
      @confirm="confirmDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { listWorkflows, createWorkflow, updateWorkflow, deleteWorkflow, type Workflow, type WorkflowStatus } from '@/api/workflow';
import EmptyState from '@erp-new-frontend-monorepo/components/src/EmptyState.vue';
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
/**
 * @brief 工作流列表页面
 * 提供工作流的增删改查功能
 */
const router = useRouter();
const $q = useQuasar();

// ============ 状态 ============
const loading = ref(false);
const searchKeyword = ref('');
const filterStatus = ref<WorkflowStatus | null>(null);
const workflowList = ref<Workflow[]>([]);

const showDialog = ref(false);
const showDeleteDialog = ref(false);
const isEdit = ref(false);
const currentWorkflow = ref<Workflow | null>(null);

const formData = ref<{
  name: string;
  description: string;
  status: WorkflowStatus;
}>({
  name: '',
  description: '',
  status: 'draft',
});

// ============ 表格配置 ============
const columns = [
  { name: 'name', label: '名称', field: 'name', align: 'left' as const, sortable: true },
  { name: 'description', label: '描述', field: 'description', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'version', label: '版本', field: 'version', align: 'center' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'center' as const, sortable: true },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
];

const pagination = ref({
  page: 1,
  rowsPerPage: 20,
  rowsNumber: 0,
});

// ============ 选项 ============
const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发布', value: 'published' },
  { label: '已禁用', value: 'disabled' },
];

// ============ 方法 ============
function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    draft: 'grey',
    published: 'positive',
    disabled: 'negative',
  };
  return colors[status] || 'grey';
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    draft: '草稿',
    published: '已发布',
    disabled: '已禁用',
  };
  return labels[status] || status;
}

async function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  loading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: props.pagination.page,
      page_size: props.pagination.rowsPerPage,
    };
    if (filterStatus.value) params.status = filterStatus.value;
    if (searchKeyword.value) params.keyword = searchKeyword.value;
    const response = await listWorkflows(params);

    const respData = response.data;
    workflowList.value = respData?.data?.list || []; // API 返回 list
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  } catch {
    $q.notify({ type: 'negative', message: '获取工作流列表失败' });
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  pagination.value.page = 1;
  void onTableRequest({ pagination: pagination.value });
}

function handleCreate() {
  isEdit.value = false;
  formData.value = { name: '', description: '', status: 'draft' };
  showDialog.value = true;
}

function handleEdit(row: Workflow) {
  isEdit.value = true;
  currentWorkflow.value = row;
  formData.value = {
    name: row.name,
    description: row.description || '',
    status: row.status || undefined,
  };
  showDialog.value = true;
}

function handleView(row: Workflow) {
  void router.push(`/workflow/${row.id}/design`);
}

function handleDelete(row: Workflow) {
  currentWorkflow.value = row;
  showDeleteDialog.value = true;
}

async function handleSubmit() {
  try {
    if (isEdit.value && currentWorkflow.value) {
      await updateWorkflow(currentWorkflow.value.id, formData.value);
      $q.notify({ type: 'positive', message: '更新成功' });
    } else {
      await createWorkflow(formData.value);
      $q.notify({ type: 'positive', message: '创建成功' });
    }
    showDialog.value = false;
    void onTableRequest({ pagination: pagination.value });
  } catch {
    $q.notify({ type: 'negative', message: '获取工作流列表失败' });
  }
}

async function confirmDelete() {
  if (!currentWorkflow.value) return;

  try {
    await deleteWorkflow(currentWorkflow.value.id);
    $q.notify({ type: 'positive', message: '删除成功' });
    showDeleteDialog.value = false;
    void onTableRequest({ pagination: pagination.value });
  } catch {
    $q.notify({ type: 'negative', message: '删除失败' });
  }
}

// ============ 生命周期 ============
onMounted(() => {
  void onTableRequest({ pagination: pagination.value });
});
</script>
