/**
 * @file useI18nT.ts
 * @description Safe i18n translation function with fallback
 * Falls back to the key itself (not crash) when translation missing
 */
import { useI18n } from 'vue-i18n'

export interface UseI18nTReturn {
  i18nT: (key: string, fallback?: string) => string
  t: (key: string, vars?: Record<string, unknown>) => string
  te: (key: string) => boolean
  locale: { value: string }
}

/**
 * Translation function that never crashes.
 * - If key exists: returns translated string
 * - If key missing with string fallback: returns fallback
 * - If key missing without fallback: returns key itself (for debugging)
 */
export function useI18nT(this: void): UseI18nTReturn {
  const i18n = useI18n()

  function t(key: string, vars?: Record<string, unknown>): string {
    return i18n.t(key, vars as Record<string, string | number>)
  }

  function te(key: string): boolean {
    return i18n.te(key)
  }

  function i18nT(key: string, fallback?: string): string {
    if (te(key)) {
      return t(key)
    }
    if (fallback !== undefined) {
      return fallback
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
 */
export function createI18nTInstance(this: void): UseI18nTReturn {
  const i18nGlobal = useI18n()

  function t(key: string, vars?: Record<string, unknown>): string {
    return i18nGlobal.t(key, vars as Record<string, string | number>)
  }

  function i18nT(key: string, fallback?: string): string {
    if (i18nGlobal.te(key)) {
      return t(key)
    }
    if (fallback !== undefined) {
      return fallback
    }
    if (import.meta.env?.DEV) {
      console.warn(`[i18n] Missing key: "${key}" (locale: ${i18nGlobal.locale.value})`)
    }
    return key
  }

  return { i18nT, t, te: i18nGlobal.te, locale: i18nGlobal.locale }
}
