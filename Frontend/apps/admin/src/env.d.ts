/**
 * @file env.d.ts
 * @description 全局类型声明
 */

/// <reference types="vite/client" />

// Vite 环境变量
interface ImportMetaEnv {
  readonly VITE_APP_TITLE: string;
  readonly VITE_API_BASE_URL: string;
  readonly [key: string]: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Quasar 类型声明
declare module '#q-app/wrappers' {
  import type { App, ComponentPublicInstance } from 'vue';
  export function createApp(): App<ComponentPublicInstance>;
  
  // Quasar boot function
  export function defineBoot<T = Record<string, unknown>>(boot: (params: T) => void | Promise<void>): void;
}

// 组件模块声明
declare module '@/components/*' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

// Vue 文件声明
declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

// Quasar Dialog Ref 类型
interface QuasarDialog<T = unknown> {
  onOk: (callback: (data?: T) => void) => void;
  onCancel: (callback: () => void) => void;
  onDialogOK: (data?: T) => void;
  onDialogCancel: () => void;
  close: () => void;
  hide: () => void;
  getComponent: () => unknown;
  getRef: () => unknown;
}

// 静态资源声明
declare module '*.png' {
  const value: string;
  export default value;
}

declare module '*.svg' {
  const value: string;
  export default value;
}

declare module '*.jpg' {
  const value: string;
  export default value;
}

declare module '*.jpeg' {
  const value: string;
  export default value;
}

declare module '*.gif' {
  const value: string;
  export default value;
}

declare module '*.webp' {
  const value: string;
  export default value;
}

// Socket.io 类型声明
declare module 'socket.io-client' {
  export interface Socket {
    id: string;
    connected: boolean;
    on(event: string, listener: (...args: unknown[]) => void): void;
    emit(event: string, ...args: unknown[]): void;
    disconnect(): void;
  }
  export function io(url: string, options?: unknown): Socket;
}

// DOMPurify 类型声明
declare module 'dompurify' {
  interface Config {
    ALLOWED_TAGS?: string[];
    ALLOWED_ATTR?: string[];
    ALLOW_DATA_ATTR?: boolean;
    ADD_ATTR?: string[];
    FORBID_TAGS?: string[];
    FORBID_ATTR?: string[];
  }

  interface SanitizeConfig extends Config {
    RETURN_TRUSTED_TYPE?: boolean;
  }

  interface DOMPurify {
    sanitize(dirty: string | Node, config?: SanitizeConfig): string;
    sanitize(dirty: string | Node, config?: SanitizeConfig & { RETURN_TRUSTED_TYPE: true }): TrustedHTML;
    isSupported(): boolean;
    setConfig(config: Config): void;
    clearConfig(): void;
    addHook(hook: string, cb: (node: Node, data: object, config: Config) => void): void;
    removeHook(hook: string): void;
    removeHooks(hook: string): void;
    removeAllHooks(): void;
  }

  const DOMPurify: DOMPurify;
  export default DOMPurify;
  export function sanitize(dirty: string | Node, config?: SanitizeConfig): string;
  export function isSupported(): boolean;
  export function setConfig(config: Config): void;
  export function clearConfig(): void;
}