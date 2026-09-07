/**
 * @file format.ts
 * @description 格式化工具函数（日期、数字、文件大小）
 */

/**
 * 格式化日期
 * @param date 日期值
 * @param format 输出格式，默认 'YYYY-MM-DD HH:mm:ss'
 */
export function formatDate(date: Date | string | number, format = 'YYYY-MM-DD HH:mm:ss'): string {
  const d = new Date(normalizeTimestamp(date));
  if (isNaN(d.getTime())) return '';

  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  const hours = String(d.getHours()).padStart(2, '0');
  const minutes = String(d.getMinutes()).padStart(2, '0');
  const seconds = String(d.getSeconds()).padStart(2, '0');

  return format
    .replace('YYYY', String(year))
    .replace('MM', month)
    .replace('DD', day)
    .replace('HH', hours)
    .replace('mm', minutes)
    .replace('ss', seconds);
}

/**
 * 归一化时间戳：后端返回 epoch 秒（10 位），JS Date 需要毫秒（13 位）。
 * 10 位数字视为秒自动 ×1000；13 位及以上保持不变。
 */
export function normalizeTimestamp(date: Date | string | number): Date | string | number {
  if (typeof date === 'number' && date > 0 && date < 1e12) {
    return date * 1000;
  }
  // 字符串形式的数字时间戳同样处理（如 "1785641927"）
  if (typeof date === 'string' && /^\d{10}$/.test(date.trim())) {
    return Number(date) * 1000;
  }
  return date;
}

/** 相对时间（刚刚、几分钟前等） */
export function relativeTime(date: Date | string | number): string {
  const d = new Date(date);
  const now = new Date();
  const diff = now.getTime() - d.getTime();

  if (diff < 60000) return '刚刚';
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
  if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`;
  return formatDate(d, 'YYYY-MM-DD');
}

/** 数字格式化（千分位） */
export function formatNumber(num: number): string {
  if (num === null || num === undefined) return '';
  return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

/** 文件大小格式化 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
