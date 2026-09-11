/**
 * 统一解析 eBike API 返回格式
 *
 * alova httpClient 的 transformResponse 会将后端响应包装为
 *   { code: number, data: unknown, message?: string, ... }
 * 而各端点返回的 data 可能是数组、{ cars: [] }、或嵌套对象。
 *
 * 本函数处理所有可能情况，返回统一数组。
 */
export function parseCarResponse(resp: Record<string, unknown> | unknown[]): unknown[] {
  // resp 可能是 alova 的完整响应对象，也可能已经是 data 部分
  const data = (resp as Record<string, unknown>).data ?? resp;

  // 情况1: 直接是数组（例如 /car 返回 []）
  if (Array.isArray(data)) {
    return data;
  }

  // 情况2: 嵌套对象 { cars: [...] } 或 { data: [...] }
  if (data && typeof data === 'object' && !Array.isArray(data)) {
    const obj = data as Record<string, unknown>;
    // 优先取 cars（后端统一字段）
    if (Array.isArray(obj.cars)) {
      return obj.cars;
    }
    // 兜底取 data.data（深层嵌套）
    if (Array.isArray(obj.data)) {
      return obj.data;
    }
  }

  // 情况3: 都不是，返回空数组
  return [];
}
