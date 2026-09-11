<template>
  <q-btn-dropdown
    v-model="showDropdown"
    flat
    no-caps
    class="language-switcher"
    :class="{ 'language-switcher--active': showDropdown }"
  >
    <template v-slot:label>
      <div class="row items-center no-wrap q-gutter-xs">
        <q-icon :name="currentLanguage?.icon || 'language'" size="20px" />
        <span class="text-body2">{{ currentLanguage?.name || locale }}</span>
        <q-icon name="expand_more" size="16px" />
      </div>
    </template>

    <q-list style="min-width: 200px; max-height: 400px;">
      <q-item
        v-for="lang in availableLanguages"
        :key="lang.code"
        clickable
        v-close-popup
        @click="handleLanguageChange(lang.code)"
        :class="{ 'bg-primary text-white': lang.code === locale }"
      >
        <q-item-section avatar>
          <span>{{ lang.icon }}</span>
        </q-item-section>
        <q-item-section>
          <q-item-label>{{ lang.name }}</q-item-label>
          <q-item-label caption>{{ lang.nativeName }}</q-item-label>
        </q-item-section>
        <q-item-section side v-if="lang.code === locale">
          <q-icon name="check" />
        </q-item-section>
      </q-item>

      <q-separator />

      <q-item clickable v-close-popup @click="handleOpenTranslator">
        <q-item-section avatar><q-icon name="translate" /></q-item-section>
        <q-item-section>
          <q-item-label>{{ $t('i18n.openTranslator') || '翻译管理' }}</q-item-label>
        </q-item-section>
      </q-item>
    </q-list>
  </q-btn-dropdown>
</template>

<script setup lang="ts">
/**
 * @file LanguageSwitcher.vue
 * @description 语言切换器组件
 * @date 2026-04-04
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

const $q = useQuasar();
const { locale: i18nLocale } = useI18n();

const showDropdown = ref(false);

const supportedLanguages = [
  { code: 'zh-CN', name: '简体中文', nativeName: '简体中文', icon: '🇨🇳', order: 1 },
  { code: 'en-US', name: 'English', nativeName: 'English', icon: '🇺🇸', order: 2 },
];

// 修复 (fix-plan-20260806 P17): 原 5 种语言选项但 i18n 仅注册 zh-CN/en-US，
// zh-TW/ja-JP/ko-KR 无翻译资源会回退到默认语言，切换无实际效果。
// 现仅显示有翻译资源的语言；代码生成语言列表，避免硬编码漂移。
const locale = computed(() => i18nLocale.value);

const availableLanguages = computed(() =>
  supportedLanguages.filter((lang) => lang.order <= 5)
);

const currentLanguage = computed(() =>
  supportedLanguages.find((lang) => lang.code === locale.value)
);

function handleLanguageChange(newLocale: string) {
  // 修复 (fix-plan-20260806 P17): 原仅改本地 ref + localStorage，
  // 不调用 i18n.global.locale，界面语言实际不变（纯UI摆设）。
  i18nLocale.value = newLocale;
  localStorage.setItem('user-locale', newLocale);
  $q.notify({
    type: 'positive',
    message: `语言已切换为 ${supportedLanguages.find((l) => l.code === newLocale)?.name}`,
    timeout: 2000,
  });
}

function handleOpenTranslator() {
  $q.notify({ type: 'info', message: '翻译管理功能开发中' });
}
</script>

<style scoped>
.language-switcher {
  border-radius: 4px;
  transition: all 0.2s;
}
.language-switcher:hover {
  background: rgba(0, 0, 0, 0.05);
}
.language-switcher--active {
  background: rgba(0, 0, 0, 0.08);
}
.body--dark .language-switcher:hover,
.body--dark .language-switcher--active {
  background: rgba(255, 255, 255, 0.1);
}
</style>
