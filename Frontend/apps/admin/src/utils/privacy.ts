/**
 * @file privacy.ts
 * @description 敏感信息脱敏工具
 * @date 2026-04-03
 */

// ============ 脱敏全局开关 ============

/**
 * @brief 脱敏功能全局开关
 * @description 设置为 false 可临时禁用所有脱敏功能（用于调试）
 */
let privacyEnabled = true;

/**
 * @brief 启用脱敏功能
 */
export function enablePrivacy() {
  privacyEnabled = true;
}

/**
 * @brief 禁用脱敏功能
 */
export function disablePrivacy() {
  privacyEnabled = false;
}

/**
 * @brief 检查脱敏功能是否启用
 */
export function isPrivacyEnabled(): boolean {
  return privacyEnabled;
}

/**
 * @brief 手机号脱敏
 * 显示前3位和后4位，中间用 * 替代
 * 例如: 13812345678 -> 138****5678
 */
export function maskPhone(phone: string | number | null | undefined): string {
  if (!phone) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return String(phone);
  const str = String(phone);
  if (str.length < 7) return str;
  return `${str.substring(0, 3)}****${str.substring(str.length - 4)}`;
}

/**
 * @brief 邮箱脱敏
 * 显示 @ 前的首尾字符，中间用 * 替代
 * 例如: test@example.com -> t***t@example.com
 */
export function maskEmail(email: string | null | undefined): string {
  if (!email) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return email;
  const atIndex = email.indexOf('@');
  if (atIndex <= 1) return email;
  if (atIndex <= 3) {
    return `${email[0]}***${email.substring(atIndex)}`;
  }
  return `${email.substring(0, atIndex - 1)}***${email.substring(atIndex)}`;
}

/**
 * @brief 身份证号脱敏
 * 显示前3位和后4位，中间用 * 替代
 * 例如: 110101199001011234 -> 110***********1234
 */
export function maskIdCard(idCard: string | null | undefined): string {
  if (!idCard) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return idCard;
  const str = idCard.trim();
  if (str.length < 8) return str;
  return `${str.substring(0, 3)}***********${str.substring(str.length - 4)}`;
}

/**
 * @brief 银行卡号脱敏
 * 显示前4位和后4位，中间用 * 替代
 * 例如: 6222021234567890123 -> 6222********7890123
 */
export function maskBankCard(cardNumber: string | number | null | undefined): string {
  if (!cardNumber) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return String(cardNumber);
  const str = String(cardNumber).replace(/\s/g, '');
  if (str.length < 8) return str;
  return `${str.substring(0, 4)}********${str.substring(str.length - 4)}`;
}

/**
 * @brief 姓名脱敏
 * 单名显示第一个字 + *，复名显示第一个和最后一个字
 * 例如: 张三 -> 张*，欧阳锋 -> 欧*锋
 */
export function maskName(name: string | null | undefined): string {
  if (!name) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return name;
  const len = name.length;
  if (len === 1) return name;
  if (len === 2) return `${name[0]}*`;
  return `${name[0]}${'*'.repeat(len - 2)}${name[len - 1]}`;
}

/**
 * @brief 地址脱敏
 * 只显示省和市，详细地址用 * 替代
 * 例如: 北京市朝阳区某某街道123号 -> 北京市朝阳区***
 */
export function maskAddress(address: string | null | undefined): string {
  if (!address) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return address;
  const detailPattern = /(街道|路|号|巷|弄|楼|栋|单元|室|栋)/i;
  const match = address.match(detailPattern);
  if (match && match.index) {
    return `${address.substring(0, match.index + match[0].length)}***`;
  }
  // 如果没有匹配到详细地址标识，保留前12个字符
  if (address.length > 12) {
    return `${address.substring(0, 12)}***`;
  }
  return address;
}

/**
 * @brief 密码脱敏
 * 全部显示为 *
 */
export function maskPassword(password: string | null | undefined): string {
  if (!password) return '-';
  return '*'.repeat(Math.min(password.length, 12));
}

/**
 * @brief IP 地址脱敏
 * 显示前两段，后两段用 * 替代
 * 例如: 192.0.2.100 -> 192.0.2.*
 */
export function maskIpAddress(ip: string | null | undefined): string {
  if (!ip) return '-';
  // 调试模式下返回原始值
  if (!privacyEnabled) return ip;
  const parts = ip.split('.');
  if (parts.length !== 4) return ip;
  return `${parts[0]}.${parts[1]}.*.*`;
}

/**
 * @brief 统一脱敏接口
 * 根据字段类型自动选择脱敏方式
 */
export interface MaskOptions {
  type: 'phone' | 'email' | 'idCard' | 'bankCard' | 'name' | 'address' | 'password' | 'ip' | 'custom';
  customReplacement?: string;
}

export function maskByType(value: string | number | null | undefined, type: MaskOptions['type']): string {
  switch (type) {
    case 'phone':
      return maskPhone(value);
    case 'email':
      return maskEmail(value as string);
    case 'idCard':
      return maskIdCard(value as string);
    case 'bankCard':
      return maskBankCard(value);
    case 'name':
      return maskName(value as string);
    case 'address':
      return maskAddress(value as string);
    case 'password':
      return maskPassword(value as string);
    case 'ip':
      return maskIpAddress(value as string);
    case 'custom':
      if (value) {
        const str = String(value);
        return str.substring(0, 2) + '*'.repeat(Math.min(str.length - 4, 8)) + str.substring(str.length - 2);
      }
      return '-';
    default:
      return String(value || '-');
  }
}

/**
 * @brief 批量脱敏对象中的敏感字段
 * @param data 原始数据对象
 * @param fields 需要脱敏的字段配置
 * @returns 脱敏后的数据
 *
 * @example
 * const user = { phone: '13812345678', email: 'test@example.com', password: '123456' };
 * const masked = maskFields(user, {
 *   phone: 'phone',
 *   email: 'email',
 *   password: 'password'
 * });
 * // 结果: { phone: '138****5678', email: 't***t@example.com', password: '********' }
 */
export function maskFields<T extends Record<string, unknown>>(
  data: T,
  fields: Record<string, MaskOptions['type']>
): T {
  const result = { ...data };

  for (const [field, type] of Object.entries(fields)) {
    if (field in result && result[field] !== null && result[field] !== undefined) {
      (result as Record<string, unknown>)[field] = maskByType(
        result[field] as string | number,
        type
      );
    }
  }

  return result;
}
