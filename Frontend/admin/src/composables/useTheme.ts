/**
 * @file useTheme.ts
 * @description 主题管理组合式函数
 * @date 2026-04-03
 * @note presetThemes 和 themeModeOptions 已移至 stores/theme.ts，避免重复定义
 */

import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useThemeStore, presetThemes, themeModeOptions, type PresetTheme, type ThemeMode } from '@/stores/theme';

// 重新导出共享类型和常量
export { presetThemes, themeModeOptions };
export type { PresetTheme, ThemeMode };

/**
 * @brief 主题配置接口
 */
export interface ThemeConfig {
  mode: ThemeMode;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  isDark: boolean;
}

/**
 * @brief 主题组合式函数
 * @description 提供组件级别的主题管理功能，内部使用 useThemeStore
 */
export function useTheme() {
  const $q = useQuasar();
  const themeStore = useThemeStore();

  /** 当前主题配置（响应式） */
  const themeConfig = ref<ThemeConfig>({
    mode: themeStore.mode,
    primaryColor: themeStore.primaryColor,
    secondaryColor: themeStore.secondaryColor,
    accentColor: themeStore.accentColor,
    isDark: themeStore.isDark,
  });

  /** 当前主题模式 */
  const themeMode = ref<ThemeMode>(themeStore.mode);

  /**
   * @brief 应用主题颜色
   */
  function applyTheme(config: Partial<ThemeConfig>) {
    // 更新 Quasar 配置
    $q.dark.set(config.isDark ?? themeStore.isDark);

    // 更新 CSS 变量
    const root = document.documentElement;
    if (config.primaryColor) {
      root.style.setProperty('--q-primary', config.primaryColor);
    }
    if (config.secondaryColor) {
      root.style.setProperty('--q-secondary', config.secondaryColor);
    }
    if (config.accentColor) {
      root.style.setProperty('--q-accent', config.accentColor);
    }
  }

  /**
   * @brief 设置主题模式
   */
  function setThemeMode(mode: ThemeMode) {
    themeStore.setMode(mode);
    themeMode.value = mode;
    themeConfig.value.mode = mode;
    themeConfig.value.isDark = themeStore.isDark;
  }

  /**
   * @brief 设置主题颜色
   */
  function setThemeColor(preset: PresetTheme) {
    themeStore.setThemeColor(preset);
    themeConfig.value.primaryColor = preset.primary;
    themeConfig.value.secondaryColor = preset.secondary;
    themeConfig.value.accentColor = preset.accent;
    applyTheme(themeConfig.value);
  }

  /**
   * @brief 重置主题
   */
  function resetTheme() {
    setThemeMode('light');
    setThemeColor(presetThemes[0]!);
  }

  /**
   * @brief 加载主题
   */
  function loadTheme() {
    themeStore.init();
    themeConfig.value = {
      mode: themeStore.mode,
      primaryColor: themeStore.primaryColor,
      secondaryColor: themeStore.secondaryColor,
      accentColor: themeStore.accentColor,
      isDark: themeStore.isDark,
    };
    themeMode.value = themeStore.mode;
  }

  // 生命周期
  onMounted(() => {
    loadTheme();
    applyTheme(themeConfig.value);
  });

  return {
    themeMode,
    themeConfig,
    presetThemes,
    themeModeOptions,
    setThemeMode,
    setThemeColor,
    resetTheme,
    applyTheme,
    loadTheme,
  };
}

/**
 * @brief 暗色主题检测
 */
export function useDarkMode() {
  const themeStore = useThemeStore();
  return { isDark: themeStore.isDark };
}
