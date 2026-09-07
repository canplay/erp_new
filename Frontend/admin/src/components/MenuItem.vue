<template>
  <q-item
    v-if="!menu.children || menu.children.length === 0"
    clickable
    :to="menu.path"
    :active="$route.path === menu.path"
    active-class="bg-primary text-white"
  >
    <q-item-section avatar>
      <q-icon :name="menu.icon" />
    </q-item-section>
    <q-item-section>
      <q-item-label>{{ getLabel(menu) }}</q-item-label>
    </q-item-section>
  </q-item>

  <!-- 带子菜单的情况 -->
  <q-expansion-item
    v-else
    :icon="menu.icon"
    :label="getLabel(menu)"
    :header-inset-level="0"
    dense
  >
    <MenuItem
      v-for="child in menu.children!"
      :key="child.path ?? child.name ?? ''"
      :menu="child"
      :depth="depth + 1"
    />
  </q-expansion-item>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

interface MenuConfig {
  path?: string;
  name?: string;
  title?: string;
  label?: string;
  icon?: string;
  permissions?: string[];
  children?: MenuConfig[];
}


withDefaults(
  defineProps<{
    menu: MenuConfig;
    depth?: number;
  }>(),
  {
    depth: 0,
  }
);

/**
 * @brief 获取菜单标签
 * 支持多种格式: title, name, label
 */
const { t } = useI18n();
function getLabel(menu: MenuConfig): string {
  if (menu.title) {
    return t(menu.title);
  }
  if (menu.label) {
    return menu.label;
  }
  if (menu.name) {
    return menu.name;
  }
  return menu.path || '';
}
</script>
