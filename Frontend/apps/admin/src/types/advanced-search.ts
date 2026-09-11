/**
 * 高级搜索类型定义
 *
 * @date 2026-05-16
 */

/**
 * 搜索条件
 */
export interface SearchCondition {
  field: string
  operator: string
  value: unknown
}

/**
 * 高级搜索筛选器
 * 兼容新旧两种使用方式
 */
export interface AdvancedFilters {
  /** 关键字 */
  keyword?: string
  /** 开始日期 */
  start_date?: string
  /** 结束日期 */
  end_date?: string
  /** 状态 */
  status?: string | number | null
  /** 类型 */
  type?: string | number | null
  /** 用户 */
  user?: string
  /** 模块 */
  module?: string
  /** 操作 */
  action?: string
  /** 角色 */
  role?: string | number | null
}
