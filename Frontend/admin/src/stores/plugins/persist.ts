/**
 * @file persist.ts
 * @description Pinia 持久化插件
 * @date 2026-04-03
 */

import { watch } from 'vue';
import type { PiniaPluginContext } from 'pinia';
import { logger } from '@/utils/logger';

/**
 * @brief 持久化配置选项
 */
export interface PersistOptions {
  /**
   * @brief 存储的 key 前缀
   */
  key?: string;

  /**
   * @brief 要持久化的状态路径（支持嵌套路径，如 'user.info.name'）
   */
  paths?: string[];

  /**
   * @brief 存储方式
   */
  storage?: 'local' | 'session';

  /**
   * @brief 自定义序列化函数
   */
  serializer?: {
    serialize: (value: unknown) => string;
    deserialize: (value: string) => unknown;
  };
}

/**
 * @brief 默认选项
 */
const defaultOptions: Required<PersistOptions> = {
  key: '',
  paths: [],
  storage: 'local',
  serializer: {
    serialize: JSON.stringify,
    deserialize: JSON.parse,
  },
};

/**
 * @brief Pinia 持久化插件
 *
 * @example
 * // main.ts
 * import { createPinia } from 'pinia';
 * import { createPersistedState } from '@/stores/plugins/persist';
 *
 * const pinia = createPinia();
 * pinia.use(createPersistedState());
 *
 * // store 使用
 * export const useAuthStore = defineStore('auth', () => {
 *   const token = ref<string | null>(null);
 *   const userInfo = ref<UserInfo | null>(null);
 *
 *   return {
 *     token,
 *     userInfo,
 *   };
 * }, {
 *   persist: {
 *     key: 'auth',
 *     paths: ['token', 'userInfo'],
 *   },
 * });
 */
export function createPersistedState() {
  return ({ store }: PiniaPluginContext) => {
    // 获取持久化配置
    const persistOptions = (store.$options as { persist?: PersistOptions }).persist;

    // 如果没有配置持久化，则不处理
    if (!persistOptions) return;

    // 合并选项
    const options: Required<PersistOptions> = {
      ...defaultOptions,
      ...persistOptions,
    };

    // 生成存储 key
    const storageKey = options.key || `pinia-${store.$id}`;
    const storage = options.storage === 'session' ? sessionStorage : localStorage;

    // ============ 从存储恢复状态 ============
    try {
      const storedValue = storage.getItem(storageKey);
      if (storedValue) {
        const deserializedValue = options.serializer.deserialize(storedValue);

        // 如果指定了 paths，只恢复指定的路径
        if (options.paths.length > 0) {
          for (const path of options.paths) {
            const value = getNestedValue(deserializedValue, path);
            if (value !== undefined) {
              setNestedValue(store.$state, path, value);
            }
          }
        } else {
          // 否则恢复整个状态
          store.$patch(deserializedValue as Parameters<typeof store.$patch>[0]);
        }
      }
    } catch (error) {
      logger.error(`[Pinia Persist] Failed to restore state for store "${store.$id}"`, error);
    }

    // ============ 监听状态变化并持久化 ============
    watch(
      () => store.$state,
      (state) => {
        try {
          // 如果指定了 paths，只保存指定的路径
          if (options.paths.length > 0) {
            const partialState: Record<string, unknown> = {};
            for (const path of options.paths) {
              const value = getNestedValue(state, path);
              if (value !== undefined) {
                setNestedValue(partialState, path, value);
              }
            }
            storage.setItem(storageKey, options.serializer.serialize(partialState));
          } else {
            // 否则保存整个状态
            storage.setItem(storageKey, options.serializer.serialize(state));
          }
        } catch (error) {
          logger.error(`[Pinia Persist] Failed to persist state for store "${store.$id}"`, error);
        }
      },
      { deep: true },
    );

    // ============ 清除持久化数据的方法 ============
    if (!store.$persist) {
      store.$persist = () => {
        storage.removeItem(storageKey);
      };
    }
  };
}

// ============ 工具函数 ============

/**
 * @brief 获取嵌套属性值
 */
function getNestedValue(obj: unknown, path: string): unknown {
  const keys = path.split('.');
  let value: unknown = obj;

  for (const key of keys) {
    if (value === null || value === undefined) return undefined;
    value = (value as Record<string, unknown>)[key];
  }

  return value;
}

/**
 * @brief 设置嵌套属性值
 */
function setNestedValue(obj: Record<string, unknown>, path: string, value: unknown): void {
  const keys = path.split('.');
  const lastKey = keys.pop();

  if (!lastKey) return;

  let current: Record<string, unknown> = obj;

  for (const key of keys) {
    if (!(key in current) || typeof current[key] !== 'object') {
      current[key] = {};
    }
    current = current[key] as Record<string, unknown>;
  }

  current[lastKey] = value;
}

export default createPersistedState;

