/**
 * @file error-code.ts
 * @description API 错误码定义
 * @date 2026-04-03
 */

/**
 * @brief HTTP 状态码
 */
export const HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  NO_CONTENT: 204,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  TIMEOUT: 408,
  CONFLICT: 409,
  UNPROCESSABLE_ENTITY: 422,
  TOO_MANY_REQUESTS: 429,
  INTERNAL_SERVER_ERROR: 500,
  BAD_GATEWAY: 502,
  SERVICE_UNAVAILABLE: 503,
  GATEWAY_TIMEOUT: 504,
} as const;

/**
 * @brief 业务错误码（需要与后端保持一致）
 */
export const BUSINESS_CODE = {
  // 通用错误 (1000-1999)
  SUCCESS: 0,
  UNKNOWN_ERROR: 1000,
  PARAM_ERROR: 1001,
  PARAM_MISSING: 1002,
  DATA_NOT_FOUND: 1003,
  DATA_DUPLICATE: 1004,
  OPERATION_FAILED: 1005,
  PERMISSION_DENIED: 1006,

  // 认证错误 (2000-2999)
  AUTH_TOKEN_EXPIRED: 2001,
  AUTH_TOKEN_INVALID: 2002,
  AUTH_TOKEN_MISSING: 2003,
  AUTH_CREDENTIALS_INVALID: 2004,
  AUTH_ACCOUNT_DISABLED: 2005,
  AUTH_ACCOUNT_LOCKED: 2006,
  AUTH_CODE_ERROR: 2007,
  AUTH_CODE_EXPIRED: 2008,
  AUTH_PASSWORD_EXPIRED: 2009,

  // 用户错误 (3000-3999)
  USER_NOT_FOUND: 3001,
  USER_ALREADY_EXISTS: 3002,
  USER_DISABLED: 3003,
  USER_ROLE_ERROR: 3004,

  // 资源错误 (4000-4999)
  RESOURCE_NOT_FOUND: 4001,
  RESOURCE_ALREADY_EXISTS: 4002,
  RESOURCE_LIMIT_EXCEEDED: 4003,

  // 文件错误 (5000-5999)
  FILE_UPLOAD_FAILED: 5001,
  FILE_TYPE_NOT_ALLOWED: 5002,
  FILE_SIZE_EXCEEDED: 5003,
  FILE_NOT_FOUND: 5004,

  // 验证错误 (6000-6999)
  VALIDATION_FAILED: 6001,
  VALIDATION_CODE_ERROR: 6002,
} as const;

/**
 * @brief 错误码消息映射
 */
export const ERROR_MESSAGES: Record<number, string> = {
  // 通用错误
  [BUSINESS_CODE.SUCCESS]: '操作成功',
  [BUSINESS_CODE.UNKNOWN_ERROR]: '未知错误，请稍后重试',
  [BUSINESS_CODE.PARAM_ERROR]: '参数错误',
  [BUSINESS_CODE.PARAM_MISSING]: '缺少必要参数',
  [BUSINESS_CODE.DATA_NOT_FOUND]: '数据不存在',
  [BUSINESS_CODE.DATA_DUPLICATE]: '数据已存在',
  [BUSINESS_CODE.OPERATION_FAILED]: '操作失败',
  [BUSINESS_CODE.PERMISSION_DENIED]: '权限不足',

  // 认证错误
  [BUSINESS_CODE.AUTH_TOKEN_EXPIRED]: '登录已过期，请重新登录',
  [BUSINESS_CODE.AUTH_TOKEN_INVALID]: '登录凭证无效，请重新登录',
  [BUSINESS_CODE.AUTH_TOKEN_MISSING]: '请先登录',
  [BUSINESS_CODE.AUTH_CREDENTIALS_INVALID]: '用户名或密码错误',
  [BUSINESS_CODE.AUTH_ACCOUNT_DISABLED]: '账号已被禁用',
  [BUSINESS_CODE.AUTH_ACCOUNT_LOCKED]: '账号已被锁定',
  [BUSINESS_CODE.AUTH_CODE_ERROR]: '验证码错误',
  [BUSINESS_CODE.AUTH_CODE_EXPIRED]: '验证码已过期',
  [BUSINESS_CODE.AUTH_PASSWORD_EXPIRED]: '密码已过期，请修改密码',

  // 用户错误
  [BUSINESS_CODE.USER_NOT_FOUND]: '用户不存在',
  [BUSINESS_CODE.USER_ALREADY_EXISTS]: '用户已存在',
  [BUSINESS_CODE.USER_DISABLED]: '用户已被禁用',
  [BUSINESS_CODE.USER_ROLE_ERROR]: '用户角色错误',

  // 资源错误
  [BUSINESS_CODE.RESOURCE_NOT_FOUND]: '资源不存在',
  [BUSINESS_CODE.RESOURCE_ALREADY_EXISTS]: '资源已存在',
  [BUSINESS_CODE.RESOURCE_LIMIT_EXCEEDED]: '资源数量已达上限',

  // 文件错误
  [BUSINESS_CODE.FILE_UPLOAD_FAILED]: '文件上传失败',
  [BUSINESS_CODE.FILE_TYPE_NOT_ALLOWED]: '不支持的文件类型',
  [BUSINESS_CODE.FILE_SIZE_EXCEEDED]: '文件大小超出限制',
  [BUSINESS_CODE.FILE_NOT_FOUND]: '文件不存在',

  // 验证错误
  [BUSINESS_CODE.VALIDATION_FAILED]: '数据验证失败',
  [BUSINESS_CODE.VALIDATION_CODE_ERROR]: '验证码错误',
};

/**
 * @brief 获取错误消息
 */
export function getErrorMessage(code: number): string {
  return ERROR_MESSAGES[code] || '操作失败，请稍后重试';
}

/**
 * @brief 判断是否为认证错误
 */
export function isAuthError(code: number): boolean {
  return code >= 2000 && code < 3000;
}

/**
 * @brief 判断是否为权限错误
 */
export function isPermissionError(code: number): boolean {
  return code === BUSINESS_CODE.PERMISSION_DENIED || code === HTTP_STATUS.FORBIDDEN;
}

/**
 * @brief 判断是否为需要重新登录的错误
 */
export function needRelogin(code: number): boolean {
  return (
    isAuthError(code) ||
    code === BUSINESS_CODE.AUTH_TOKEN_EXPIRED ||
    code === BUSINESS_CODE.AUTH_TOKEN_INVALID ||
    code === BUSINESS_CODE.AUTH_TOKEN_MISSING
  );
}
