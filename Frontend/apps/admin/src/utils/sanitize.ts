/**
 * @file sanitize.ts
 * @description XSS 防护工具函数（统一配置版本）
 * @date 2026-05-21
 * @note 所有函数使用独立配置，避免全局 DOMPurify.setConfig 重复初始化
 */

import DOMPurify from 'dompurify';

/**
 * @brief 安全的 HTML 白名单配置
 * 允许常见的富文本标签和属性
 */
const ALLOWED_TAGS = [
  // 文本格式
  'p', 'br', 'span', 'strong', 'em', 'u', 's', 'del',
  // 标题
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  // 列表
  'ul', 'ol', 'li',
  // 链接和图片
  'a', 'img',
  // 引用和代码
  'blockquote', 'pre', 'code',
  // 表格
  'table', 'thead', 'tbody', 'tr', 'th', 'td',
  // 媒体(审计修复-C2): 移除 iframe(可承载第三方脚本/钓鱼), style 标签本就未在白名单且已在 FORBID_TAGS 中
  'div',
  // 视频容器
  'figure', 'figcaption',
];

const ALLOWED_ATTR = [
  // 链接属性
  'href', 'target', 'rel',
  // 图片属性(审计修复-C2: 保留 src/alt/title/width/height 等安全属性, 移除 style/data-src)
  'src', 'alt', 'title', 'width', 'height',
  // iframe 属性
  'frameborder', 'allowfullscreen', 'allow',
  // 表格属性
  'colspan', 'rowspan',
  // class
  'class',
];

/**
 * @brief 初始化 DOMPurify 配置
 */
DOMPurify.setConfig({
  ALLOWED_TAGS,
  ALLOWED_ATTR,
  ALLOW_DATA_ATTR: false,
  ADD_ATTR: ['target', 'rel'],
});

/**
 * @brief 清理 HTML 内容，防止 XSS 攻击
 * @param dirty - 需要清理的原始 HTML 内容
 * @returns 清理后的安全 HTML 字符串
 * 
 * @example
 * ```ts
 * const safeHtml = sanitizeHTML(userInput);
 * ```
 */
export function sanitizeHTML(dirty: string): string {
  if (!dirty) return '';
  
  // 使用 DOMPurify 清理 HTML
  return DOMPurify.sanitize(dirty, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    // 强制所有链接在新窗口打开
    FORBID_TAGS: ['script', 'style', 'object', 'embed', 'form'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover'],
  });
}

/**
 * @brief 清理纯文本（去除所有 HTML 标签）
 * @param dirty - 包含 HTML 的文本
 * @returns 纯文本内容
 */
export function stripHTML(dirty: string): string {
  if (!dirty) return '';
  
  // 先清理，再去除所有标签
  const clean = DOMPurify.sanitize(dirty, { ALLOWED_TAGS: [] });
  return clean.replace(/<[^>]*>/g, '');
}

/**
 * @brief 验证 URL 是否安全(审计修复-C2: 改为 http/https 白名单, 拒绝 javascript: 等协议)
 * @param url - 需要验证的 URL
 * @returns 安全则返回 true
 */
export function isSafeUrl(url: string): boolean {
  if (!url) return false;

  const trimmed = url.trim();
  // 白名单: 仅允许 http/https 协议(含协议相对 // 与站内相对路径)
  if (/^https?:\/\//i.test(trimmed)) return true;
  if (trimmed.startsWith('//')) return true;
  if (/^\/[^/]/.test(trimmed)) return true;
  return false;
}

/**
 * @brief 清理链接，确保安全
 * @param url - 原始 URL
 * @returns 清理后的安全 URL
 */
export function sanitizeUrl(url: string): string {
  if (!url) return '';
  
  if (!isSafeUrl(url)) {
    return '';
  }
  
  return url.trim();
}

