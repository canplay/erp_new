/**
 * @file object.ts
 * @description 对象/函数工具（深拷贝、防抖、节流、pick、omit）
 */

/** 深拷贝（使用原生 structuredClone，降级处理不支持的类型） */
export function deepClone<T>(obj: T): T {
  try {
    return structuredClone(obj);
  } catch {
    if (obj === null || typeof obj !== 'object') return obj;
    if (obj instanceof Date) return new Date(obj.getTime()) as T;
    if (obj instanceof Array) return obj.map((item) => deepClone(item)) as T;
    if (obj instanceof Object) {
      const clonedObj: Record<string, unknown> = {};
      for (const key in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
          clonedObj[key] = deepClone((obj as Record<string, unknown>)[key]);
        }
      }
      return clonedObj as T;
    }
    return obj;
  }
}

/** 从对象中提取指定键 */
export function pick<T extends Record<string, unknown>, K extends keyof T>(
  obj: T, keys: K[]
): Pick<T, K> {
  const result = {} as Pick<T, K>;
  keys.forEach((key) => {
    if (key in obj) result[key] = obj[key];
  });
  return result;
}

/** 从对象中排除指定键 */
export function omit<T extends Record<string, unknown>, K extends keyof T>(
  obj: T, keys: K[]
): Omit<T, K> {
  const result = { ...obj };
  keys.forEach((key) => { delete result[key]; });
  return result;
}

/** 防抖函数 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T, wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  return function (this: unknown, ...args: Parameters<T>) {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func.apply(this, args), wait);
  };
}

/** 节流函数 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T, limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;
  return function (this: unknown, ...args: Parameters<T>) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => { inThrottle = false; }, limit);
    }
  };
}
