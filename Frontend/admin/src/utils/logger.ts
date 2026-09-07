/**
 * @file logger.ts
 * @description 日志工具模块 — 提供统一的日志输出接口
 * @date 2026-08-15
 */

const noop = () => {};

const isDev = typeof process !== 'undefined' && process.env?.NODE_ENV !== 'production';

export const logger = {
  trace: isDev ? console.debug?.bind(console) : noop,
  debug: isDev ? console.debug?.bind(console) : noop,
  info: isDev ? console.info.bind(console) : noop,
  warn: isDev ? console.warn.bind(console) : noop,
  error: isDev ? console.error.bind(console) : noop,
};

export type Logger = typeof logger;
