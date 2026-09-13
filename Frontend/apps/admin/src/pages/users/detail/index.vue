<template>
  <q-page class="erp-page">
    <q-breadcrumbs class="q-mb-sm">
      <q-breadcrumbs-el :label="i18nT('breadcrumb_users')" to="/users" />
      <q-breadcrumbs-el :label="i18nT('breadcrumb_user_detail')" />
    </q-breadcrumbs>
    <div class="row items-center q-mb-md">
      <q-btn
        flat
        round
        dense
        icon="arrow_back"
        @click="router.back()"
        :title="i18nT('raw.s4b2ea3')"
      />
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.sca349f') }}</div>
      <q-space />
      <template v-if="user">
        <q-btn
          flat
          :color="user.isActive ? 'negative' : 'positive'"
          :icon="user.isActive ? 'block' : 'check_circle'"
          :label="user.isActive ? i18nT('raw.sd64e4d') : i18nT('raw.sc8a730')"
          :loading="toggling"
          :disable="!isPlatformUser"
          :title="isPlatformUser ? '' : '租户用户请在租户端管理'"
          @click="toggleStatus"
        />
        <q-btn
          flat
          color="primary"
          icon="badge"
          :label="i18nT('raw.sd3abcb')"
          class="q-ml-sm"
          :disable="!isPlatformUser"
          :title="isPlatformUser ? '' : '租户用户请在租户端管理'"
          @click="openAssignRoles"
        />
      </template>
    </div>

    <q-card v-if="user" class="q-mb-md">
      <q-card-section class="row items-center">
        <q-avatar size="56px" color="primary" text-color="white" class="q-mr-md">
          {{ (user.userName || user.email || 'U').charAt(0).toUpperCase() }}
        </q-avatar>
        <div>
          <div class="text-h6">{{ fullName }}</div>
          <div class="text-caption text-grey-6">@{{ user.userName }}</div>
        </div>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-list dense padding>
          <q-item>
            <q-item-section avatar><q-icon name="mail" color="primary" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s54b275') }}</q-item-label>
              <q-item-label>{{ user.email }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="badge" color="primary" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sd84192') }}</q-item-label>
              <q-item-label>{{ fullName }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="phone" color="primary" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s5e8973') }}</q-item-label>
              <q-item-label>{{ user.phoneNumber || '—' }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar
              ><q-icon name="power_settings_new" color="primary"
            /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9fb403') }}</q-item-label>
              <q-item-label>
                <q-badge :color="user.isActive ? 'green' : 'red'">{{
                  user.isActive ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d')
                }}</q-badge>
              </q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="verified_user" color="primary" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s1b9006') }}</q-item-label>
              <q-item-label>
                <q-badge :color="user.emailConfirmed ? 'green' : 'orange'">{{
                  user.emailConfirmed ? i18nT('raw.s0dd599') : i18nT('raw.sad45f5')
                }}</q-badge>
              </q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="shield" color="primary" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s1a8426') }}</q-item-label>
              <q-item-label>
                <q-badge :color="user.twoFactorEnabled ? 'green' : 'grey'">{{
                  user.twoFactorEnabled ? i18nT('raw.s7b57ae') : i18nT('raw.sdbc7c2')
                }}</q-badge>
              </q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>

    <q-card v-if="user">
      <q-card-section class="text-h6">{{ i18nT('raw.sc747d4') }}</q-card-section>
      <q-card-section class="q-pt-none">
        <div v-if="assignedRoles.length" class="row q-gap-xs">
          <q-chip
            v-for="r in assignedRoles"
            :key="r"
            color="primary"
            text-color="white"
            size="sm"
            :label="r"
          />
        </div>
        <EmptyState
          v-else-if="rolesLoadFailed"
          title="角色信息在租户端查看"
          hint="平台跨租户查看用户角色需在对应租户的管理端操作"
          icon="lock_open"
        />
        <EmptyState v-else :title="i18nT('raw.s710590')" :hint="i18nT('raw.s1911b1')" />
      </q-card-section>
    </q-card>

    <!-- 会话管理（平台视角：按用户查/撤销，跨租户） -->
    <q-card v-if="user" class="q-mt-md">
      <q-card-section class="row items-center">
        <div class="text-h6">会话管理</div>
        <q-space />
        <q-btn
          outline
          color="negative"
          dense
          size="sm"
          icon="logout"
          label="撤销全部会话"
          :loading="sessionsLoading"
          :disable="!sessions.length"
          @click="revokeAllUserSessions"
        />
      </q-card-section>
      <q-card-section class="q-pt-none">
        <q-table
          flat
          bordered
          dense
          :rows="sessions"
          :columns="sessionColumns"
          row-key="id"
          hide-bottom
          :loading="sessionsLoading"
          :pagination="{ rowsPerPage: 0 }"
        >
          <template #body-cell-actions="{ row }">
            <q-td>
              <q-btn
                v-if="!row.isCurrent"
                dense
                flat
                size="sm"
                color="negative"
                label="撤销"
                @click="revokeSession(row)"
              />
              <q-badge v-else color="teal" label="当前" />
            </q-td>
          </template>
        </q-table>
        <EmptyState
          v-if="!sessionsLoading && !sessions.length"
          title="该用户暂无会话"
          hint="用户未登录或会话已全部过期"
        />
      </q-card-section>
    </q-card>

    <div v-if="loading" class="q-pa-xl column items-center text-grey-6">
      <q-spinner size="40px" class="q-mb-sm" />
      <div>{{ i18nT('raw.s795a79') }}</div>
    </div>
    <EmptyState v-else-if="!user" :title="i18nT('raw.sf628a7')" :hint="i18nT('raw.s5ce8a6')" />

    <!-- 分配角色弹窗 -->
    <q-dialog v-model="assignDialog">
      <q-card style="min-width: 420px">
        <q-card-section class="row items-center">
          <div class="text-h6">分配角色 — {{ fullName }}</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-card-section>
          <q-list v-if="roles.length">
            <q-item v-for="r in roles" :key="r.id" tag="label" v-ripple>
              <q-item-section>
                <q-item-label>{{ r.name }}</q-item-label>
                <q-item-label caption>{{ r.description }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-checkbox v-model="selectedRoles" :val="r.name" />
              </q-item-section>
            </q-item>
          </q-list>
          <EmptyState v-else :title="i18nT('raw.s710590')" :hint="i18nT('raw.sc34096')" />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sfe9512')"
            color="primary"
            :loading="saving"
            @click="saveRoles"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { identityApi, type IdentityUser, type IdentityRole } from '@/api';
import { EmptyState } from '@erp-new-frontend-monorepo/components';

const route = useRoute();
const router = useRouter();

const id = String((route.params as { id?: string }).id ?? '');

const user = ref<IdentityUser | null>(null);
const roles = ref<IdentityRole[]>([]);
const assignedRoles = ref<string[]>([]);
const rolesLoadFailed = ref(false);
/** 平台可管用户：root 租户用户或未标租户（租户用户由租户管理员自管，平台只读+会话管理） */
const isPlatformUser = computed(() => !user.value?.tenantId || user.value.tenantId === 'root');
const loading = ref(false);
const toggling = ref(false);
const assignDialog = ref(false);
const selectedRoles = ref<string[]>([]);
const saving = ref(false);

const fullName = computed(
  () => `${user.value?.firstName || ''} ${user.value?.lastName || ''}`.trim() || '—',
);

async function load() {
  loading.value = true;
  try {
    user.value = await identityApi.user(id);
    if (user.value) {
      // 跨租户查看时 userRoles 为租户内端点（root 查租户用户 404）→ 角色区降级为只读提示
      try {
        const ur = (await identityApi.userRoles(id)) || [];
        assignedRoles.value = ur
          .map(
            (r: { roleName?: string; roleId?: string; enabled?: boolean }) =>
              r.roleName || r.roleId || '',
          )
          .filter(Boolean);
        rolesLoadFailed.value = false;
      } catch {
        assignedRoles.value = [];
        rolesLoadFailed.value = true;
      }
    }
  } finally {
    loading.value = false;
  }
}

async function toggleStatus() {
  if (!user.value) return;
  toggling.value = true;
  try {
    await identityApi.toggleUserStatus(id, !user.value.isActive);
    await load();
  } finally {
    toggling.value = false;
  }
}

function openAssignRoles() {
  selectedRoles.value = [...assignedRoles.value];
  assignDialog.value = true;
}

async function saveRoles() {
  saving.value = true;
  try {
    await identityApi.assignUserRoles(id, selectedRoles.value);
    assignDialog.value = false;
    await load();
  } finally {
    saving.value = false;
  }
}

// ── 会话管理（按用户，平台视角） ──
interface UserSessionRow {
  id: string;
  deviceType?: string;
  browser?: string;
  ipAddress?: string;
  lastActivityAt?: string;
  isCurrent?: boolean;
}
const sessions = ref<UserSessionRow[]>([]);
const sessionsLoading = ref(false);
const sessionColumns: {
  name: string;
  label: string;
  field: string;
  align: 'left' | 'right' | 'center';
}[] = [
  { name: 'device', label: '设备', field: 'deviceType', align: 'left' },
  { name: 'ipAddress', label: 'IP', field: 'ipAddress', align: 'left' },
  { name: 'lastActivityAt', label: '最近活跃', field: 'lastActivityAt', align: 'left' },
  { name: 'actions', label: '', field: 'actions', align: 'right' },
];

async function loadSessions() {
  sessionsLoading.value = true;
  try {
    const rows = (await identityApi.userSessions(id)) || [];
    sessions.value = rows as UserSessionRow[];
  } catch {
    sessions.value = [];
  } finally {
    sessionsLoading.value = false;
  }
}

async function revokeSession(row: UserSessionRow) {
  try {
    await identityApi.revokeUserSession(id, row.id);
    await loadSessions();
  } catch (e) {
    // revoke session failed - error swallowed for UX
    void e;
  }
}

async function revokeAllUserSessions() {
  sessionsLoading.value = true;
  try {
    await identityApi.revokeAllUserSessions(id);
    await loadSessions();
  } finally {
    sessionsLoading.value = false;
  }
}

onMounted(async () => {
  const res = await identityApi.roles();
  roles.value = res.items || [];
  await load();
  await loadSessions();
});
</script>
