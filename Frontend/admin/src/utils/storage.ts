/**
 * @file storage.ts
 * @description localStorage 安全操作工具
 */

/** 安全获取 localStorage */
export function getStorageItem<T>(key: string, defaultValue: T): T {
  try {
    const item = localStorage.getItem(key);
    return item === null ? defaultValue : JSON.parse(item) as T;
  } catch {
    return defaultValue;
  }
}

/** 安全设置 localStorage */
export function setStorageItem<T>(key: string, value: T): boolean {
  try {
    localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}

/** 安全删除 localStorage */
export function removeStorageItem(key: string): boolean {
  try {
    localStorage.removeItem(key);
    return true;
  } catch {
    return false;
  }
}
