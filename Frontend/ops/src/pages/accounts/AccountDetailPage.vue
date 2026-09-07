<template>
  <q-page class="q-pa-md">
    <q-btn flat icon="arrow_back" label="返回" @click="$router.back()" class="q-mb-md" />
    <div v-if="account">
      <div class="row items-center">
        <q-avatar size="64px" class="q-mr-md"><q-icon :name="platformIcon(account.platform)" size="40px" /></q-avatar>
        <div>
          <div class="text-h4">{{ account.account_name }}</div>
          <div class="text-grey">{{ account.platform }} · {{ account.is_active ? '正常' : '停用' }}</div>
        </div>
      </div>
      <q-separator class="q-my-md" />
      <div class="row q-col-gutter-md">
        <div class="col-12 col-md-6">
          <q-card bordered>
            <q-card-section><div class="text-h6">账号信息</div></q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>平台</q-item-label><q-item-label caption>{{ account.platform }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>名称</q-item-label><q-item-label caption>{{ account.account_name }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>平台ID</q-item-label><q-item-label caption>{{ account.account_id || '-' }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>状态</q-item-label><q-item-label caption><q-badge :color="account.is_active ? 'green' : 'grey'">{{ account.is_active ? '启用' : '停用' }}</q-badge></q-item-label></q-item-section></q-item>
            </q-list>
          </q-card>
        </div>
        <div class="col-12 col-md-6">
          <q-card bordered>
            <q-card-section><div class="text-h6">数据概览</div></q-card-section>
            <q-card-section class="text-center text-grey">
              <q-icon name="bar_chart" size="48px" />
              <div class="q-mt-sm">对接平台 API 后可查看数据</div>
            </q-card-section>
          </q-card>
        </div>
      </div>
    </div>
    <div v-else class="text-center q-py-xl"><q-spinner size="40px" /><div class="q-mt-sm text-grey">加载中...</div></div>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useAccountStore } from '@/stores/account';
import type { SocialAccount } from '@/api/accounts';

const route = useRoute();
const accountStore = useAccountStore();
const account = ref<SocialAccount | null>(null);

function platformIcon(p: string) {
  return ({ weibo: 'alternate_email', bilibili: 'smart_display', xiaohongshu: 'photo', douyin: 'music_video', wechat_mp: 'chat' } as Record<string, string>)[p] || 'public';
}

onMounted(() => {
  const id = (route.params as Record<string, string>).id ?? '';
  if (id) {
    const found = accountStore.accounts.find((a) => a.id === id);
    if (found) {
      account.value = found;
    }
  }
});
</script>
