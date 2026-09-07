<template>
  <q-drawer
    v-model="localDrawerOpen"
    show-if-above
    persistent
    bordered
    :width="240"
    :breakpoint="768"
    :overlay="isMobile"
    side="left"
  >
    <q-scroll-area class="fit" :bar-style="{ right: '4px' }">
      <!-- 用户信息卡片 -->
      <div class="drawer-header q-pa-md">
        <div class="text-center">
          <q-avatar size="64px" color="primary" text-color="white" class="q-mb-sm">
            {{ userAvatarText }}
          </q-avatar>
          <div class="text-subtitle1 text-weight-bold">
            {{ username || $t('common.admin') }}
          </div>
          <div class="text-caption text-grey-6">
            {{ role === 'admin' ? $t('user.admin') : $t('user.normalUser') }}
          </div>
        </div>
      </div>

      <!-- 菜单列表（分类结构，按权限过滤） -->
      <q-list padding>
        <template v-for="(category, catIdx) in visibleCategories" :key="`cat-${catIdx}`">
          <!-- 分类可折叠展开 -->
          <q-expansion-item
            :icon="category.icon"
            :label="$t(category.title)"
            :header-inset-level="0"
            dense
            default-opened
          >
            <template v-for="(item, idx) in category.items" :key="item.path ?? `item-${idx}`">
              <q-item
                clickable
                :to="item.path"
                :active="$route.path === item.path"
                active-class="bg-primary text-white"
              >
                <q-item-section avatar>
                  <q-icon :name="item.icon" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ $t(item.title ?? '') }}</q-item-label>
                </q-item-section>
              </q-item>
            </template>
          </q-expansion-item>
        </template>
      </q-list>
    </q-scroll-area>

    <!-- 移动端遮罩层 -->
    <div v-if="isMobile && localDrawerOpen" class="drawer-overlay" @click="localDrawerOpen = false" />
  </q-drawer>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { MenuCategory } from '@/config/menu';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const route = useRoute();

const props = defineProps<{
  leftDrawerOpen: boolean;
  isMobile: boolean;
  userAvatarText: string;
  username?: string;
  role?: string;
  visibleCategories: MenuCategory[];
}>();

const emit = defineEmits<{
  'update:leftDrawerOpen': [value: boolean];
}>();

const localDrawerOpen = ref(props.leftDrawerOpen);

watch(
  () => props.leftDrawerOpen,
  (val) => {
    localDrawerOpen.value = val;
  }
);

watch(
  localDrawerOpen,
  (val) => {
    emit('update:leftDrawerOpen', val);
  }
);
</script>
