/**
 * @file importExport.ts
 * @description 导入导出类型定义
 * @date 2026-04-04
 */

/**
 * @brief 导入模板配置
 */
export interface ImportTemplate {
  /** 模板ID */
  id: string;
  /** 模板名称 */
  name: string;
  /** 实体类型 */
  entity_type: string;
  /** 描述 */
  description?: string;
  /** 列配置 */
  columns: ImportTemplateColumn[];
  /** 示例数据行数 */
  sampleRowCount: number;
  /** 是否为默认模板 */
  is_default: boolean;
  /** 创建者 */
  created_by: string;
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 导入模板列配置
 */
export interface ImportTemplateColumn {
  /** 字段名 */
  field: string;
  /** Excel表头名 */
  header: string;
  /** 是否必填 */
  required: boolean;
  /** 数据类型 */
  dataType: 'string' | 'number' | 'date' | 'boolean' | 'select';
  /** 枚举选项（select类型用） */
  options?: Array<{ label: string; value: unknown }>;
  /** 校验规则 */
  validation?: ImportValidationRule;
  /** 值映射（导入时转换） */
  valueMapping?: Record<string, unknown>;
  /** 默认值 */
  defaultValue?: unknown;
  /** 宽度（Excel） */
  width?: number;
  /** 颜色（Excel） */
  color?: string;
}

/**
 * @brief 导入校验规则
 */
export interface ImportValidationRule {
  /** 规则类型 */
  type: 'required' | 'type' | 'range' | 'pattern' | 'custom' | 'unique';
  /** 规则配置 */
  config?: {
    /** 最小值/最小长度 */
    min?: number;
    /** 最大值/最大长度 */
    max?: number;
    /** 正则表达式 */
    pattern?: string;
    /** 自定义错误消息 */
    message?: string;
    /** 自定义校验函数 */
    customValidator?: string;
  };
}

/**
 * @brief 导出任务状态
 */
export type ExportTaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';

/**
 * @brief 导出任务
 */
export interface ExportTask {
  /** 任务ID */
  id: string;
  /** 任务名称 */
  name: string;
  /** 状态 */
  status: ExportTaskStatus;
  /** 导出格式 */
  format: 'xlsx' | 'csv' | 'json';
  /** 总记录数 */
  totalCount: number;
  /** 已导出记录数 */
  exportedCount: number;
  /** 进度百分比 */
  progress: number;
  /** 文件大小（字节） */
  fileSize?: number;
  /** 下载链接 */
  downloadUrl?: string;
  /** 过期时间 */
  expires_at?: string;
  /** 错误信息 */
  error?: string;
  /** 创建时间 */
  created_at: string;
  /** 完成时间 */
  completed_at?: string;
}

/**
 * @brief 导出任务参数
 */
export interface ExportTaskParams {
  /** 实体类型 */
  entity_type: string;
  /** 导出格式 */
  format: 'xlsx' | 'csv' | 'json';
  /** 字段列表 */
  fields: string[];
  /** 查询条件 */
  filters?: Record<string, unknown>;
  /** 排序 */
  sort?: {
    field: string;
    order: 'asc' | 'desc';
  };
  /** 导出范围 */
  range: 'all' | 'filtered' | 'selected';
  /** 高级选项 */
  options?: {
    includeHeader?: boolean;
    includeIndex?: boolean;
    sheetName?: string;
    title?: string;
    description?: string;
    compress?: boolean;
  };
}

/**
 * @brief 导入任务状态
 */
export type ImportTaskStatus = 'pending' | 'uploading' | 'parsing' | 'validating' | 'importing' | 'completed' | 'failed';

/**
 * @brief 导入任务
 */
export interface ImportTask {
  /** 任务ID */
  id: string;
  /** 任务名称 */
  name: string;
  /** 模板ID */
  templateId?: string;
  /** 状态 */
  status: ImportTaskStatus;
  /** 文件名 */
  fileName: string;
  /** 总记录数 */
  totalCount: number;
  /** 成功数 */
  successCount: number;
  /** 失败数 */
  failCount: number;
  /** 进度百分比 */
  progress: number;
  /** 错误报告链接 */
  errorReportUrl?: string;
  /** 错误信息 */
  error?: string;
  /** 创建时间 */
  created_at: string;
  /** 完成时间 */
  completed_at?: string;
}

/**
 * @brief 导入任务参数
 */
export interface ImportTaskParams {
  /** 模板ID */
  templateId: string;
  /** 导入模式 */
  mode: 'insert' | 'update' | 'upsert';
  /** 冲突处理策略（upsert模式用） */
  conflictStrategy?: 'skip' | 'update' | 'error';
  /** 是否模拟运行 */
  dryRun?: boolean;
  /** 跳过错误行 */
  skipErrors?: boolean;
}

/**
 * @brief 导入结果详情
 */
export interface ImportResultDetail {
  /** 总数 */
  total: number;
  /** 成功数 */
  success: number;
  /** 失败数 */
  failed: number;
  /** 跳过数 */
  skipped: number;
  /** 错误列表 */
  errors: ImportErrorDetail[];
  /** 警告列表 */
  warnings: ImportWarningDetail[];
}

/**
 * @brief 导入错误详情
 */
export interface ImportErrorDetail {
  /** 行号 */
  row: number;
  /** 字段 */
  field: string;
  /** 值 */
  value: unknown;
  /** 错误消息 */
  message: string;
  /** 错误类型 */
  type: 'validation' | 'duplicate' | 'reference' | 'system';
}

/**
 * @brief 导入警告详情
 */
export interface ImportWarningDetail {
  /** 行号 */
  row: number;
  /** 字段 */
  field: string;
  /** 警告消息 */
  message: string;
  /** 警告类型 */
  type: 'truncated' | 'converted' | 'implicit';
}

/**
 * @brief 预定义的导入模板
 */
export const DEFAULT_IMPORT_TEMPLATES: Omit<ImportTemplate, 'id' | 'created_at' | 'updated_at'>[] = [
  {
    name: '用户导入模板',
    entity_type: 'user',
    description: '标准用户数据导入模板',
    is_default: true,
    created_by: 'system',
    sampleRowCount: 5,
    columns: [
      { field: 'username', header: '用户名', required: true, dataType: 'string', validation: { type: 'required' } },
      { field: 'nickname', header: '昵称', required: false, dataType: 'string' },
      { field: 'email', header: '邮箱', required: true, dataType: 'string', validation: { type: 'required' } },
      { field: 'phone', header: '手机号', required: false, dataType: 'string' },
      { field: 'role', header: '角色', required: false, dataType: 'select', options: [
        { label: '管理员', value: 'admin' },
        { label: '普通用户', value: 'user' },
        { label: '访客', value: 'guest' },
      ]},
      { field: 'status', header: '状态', required: false, dataType: 'select', options: [
        { label: '启用', value: 1 },
        { label: '禁用', value: 0 },
      ], defaultValue: 1 },
    ],
  },
  {
    name: '客户导入模板',
    entity_type: 'customer',
    description: '客户数据导入模板',
    is_default: true,
    created_by: 'system',
    sampleRowCount: 5,
    columns: [
      { field: 'name', header: '客户名称', required: true, dataType: 'string', validation: { type: 'required' } },
      { field: 'contact', header: '联系人', required: false, dataType: 'string' },
      { field: 'phone', header: '联系电话', required: false, dataType: 'string' },
      { field: 'email', header: '邮箱', required: false, dataType: 'string' },
      { field: 'level', header: '客户等级', required: false, dataType: 'select', options: [
        { label: 'VIP', value: 'vip' },
        { label: '普通', value: 'normal' },
        { label: '潜在', value: 'potential' },
      ]},
    ],
  },
];

/**
 * @brief 导出格式选项
 */
export const EXPORT_FORMAT_OPTIONS = [
  { label: 'Excel (.xlsx)', value: 'xlsx', icon: 'table_chart' },
  { label: 'CSV (.csv)', value: 'csv', icon: 'grid_on' },
  { label: 'JSON (.json)', value: 'json', icon: 'data_object' },
];

/**
 * @brief 导入模式选项
 */
export const IMPORT_MODE_OPTIONS = [
  { label: '仅新增', value: 'insert', description: '只插入新记录，忽略已存在的' },
  { label: '仅更新', value: 'update', description: '只更新已存在的记录，忽略新的' },
  { label: '新增或更新', value: 'upsert', description: '新记录插入，已存在的更新' },
];
