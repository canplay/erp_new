/**
 * 参数命名转换工具
 * camelCase → snake_case 转换，用于前端参数与后端 Rust 接口对接
 */

const CAMEL_TO_SNAKE_REGEX = /([a-z0-9])([A-Z])/g;

/**
 * 驼峰转蛇形：user_id → user_id, page_size → page_size
 */
function camelToSnake(key: string): string {
  return key.replace(CAMEL_TO_SNAKE_REGEX, '$1_$2').toLowerCase();
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return Object.prototype.toString.call(value) === '[object Object]';
}

/**
 * 将对象的所有 key 从 camelCase 递归转换为 snake_case
 * - 跳过 null / undefined / 非对象类型
 * - 处理嵌套对象和数组
 * - 已为 snake_case 的 key 保持不变（无大写字母可匹配）
 */
export function toSnakeCase<T>(obj: T): T {
  if (obj === null || obj === undefined) return obj;

  if (Array.isArray(obj)) {
    return obj.map(toSnakeCase) as T;
  }

  if (!isPlainObject(obj)) return obj;

  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    result[camelToSnake(key)] = toSnakeCase(value);
  }
  return result as T;
}
