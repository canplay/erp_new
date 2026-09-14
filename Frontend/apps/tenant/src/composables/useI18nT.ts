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
 */
export function useI18nT(this: void): I18nTParams {
  const { t: tInternal, te: teInternal, locale } = useI18n()

  const t = (key: I18nKey, vars?: I18nVars): string => tInternal(key, vars)
  const te = (key: I18nKey): boolean => teInternal(key)

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

  return { t, te, i18nT, locale }
}

/**
 * Standalone i18n instance creator (for use outside component setup).
 */
export function createI18nT(this: void): I18nTParams {
  return useI18nT()
}
