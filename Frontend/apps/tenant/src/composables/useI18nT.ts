/**
 * @file Use I18n T composable with full type safety
 * @description Safe i18n translation function with fallback
 * Falls back to the key itself (not crash) when translation missing
 */
import { useI18n } from 'vue-i18n'

export type I18nKey = string
export type I18nVars = Record<string, string | number>
export type I18nFallback = string | Record<string, unknown>

export interface I18nTParams {
  t: (key: I18nKey, vars?: I18nVars) => string
  te: (key: I18nKey) => boolean
  locale: { value: string }
  i18nT: (key: I18nKey, fallback?: I18nFallback) => string
}

/**
 * Returns a safe i18n translation function.
 * - If the key exists in the translation registry, `t` is used.
 * - If the key is missing AND a string `fallback` is given, the fallback is returned.
 * - If the key is missing AND no fallback, the key itself is returned (useful in dev).
 */
export function useI18nT(this: void): I18nTParams {
  const i18n = useI18n()

  const t = (key: I18nKey, vars?: I18nVars): string => i18n.t(key, vars)
  const te = (key: I18nKey): boolean => i18n.te(key)

  const i18nT = (key: I18nKey, fallback?: I18nFallback): string => {
    if (te(key)) {
      return t(key)
    }
    if (typeof fallback === 'string') {
      return fallback
    }
    if (import.meta.env?.DEV) {
      console.warn(`[i18n] Missing translation key: "${key}"`)
    }
    return key
  }

  return { t, te, i18nT, locale: i18n.locale }
}

/**
 * Standalone i18n instance creator (for use outside component setup).
 */
export function createI18nT(this: void): I18nTParams {
  return useI18nT()
}
