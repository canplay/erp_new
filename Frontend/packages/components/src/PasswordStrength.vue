/**
 * @file PasswordStrength.vue
 * @description 密码强度校验组件
 * @date 2026-04-03
 */

<template>
  <div class="password-strength">
    <!-- 密码强度指示条 -->
    <div class="strength-bar q-mb-sm">
      <div
        class="strength-fill"
        :class="strengthClass"
        :style="{ width: strengthWidth }"
      />
    </div>

    <!-- 强度文字提示 -->
    <div class="strength-text row justify-between items-center text-caption">
      <span>{{ $t('passwordStrength.strength') }}：{{ strengthLabel }}</span>
      <span class="text-grey-6">{{ requirementText }}</span>
    </div>

    <!-- 密码要求清单 -->
    <div class="requirements q-mt-sm">
      <div
        v-for="req in requirements"
        :key="req.key"
        class="requirement row items-center q-py-xs"
      >
        <q-icon
          :name="req.met ? 'check_circle' : 'radio_button_unchecked'"
          :color="req.met ? 'positive' : 'grey-5'"
          size="16px"
          class="q-mr-xs"
        />
        <span :class="req.met ? 'text-positive' : 'text-grey-6'">
          {{ req.label }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

// Password requirement interface
interface Requirement {
  key: string;
  label: string;
  met: boolean;
}

const { t } = useI18n();
const props = defineProps<{
  password: string;
}>();


// Password strength levels
type StrengthLevel = 'empty' | 'weak' | 'medium' | 'strong';

// Strength configuration - 使用 i18n
const strengthConfig = computed(() => ({
  empty: { label: t('passwordStrength.empty'), width: '0%', class: '' },
  weak: { label: t('passwordStrength.weak'), width: '33%', class: 'strength-weak' },
  medium: { label: t('passwordStrength.medium'), width: '66%', class: 'strength-medium' },
  strong: { label: t('passwordStrength.strong'), width: '100%', class: 'strength-strong' },
}));

// Password validation rules - 使用 i18n
const rules = computed(() => [
  { key: 'length', label: t('passwordStrength.length'), test: (p: string) => p.length >= 8 },
  { key: 'lowercase', label: t('passwordStrength.lowercase'), test: (p: string) => /[a-z]/.test(p) },
  { key: 'uppercase', label: t('passwordStrength.uppercase'), test: (p: string) => /[A-Z]/.test(p) },
  { key: 'number', label: t('passwordStrength.number'), test: (p: string) => /\d/.test(p) },
  { key: 'special', label: t('passwordStrength.special'), test: (p: string) => /[!@#$%^&*()_+\-=[\]{};':"\\|,.<>?]/.test(p) },
]);

/**
 * @brief Calculate password strength level
 */
function getStrengthLevel(password: string): StrengthLevel {
  if (!password) return 'empty';

  const metCount = rules.value.filter((rule) => rule.test(password)).length;

  if (metCount <= 2) return 'weak';
  if (metCount <= 3) return 'medium';
  return 'strong';
}

/**
 * @brief Check each requirement
 */
const requirements = computed<Requirement[]>(() => {
  return rules.value.map((rule) => ({
    ...rule,
    met: props.password ? rule.test(props.password) : false,
  }));
});

/**
 * @brief Current strength level
 */
const strengthLevel = computed<StrengthLevel>(() => getStrengthLevel(props.password));

/**
 * @brief Strength label
 */
const strengthLabel = computed(() => strengthConfig.value[strengthLevel.value].label);

/**
 * @brief Strength bar width
 */
const strengthWidth = computed(() => strengthConfig.value[strengthLevel.value].width);

/**
 * @brief Strength bar CSS class
 */
const strengthClass = computed(() => strengthConfig.value[strengthLevel.value].class);

/**
 * @brief Met requirements count text
 */
const requirementText = computed(() => {
  const met = requirements.value.filter((r) => r.met).length;
  return t('passwordStrength.requirementMet', { met, total: rules.value.length }).replace('{met}', String(met)).replace('{total}', String(rules.value.length)) || `${met}/${rules.value.length} ${t('passwordStrength.requirementMet')}`;
});
</script>

<style scoped>
.password-strength {
  padding: 8px 0;
}

/* 强度指示条 */
.strength-bar {
  height: 4px;
  background: #e0e0e0;
  border-radius: 2px;
  overflow: hidden;
}

.strength-fill {
  height: 100%;
  transition: all 0.3s ease;
  border-radius: 2px;
}

/* 强度颜色 */
.strength-weak {
  background: linear-gradient(90deg, #ff4757, #ff6b81);
}

.strength-medium {
  background: linear-gradient(90deg, #ffa502, #ff6348);
}

.strength-strong {
  background: linear-gradient(90deg, #2ed573, #7bed9f);
}

/* 要求清单 */
.requirement {
  transition: color 0.2s ease;
}

.requirement span {
  font-size: 12px;
}
</style>
