/**
 * @file apiGovernance.ts
 * @description API 治理类型定义
 * @date 2026-04-04
 */

/**
 * @brief API 调用统计
 */
export interface ApiCallStatistics {
  /** 总调用次数 */
  totalCalls: number;
  /** 成功次数 */
  successCalls: number;
  /** 失败次数 */
  failedCalls: number;
  /** 错误率 (%) */
  errorRate: number;
  /** 平均响应时间 (ms) */
  avg_response_time: number;
  /** P50 响应时间 (ms) */
  p50ResponseTime: number;
  /** P90 响应时间 (ms) */
  p90ResponseTime: number;
  /** P95 响应时间 (ms) */
  p95_response_time: number;
  /** P99 响应时间 (ms) */
  p99ResponseTime: number;
  /** 最大响应时间 (ms) */
  max_response_time: number;
  /** 最小响应时间 (ms) */
  min_response_time: number;
  /** 总数据量 (bytes) */
  totalDataSize: number;
  /** QPS (每秒请求数) */
  qps: number;
  /** 时间范围 */
  timeRange: {
    start: number;
    end: number;
  };
}

/**
 * @brief API 端点统计
 */
export interface ApiEndpointStatistics {
  /** 端点路径 */
  path: string;
  /** 请求方法 */
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  /** 调用次数 */
  callCount: number;
  /** 成功次数 */
  successCount: number;
  /** 失败次数 */
  failedCount: number;
  /** 平均响应时间 (ms) */
  avg_response_time: number;
  /** P95 响应时间 (ms) */
  p95_response_time: number;
  /** 最后调用时间 */
  lastCalledAt: number;
  /** 错误率 (%) */
  errorRate: number;
  /** 分类 */
  category: string;
}

/**
 * @brief API 日志条目
 */
export interface ApiLogEntry {
  /** 日志ID */
  id: string;
  /** 请求ID */
  requestId: string;
  /** 请求方法 */
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  /** 请求路径 */
  path: string;
  /** 查询参数 */
  queryParams?: Record<string, string>;
  /** 请求头 */
  headers: Record<string, string>;
  /** 请求体大小 (bytes) */
  requestSize: number;
  /** 请求体 */
  request_body?: string;
  /** 响应状态码 */
  status_code: number;
  /** 响应头 */
  responseHeaders: Record<string, string>;
  /** 响应体大小 (bytes) */
  responseSize: number;
  /** 响应体 */
  response_body?: string;
  /** 响应时间 (ms) */
  response_time: number;
  /** 客户端IP */
  clientIp: string;
  /** 用户ID (如果有) */
  user_id?: number;
  /** 用户名 (如果有) */
  username?: string;
  /** 错误信息 (如果有) */
  error?: string;
  /** 时间戳 */
  timestamp: number;
}

/**
 * @brief API 告警规则
 */
export interface ApiAlertRule {
  /** 规则ID */
  id: string;
  /** 规则名称 */
  name: string;
  /** 规则类型 */
  type: 'error_rate' | 'response_time' | 'qps_threshold' | 'data_size';
  /** 端点模式 (支持通配符) */
  endpointPattern: string;
  /** 方法 (空表示所有方法) */
  method?: string;
  /** 阈值 */
  threshold: number;
  /** 比较操作符 */
  operator: 'gt' | 'lt' | 'eq' | 'gte' | 'lte';
  /** 持续时间 (秒) */
  duration: number;
  /** 告警级别 */
  level: 'info' | 'warning' | 'critical';
  /** 是否启用 */
  enabled: boolean;
  /** 创建时间 */
  created_at: string;
}

/**
 * @brief API 告警
 */
export interface ApiAlert {
  /** 告警ID */
  id: string;
  /** 关联的规则 */
  ruleId: string;
  /** 规则名称 */
  ruleName: string;
  /** 端点 */
  endpoint: string;
  /** 方法 */
  method: string;
  /** 告警级别 */
  level: 'info' | 'warning' | 'critical';
  /** 告警消息 */
  message: string;
  /** 当前值 */
  currentValue: number;
  /** 阈值 */
  threshold: number;
  /** 状态 */
  status: 'active' | 'acknowledged' | 'resolved';
  /** 触发时间 */
  triggeredAt: string;
  /** 解决时间 */
  resolved_at?: string;
}

/**
 * @brief API 响应时间分布
 */
export interface ApiResponseTimeDistribution {
  /** 分桶范围 (ms) */
  bucket: string;
  /** 范围下限 */
  min: number;
  /** 范围上限 */
  max: number;
  /** 调用次数 */
  count: number;
  /** 占比 (%) */
  percentage: number;
}

/**
 * @brief API 趋势数据点
 */
export interface ApiTrendPoint {
  /** 时间戳 */
  timestamp: number;
  /** 调用次数 */
  callCount: number;
  /** 错误次数 */
  errorCount: number;
  /** 平均响应时间 (ms) */
  avg_response_time: number;
  /** P95 响应时间 (ms) */
  p95_response_time: number;
}

/**
 * @brief API 分类统计
 */
export interface ApiCategoryStatistics {
  /** 分类名称 */
  category: string;
  /** 调用次数 */
  callCount: number;
  /** 平均响应时间 (ms) */
  avg_response_time: number;
  /** 错误率 (%) */
  errorRate: number;
}

/**
 * @brief API 性能基线
 */
export interface ApiPerformanceBaseline {
  /** 端点 */
  endpoint: string;
  /** 方法 */
  method: string;
  /** 基线平均响应时间 (ms) */
  baselineAvg: number;
  /** 基线 P95 (ms) */
  baselineP95: number;
  /** 当前平均 (ms) */
  currentAvg: number;
  /** 当前 P95 (ms) */
  currentP95: number;
  /** 偏差 (%) */
  deviation: number;
  /** 状态 */
  status: 'normal' | 'degraded' | 'critical';
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 批量操作请求
 */
export interface BatchApiOperation {
  /** 操作类型 */
  type: 'delete' | 'export' | 'analyze';
  /** 日志ID列表 */
  ids: string[];
  /** 筛选条件 (用于导出和分析) */
  filter?: ApiLogFilter;
}

/**
 * @brief API 日志筛选条件
 */
export interface ApiLogFilter {
  /** 方法 */
  method?: string;
  /** 路径关键词 */
  pathKeyword?: string;
  /** 状态码 */
  status_code?: number;
  /** 最小响应时间 (ms) */
  min_response_time?: number;
  /** 最大响应时间 (ms) */
  max_response_time?: number;
  /** 用户ID */
  user_id?: number;
  /** 开始时间 */
  start_time?: number;
  /** 结束时间 */
  end_time?: number;
  /** 是否只显示错误 */
  errorsOnly?: boolean;
}

/**
 * @brief API 日志排序
 */
export interface ApiLogSort {
  /** 排序字段 */
  field: 'timestamp' | 'response_time' | 'status_code';
  /** 排序方向 */
  direction: 'asc' | 'desc';
}
