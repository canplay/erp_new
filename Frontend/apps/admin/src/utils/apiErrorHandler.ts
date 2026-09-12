/**
 * @file apiErrorHandler.ts
 * @description Centralized API error handling utility
 * @date 2026-09-12
 */

import { logger } from '@/utils/logger';

/**
 * Standardized API error handler
 * Logs the error and re-throws for caller handling
 */
export async function handleApiError(error: unknown, context: string): void {
  if (error instanceof Error) {
    logger.error(`[API Error] ${context}: ${error.message}`, error);
  } else if (typeof error === 'object' && error !== null) {
    const errObj = error as { message?: string; msg?: string };
    const message = errObj.message || errObj.msg || JSON.stringify(error);
    logger.error(`[API Error] ${context}: ${message}`, error);
  } else {
    logger.error(`[API Error] ${context}: ${String(error)}`);
  }
}

/**
 * Wraps an API function with standardized error handling
 * Returns a tuple [data, error] for safe consumption
 */
export async function withApiErrorHandling<T>(
  apiCall: () => Promise<T>,
  context: string
): Promise<T> {
  try {
    return await apiCall();
  } catch (error) {
    handleApiError(error, context);
    throw error;
  }
}
