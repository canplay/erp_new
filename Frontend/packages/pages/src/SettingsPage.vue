<template>
  <q-page class="settings-page q-pa-md">
    <div class="text-h5 q-mb-lg text-weight-bold">设置</div>

    <!-- 用户信息 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section>
        <div class="text-subtitle1 text-weight-bold q-mb-sm">用户信息</div>
        <div class="row items-center no-wrap">
          <q-avatar size="64px" color="primary" text-color="white" class="q-mr-md">
            {{ displayNameInitial }}
          </q-avatar>
          <div>
            <div class="text-weight-bold">{{ displayName }}</div>
            <div class="text-caption text-grey-6">{{ auth.userId }}</div>
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- 服务器配置 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section>
        <div class="text-subtitle1 text-weight-bold q-mb-sm">服务器</div>
        <q-item>
          <q-item-section>
            <q-item-label>Matrix 服务器</q-item-label>
            <q-item-label caption>
              <a :href="serverUrl" target="_blank" class="text-primary">{{ serverUrl }}</a>
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-card-section>
    </q-card>

    <!-- 操作 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section>
        <div class="text-subtitle1 text-weight-bold q-mb-sm">操作</div>
        <q-list separator>
          <q-item clickable v-ripple @click="clearCache">
            <q-item-section avatar>
              <q-icon name="delete_sweep" color="orange" />
            </q-item-section>
            <q-item-section>
              <q-item-label>清除本地缓存</q-item-label>
              <q-item-label caption>清除消息和房间的本地缓存数据</q-item-label>
            </q-item-section>
          </q-item>
          <q-item clickable v-ripple @click="handleLogout">
            <q-item-section avatar>
              <q-icon name="logout" color="red" />
            </q-item-section>
            <q-item-section>
              <q-item-label class="text-red">退出登录</q-item-label>
              <q-item-label caption>登出并清除登录凭证</q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>

    <div class="text-center text-grey-5 text-caption q-mt-xl">
      <div>Social v0.0.1</div>
      <div>基于 Matrix 协议 | Tuwunel 后端</div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useQuasar } from 'quasar';
import { config } from '@/services/config';

const $q = useQuasar();
const router = useRouter();
const auth = useAuthStore();

const serverUrl = computed(() => config.tuwunelUrl);

const displayName = computed(() => {
  if (!auth.userId) return '用户';
  return (auth.userId.split(':')[0] ?? '').replace('@', '');
});

const displayNameInitial = computed(() => {
  return (displayName.value ?? '?').charAt(0).toUpperCase();
});

function clearCache() {
  $q.dialog({
    title: '清除缓存',
    message: '确定要清除本地缓存数据吗？',
    cancel: true,
  }).onOk(() => {
    localStorage.removeItem('matrix_access_token');
    localStorage.removeItem('matrix_user_id');
    localStorage.removeItem('matrix_device_id');
    $q.notify({ type: 'positive', message: '缓存已清除' });
  });
}

function handleLogout() {
  $q.dialog({
    title: '退出登录',
    message: '确定要退出当前账号吗？',
    ok: '确认退出',
    cancel: true,
    persistent: true,
  }).onOk(() => {
    auth.logout().then(() => {
      void router.push('/login');
      $q.notify({ type: 'info', message: '已退出登录' });
    }).catch((e: unknown) => {
      $q.notify({ type: 'negative', message: '退出失败: ' + (e instanceof Error ? e.message : '未知错误') });
    });
  });
}
</script>
