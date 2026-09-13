/**
 * @file i18n.ts
 * @description 多语言增强类型定义
 * @date 2026-04-04
 */

/**
 * @brief 支持的语言
 */
export interface SupportedLanguage {
  /** 语言代码 */
  code: string;
  /** 语言名称（中文） */
  name: string;
  /** 语言名称（英文） */
  nameEn: string;
  /** 本地化名称 */
  nativeName: string;
  /** 排序 */
  order: number;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * @brief 翻译条目
 */
export interface TranslationEntry {
  /** 翻译key */
  key: string;
  /** 翻译值 */
  value: string;
  /** 上下文/注释 */
  context?: string;
  /** 是否已翻译 */
  isTranslated: boolean;
  /** 是否已审核 */
  isReviewed: boolean;
  /** 最后修改时间 */
  updated_at: string;
}

/**
 * @brief 翻译文件
 */
export interface TranslationFile {
  /** 文件路径 */
  path: string;
  /** 语言代码 */
  locale: string;
  /** 翻译条目 */
  entries: TranslationEntry[];
  /** 翻译进度 */
  progress: number;
  /** 最后同步时间 */
  lastSyncedAt: string;
}

/**
 * @brief 翻译统计
 */
export interface TranslationStatistics {
  /** 语言代码 */
  locale: string;
  /** 总条目数 */
  totalEntries: number;
  /** 已翻译数 */
  translatedEntries: number;
  /** 已审核数 */
  reviewedEntries: number;
  /** 翻译进度 (%) */
  progress: number;
  /** 待翻译数 */
  pendingEntries: number;
  /** 待审核数 */
  pendingReviewEntries: number;
}

/**
 * @brief 用户翻译偏好
 */
export interface UserTranslationPreference {
  /** 用户ID */
  user_id: number;
  /** 首选语言 */
  preferredLocale: string;
  /** 备用语言 */
  fallbackLocale: string;
  /** 自动检测语言 */
  autoDetect: boolean;
  /** 显示语言名称 */
  showLanguageName: boolean;
  /** 自定义翻译 */
  customTranslations: Record<string, Record<string, string>>;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 翻译一致性检查结果
 */
export interface TranslationConsistencyCheck {
  /** 检查类型 */
  type: 'missing' | 'empty' | 'placeholder_mismatch' | 'html_tags';
  /** 翻译key */
  key: string;
  /** 受影响的语言 */
  affectedLocales: string[];
  /** 严重程度 */
  severity: 'error' | 'warning' | 'info';
  /** 描述 */
  description: string;
}

/**
 * @brief 翻译命名空间
 */
export interface TranslationNamespace {
  /** 命名空间名称 */
  name: string;
  /** 描述 */
  description?: string;
  /** 包含的key数量 */
  keyCount: number;
  /** 已翻译数量 */
  translatedCount: number;
  /** 是否为核心命名空间 */
  isCore: boolean;
}

/**
 * @brief 批量翻译请求
 */
export interface BatchTranslateRequest {
  /** 源语言 */
  sourceLocale: string;
  /** 目标语言 */
  targetLocale: string;
  /** 需要翻译的keys */
  keys: string[];
  /** 翻译引擎 */
  engine?: 'google' | 'deepl' | 'openai' | 'azure';
  /** 语气风格 */
  tone?: 'formal' | 'informal' | 'technical';
}

/**
 * @brief 翻译历史记录
 */
export interface TranslationHistory {
  /** 记录ID */
  id: string;
  /** 翻译key */
  key: string;
  /** 语言代码 */
  locale: string;
  /** 原值 */
  oldValue?: string;
  /** 新值 */
  newValue: string;
  /** 操作人 */
  operator: {
    id: number;
    name: string;
  };
  /** 操作类型 */
  action: 'create' | 'update' | 'delete';
  /** 操作时间 */
  operatedAt: string;
}
