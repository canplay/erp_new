<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5">社交账号</div>
      <q-space />
      <q-btn color="primary" icon="add" label="添加账号" @click="showAddDialog = true" />
    </div>
    <q-table :rows="accountStore.accounts" :columns="columns" row-key="id" flat bordered :loading="accountStore.loading">
      <template v-slot:body-cell-platform="{ row }">
        <td><q-badge :color="platformColor(row.platform)">{{ row.platform }}</q-badge></td>
      </template>
      <template v-slot:body-cell-is_active="{ row }">
        <td><q-badge :color="row.is_active ? 'green' : 'grey'">{{ row.is_active ? '正常' : '停用' }}</q-badge></td>
      </template>
      <template v-slot:body-cell-actions="{ row }">
        <td class="text-center"><q-btn dense flat icon="delete" color="negative" @click="confirmDelete(row)" /></td>
      </template>
    </q-table>
    <q-dialog v-model="showAddDialog">
      <q-card style="min-width:400px">
        <q-card-section><div class="text-h6">添加社交账号</div></q-card-section>
        <q-card-section>
          <q-select v-model="form.platform" :options="platforms" label="平台" outlined />
          <q-input v-model="form.account_name" label="账号名称" outlined class="q-mt-sm" />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="取消" v-close-popup />
          <q-btn color="primary" label="保存" @click="saveAccount" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useAccountStore } from '@/stores/account';
import type { SocialAccount } from '@/api/accounts';
import { useI18n } from 'vue-i18n';

const $q = useQuasar();
void useI18n();
const accountStore = useAccountStore();

const columns = [
  { name: 'account_name', label: '名称', field: 'account_name', align: 'left' as const },
  { name: 'platform', label: '平台', field: 'platform', align: 'left' as const },
  { name: 'is_active', label: '状态', field: 'is_active', align: 'left' as const },
  { name: 'created_at', label: '添加时间', field: 'created_at', align: 'left' as const },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
];
const platforms = ['weibo', 'wechat_mp', 'xiaohongshu', 'douyin', 'bilibili', 'custom'];
const showAddDialog = ref(false);
const form = ref({ platform: '', account_name: '' });

function platformColor(p: string) {
  const map: Record<string, string> = { weibo: 'red', bilibili: 'blue', xiaohongshu: 'orange', douyin: 'purple', wechat_mp: 'green', custom: 'grey' };
  return map[p] || 'grey';
}

async function saveAccount() {
  try {
    await accountStore.createAccount(form.value);
    showAddDialog.value = false;
    form.value = { platform: '', account_name: '' };
    $q.notify({ type: 'positive', message: '账号已添加' });
  } catch {
    $q.notify({ type: 'negative', message: '添加失败' });
  }
}

function confirmDelete(row: SocialAccount) {
  $q.dialog({
    title: '确认删除',
    message: `确定删除账号「${row.account_name}」？此操作不可撤销。`,
    cancel: true,
    persistent: true,
  }).onOk(() => {
    void (async () => {
      try {
        await accountStore.deleteAccount(row.id);
        $q.notify({ type: 'positive', message: '已删除' });
      } catch {
        $q.notify({ type: 'negative', message: '删除失败' });
      }
    })();
  });
}

onMounted(() => void accountStore.fetchAccounts());
</script>
