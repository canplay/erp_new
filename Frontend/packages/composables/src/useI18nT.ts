/**
 * @file Use I18n T composable with full type safety
 * Returns a translation function `t` and a safe `i18nT` that falls back
 * to key or provided string instead of throwing.
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
