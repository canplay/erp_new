<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('department.title') }}</div>

    <div class="row q-col-gutter-md">
      <!-- 部门树 -->
      <div class="col-12 col-md-4">
        <q-card bordered>
          <q-card-section>
            <div class="row items-center justify-between q-mb-md">
              <div class="text-h6">{{ $t('department.tree') }}</div>
              <q-btn color="positive" size="sm" icon="add" round @click="openDepartmentDialog()" />
            </div>
            <q-tree
              :nodes="departmentTree"
              node-key="id"
              label-key="name"
              :selected="selectedDepartmentId"
              selection-strategy="leaf"
              default-expand-all
              @update:selected="onDepartmentSelect"
            >
              <template v-slot:default-header="props">
                <div class="row items-center justify-between" style="width: 100%">
                  <div class="row items-center">
                    <q-icon
                      :name="props.node.parent_id ? 'folder' : 'corporate_fare'"
                      :color="props.node.parent_id ? 'secondary' : 'primary'"
                      class="q-mr-sm"
                    />
                    <span>{{ props.node.name }}</span>
                  </div>
                  <div class="row q-gutter-xs">
                    <q-btn flat dense size="sm" icon="add" color="positive" @click.stop="openDepartmentDialog(props.node)" />
                    <q-btn flat dense size="sm" icon="edit" color="primary" @click.stop="openDepartmentDialog(props.node, true)" />
                    <q-btn flat dense size="sm" icon="delete" color="negative" @click.stop="handleDelete(props.node)" />
                  </div>
                </div>
              </template>
            </q-tree>
          </q-card-section>
        </q-card>
      </div>

      <!-- 部门详情 -->
      <div class="col-12 col-md-8">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6">{{ $t('department.details') }}</div>
          </q-card-section>

          <q-separator />

          <q-card-section v-if="currentDepartment">
            <q-list dense>
              <q-item>
                <q-item-section>
                  <q-item-label caption>{{ $t('department.id') }}</q-item-label>
                  <q-item-label class="text-body1">{{ currentDepartment.id }}</q-item-label>
                </q-item-section>
              </q-item>
              <q-item>
                <q-item-section>
                  <q-item-label caption>{{ $t('department.name') }}</q-item-label>
                  <q-item-label class="text-body1">{{ currentDepartment.name }}</q-item-label>
                </q-item-section>
              </q-item>
              <q-item v-if="currentDepartment.parent_id">
                <q-item-section>
                  <q-item-label caption>{{ $t('department.parentDepartment') }}</q-item-label>
                  <q-item-label class="text-body1">{{ getParentName(currentDepartment.parent_id) }}</q-item-label>
                </q-item-section>
              </q-item>
              <q-item v-if="currentDepartment.leader_name">
                <q-item-section>
                  <q-item-label caption>{{ $t('department.leader') }}</q-item-label>
                  <q-item-label class="text-body1">{{ currentDepartment.leader_name }}</q-item-label>
                </q-item-section>
              </q-item>
              <q-item v-if="currentDepartment.description">
                <q-item-section>
                  <q-item-label caption>{{ $t('department.description') }}</q-item-label>
                  <q-item-label class="text-body1">{{ currentDepartment.description }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
          <q-card-section v-else>
            <div class="text-grey-5 text-center q-pa-lg">
              {{ $t('department.selectToView') }}
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 创建/编辑部门弹窗 -->
    <q-dialog v-model="showDepartmentDialog" persistent>
      <q-card style="min-width: 450px">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? $t('department.edit') : $t('department.create') }}</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
            v-model="departmentForm.name"
            :label="$t('department.name')"
            outlined
            class="q-mb-md"
          />
          <q-select
            v-model="departmentForm.parent_id"
            :options="parentDepartmentOptions"
            :label="$t('department.parentDepartment')"
            outlined
            clearable
            emit-value
            map-options
            class="q-mb-md"
          />
          <q-select
            v-model="departmentForm.leader_id"
            :options="userOptions"
            :label="$t('department.leader')"
            outlined
            clearable
            emit-value
            map-options
            class="q-mb-md"
          />
          <q-input
            v-model="departmentForm.description"
            :label="$t('department.description')"
            outlined
            type="textarea"
            class="q-mb-md"
          />
          <q-input
            v-model.number="departmentForm.sort_order"
            :label="$t('department.sort_order')"
            outlined
            type="number"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.confirm')" @click="saveDepartment" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 确认删除弹窗 -->
    <ConfirmDialog
      ref="deleteDialogRef"
      :title="$t('department.confirmDelete')"
      :message="$t('department.confirmDeleteMessage', { name: pendingDeleteDepartment?.name ?? '' })"
      confirm-color="negative"
      @confirm="doDeleteDepartment"
    />
  </q-page>
</template>

<script setup lang="ts">
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import { useDepartmentList } from '@erp-new-frontend-monorepo/composables/src/useDepartmentList';;

const {
  selectedDepartmentId,
  showDepartmentDialog,
  isEdit,
  deleteDialogRef,
  pendingDeleteDepartment,
  departmentForm,
  departmentTree,
  currentDepartment,
  parentDepartmentOptions,
  userOptions,
  getParentName,
  onDepartmentSelect,
  openDepartmentDialog,
  saveDepartment,
  handleDelete,
  doDeleteDepartment,
} = useDepartmentList();
</script>

<style scoped>
/* 部门列表页样式 */
</style>
