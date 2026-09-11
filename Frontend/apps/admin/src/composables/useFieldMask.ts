import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  MaskStrategy,
  MASK_STRATEGY_OPTIONS,
  FieldMaskUtils,
  DEFAULT_FIELD_MASKS,
} from '@/types/fieldMask';
import type { FieldMaskRule } from '@/types/fieldMask';

export interface FieldMaskItem extends FieldMaskRule {
  fieldType: 'string' | 'number' | 'date' | 'phone' | 'email' | 'idcard' | 'bankcard';
  sampleValue: string;
}

interface SampleValues {
  [entityType: string]: Record<string, string>;
}

const sampleValues: SampleValues = {
  user: {
    phone: '13812345678',
    idcard: '110101199001011234',
    email: 'zhangsan@example.com',
    bankCard: '6222021234567890123',
    realName: '张三',
    address: '北京市朝阳区建国路88号',
  },
  customer: {
    contactPhone: '13987654321',
    contactIdCard: '310101199505051234',
    bankAccount: '6222020987654321098',
  },
  order: {
    buyerPhone: '13666666666',
    buyerAddress: '上海市浦东新区世纪大道100号',
  },
};

export function useFieldMask() {
  const $q = useQuasar();
  const { t } = useI18n();

  const selectedEntityType = ref('user');
  const selectedRole = ref<string | null>(null);
  const searchQuery = ref('');
  const fieldMasks = ref<FieldMaskItem[]>([]);
  const batchStrategy = ref<MaskStrategy>(MaskStrategy.NONE);
  const selectedFields = ref<string[]>([]);
  const previewRule = ref<FieldMaskItem | null>(null);
  const maskedPreview = ref('');
  const showPreviewDialog = ref(false);

  const entityTypeOptions = [
    { label: '用户信息', value: 'user' },
    { label: '客户信息', value: 'customer' },
    { label: '订单信息', value: 'order' },
  ];

  const roleOptions = [
    { label: '管理员', value: 'admin' },
    { label: '普通用户', value: 'user' },
    { label: '访客', value: 'guest' },
  ];

  const filteredFieldMasks = computed(() => {
    let result = fieldMasks.value.filter((m) => m.entity_type === selectedEntityType.value);

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (m) =>
          m.field.toLowerCase().includes(query) ||
          m.fieldName.toLowerCase().includes(query)
      );
    }

    return result;
  });

  function getFieldTypeFromStrategy(strategy: MaskStrategy): string {
    const mapping: Record<MaskStrategy, string> = {
      [MaskStrategy.PHONE]: 'phone',
      [MaskStrategy.EMAIL]: 'email',
      [MaskStrategy.ID_CARD]: 'idcard',
      [MaskStrategy.BANK_CARD]: 'bankcard',
      [MaskStrategy.NAME]: 'string',
      [MaskStrategy.ADDRESS]: 'string',
      [MaskStrategy.AMOUNT]: 'number',
      [MaskStrategy.HIDE]: 'string',
      [MaskStrategy.CUSTOM]: 'string',
      [MaskStrategy.NONE]: 'string',
      [MaskStrategy.REVEAL]: 'string',
      [MaskStrategy.HASH]: 'string',
    };
    return mapping[strategy] || 'string';
  }

  function getDefaultVisiblePrefix(strategy: MaskStrategy): number {
    const prefixes: Partial<Record<MaskStrategy, number>> = {
      [MaskStrategy.PHONE]: 3,
      [MaskStrategy.ID_CARD]: 6,
      [MaskStrategy.BANK_CARD]: 0,
      [MaskStrategy.NAME]: 1,
    };
    return prefixes[strategy] ?? 0;
  }

  function getDefaultVisibleSuffix(strategy: MaskStrategy): number {
    const suffixes: Partial<Record<MaskStrategy, number>> = {
      [MaskStrategy.PHONE]: 4,
      [MaskStrategy.ID_CARD]: 4,
      [MaskStrategy.BANK_CARD]: 4,
      [MaskStrategy.NAME]: 1,
    };
    return suffixes[strategy] ?? 0;
  }

  function getDefaultSample(fieldType: string): string {
    const samples: Record<string, string> = {
      phone: '138****5678',
      email: 'a***@example.com',
      idcard: '110101***********1234',
      bankcard: '**** **** **** 1234',
      string: '示例文本',
      number: '********',
      date: '****-**-**',
    };
    return samples[fieldType] || '***';
  }

  function initFieldMasks(initialRules?: FieldMaskRule[]) {
    const defaultMasksForEntity = DEFAULT_FIELD_MASKS.filter(
      (m) => m.entity_type === selectedEntityType.value
    );

    const existingMap = new Map(
      initialRules
        ?.filter((r) => r.entity_type === selectedEntityType.value)
        .map((r) => [r.field, r]) || []
    );

    fieldMasks.value = defaultMasksForEntity.map((m) => {
      const existing = existingMap.get(m.field);
      const samples = sampleValues[selectedEntityType.value] || {};
      const fieldType = getFieldTypeFromStrategy(m.strategy);

      return {
        ...m,
        id: existing?.id || Math.random(),
        strategy: existing?.strategy || m.strategy,
        customPattern: existing?.customPattern || m.customPattern,
        replaceChar: existing?.replaceChar || m.replaceChar || '*',
        visiblePrefix: existing?.visiblePrefix ?? m.visiblePrefix ?? getDefaultVisiblePrefix(m.strategy),
        visibleSuffix: existing?.visibleSuffix ?? m.visibleSuffix ?? getDefaultVisibleSuffix(m.strategy),
        defaultValue: existing?.defaultValue || m.defaultValue,
        enabled: existing?.enabled ?? m.enabled ?? true,
        sampleValue: samples[m.field] || getDefaultSample(fieldType),
      } as FieldMaskItem;
    });
  }

  function handleStrategyChange(rule: FieldMaskItem) {
    if (rule.strategy !== MaskStrategy.CUSTOM) {
      delete rule.customPattern;
    }
    if (rule.strategy !== MaskStrategy.HIDE) {
      delete rule.defaultValue;
    }
    rule.visiblePrefix = getDefaultVisiblePrefix(rule.strategy);
    rule.visibleSuffix = getDefaultVisibleSuffix(rule.strategy);

    handleRuleChange(rule);
  }

  function handleRuleChange(rule: { field: string; [key: string]: unknown }) {
    const result = FieldMaskUtils.applyMask(rule.sampleValue, rule.strategy, {
      ...(rule.customPattern !== undefined ? { customPattern: rule.customPattern } : {}),
      ...(rule.replaceChar !== undefined ? { replaceChar: rule.replaceChar } : {}),
      ...(rule.visiblePrefix !== undefined ? { visiblePrefix: rule.visiblePrefix } : {}),
      ...(rule.visibleSuffix !== undefined ? { visibleSuffix: rule.visibleSuffix } : {}),
      ...(rule.defaultValue !== undefined ? { defaultValue: rule.defaultValue } : {}),
    });
    rule.sampleValue = result.maskedValue;
  }

  function showPreview(rule: FieldMaskItem) {
    previewRule.value = rule;

    const samples = sampleValues[selectedEntityType.value] || {};
    const originalValue = samples[rule.field] || '13812345678';

    const result = FieldMaskUtils.applyMask(originalValue, rule.strategy, {
      ...(rule.customPattern !== undefined ? { customPattern: rule.customPattern } : {}),
      ...(rule.replaceChar !== undefined ? { replaceChar: rule.replaceChar } : {}),
      ...(rule.visiblePrefix !== undefined ? { visiblePrefix: rule.visiblePrefix } : {}),
      ...(rule.visibleSuffix !== undefined ? { visibleSuffix: rule.visibleSuffix } : {}),
      ...(rule.defaultValue !== undefined ? { defaultValue: rule.defaultValue } : {}),
    });

    maskedPreview.value = result.maskedValue;
    showPreviewDialog.value = true;
  }

  function applyBatchStrategy() {
    if (selectedFields.value.length === 0) {
      $q.notify({
        type: 'warning',
        message: '请先选择要应用规则的字段',
      });
      return;
    }

    fieldMasks.value.forEach((rule) => {
      if (selectedFields.value.includes(rule.field)) {
        rule.strategy = batchStrategy.value;
        handleStrategyChange(rule);
      }
    });

    $q.notify({
      type: 'positive',
      message: `已成功应用到 ${selectedFields.value.length} 个字段`,
    });
  }

  function saveAllRules() {
    const rules: FieldMaskRule[] = fieldMasks.value.map((rule) => ({
      id: rule.id,
      entity_type: rule.entity_type,
      field: rule.field,
      fieldName: rule.fieldName,
      strategy: rule.strategy,
      ...(rule.customPattern !== undefined ? { customPattern: rule.customPattern } : {}),
      ...(rule.replaceChar !== undefined ? { replaceChar: rule.replaceChar } : {}),
      ...(rule.visiblePrefix !== undefined ? { visiblePrefix: rule.visiblePrefix } : {}),
      ...(rule.visibleSuffix !== undefined ? { visibleSuffix: rule.visibleSuffix } : {}),
      ...(rule.defaultValue !== undefined ? { defaultValue: rule.defaultValue } : {}),
      enabled: rule.enabled,
      priority: rule.priority,
      created_at: rule.created_at,
      updated_at: new Date().toISOString(),
    }));

    logger.info('【字段脱敏规则】保存的数据：', rules);
    emitSaved(rules);

    $q.notify({
      type: 'positive',
      message: '脱敏规则保存成功',
    });
  }

  // We need to get the emit from the component. Use a callback approach.
  let emitSaved: (rules: FieldMaskRule[]) => void = () => {};

  function setOnSaved(callback: (rules: FieldMaskRule[]) => void) {
    emitSaved = callback;
  }

  watch(selectedEntityType, () => {
    initFieldMasks();
  });

  return {
    selectedEntityType,
    selectedRole,
    searchQuery,
    fieldMasks,
    batchStrategy,
    selectedFields,
    previewRule,
    maskedPreview,
    showPreviewDialog,
    entityTypeOptions,
    roleOptions,
    filteredFieldMasks,
    initFieldMasks,
    handleStrategyChange,
    handleRuleChange,
    showPreview,
    applyBatchStrategy,
    saveAllRules,
    setOnSaved,
  };
}
