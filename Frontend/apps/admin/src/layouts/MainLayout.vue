<template>
  <q-layout view="hHh Lpr lff">
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-btn
          flat
          dense
          round
          icon="menu"
          @click="leftDrawerOpen = !leftDrawerOpen"
          :title="'菜单'"
        />
        <q-toolbar-title>特种设备安全管理 · 监管后台</q-toolbar-title>
        <q-space />
        <MeilisearchBox class="q-mr-sm" style="min-width: 280px; max-width: 400px" />
        <q-btn flat round icon="notifications" to="/messages" :title="'通知'">
          <q-badge v-if="exportStore.pendingCount" color="amber" floating transparent>
            {{ exportStore.pendingCount }}
          </q-badge>
        </q-btn>
        <q-btn flat round icon="account_circle" to="/profile" :title="'账户'" />
        <q-btn flat round icon="logout" :title="i18nT('nav_logout')" @click="onLogout" />
      </q-toolbar>
    </q-header>

    <q-drawer v-model="leftDrawerOpen" show-if-above bordered class="bg-grey-1 flex column no-wrap">
      <q-scroll-area class="col" style="flex: 1 1 auto">
        <q-list padding>
          <q-item-label header>监管后台导航</q-item-label>

          <template v-for="(group, groupIndex) in visibleGroups" :key="group.key">
            <q-expansion-item
              :icon="group.icon"
              :label="group.label"
              :default-opened="groupIndex === 0"
              expand-separator
            >
              <q-list class="q-pl-md">
                <q-item
                  v-for="link in group.links"
                  :key="link.to"
                  clickable
                  :to="link.to"
                  v-ripple
                  exact
                  active-class="text-primary"
                >
                  <q-item-section avatar><q-icon :name="link.icon" /></q-item-section>
                  <q-item-section>{{ link.title }}</q-item-section>
                </q-item>
              </q-list>
            </q-expansion-item>
          </template>

          <!-- 下载中心：异步导出/下载通知 -->
          <q-expansion-item
            icon="download"
            :label="i18nT('nav_download_center')"
            :caption="exportStore.pendingCount ? `${exportStore.pendingCount} 个生成中` : undefined"
            expand-separator
          >
            <q-list class="q-pl-md">
              <q-item v-if="!exportStore.tasks.length">
                <q-item-section>
                  <span class="text-grey-6 text-caption">暂无导出任务</span>
                </q-item-section>
              </q-item>
              <q-item v-for="t in exportStore.tasks" :key="t.id" class="q-pa-xs">
                <q-item-section avatar>
                  <q-spinner v-if="t.status === 'pending'" color="amber" size="20px" />
                  <q-icon v-else-if="t.status === 'ready'" name="check_circle" color="positive" />
                  <q-icon v-else name="error" color="negative" />
                </q-item-section>
                <q-item-section>
                  <q-item-label lines="1">{{ t.label }}</q-item-label>
                  <q-item-label caption>
                    <a v-if="t.status === 'ready' && t.url" :href="t.url" download>下载</a>
                    <span v-else-if="t.status === 'pending'">生成中…</span>
                    <span v-else>失败</span>
                  </q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-btn
                    dense
                    flat
                    round
                    icon="close"
                    size="sm"
                    @click="exportStore.dismiss(t.id)"
                  />
                </q-item-section>
              </q-item>
            </q-list>
          </q-expansion-item>
        </q-list>
      </q-scroll-area>

      <!-- 系统/平台版本信息 -->
      <q-separator />
      <div class="q-pa-sm text-caption text-grey-7">
        <div class="row items-center no-wrap">
          <q-icon name="verified_user" size="16px" class="q-mr-xs" />
          <span>监管后台</span>
          <q-space />
          <span>v{{ appVersion }}</span>
        </div>
        <div class="text-grey-5 q-mt-xs">特种设备安全管理系统 · 平台端</div>
      </div>
    </q-drawer>

    <q-page-container>
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { logout } from '@erp-new-frontend-monorepo/boot/alova';
import { useAuthStore } from '@erp-new-frontend-monorepo/stores';
import { CAP_SCOPE, hasCap } from '@erp-new-frontend-monorepo/capabilities';
import { useExportTasksStore } from '@/stores/exportTasks';
import { MeilisearchBox } from '@erp-new-frontend-monorepo/components';

const router = useRouter();
const leftDrawerOpen = ref(false);
const exportStore = useExportTasksStore();

// 前端版本（package.json version 固定 0.0.1；硬编码避免 <script setup> 顶层 await 使组件变 async 白屏）
const appVersion = ref('0.0.1');

async function onLogout() {
  logout();
  await router.replace('/auth/login');
}

