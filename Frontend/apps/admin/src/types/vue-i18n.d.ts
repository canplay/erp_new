/**
 * Vue i18n 全局类型声明
 * 使模板中的 $t / $tc / $te 可用
 */
declare module 'vue' {
  interface ComponentCustomProperties {
    $t: (key: string, ...args: unknown[]) => string;
    $tc: (key: string, ...args: unknown[]) => string;
    $te: (key: string) => boolean;
  }
}

export {};
