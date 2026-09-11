/**
 * 请求缓存工具模块
 *
 * 提供请求结果缓存、缓存失效和内存管理功能
 */

/**
 * 缓存项结构
 */
interface CacheItem<T> {
  data: T
  timestamp: number
  ttl: number
}

/**
 * 缓存配置选项
 */
export interface CacheOptions {
  /** 默认 TTL（毫秒），默认 5 分钟 */
  defaultTtl?: number
  /** 最大缓存数量，默认 100 */
  maxSize?: number
}

/**
 * 默认缓存配置
 */
const DEFAULT_OPTIONS: Required<CacheOptions> = {
  defaultTtl: 5 * 60 * 1000,
  maxSize: 100,
}

/**
 * 内存缓存类
 */
export class MemoryCache {
  private cache: Map<string, CacheItem<unknown>>
  private readonly options: Required<CacheOptions>

  constructor(options: CacheOptions = {}) {
    this.options = { ...DEFAULT_OPTIONS, ...options }
    this.cache = new Map()
  }

  get<T>(key: string): T | null {
    const item = this.cache.get(key) as CacheItem<T> | undefined
    if (!item) return null
    if (Date.now() - item.timestamp > item.ttl) {
      this.cache.delete(key)
      return null
    }
    return item.data
  }

  set<T>(key: string, data: T, ttl?: number): void {
    if (this.cache.size >= this.options.maxSize) {
      this.evictLRU()
    }
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttl ?? this.options.defaultTtl,
    })
  }

  delete(key: string): boolean {
    return this.cache.delete(key)
  }

  clear(): void {
    this.cache.clear()
  }

  has(key: string): boolean {
    const item = this.cache.get(key)
    if (!item) return false
    if (Date.now() - item.timestamp > item.ttl) {
      this.cache.delete(key)
      return false
    }
    return true
  }

  private evictLRU(): void {
    const entries = Array.from(this.cache.entries())
      .sort((a, b) => a[1].timestamp - b[1].timestamp)
    const first = entries[0]
    if (first) {
      this.cache.delete(first[0])
    }
  }
}

/**
 * 请求缓存装饰器
 * 用于缓存 HTTP 请求结果
 */
export function cached<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  options: {
    keyGenerator?: (...args: Parameters<T>) => string
    ttl?: number
    cache?: MemoryCache
    condition?: (result: Awaited<ReturnType<T>>) => boolean
  } = {}
): T {
  const cache = options.cache ?? new MemoryCache({ ...(options.ttl ? { defaultTtl: options.ttl } : {}) })

  const defaultKeyGenerator = (...args: Parameters<T>) =>
    `${fn.name || 'anonymous'}:${JSON.stringify(args)}`

  const keyGenerator = options.keyGenerator ?? defaultKeyGenerator

  return (async (...args: Parameters<T>) => {
    const key = keyGenerator(...args)
    const cachedResult = cache.get(key)
    if (cachedResult !== null) {
      return cachedResult
    }
    const result = await fn(...args)
    if (!options.condition || options.condition(result as Awaited<ReturnType<T>>)) {
      cache.set(key, result)
    }
    return result
  }) as T
}
