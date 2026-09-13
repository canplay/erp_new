/**
 * @file useCommandPalette.ts
 * @description 命令面板组合式函数（已合并到 useShortcuts.ts）
 * @date 2026-05-06
 */

import { ref } from 'vue'
import type CommandPalette from '@erp-new-frontend-monorepo/components/src/CommandPalette/Main.vue'

// 单例模式 - 确保全局只有一个实例
let instance: ReturnType<typeof ref<InstanceType<typeof CommandPalette> | null>> | null = null

/**
 * @brief 命令面板组合式函数
 * @description 提供全局命令面板的访问接口
 */
export function useCommandPalette() {
  // 如果实例不存在，创建一个新的 ref
  if (!instance) {
    instance = ref<InstanceType<typeof CommandPalette> | null>(null)
  }

  /**
   * @brief 打开命令面板
   */
  function open(): void {
    // 通过 DOM 事件触发打开
    document.dispatchEvent(new CustomEvent('open-command-palette'))
  }

  /**
   * @brief 关闭命令面板
   */
  function close(): void {
    document.dispatchEvent(new CustomEvent('close-command-palette'))
  }

  return {
    open,
    close
  }
}

/**
 * @brief 注册命令面板实例
 * @description 由 CommandPalette 组件调用
 */
export function registerCommandPalette(instanceRef: typeof instance): void {
  instance = instanceRef
}