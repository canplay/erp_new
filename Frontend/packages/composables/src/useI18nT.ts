/**
 * @file useI18nT.ts
 * @description Safe i18n translation function with fallback
 * Falls back to the key itself (not crash) when translation missing
 */
import { useI18n } from 'vue-i18n'

/**
 * Translation function that never crashes.
 * - If key exists: returns translated string
 * - If key missing: returns the key itself (for debugging)
 * - Supports {param} interpolation via second argument
 */
export function useI18nT(this: void) {
  const i18n = useI18n()

  /** @this: void */
  function t(key: string, vars?: Record<string, string | number>): string {
    return i18n.t(key, vars)
  }

  /** @this: void */
  function te(key: string): boolean {
    return i18n.te(key)
  }

  /**
   * Safe translate function
   * @param key - i18n key (e.g. 'common.save')
   * @param vars - optional variables for interpolation
   */
  function i18nT(key: string, fallbackOrVars?: string | Record<string, unknown>): string {
    if (te(key)) {
      const vars = typeof fallbackOrVars === 'object' ? fallbackOrVars as Record<string, string | number> : undefined;
      return t(key, vars)
    }
    // Key doesn't exist - return fallback string or key itself
    if (typeof fallbackOrVars === 'string') {
      return fallbackOrVars
    }
    if (import.meta.env?.DEV) {
      console.warn(`[i18n] Missing key: "${key}" (locale: ${i18n.locale.value})`)
    }
    return key
  }

  return { i18nT, t, te, locale: i18n.locale }
}

/**
 * Standalone version for use outside <script setup>
 * Import this if you need to use i18nT in templates without useI18n
 */
export function createI18nTInstance(this: void) {
  const i18nGlobal = useI18n()

  /** @this: void */
  function t(key: string, vars?: Record<string, string | number>): string {
    return i18nGlobal.t(key, vars)
  }

  return {
    i18nT: (key: string, fallbackOrVars?: string | Record<string, unknown>): string => {
      if (i18nGlobal.te(key)) {
        const vars = typeof fallbackOrVars === 'object' ? fallbackOrVars as Record<string, string | number> : undefined;
        return t(key, vars)
      }
      if (typeof fallbackOrVars === 'string') {
        return fallbackOrVars
      }
      if (import.meta.env?.DEV) {
        console.warn(`[i18n] Missing key: "${key}" (locale: ${i18nGlobal.locale.value})`)
      }
      return key
    },
    ...i18nGlobal
  }
}
