<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.s92b96a') }}</div>
      <q-space />
      <q-btn color="primary" icon="add" :label="i18nT('raw.s974d0e')" @click="openCreate" />
    </div>

    <PageTable
      v-if="loading || rows.length > 0"
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      @request="onRequest"
    >
      <template #body-cell-permissions="{ row }">
        <q-td>
          <div class="permission-chips">
            <q-chip
              v-for="p in (row.permissions || []).slice(0, 5)"
              :key="p"
              size="xs"
              color="primary"
              text-color="white"
              :label="shortName(p)"
            />
            <span v-if="(row.permissions || []).length > 5" class="text-grey text-caption"
              >+{{ (row.permissions || []).length - 5 }}</span
            >
          </div>
        </q-td>
      </template>
      <template #body-cell-actions="{ row }">
        <q-td>
          <q-btn
            flat
            dense
            color="negative"
            icon="delete"
            size="sm"
            @click="confirmDelete(row)"
            :title="i18nT('raw.sa18077')"
          />
        </q-td>
      </template>
    </PageTable>
    <EmptyState v-else :title="i18nT('raw.s710590')" :hint="i18nT('raw.s2bfa56')" />

    <!-- 新建角色弹窗 -->
    <q-dialog v-model="createDialog">
      <q-card style="min-width: 420px">
        <q-card-section class="text-h6">{{ i18nT('raw.s974d0e') }}</q-card-section>
        <q-card-section>
          <q-form @submit="createRole" class="q-gutter-sm">
            <q-input
              v-model="form.name"
              :label="i18nT('raw.sc3140c')"
              filled
              dense
              :rules="[(v) => !!v || i18nT('raw.sc9912d')]"
              lazy-rules
            />
            <q-input
              v-model="form.description"
              :label="i18nT('raw.s9e58f1')"
              filled
              dense
              type="textarea"
            />
            <div v-if="createError" class="text-negative text-caption">{{ createError }}</div>
            <q-btn
              type="submit"
              :label="i18nT('raw.s3089ce')"
              color="primary"
              class="full-width"
              :loading="creating"
            />
          </q-form>
        </q-card-section>
      </q-card>
    </q-dialog>

    <!-- 删除角色确认弹窗 -->
    <q-dialog v-model="deleteDialog">
      <q-card style="min-width: 400px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ i18nT('raw.sa18077') }}</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-card-section>
          确认删除角色 <b>{{ deleteTarget?.name }}</b
          >？该操作不可撤销。
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sbd7449')"
            color="negative"
            :loading="deleting"
            @click="doDelete"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { identityApi, type AdminRole } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const columns = [
  { name: 'name', label: i18nT('raw.sc3140c'), field: 'name', sortable: true },
  { name: 'description', label: i18nT('raw.s9e58f1'), field: 'description' },
  { name: 'permissions', label: i18nT('raw.s1aea70'), field: 'permissions' },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions' },
];

const { rows, total, loading, load, page, size, goToPage, setSize } = useTableState<AdminRole>({
  size: 20,
});
const createDialog = ref(false);
const creating = ref(false);
const form = ref({ name: '', description: '' });
const createError = ref('');
const deleteDialog = ref(false);
const deleteTarget = ref<AdminRole | null>(null);
const deleting = ref(false);

const $q = useQuasar();

function shortName(perm: string) {
  return perm.replace(/^Permissions\./, '').replace(/\./g, ' ');
}

async function doLoad() {
  await load(() =>
    identityApi.roles(page.value, size.value).then((res) => {
      rows.value = res.items;
      total.value = res.total;
    }),
  );
}

function onRequest({ page: p, rowsPerPage }: { page: number; rowsPerPage: number }) {
  if (rowsPerPage !== size.value) {
    void setSize(rowsPerPage);
  } else {
    void goToPage(p);
  }
}

function openCreate() {
  form.value = { name: '', description: '' };
  createError.value = '';
  createDialog.value = true;
}

async function createRole() {
  creating.value = true;
  createError.value = '';
  try {
    const payload: { name: string; description?: string } = { name: form.value.name.trim() };
    if (form.value.description.trim()) payload.description = form.value.description.trim();
    await identityApi.createRole(payload);
    createDialog.value = false;
    $q.notify({ type: 'positive', message: i18nT('raw.s2eeb7e') });
    await doLoad();
  } catch (e) {
    createError.value = (e as { message?: string })?.message || i18nT('raw.s20f5e0');
  } finally {
    creating.value = false;
  }
}

function confirmDelete(r: AdminRole) {
  deleteTarget.value = r;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  try {
    await identityApi.deleteRole(deleteTarget.value.id);
    deleteDialog.value = false;
    $q.notify({ type: 'positive', message: i18nT('raw.s39c435') });
    await doLoad();
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: (e as { message?: string })?.message || i18nT('raw.s75a0f4'),
    });
  } finally {
    deleting.value = false;
  }
}

onMounted(doLoad);
</script>

<style scoped>
.permission-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
  max-width: 480px;
}
</style>
