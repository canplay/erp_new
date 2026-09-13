<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.s954c98') }}</div>
      <q-space />
      <q-btn color="primary" icon="group_add" :label="i18nT('raw.s76ec22')" @click="openCreate" />
    </div>

    <q-banner v-if="errorMessage" type="negative" rounded class="q-mb-md">
      {{ errorMessage }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.sdedda3')" @click="errorMessage = ''" />
      </template>
    </q-banner>

    <PageTable
      v-if="loading || rows.length > 0"
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      @request="onRequest"
    >
      <template #body-cell-roleNames="{ row }">
        <q-td>
          <template v-if="row.roleNames?.length">
            <q-chip
              v-for="role in row.roleNames"
              :key="role"
              dense
              size="sm"
              color="primary"
              text-color="white"
              class="q-mr-xs"
            >
              {{ role }}
            </q-chip>
          </template>
          <span v-else class="text-grey-6">—</span>
        </q-td>
      </template>
      <template #body-cell-createdAt="{ row }">
        <q-td>{{ formatDate(row.createdAt) }}</q-td>
      </template>
      <template #body-cell-actions="{ row }">
        <q-td>
          <q-btn
            flat
            dense
            icon="group"
            size="sm"
            @click="openMembers(row)"
            :title="i18nT('raw.se607fa')"
          />
          <q-btn
            flat
            dense
            icon="delete"
            size="sm"
            :loading="removingId === row.id"
            @click="confirmRemoveGroup(row)"
            :title="i18nT('raw.sbd7449')"
          />
        </q-td>
      </template>
    </PageTable>
    <EmptyState v-else :title="i18nT('raw.s32fbaa')" :hint="i18nT('raw.s8a5470')" />

    <ConfirmDialog
      v-model="deleteDialog"
      :title="i18nT('raw.sbd7449')"
      :message="`确认删除用户组「${deleteTarget?.name}」？该操作不可恢复。`"
      :loading="deleteConfirming"
      @confirm="doRemoveGroup"
    />

    <q-dialog v-model="createDialog">
      <q-card style="min-width: 420px">
        <q-card-section class="text-h6">{{ i18nT('raw.s76ec22') }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="form.name"
            :label="i18nT('raw.scef53c')"
            filled
            :rules="[(v) => !!v || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
          <q-input
            v-model="form.description"
            :label="i18nT('raw.s9e58f1')"
            filled
            type="textarea"
            class="q-mt-md"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.s3089ce')"
            color="primary"
            :loading="creating"
            @click="createGroup"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="membersDialog">
      <q-card
        style="
          min-width: 560px;
          max-width: 90vw;
          display: flex;
          flex-direction: column;
          max-height: 85vh;
        "
      >
        <q-card-section class="row items-center">
          <div class="text-h6">成员管理 — {{ currentGroup?.name }}</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-card-section>
          <div class="row items-center q-gutter-sm">
            <q-input
              v-model="newMemberId"
              :label="i18nT('raw.sb99d44')"
              filled
              dense
              class="col"
              @keyup.enter="addMember"
            />
            <q-btn
              color="primary"
              icon="person_add"
              :label="i18nT('raw.sd885c1')"
              :loading="addingMember"
              @click="addMember"
            />
          </div>
          <div class="q-mt-md" style="flex: 1 1 auto; min-height: 0; display: flex">
            <q-scroll-area class="col">
              <PageTable
                v-if="loadingMembers || members.length > 0"
                :rows="members"
                :columns="memberColumns"
                :row-key="rowKey"
                :loading="loadingMembers"
                :total="members.length"
              >
                <template #body-cell-actions="{ row }">
                  <q-td>
                    <q-btn
                      flat
                      dense
                      icon="person_remove"
                      size="sm"
                      :loading="removingMemberId === memberId(row)"
                      @click="removeMember(row)"
                      :title="i18nT('raw.sfcbd09')"
                    />
                  </q-td>
                </template>
              </PageTable>
              <EmptyState v-else :title="i18nT('raw.s9bd662')" :hint="i18nT('raw.sce31ad')" />
            </q-scroll-area>
          </div>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { groupsApi, type GroupItem } from '../../api';
import { EmptyState, PageTable, ConfirmDialog } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

interface GroupMember {
  id?: string;
  userId?: string;
  userName?: string;
  name?: string;
  email?: string;
}

const $q = useQuasar();

const { rows, total, loading, page, size, load } = useTableState<GroupItem>({ size: 20 });
const errorMessage = ref('');
const createDialog = ref(false);
const creating = ref(false);
const form = ref({ name: '', description: '' });
const removingId = ref<string | null>(null);
const deleteTarget = ref<GroupItem | null>(null);
const deleteDialog = ref(false);
const deleteConfirming = ref(false);

const membersDialog = ref(false);
const currentGroup = ref<GroupItem | null>(null);
const members = ref<GroupMember[]>([]);
const loadingMembers = ref(false);
const newMemberId = ref('');
const addingMember = ref(false);
const removingMemberId = ref<string | null>(null);

const columns = [
  { name: 'name', label: i18nT('raw.scef53c'), field: 'name', align: 'left' as const },
  {
    name: 'description',
    label: i18nT('raw.s9e58f1'),
    field: 'description',
    align: 'left' as const,
  },
  {
    name: 'memberCount',
    label: i18nT('raw.s102887'),
    field: 'memberCount',
    align: 'center' as const,
  },
  { name: 'roleNames', label: i18nT('raw.s44225f'), field: 'roleNames', align: 'left' as const },
  { name: 'createdAt', label: i18nT('raw.saf9956'), field: 'createdAt', align: 'left' as const },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions', align: 'right' as const },
];

const memberColumns = [
  {
    name: 'userId',
    label: i18nT('raw.sb99d44'),
    field: (r: GroupMember) => memberId(r),
    align: 'left' as const,
  },
  {
    name: 'userName',
    label: i18nT('raw.s9fcdad'),
    field: (r: GroupMember) => r.userName ?? r.name ?? '—',
    align: 'left' as const,
  },
  {
    name: 'email',
    label: i18nT('raw.s54b275'),
    field: (r: GroupMember) => r.email ?? '—',
    align: 'left' as const,
  },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions', align: 'right' as const },
];

function memberId(m: GroupMember) {
  return m.userId ?? m.id ?? '';
}

function rowKey(r: GroupMember) {
  return memberId(r);
}

function toast(type: 'positive' | 'negative', message: string) {
  if (typeof $q.notify === 'function') {
    $q.notify({ type, message });
  }
}

async function loadGroups() {
  const res = await groupsApi.list(page.value, size.value);
  rows.value = res.items || [];
  total.value = res.total || 0;
}

async function onRequest({ page: p, rowsPerPage: s }: { page: number; rowsPerPage: number }) {
  page.value = p;
  size.value = s;
  await load(loadGroups);
}

function openCreate() {
  form.value = { name: '', description: '' };
  createDialog.value = true;
}

async function createGroup() {
  if (!form.value.name.trim()) return;
  creating.value = true;
  try {
    const payload: { name: string; description?: string } = { name: form.value.name.trim() };
    if (form.value.description.trim()) {
      payload.description = form.value.description.trim();
    }
    await groupsApi.create(payload);
    createDialog.value = false;
    toast('positive', i18nT('raw.s64e540'));
    await load(loadGroups);
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.sb62cb3');
  } finally {
    creating.value = false;
  }
}

function confirmRemoveGroup(g: GroupItem) {
  deleteTarget.value = g;
  deleteDialog.value = true;
}

async function removeGroup(g: GroupItem) {
  removingId.value = g.id;
  try {
    await groupsApi.remove(g.id);
    toast('positive', i18nT('raw.s83916d'));
    await load(loadGroups);
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.s2324d0');
  } finally {
    removingId.value = null;
  }
}

async function doRemoveGroup() {
  if (!deleteTarget.value) return;
  deleteConfirming.value = true;
  try {
    await removeGroup(deleteTarget.value);
    deleteDialog.value = false;
  } finally {
    deleteConfirming.value = false;
  }
}

async function openMembers(g: GroupItem) {
  currentGroup.value = g;
  members.value = [];
  newMemberId.value = '';
  membersDialog.value = true;
  await loadMembers();
}

async function loadMembers() {
  if (!currentGroup.value) return;
  loadingMembers.value = true;
  try {
    members.value = (await groupsApi.members(currentGroup.value.id)) as GroupMember[];
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.sfa9d27');
  } finally {
    loadingMembers.value = false;
  }
}

async function addMember() {
  if (!currentGroup.value) return;
  const userId = newMemberId.value.trim();
  if (!userId) return;
  addingMember.value = true;
  try {
    await groupsApi.addMember(currentGroup.value.id, userId);
    newMemberId.value = '';
    toast('positive', i18nT('raw.s5917c1'));
    await load(loadMembers);
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.sf2072f');
  } finally {
    addingMember.value = false;
  }
}

async function removeMember(m: GroupMember) {
  if (!currentGroup.value) return;
  const userId = memberId(m);
  if (!userId) return;
  removingMemberId.value = userId;
  try {
    await groupsApi.removeMember(currentGroup.value.id, userId);
    toast('positive', i18nT('raw.s457eaa'));
    await load(loadMembers);
    await load(loadGroups);
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.s32bb41');
  } finally {
    removingMemberId.value = null;
  }
}

onMounted(() => load(loadGroups));
</script>
