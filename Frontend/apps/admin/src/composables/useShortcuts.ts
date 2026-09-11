/**
 * @file useShortcuts.ts
 * @description 快捷键组合式函数（合并 useGlobalShortcuts/usePresetShortcuts）
 * @date 2026-04-03
 * @note 合并了三个重叠的函数：useShortcuts、useGlobalShortcuts、usePresetShortcuts
 */

import { onMounted, onUnmounted } from 'vue';
import { useQuasar } from 'quasar';
import { useRouter } from 'vue-router';

/**
 * @brief 快捷键配置接口
 */
export interface Shortcut {
  /** 键名 */
  key: string;
  /** 修饰键 */
  modifiers?: ('ctrl' | 'alt' | 'shift' | 'meta')[];
  /** 描述 */
  description: string;
  /** 回调函数 */
  callback: () => void;
  /** 是否启用 */
  enabled?: boolean;
}

/**
 * @brief 快捷键组合式函数
 */
export function useShortcuts() {
  /** 快捷键列表 */
  const shortcuts: Shortcut[] = [];

  /** 键盘事件处理器 */
  function handleKeyDown(event: KeyboardEvent) {
    // 忽略在输入框中的快捷键
    const target = event.target as HTMLElement;
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) {
      return;
    }

    for (const shortcut of shortcuts) {
      if (shortcut.enabled === false) continue;

      const modifiers = shortcut.modifiers || [];
      const key = shortcut.key.toLowerCase();

      // 检查修饰键
      const ctrlMatch =
        modifiers.includes('ctrl') === (event.ctrlKey || event.metaKey);
      const altMatch = modifiers.includes('alt') === event.altKey;
      const shiftMatch = modifiers.includes('shift') === event.shiftKey;
      const metaMatch = modifiers.includes('meta') === event.metaKey;

      // 检查主键
      const keyMatch = event.key.toLowerCase() === key;

      if (keyMatch && ctrlMatch && altMatch && shiftMatch && metaMatch) {
        event.preventDefault();
        event.stopPropagation();
        shortcut.callback();
        return;
      }
    }
  }

  /**
   * @brief 注册快捷键
   */
  function registerShortcut(shortcut: Shortcut) {
    shortcuts.push(shortcut);
  }

  /**
   * @brief 注销快捷键
   */
  function unregisterShortcut(key: string, modifiers?: string[]) {
    const index = shortcuts.findIndex(
      (s) =>
        s.key.toLowerCase() === key.toLowerCase() &&
        JSON.stringify(s.modifiers?.sort()) === JSON.stringify(modifiers?.sort())
    );
    if (index > -1) {
      shortcuts.splice(index, 1);
    }
  }

  /**
   * @brief 启用快捷键
   */
  function enableShortcut(key: string) {
    const shortcut = shortcuts.find((s) => s.key.toLowerCase() === key.toLowerCase());
    if (shortcut) {
      shortcut.enabled = true;
    }
  }

  /**
   * @brief 禁用快捷键
   */
  function disableShortcut(key: string) {
    const shortcut = shortcuts.find((s) => s.key.toLowerCase() === key.toLowerCase());
    if (shortcut) {
      shortcut.enabled = false;
    }
  }

  // 生命周期
  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
    shortcuts.length = 0;
  });

  return {
    registerShortcut,
    unregisterShortcut,
    enableShortcut,
    disableShortcut,
  };
}

/**
 * @brief 全局快捷键 Hook
 * 在 Layout 或 App.vue 中调用一次即可
 */
export function useGlobalShortcuts() {
  /** 快捷键列表 */
  const shortcuts: Array<{
    key: string;
    modifiers?: string[];
    description: string;
    handler: () => void;
  }> = [];

  /** 键盘事件处理器 */
  function handleKeyDown(event: KeyboardEvent) {
    // 忽略在输入框中的快捷键
    const target = event.target as HTMLElement;
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) {
      // 但允许 Escape 键
      if (event.key !== 'Escape') {
        return;
      }
    }

    for (const shortcut of shortcuts) {
      const modifiers = shortcut.modifiers || [];
      const key = shortcut.key.toLowerCase();

      // 检查修饰键
      const ctrlMatch =
        modifiers.includes('ctrl') === (event.ctrlKey || event.metaKey);
      const altMatch = modifiers.includes('alt') === event.altKey;
      const shiftMatch = modifiers.includes('shift') === event.shiftKey;
      const metaMatch = modifiers.includes('meta') === event.metaKey;

      // 检查主键
      const keyMatch = event.key.toLowerCase() === key;

      if (keyMatch && ctrlMatch && altMatch && shiftMatch && metaMatch) {
        event.preventDefault();
        event.stopPropagation();
        shortcut.handler();
        return;
      }
    }
  }

  /**
   * @brief 注册全局快捷键
   */
  function registerGlobalShortcut(
    key: string,
    description: string,
    handler: () => void,
    modifiers?: string[]
  ) {
    shortcuts.push({ key, modifiers: modifiers ?? [], description, handler });
  }

  /**
   * @brief 注册快捷键（带配置）
   */
  function register(config: {
    key: string;
    modifiers?: string[];
    description: string;
    handler: () => void;
  }) {
    shortcuts.push(config);
  }

  /** 快捷键列表（用于显示） */
  function getShortcutList() {
    return shortcuts.map((s) => ({
      key: [...(s.modifiers || []), s.key],
      description: s.description,
    }));
  }

  // 生命周期
  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
    shortcuts.length = 0;
  });

  return {
    registerGlobalShortcut,
    register,
    getShortcutList,
  };
}

/**
 * @brief 预定义的全局快捷键
 * @description 在 Layout 或 App.vue 中调用此函数注册预设快捷键
 */
export function usePresetShortcuts() {
  const $q = useQuasar();
  const router = useRouter();
  const { registerGlobalShortcut } = useGlobalShortcuts();

  // Ctrl+Shift+? 打开快捷键帮助
  registerGlobalShortcut('?', '打开快捷键帮助', () => {
    $q.notify({
      type: 'info',
      message: '快捷键: Ctrl+K 搜索, Ctrl+N 新建, Esc 关闭弹窗',
      timeout: 3000,
    });
  });

  // Ctrl+K 搜索
  registerGlobalShortcut('k', '全局搜索', () => {
    // 触发全局搜索
    const searchInput = document.querySelector('.global-search input');
    if (searchInput instanceof HTMLInputElement) {
      searchInput.focus();
    } else {
      void router.push({ name: 'Dashboard' });
    }
  }, ['ctrl']);

  // Ctrl+N 新建（通用）
  registerGlobalShortcut('n', '新建', () => {
    // 向外触发新建事件
    window.dispatchEvent(new CustomEvent('shortcut:new'));
  }, ['ctrl']);

  // Escape 关闭弹窗
  registerGlobalShortcut('escape', '关闭弹窗/取消', () => {
    // 触发关闭事件
    window.dispatchEvent(new CustomEvent('shortcut:escape'));
  });
}
