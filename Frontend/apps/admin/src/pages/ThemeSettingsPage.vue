<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('theme.title') }}</div>

    <!-- 主题模式 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-md">{{ $t('theme.mode') }}</div>
        <div class="row q-col-gutter-md">
          <div v-for="option in themeModeOptions" :key="option.value" class="col-12 col-sm-4">
            <q-btn
              :color="themeMode === option.value ? 'primary' : 'grey'"
              :icon="option.icon"
              :label="option.label"
              class="full-width"
              @click="setThemeMode(option.value as ThemeMode)"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- 主题颜色 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-md">{{ $t('theme.colors') }}</div>
        <div class="row q-col-gutter-md">
          <div v-for="preset in presetThemes" :key="preset.name" class="col-6 col-sm-4 col-md-2">
            <q-card
              class="theme-card cursor-pointer"
              :class="{ active: currentPrimary === preset.primary }"
              @click="setThemeColor(preset)"
            >
              <q-card-section class="theme-preview" :style="{ background: preset.primary }">
                <q-icon name="check" v-if="currentPrimary === preset.primary" color="white" />
              </q-card-section>
              <q-card-section class="q-pa-sm">
                <div class="text-center">{{ $t(`theme.${preset.name}`) }}</div>
              </q-card-section>
            </q-card>
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- 预览区域 -->
    <q-card bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-md">{{ $t('theme.preview') }}</div>
        <div class="preview-area">
          <q-btn color="primary" :label="$t('common.save')" class="q-mr-sm" />
          <q-btn color="secondary" :label="$t('common.cancel')" class="q-mr-sm" />
          <q-btn color="accent" :label="$t('common.add')" />
          <q-space class="q-my-sm" />
          <q-input v-model="previewText" outlined dense :placeholder="$t('theme.previewText')" class="q-mb-md" />
          <q-toggle v-model="previewToggle" :label="$t('theme.toggle')" class="q-mr-md" />
          <q-checkbox v-model="previewCheckbox" :label="$t('theme.checkbox')" class="q-mr-md" />
          <q-badge color="primary" text-color="white" label="Badge" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 操作按钮 -->
    <div class="q-mt-md text-right">
      <q-btn flat color="grey" :label="$t('common.reset')" @click="handleReset" />
      <q-btn color="primary" :label="$t('common.save')" @click="handleSave" />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useTheme, type ThemeMode } from '@/composables/useTheme';

const { t } = useI18n();
const $q = useQuasar();

const {
  themeMode,
  presetThemes,
  themeModeOptions,
  setThemeMode,
  setThemeColor,
  resetTheme,
} = useTheme();

// 预览状态
const previewText = ref('');
const previewToggle = ref(true);
const previewCheckbox = ref(false);

// 当前主题色
const currentPrimary = computed(() => {
  const preset = presetThemes.find((p) => p.primary === themeMode.value);
  return preset?.primary ?? presetThemes[0]?.primary ?? '#1976D2';
});

/**
 * @brief 重置主题
 */
function handleReset() {
  resetTheme();
  $q.notify({ type: 'positive', message: t('theme.resetSuccess') });
}

/**
 * @brief 保存设置
 */
function handleSave() {
  $q.notify({ type: 'positive', message: t('common.success') });
}
</script>

<style scoped>
.theme-card {
  border: 2px solid transparent;
  transition: all 0.3s ease;
}

.theme-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.theme-card.active {
  border-color: var(--q-primary);
}

.theme-preview {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-area {
  padding: 20px;
  border: 1px dashed #ccc;
  border-radius: 8px;
}
</style>
