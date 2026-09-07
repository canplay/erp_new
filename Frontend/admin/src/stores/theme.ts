/**
 * @file theme.ts
 * @description 主题状态管理
 * @date 2026-04-02
 * @note 整合了 useTheme.ts 的部分功能，避免重复代码
 */

import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { getStorageItem, setStorageItem } from '@/utils/storage';

export type ThemeMode = 'light' | 'dark' | 'auto';

/**
 * @brief 预设主题颜色接口
 */
export interface PresetTheme {
  name: string;
  primary: string;
  secondary: string;
  accent: string;
}

/**
 * @brief 主题模式选项
 */
export const themeModeOptions = [
  { label: '浅色模式', value: 'light' as ThemeMode, icon: 'light_mode' },
  { label: '深色模式', value: 'dark' as ThemeMode, icon: 'dark_mode' },
  { label: '跟随系统', value: 'auto' as ThemeMode, icon: 'contrast' },
];

/**
 * @brief 预设主题列表
 */
export const presetThemes: PresetTheme[] = [
  { name: 'blue', primary: '#1976d2', secondary: '#26a69a', accent: '#ff6f00' },
  { name: 'purple', primary: '#7b1fa2', secondary: '#00bcd4', accent: '#ffc107' },
  { name: 'green', primary: '#388e3c', secondary: '#7b1fa2', accent: '#ff5722' },
  { name: 'orange', primary: '#f57c00', secondary: '#0288d1', accent: '#7b1fa2' },
  { name: 'red', primary: '#d32f2f', secondary: '#0288d1', accent: '#ffc107' },
  { name: 'teal', primary: '#00796b', secondary: '#f57c00', accent: '#7b1fa2' },
];

export const useThemeStore = defineStore('theme', () => {
  const $q = useQuasar();

  // 获取保存的主题模式，默认为 'auto'
  const savedMode = (getStorageItem<ThemeMode>('theme_mode', 'auto') || 'auto');
  const mode = ref<ThemeMode>(savedMode);

  // 主题颜色配置
  const primaryColor = ref<string>('#1976d2');
  const secondaryColor = ref<string>('#26a69a');
  const accentColor = ref<string>('#ff6f00');

  // 计算当前实际主题
  const isDark = ref<boolean>(
    mode.value === 'dark' || (mode.value === 'auto' && $q.dark.isActive)
  );

  // 监听系统主题变化
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

  /**
   * @brief 处理系统主题变化
   */
  function handleSystemThemeChange() {
    if (mode.value === 'auto') {
      isDark.value = mediaQuery.matches;
      applyThemeColors();
    }
  }

  /**
   * @brief 应用主题颜色到 CSS 变量
   */
  function applyThemeColors() {
    const root = document.documentElement;
    root.style.setProperty('--q-primary', primaryColor.value);
    root.style.setProperty('--q-secondary', secondaryColor.value);
    root.style.setProperty('--q-accent', accentColor.value);
  }

  /**
   * @brief 从 localStorage 加载主题配置
   */
  function loadThemeConfig() {
    const saved = getStorageItem<string>('themeConfig', '');
    if (saved) {
      try {
        const config = JSON.parse(saved);
        if (config.primaryColor) primaryColor.value = config.primaryColor;
        if (config.secondaryColor) secondaryColor.value = config.secondaryColor;
        if (config.accentColor) accentColor.value = config.accentColor;
      } catch {
        // 忽略解析错误
      }
    }
  }

  // 初始化
  function init() {
    // 加载主题配置
    loadThemeConfig();
    // 应用保存的主题
    applyMode(mode.value);
    // 应用主题颜色
    applyThemeColors();
    // 添加媒体查询监听器
    mediaQuery.addEventListener('change', handleSystemThemeChange);
  }

  // 应用主题模式
  function applyMode(newMode: ThemeMode) {
    mode.value = newMode;
    setStorageItem('theme_mode', newMode);

    switch (newMode) {
      case 'light':
        $q.dark.set(false);
        isDark.value = false;
        break;
      case 'dark':
        $q.dark.set(true);
        isDark.value = true;
        break;
      case 'auto':
        isDark.value = mediaQuery.matches;
        break;
    }
  }

  // 切换主题
  function toggleTheme() {
    if (mode.value === 'light') {
      applyMode('dark');
    } else if (mode.value === 'dark') {
      applyMode('light');
    } else {
      // auto 模式下，切换到与当前系统相反的主题
      applyMode(mediaQuery.matches ? 'light' : 'dark');
    }
  }

  // 设置特定模式
  function setMode(newMode: ThemeMode) {
    applyMode(newMode);
  }

  /**
   * @brief 设置主题颜色
   * @param preset - 预设主题
   */
  function setThemeColor(preset: PresetTheme) {
    primaryColor.value = preset.primary;
    secondaryColor.value = preset.secondary;
    accentColor.value = preset.accent;
    applyThemeColors();
    // 保存到 localStorage
    setStorageItem('themeConfig', {
      primaryColor: primaryColor.value,
      secondaryColor: secondaryColor.value,
      accentColor: accentColor.value,
    });
  }

  return {
    mode,
    isDark,
    primaryColor,
    secondaryColor,
    accentColor,
    init,
    toggleTheme,
    setMode,
    setThemeColor,
    applyThemeColors,
  };
});