interface Link {
  title: string;
  icon: string;
  to: string;
  cap: string;
}
interface Group {
  key: string;
  label: string;
  icon: string;
  links: Link[];
}

const groups: Group[] = [
  {
    key: 'overview',
    label: '总览概览',
    icon: 'dashboard',
    links: [
      { title: '总览', icon: 'dashboard', to: '/', cap: 'equipment' },
      { title: '数据分析', icon: 'insights', to: '/analytics', cap: 'analytics' },
      { title: '态势感知', icon: 'monitor_heart', to: '/health', cap: 'health' },
    ],
  },
  {
    key: 'safety',
    label: '安全日周月管控',
    icon: 'shield',
    links: [
      { title: '日管控监控', icon: 'check_circle', to: '/daily-control', cap: 'daily-control' },
      { title: '周排查监控', icon: 'search', to: '/weekly-check', cap: 'weekly-check' },
      { title: '月调度监控', icon: 'calendar_month', to: '/monthly-report', cap: 'monthly-report' },
      { title: '隐患统计', icon: 'warning', to: '/hidden-danger', cap: 'hidden-danger' },
      { title: '知识图谱', icon: 'hub', to: '/knowledge-graph', cap: 'knowledge-graph' },
    ],
  },
  {
    key: 'equipment',
    label: '设备与档案',
    icon: 'inventory_2',
    links: [
      { title: '设备档案', icon: 'inventory_2', to: '/equipment', cap: 'equipment' },
      { title: '商品目录', icon: 'storefront', to: '/catalog', cap: 'catalog' },
    ],
  },
  {
    key: 'people',
    label: '人员与组织',
    icon: 'badge',
    links: [
      { title: '人员管理', icon: 'badge', to: '/personnel', cap: 'personnel' },
      { title: '用户管理', icon: 'people', to: '/users', cap: 'users-platform' },
      { title: '角色管理', icon: 'manage_accounts', to: '/roles', cap: 'roles' },
      { title: '用户组', icon: 'groups', to: '/groups', cap: 'user-groups' },
    ],
  },
  {
    key: 'platform',
    label: '平台运营',
    icon: 'hub',
    links: [
      { title: '租户管理', icon: 'domain', to: '/tenants', cap: 'tenant-management' },
      { title: '监管对接', icon: 'policy', to: '/regulatory', cap: 'regulatory' },
      { title: '工单', icon: 'support_agent', to: '/tickets', cap: 'tickets' },
      { title: '模拟登录', icon: 'swap_horiz', to: '/impersonation', cap: 'impersonation' },
    ],
  },
  {
    key: 'system',
    label: '系统管理',
    icon: 'settings',
    links: [
      { title: '审计日志', icon: 'receipt_long', to: '/audits', cap: 'audit-platform' },
      { title: '会话管理', icon: 'devices', to: '/sessions', cap: 'sessions-platform' },
      { title: 'Webhook', icon: 'webhook', to: '/webhooks', cap: 'webhooks' },
      { title: '文件管理', icon: 'folder', to: '/files', cap: 'files' },
      { title: '智能识别', icon: 'document_scanner', to: '/ocr', cap: 'ocr' },
    ],
  },
  {
    key: 'personal',
    label: '个人与消息',
    icon: 'person',
    links: [
      { title: '消息中心', icon: 'notifications', to: '/messages', cap: 'messages' },
      { title: '聊天', icon: 'chat', to: '/chat', cap: 'chat' },
      { title: '个人中心', icon: 'account_circle', to: '/profile', cap: 'profile' },
      { title: '套餐订阅', icon: 'payments', to: '/billing', cap: 'billing' },
    ],
  },
];

// 平台管理功能：仅 Admin/超级管理员可见（能力门控单一可信源）
const ADMIN_ROLES = ['Admin', 'SuperAdmin', '超级管理员', 'admin', 'superadmin'];
const roleNames = ref<string[]>([]);
const authStore = useAuthStore();

onMounted(async () => {
  try {
    const me = await authStore.fetchUser();
    roleNames.value = me?.roleNames || [];
  } catch {
    // 获取角色失败时默认放行（显示全量菜单）
  }
});

const isAdmin = computed(() => roleNames.value.some((r) => ADMIN_ROLES.includes(r)));

const visibleGroups = computed(() =>
  groups
    .map((g) => ({
      ...g,
      links: g.links.filter((l) => {
        // 能力门控（platform 视角）：shared/platform 能力才显示，tenant 专有入口隐藏
        if (!hasCap(l.cap, 'platform')) return false;
        // 平台专有能力额外叠加管理员角色门控（最小权限）
        if (CAP_SCOPE[l.cap] === 'platform' && !isAdmin.value) return false;
        return true;
      }),
    }))
    .filter((g) => g.links.length > 0),
);
</script>
