<template>
  <q-page class="erp-page">
    <q-breadcrumbs class="q-mb-sm">
      <q-breadcrumbs-el :label="i18nT('breadcrumb_roles')" to="/roles" />
      <q-breadcrumbs-el :label="i18nT('breadcrumb_role_detail')" />
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
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.s5760a9') }}</div>
    </div>

    <q-card v-if="role">
      <q-card-section class="row items-center">
        <q-avatar
          size="48px"
          color="secondary"
          text-color="white"
          icon="admin_panel_settings"
          class="q-mr-md"
        />
        <div>
          <div class="text-h6">{{ role.name }}</div>
          <div class="text-caption text-grey-6">{{ role.description || '—' }}</div>
        </div>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <div class="row items-center q-mb-sm">
          <div class="text-subtitle2">{{ i18nT('raw.s360f75') }}</div>
          <q-space />
          <q-btn
            outline
            dense
            size="sm"
            color="primary"
            icon="edit"
            label="编辑权限"
            :loading="permsLoading"
            @click="openPermEditor"
          />
        </div>
        <div v-if="role.permissions?.length" class="permission-chips">
          <q-chip
            v-for="p in role.permissions"
            :key="p"
            size="sm"
            color="primary"
            text-color="white"
            :label="shortName(p)"
          />
        </div>
        <EmptyState
          v-else
          :title="i18nT('raw.s13c99a')"
          :hint="i18nT('raw.s9b1be3')"
          icon="lock_open"
        />
      </q-card-section>
    </q-card>

    <div v-if="loading" class="q-pa-xl column items-center text-grey-6">
      <q-spinner size="40px" class="q-mb-sm" />
      <div>{{ i18nT('raw.s795a79') }}</div>
    </div>
    <EmptyState v-else-if="!role" :title="i18nT('raw.s58a825')" :hint="i18nT('raw.sfc3230')" />

    <!-- 权限编辑弹窗（按资源分组勾选 → PUT roles/{id}/permissions） -->
    <q-dialog v-model="permDialog" persistent>
      <q-card style="min-width: 720px; max-width: 92vw">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">编辑角色权限 · {{ role?.name }}</div>
          <q-space />
          <q-btn flat round dense icon="close" @click="permDialog = false" />
        </q-card-section>
        <q-separator />
        <q-card-section class="q-pt-none" style="max-height: 62vh">
          <q-scroll-area style="height: 56vh">
            <div v-if="permsLoading" class="text-grey-6 q-pa-md">加载权限目录…</div>
            <div v-for="group in groupedCatalog" :key="group.resource" class="q-mt-md">
              <div class="row items-center q-mb-xs">
                <div class="text-subtitle2 text-primary">{{ group.resource }}</div>
                <q-space />
                <q-btn
                  flat
                  dense
                  size="xs"
                  :label="groupAllChecked(group) ? '清空' : '全选'"
                  @click="toggleGroup(group)"
                />
              </div>
              <div class="q-pl-sm">
                <template v-for="p in group.items" :key="p.name">
                  <q-checkbox
                    :model-value="permSelected.includes(p.name)"
                    :label="shortName(p.name)"
                    dense
                    @update:model-value="togglePerm(p.name)"
                  />
                  <div class="text-caption text-grey-6 q-pl-sm" style="margin-top: -4px">
                    {{ p.description }}
                  </div>
                </template>
              </div>
            </div>
          </q-scroll-area>
        </q-card-section>
        <q-separator />
        <q-card-actions align="right" class="q-px-md q-pb-md">
          <q-btn flat label="取消" color="grey-7" @click="permDialog = false" />
          <q-btn color="primary" label="保存权限" :loading="saving" @click="savePerms" />
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
import { useQuasar } from 'quasar';
import { identityApi, type IdentityRole, type PermissionCatalogEntry } from '@/api';
import { EmptyState } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const route = useRoute();
const router = useRouter();
const $q = useQuasar();

const id = String((route.params as { id?: string }).id ?? '');

const role = ref<IdentityRole | null>(null);
const { loading, load } = useTableState<IdentityRole>();

function shortName(perm: string) {
  return perm.replace(/^Permissions\./, '').replace(/\./g, ' ');
}

// ── 权限编辑 ──
const permDialog = ref(false);
const permSelected = ref<string[]>([]);
const permsLoading = ref(false);
const saving = ref(false);
const catalog = ref<PermissionCatalogEntry[]>([]);

const groupedCatalog = computed(() => {
  const groups = new Map<string, PermissionCatalogEntry[]>();
  for (const p of catalog.value) {
    const g = groups.get(p.resource) ?? [];
    g.push(p);
    groups.set(p.resource, g);
  }
  return [...groups.entries()]
    .map(([resource, items]) => ({ resource, items }))
    .sort((a, b) => a.resource.localeCompare(b.resource));
});

function togglePerm(name: string) {
  permSelected.value = permSelected.value.includes(name)
    ? permSelected.value.filter((x) => x !== name)
    : [...permSelected.value, name];
}

function groupAllChecked(group: { resource: string; items: PermissionCatalogEntry[] }) {
  return group.items.every((p) => permSelected.value.includes(p.name));
}

function toggleGroup(group: { resource: string; items: PermissionCatalogEntry[] }) {
  const names = group.items.map((p) => p.name);
  const allChecked = groupAllChecked(group);
  permSelected.value = allChecked
    ? permSelected.value.filter((x) => !names.includes(x))
    : [...new Set([...permSelected.value, ...names])];
}

async function openPermEditor() {
  if (!role.value) return;
  permsLoading.value = true;
  try {
    if (!catalog.value.length) {
      catalog.value = await identityApi.permissionCatalog();
    }
    const cur = await identityApi.rolePermissions(id);
    permSelected.value = Array.isArray(cur) ? cur : [];
    permDialog.value = true;
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: `加载权限失败: ${e instanceof Error ? e.message : String(e)}`,
    });
  } finally {
    permsLoading.value = false;
  }
}

async function savePerms() {
  saving.value = true;
  try {
    await identityApi.updateRolePermissions(id, permSelected.value);
    $q.notify({ type: 'positive', message: '角色权限已保存' });
    permDialog.value = false;
    // 重载角色（权限 chips 同步）
    await load(async () => {
      role.value = await identityApi.role(id);
    });
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: `保存失败: ${e instanceof Error ? e.message : String(e)}`,
    });
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  await load(async () => {
    role.value = await identityApi.role(id);
  });
});
</script>

<style scoped>
.permission-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
</style>
