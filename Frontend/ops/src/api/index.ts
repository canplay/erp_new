/**
 * @file index.ts
 * @description API 统一入口 — 基于 alova 的 HTTP 客户端
 * @date 2026-07-06
 */

import { httpClient, type ApiResponse, type PaginationParams, type PaginatedResponse } from '@/utils/alova';

export { httpClient, type ApiResponse, type PaginationParams, type PaginatedResponse };
export default httpClient;
