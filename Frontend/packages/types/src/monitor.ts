/**
 * @file monitor.ts
 * @description 系统健康监控类型定义
 * @date 2026-04-04
 */

/**
 * @brief 性能指标
 */
export interface PerformanceMetrics {
  /** 首次内容绘制 (FCP) */
  FCP: number;
  /** 最大内容绘制 (LCP) */
  LCP: number;
  /** 累积布局偏移 (CLS) */
  CLS: number;
  /** 首次输入延迟 (FID) / 交互到下一帧绘制 (INP) */
  FID: number;
  /** 页面加载时间 */
  pageLoadTime: number;
  /** DOM 构建完成时间 */
  DOMContentLoaded: number;
  /** 页面完全加载时间 */
  loadComplete: number;
  /** 时间戳 */
  timestamp: number;
}

/**
 * @brief WebSocket 连接状态
 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'reconnecting';

/**
 * @brief WebSocket 质量指标
 */
export interface WebSocketMetrics {
  /** 连接状态 */
  status: ConnectionStatus;
  /** 连接建立时间 */
  connectedAt: number | null;
  /** 最后心跳时间 */
  lastHeartbeatAt: number | null;
  /** 重连次数 */
  reconnectAttempts: number;
  /** 消息发送数 */
  messagesSent: number;
  /** 消息接收数 */
  messagesReceived: number;
  /** 错误数 */
  errorCount: number;
  /** 平均响应时间 (ms) */
  avg_response_time: number;
  /** 连接质量评分 (0-100) */
  qualityScore: number;
}

/**
 * @brief API 调用记录
 */
export interface ApiCallRecord {
  /** 调用ID */
  id: string;
  /** 请求方法 */
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  /** 请求路径 */
  path: string;
  /** 请求时间 */
  requestTime: number;
  /** 响应时间 (ms) */
  response_time: number;
  /** HTTP 状态码 */
  status: number;
  /** 请求大小 (bytes) */
  requestSize?: number;
  /** 响应大小 (bytes) */
  responseSize?: number;
  /** 是否成功 */
  success: boolean;
  /** 错误信息 */
  error?: string;
  /** 时间戳 */
  timestamp: number;
}

/**
 * @brief 前端错误记录
 */
export interface FrontendError {
  /** 错误ID */
  id: string;
  /** 错误类型 */
  type: 'javascript' | 'resource' | 'promise' | 'vue' | 'network';
  /** 错误消息 */
  message: string;
  /** 堆栈信息 */
  stack?: string;
  /** 发生错误的文件名 */
  filename?: string;
  /** 行号 */
  lineno?: number;
  /** 列号 */
  colno?: number;
  /** 用户ID (如果有) */
  user_id?: number;
  /** 用户操作 (如果有) */
  action?: string;
  /** 时间戳 */
  timestamp: number;
  /** 是否已处理 */
  resolved: boolean;
}

/**
 * @brief 系统资源使用情况
 */
export interface ResourceUsage {
  /** CPU 使用率 (%) */
  cpuUsage: number;
  /** 内存使用率 (%) */
  memoryUsage: number;
  /** 内存总量 (bytes) */
  memoryTotal: number;
  /** 内存使用量 (bytes) */
  memoryUsed: number;
  /** 存储使用率 (%) */
  storageUsage: number;
  /** 存储总量 (bytes) */
  storageTotal: number;
  /** 存储使用量 (bytes) */
  storageUsed: number;
  /** 网络状态 */
  networkStatus: 'online' | 'offline' | 'slow';
  /** 网络类型 */
  networkType?: string;
  /** 时间戳 */
  timestamp: number;
}

/**
 * @brief 健康检查结果
 */
export interface HealthCheckResult {
  /** 检查项名称 */
  name: string;
  /** 状态 */
  status: 'healthy' | 'warning' | 'critical' | 'unknown';
  /** 消息 */
  message: string;
  /** 详情 */
  details?: Record<string, unknown>;
  /** 响应时间 (ms) */
  response_time?: number;
}

/**
 * @brief 性能告警规则
 */
export interface PerformanceAlertRule {
  /** 规则ID */
  id: string;
  /** 规则名称 */
  name: string;
  /** 指标类型 */
  metricType: 'FCP' | 'LCP' | 'CLS' | 'FID' | 'apiResponseTime' | 'errorRate';
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
 * @brief 性能告警
 */
export interface PerformanceAlert {
  /** 告警ID */
  id: string;
  /** 关联的规则 */
  ruleId: string;
  /** 规则名称 */
  ruleName: string;
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
 * @brief 性能基线
 */
export interface PerformanceBaseline {
  /** 指标类型 */
  metricType: string;
  /** 基线值 */
  baseline: number;
  /** P50 值 */
  p50: number;
  /** P90 值 */
  p90: number;
  /** P95 值 */
  p95: number;
  /** P99 值 */
  p99: number;
  /** 样本数 */
  sampleCount: number;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 性能趋势数据点
 */
export interface PerformanceTrendPoint {
  /** 时间戳 */
  timestamp: number;
  /** 值 */
  value: number;
  /** 标记（异常、告警等） */
  flags?: string[];
}

/**
 * @brief 连接质量等级
 */
export type ConnectionQualityLevel = 'excellent' | 'good' | 'fair' | 'poor' | 'disconnected';

/**
 * @brief 获取连接质量等级
 */
export function getConnectionQualityLevel(score: number): ConnectionQualityLevel {
  if (score >= 90) return 'excellent';
  if (score >= 70) return 'good';
  if (score >= 50) return 'fair';
  if (score >= 20) return 'poor';
  return 'disconnected';
}

/**
 * @brief 获取连接质量等级颜色
 */
export function getConnectionQualityColor(level: ConnectionQualityLevel): string {
  const colors: Record<ConnectionQualityLevel, string> = {
    excellent: 'positive',
    good: 'info',
    fair: 'warning',
    poor: 'negative',
    disconnected: 'grey',
  };
  return colors[level];
}

/**
 * @brief 获取健康状态颜色
 */
export function getHealthStatusColor(status: 'healthy' | 'warning' | 'critical' | 'unknown'): string {
  const colors: Record<string, string> = {
    healthy: 'positive',
    warning: 'warning',
    critical: 'negative',
    unknown: 'grey',
  };
  return colors[status] ?? 'grey';
}

/**
 * @brief 获取告警级别颜色
 */
export function getAlertLevelColor(level: 'info' | 'warning' | 'critical'): string {
  const colors: Record<string, string> = {
    info: 'info',
    warning: 'warning',
    critical: 'negative',
  };
  return colors[level] ?? 'info';
}
