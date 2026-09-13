/**
 * 请求重试工具模块
 *
 * 提供指数退避重试、可配置重试次数和错误判断功能
 */

/**
 * 重试配置选项
 */
export interface RetryOptions {
  /** 最大重试次数，默认 3 */
  maxRetries?: number
  /** 初始延迟时间（毫秒），默认 1000 */
  initialDelay?: number
  /** 最大延迟时间（毫秒），默认 30000 */
  maxDelay?: number
  /** 延迟倍数，默认 2 */
  backoffMultiplier?: number
  /** 随机抖动因子（0-1），默认 0.1 */
  jitter?: number
  /** 需要重试的错误状态码 */
  retryStatusCodes?: number[]
  /** 需要重试的错误类型 */
  retryErrors?: string[]
  /** 重试回调 */
  onRetry?: (attempt: number, error: Error, delay: number) => void
}

/**
 * 默认重试配置
 */
const DEFAULT_OPTIONS: Required<RetryOptions> = {
  maxRetries: 3,
  initialDelay: 1000,
  maxDelay: 30000,
  backoffMultiplier: 2,
  jitter: 0.1,
  retryStatusCodes: [408, 429, 500, 502, 503, 504],
  retryErrors: ['network_error', 'timeout'],
  onRetry: () => {},
}

/**
 * 计算下一次重试的延迟时间
 * 使用指数退避算法并添加随机抖动
 */
export async function calculateDelay(
  attempt: number,
  options: Required<RetryOptions>
): number {
  const exponentialDelay = options.initialDelay * Math.pow(options.backoffMultiplier, attempt - 1)
  const jitterAmount = exponentialDelay * options.jitter
  const jitterValue = (Math.random() - 0.5) * 2 * jitterAmount
  return Math.max(Math.min(exponentialDelay + jitterValue, options.maxDelay), 0)
}

/**
 * 判断是否应该重试
 */
export async function shouldRetry(
  error: Error,
  options: Required<RetryOptions>
): boolean {
  const errorWithRetry = error as Error & { retryCount?: number }
  if (errorWithRetry.retryCount !== undefined && errorWithRetry.retryCount >= options.maxRetries) {
    return false
  }

  const response = (error as Error & { response?: { status?: number } }).response
  if (response?.status && options.retryStatusCodes.includes(response.status)) {
    return true
  }

  const error_message = error.message?.toLowerCase() || ''
  if (options.retryErrors.some(e => error_message.includes(e.toLowerCase()))) {
    return true
  }

  return false
}

/**
 * 异步重试装饰器
 * 用于包装可能失败的异步函数
 */
export function withRetry<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  options: RetryOptions = {}
): T {
  const opts = { ...DEFAULT_OPTIONS, ...options } as Required<RetryOptions>

  return (async (...args: Parameters<T>) => {
    let lastError: Error | null = null

    for (let attempt = 1; attempt <= opts.maxRetries + 1; attempt++) {
      try {
        return await fn(...args)
      } catch (error) {
        lastError = error as Error

        if (attempt > opts.maxRetries) {
          throw lastError
        }

        if (!shouldRetry(lastError, opts)) {
          throw lastError
        }

        const delay = calculateDelay(attempt, opts)
        opts.onRetry(attempt, lastError, delay)
        await sleep(delay)
      }
    }

    throw lastError!
  }) as T
}

/**
 * 带条件的重试函数
 * 允许自定义重试条件
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: RetryOptions & { condition?: (error: Error) => boolean } = {}
): Promise<T> {
  const opts = { ...DEFAULT_OPTIONS, ...options } as Required<RetryOptions> & { condition?: (error: Error) => boolean }
  let lastError: Error | null = null

  for (let attempt = 1; attempt <= opts.maxRetries + 1; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error as Error

      if (attempt > opts.maxRetries) {
        throw lastError
      }

      if (opts.condition && !opts.condition(lastError)) {
        throw lastError
      }

      const delay = calculateDelay(attempt, opts)
      opts.onRetry(attempt, lastError, delay)
      await sleep(delay)
    }
  }

  throw lastError!
}

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}
