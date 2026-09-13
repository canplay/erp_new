/**
 * @file formBuilder.ts
 * @description 表单生成器类型定义
 * @date 2026-04-04
 */

/**
 * @brief 表单字段类型
 */
export type FormFieldType =
  | 'input'
  | 'textarea'
  | 'number'
  | 'password'
  | 'email'
  | 'tel'
  | 'select'
  | 'multiSelect'
  | 'radio'
  | 'checkbox'
  | 'switch'
  | 'date'
  | 'time'
  | 'datetime'
  | 'daterange'
  | 'upload'
  | 'cascader'
  | 'treeSelect'
  | 'editor'
  | 'divider'
  | 'slot';

/**
 * @brief 表单字段验证规则
 */
export interface FormValidationRule {
  /** 规则类型 */
  type: 'required' | 'min' | 'max' | 'minLength' | 'maxLength' | 'pattern' | 'email' | 'url' | 'custom';
  /** 验证消息 */
  message?: string;
  /** 规则值 */
  value?: string | number | RegExp;
  /** 自定义验证函数 */
  validator?: (value: unknown) => boolean | string;
}

/**
 * @brief 表单字段选项（用于 select、radio、checkbox 等）
 */
export interface FormFieldOption {
  /** 选项标签 */
  label: string;
  /** 选项值 */
  value: string | number;
  /** 是否禁用 */
  disabled?: boolean;
  /** 层级（用于级联选择） */
  children?: FormFieldOption[];
}

/**
 * @brief 表单字段配置
 */
export interface FormFieldConfig {
  /** 字段唯一标识 */
  id: string;
  /** 字段类型 */
  type: FormFieldType;
  /** 字段名 */
  name: string;
  /** 标签 */
  label?: string;
  /** 占位提示 */
  placeholder?: string;
  /** 默认值 */
  defaultValue?: unknown;
  /** 是否禁用 */
  disabled?: boolean;
  /** 是否只读 */
  readonly?: boolean;
  /** 是否隐藏 */
  hidden?: boolean;
  /** 是否必填 */
  required?: boolean;
  /** 验证规则 */
  rules?: FormValidationRule[];
  /** 选项（用于 select、radio、checkbox 等） */
  options?: FormFieldOption[];
  /** 级联配置 */
  cascaderOptions?: FormFieldOption[];
  /** 最大值/最小值 */
  min?: number;
  max?: number;
  /** 文本类型 */
  maxlength?: number;
  /** 步进值 */
  step?: number;
  /** 行数 */
  rows?: number;
  /** 显示格式 */
  format?: string;
  /** 文件上传配置 */
  uploadConfig?: {
    action?: string;
    accept?: string;
    maxSize?: number;
    multiple?: boolean;
    autoUpload?: boolean;
  };
  /** 栅格宽度 */
  span?: number;
  /** 偏移量 */
  offset?: number;
  /** 提示信息 */
  hint?: string;
  /** 前缀图标 */
  prefixIcon?: string;
  /** 后缀图标 */
  suffixIcon?: string;
  /** 样式类名 */
  class?: string;
  /** 事件处理器 */
  events?: Record<string, (value: unknown) => void>;
}

/**
 * @brief 表单分组
 */
export interface FormGroup {
  /** 分组ID */
  id: string;
  /** 分组名称 */
  name: string;
  /** 分组描述 */
  description?: string;
  /** 字段列表 */
  fields: FormFieldConfig[];
  /** 是否折叠 */
  collapsed?: boolean;
  /** 样式类名 */
  class?: string;
}

/**
 * @brief 表单配置
 */
export interface FormConfig {
  /** 表单ID */
  id: string;
  /** 表单名称 */
  name: string;
  /** 表单描述 */
  description?: string;
  /** 表单布局 */
  layout: 'horizontal' | 'vertical' | 'inline';
  /** 标签宽度 */
  labelWidth?: number | string;
  /** 标签位置 */
  labelPosition?: 'left' | 'top' | 'right';
  /** 分组列表 */
  groups: FormGroup[];
  /** 表单初始值 */
  initialValues?: Record<string, unknown>;
  /** 是否显示操作按钮 */
  showActions?: boolean;
  /** 操作按钮配置 */
  actionsConfig?: {
    submitText?: string;
    resetText?: string;
    cancelText?: string;
    showReset?: boolean;
    showCancel?: boolean;
  };
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 表单模板
 */
export interface FormTemplate {
  /** 模板ID */
  id: string;
  /** 模板名称 */
  name: string;
  /** 模板描述 */
  description?: string;
  /** 模板分类 */
  category: string;
  /** 表单配置 */
  config: FormConfig;
  /** 使用次数 */
  usageCount: number;
  /** 收藏数 */
  favoriteCount: number;
  /** 是否公开 */
  isPublic: boolean;
  /** 创建者 */
  created_by: {
    id: number;
    name: string;
  };
  /** 创建时间 */
  created_at: string;
}

/**
 * @brief 表单提交数据
 */
export interface FormSubmitData {
  /** 表单ID */
  formId: string;
  /** 表单值 */
  values: Record<string, unknown>;
  /** 验证结果 */
  isValid: boolean;
  /** 错误信息 */
  errors?: Record<string, string>;
  /** 提交时间 */
  submittedAt: string;
}

/**
 * @brief 表单版本
 */
export interface FormVersion {
  /** 版本号 */
  version: string;
  /** 表单配置 */
  config: FormConfig;
  /** 创建者 */
  created_by: {
    id: number;
    name: string;
  };
  /** 创建时间 */
  created_at: string;
  /** 变更说明 */
  changeLog?: string;
}

/**
 * @brief 表单数据
 */
export interface FormData {
  /** 数据ID */
  id: string;
  /** 表单ID */
  formId: string;
  /** 表单值 */
  values: Record<string, unknown>;
  /** 状态 */
  status: 'draft' | 'pending' | 'approved' | 'rejected';
  /** 创建者 */
  created_by: {
    id: number;
    name: string;
  };
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * @brief 拖拽排序事件
 */
export interface DragSortEvent {
  /** 拖拽的字段ID */
  draggedId: string;
  /** 目标位置 */
  targetIndex: number;
  /** 目标分组ID */
  targetGroupId?: string;
}

/**
 * @brief 字段验证结果
 */
export interface FieldValidationResult {
  /** 字段ID */
  fieldId: string;
  /** 字段名 */
  fieldName: string;
  /** 是否通过 */
  passed: boolean;
  /** 错误消息 */
  message?: string;
}
