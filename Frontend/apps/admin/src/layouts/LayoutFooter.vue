<template>
  <q-footer elevated class="bg-grey-8 text-white">
    <!-- 桌面端页脚 -->
    <template v-if="!isMobile">
      <q-toolbar class="justify-center">
        <div class="text-caption">
          © 2026 MyAI Admin · {{ $t('common.poweredBy') }}
        </div>
      </q-toolbar>
    </template>
    <!-- 移动端底部导航 -->
    <template v-else>
      <q-toolbar class="mobile-nav">
        <div class="mobile-nav-items">
          <div
            v-for="nav in mobileNavItems"
            :key="nav.path"
            class="mobile-nav-item"
            :class="{ active: isCurrentRoute(nav.path) }"
            @click="$emit('navigateTo', nav.path)"
          >
            <q-icon :name="nav.icon" size="24px" />
            <span class="nav-label">{{ nav.label }}</span>
          </div>
        </div>
      </q-toolbar>
    </template>
  </q-footer>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

defineProps<{
  isMobile: boolean;
  mobileNavItems: Array<{ path: string; label: string; icon: string }>;
  isCurrentRoute: (path: string) => boolean;
}>();

defineEmits<{
  navigateTo: [path: string];
}>();
</script>
