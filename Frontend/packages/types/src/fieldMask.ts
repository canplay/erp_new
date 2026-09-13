/**
 * @file fieldMask.ts
 * @description 字段脱敏规则类型定义
 * @date 2026-04-04
 */

export enum MaskStrategy {
  NONE = 'none',
  HIDE = 'hide',
  PHONE = 'phone',
  EMAIL = 'email',
  ID_CARD = 'id_card',
  BANK_CARD = 'bank_card',
  NAME = 'name',
  ADDRESS = 'address',
  CUSTOM = 'custom',
  AMOUNT = 'amount',
  REVEAL = 'reveal',
  HASH = 'hash',
}

export interface FieldMaskRule {
  id: number;
  entity_type: string;
  field: string;
  fieldName: string;
  strategy: MaskStrategy;
  customPattern?: string;
  replaceChar?: string;
  visiblePrefix?: number;
  visibleSuffix?: number;
  conditionField?: string;
  conditionValue?: unknown;
  defaultValue?: string;
  enabled: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
}

export const MASK_STRATEGY_OPTIONS: Array<{ label: string; value: MaskStrategy }> = [
  { label: '不脱敏', value: MaskStrategy.NONE },
  { label: '完全隐藏', value: MaskStrategy.HIDE },
  { label: '手机号脱敏', value: MaskStrategy.PHONE },
  { label: '邮箱脱敏', value: MaskStrategy.EMAIL },
  { label: '身份证脱敏', value: MaskStrategy.ID_CARD },
  { label: '银行卡脱敏', value: MaskStrategy.BANK_CARD },
  { label: '姓名脱敏', value: MaskStrategy.NAME },
  { label: '地址脱敏', value: MaskStrategy.ADDRESS },
  { label: '金额脱敏', value: MaskStrategy.AMOUNT },
  { label: '自定义', value: MaskStrategy.CUSTOM },
];

export interface FieldMaskResult {
  originalValue: string;
  maskedValue: string;
  strategy: MaskStrategy;
  isMasked: boolean;
}

export class FieldMaskUtils {
  static applyMask(
    value: string | number | null | undefined,
    strategy: MaskStrategy,
    options: { customPattern?: string; replaceChar?: string; visiblePrefix?: number; visibleSuffix?: number; defaultValue?: string } = {}
  ): FieldMaskResult {
    const originalValue = String(value ?? '');
    const replaceChar = options.replaceChar || '*';

    if (!originalValue || strategy === MaskStrategy.NONE) {
      return { originalValue, maskedValue: originalValue, strategy, isMasked: false };
    }

    let maskedValue: string;
    switch (strategy) {
      case MaskStrategy.HIDE:
        maskedValue = options.defaultValue || '***';
        break;
      case MaskStrategy.PHONE:
        maskedValue = originalValue.length < 7
          ? originalValue.replace(/./g, replaceChar)
          : originalValue.slice(0, 3) + replaceChar.repeat(originalValue.length - 7) + originalValue.slice(-4);
        break;
      case MaskStrategy.EMAIL: {
        const parts = originalValue.split('@');
        if (parts.length === 2) {
          const local = parts[0] ?? '';
          const domain = parts[1] ?? '';
          maskedValue = (local[0] ?? '') + replaceChar.repeat(Math.max(0, local.length - 2)) + (local[local.length - 1] ?? '') + '@' + domain;
        } else {
          maskedValue = originalValue.replace(/./g, replaceChar);
        }
        break;
      }
      case MaskStrategy.ID_CARD:
        maskedValue = originalValue.length < 8
          ? originalValue.replace(/./g, replaceChar)
          : originalValue.slice(0, 6) + replaceChar.repeat(originalValue.length - 10) + originalValue.slice(-4);
        break;
      case MaskStrategy.BANK_CARD:
        maskedValue = replaceChar.repeat(originalValue.length - 4) + originalValue.slice(-4);
        break;
      case MaskStrategy.NAME:
        maskedValue = originalValue.length <= 1
          ? replaceChar
          : originalValue[0] + replaceChar.repeat(originalValue.length - 2) + originalValue[originalValue.length - 1];
        break;
      case MaskStrategy.ADDRESS: {
        const parts = originalValue.split(/[省市区县]/);
        maskedValue = parts.length > 1 ? parts[0] + '***' : originalValue.length > 10 ? originalValue.slice(0, 10) + '...' : originalValue;
        break;
      }
      case MaskStrategy.AMOUNT: {
        const parts = originalValue.split('.');
        maskedValue = parts.length === 2
          ? replaceChar.repeat((parts[0] ?? '').length) + '.' + (parts[1] ?? '')
          : replaceChar.repeat(originalValue.length);
        break;
      }
      case MaskStrategy.CUSTOM: {
        if (!options.customPattern) {
          maskedValue = originalValue.replace(/./g, replaceChar);
          break;
        }
        const [prefixLen = 0, suffixLen = 0] = options.customPattern.split('-').map(Number);
        const maskedLen = originalValue.length - prefixLen - suffixLen;
        maskedValue = originalValue.slice(0, prefixLen) + replaceChar.repeat(maskedLen > 0 ? maskedLen : 0) + originalValue.slice(originalValue.length - suffixLen);
        break;
      }
      default:
        maskedValue = originalValue;
    }

    return { originalValue, maskedValue, strategy, isMasked: maskedValue !== originalValue };
  }
}

export const DEFAULT_FIELD_MASKS: Omit<FieldMaskRule, 'id' | 'created_at' | 'updated_at'>[] = [
  { entity_type: 'user', field: 'phone', fieldName: '手机号', strategy: MaskStrategy.PHONE, enabled: true, priority: 1 },
  { entity_type: 'user', field: 'idcard', fieldName: '身份证号', strategy: MaskStrategy.ID_CARD, enabled: true, priority: 2 },
  { entity_type: 'user', field: 'email', fieldName: '邮箱', strategy: MaskStrategy.EMAIL, enabled: true, priority: 3 },
  { entity_type: 'user', field: 'bankCard', fieldName: '银行卡号', strategy: MaskStrategy.BANK_CARD, enabled: true, priority: 4 },
  { entity_type: 'user', field: 'realName', fieldName: '真实姓名', strategy: MaskStrategy.NAME, enabled: true, priority: 5 },
  { entity_type: 'user', field: 'address', fieldName: '地址', strategy: MaskStrategy.ADDRESS, enabled: true, priority: 6 },
  { entity_type: 'customer', field: 'contactPhone', fieldName: '联系电话', strategy: MaskStrategy.PHONE, enabled: true, priority: 1 },
  { entity_type: 'customer', field: 'contactIdCard', fieldName: '联系人身份证', strategy: MaskStrategy.ID_CARD, enabled: true, priority: 2 },
  { entity_type: 'customer', field: 'bankAccount', fieldName: '银行账户', strategy: MaskStrategy.BANK_CARD, enabled: true, priority: 3 },
  { entity_type: 'order', field: 'buyerPhone', fieldName: '买家手机', strategy: MaskStrategy.PHONE, enabled: true, priority: 1 },
  { entity_type: 'order', field: 'buyerAddress', fieldName: '买家地址', strategy: MaskStrategy.ADDRESS, enabled: true, priority: 2 },
];
