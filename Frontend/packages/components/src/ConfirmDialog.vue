<template>
  <q-dialog v-model="showDialog" persistent class="confirm-dialog-wrapper">
    <q-card class="confirm-dialog-card">
      <div class="confirm-dialog-header" :class="`header-${confirmColor}`">
        <q-avatar class="confirm-icon" :color="iconColor" text-color="white">
          <q-icon :name="icon" class="icon-animate" />
        </q-avatar>
        <span class="confirm-title">{{ title }}</span>
      </div>

      <q-card-section class="confirm-dialog-content">
        <div class="confirm-message">{{ message }}</div>

        <!-- 密码确认输入（敏感操作时显示） -->
        <q-input
          v-if="requirePassword"
          v-model="password"
          :type="showPassword ? 'text' : 'password'"
          :label="$t('confirmDialog.passwordPlaceholder')"
          outlined
          dense
          class="q-mt-md modern-input"
        >
          <template v-slot:prepend>
            <q-icon name="lock" />
          </template>
          <template v-slot:append>
            <q-icon
              :name="showPassword ? 'visibility_off' : 'visibility'"
              class="cursor-pointer"
              @click="showPassword = !showPassword"
            />
          </template>
        </q-input>

        <!-- 额外信息输入 -->
        <q-input
          v-if="requireInput"
          v-model="inputValue"
          :label="inputLabel"
          outlined
          dense
          class="q-mt-md modern-input"
          :rules="[(val) => !!val || inputError || $t('confirmDialog.passwordRequired')]"
        >
          <template v-slot:prepend>
            <q-icon name="edit" />
          </template>
        </q-input>
      </q-card-section>

      <q-card-actions class="confirm-dialog-actions">
        <q-btn flat :label="cancelText || $t('common.cancel')" color="grey" class="cancel-btn" @click="handleCancel" />
        <q-btn
          :label="confirmText || $t('common.confirm')"
          :color="confirmColor"
          class="confirm-btn"
          :class="`confirm-btn-${confirmColor}`"
          @click="handleConfirm"
          :disable="requirePassword && !password"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
/**
 * @file ConfirmDialog.vue
 * @description 确认对话框组件（增强版）
 * @date 2026-04-03
 * @note 新增密码二次确认和自定义输入功能
 */

import { ref, computed } from 'vue';

export interface ConfirmDialogProps {
  title?: string;
  message: string;
  icon?: string;
  iconColor?: string;
  confirmText?: string;
  cancelText?: string;
  confirmColor?: string;
  /** 是否需要密码确认 */
  requirePassword?: boolean;
  /** 是否需要额外输入 */
  requireInput?: boolean;
  /** 输入框标签 */
  inputLabel?: string;
  /** 输入错误提示 */
  inputError?: string;
}

const props = withDefaults(defineProps<ConfirmDialogProps>(), {
  icon: 'warning',
  iconColor: 'negative',
  confirmColor: 'primary',
  requirePassword: false,
  requireInput: false,
  inputLabel: '',
  inputError: '',
});

const emit = defineEmits<{
  (e: 'confirm', password?: string, inputValue?: string): void;
  (e: 'cancel'): void;
}>();

const showDialog = ref(false);
const password = ref('');
const inputValue = ref('');
const showPassword = ref(false);

/**
 * @brief 默认图标颜色
 */
const iconColor = computed(() => props.iconColor || 'negative');

/**
 * @brief 打开确认对话框
 */
function open() {
  showDialog.value = true;
  // 重置输入
  password.value = '';
  inputValue.value = '';
  showPassword.value = false;
}

/**
 * @brief 关闭对话框
 */
function close() {
  showDialog.value = false;
  password.value = '';
  inputValue.value = '';
  showPassword.value = false;
}

/**
 * @brief 确认操作
 */
function handleConfirm() {
  if (props.requirePassword && !password.value) {
    return;
  }
  if (props.requireInput && !inputValue.value) {
    return;
  }

  if (props.requirePassword) {
    emit('confirm', password.value, inputValue.value);
  } else if (props.requireInput) {
    emit('confirm', undefined, inputValue.value);
  } else {
    emit('confirm');
  }
  close();
}

/**
 * @brief 取消操作
 */
function handleCancel() {
  emit('cancel');
  close();
}

defineExpose({ open, close });
</script>

<style scoped>
/* ============ 现代确认对话框样式 ============ */
.confirm-dialog-card {
  border-radius: 16px;
  min-width: 400px;
  max-width: 90vw;
  overflow: hidden;
  animation: dialogSlideIn 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

@keyframes dialogSlideIn {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* 头部样式 */
.confirm-dialog-header {
  display: flex;
  align-items: center;
  padding: 20px 24px;
  gap: 12px;
}

.header-negative {
  background: linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%);
}

.header-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.header-warning {
  background: linear-gradient(135deg, #ffc107 0%, #fab005 100%);
}

.header-positive {
  background: linear-gradient(135deg, #51cf66 0%, #40c057 100%);
}

.header-info {
  background: linear-gradient(135deg, #339af0 0%, #228be6 100%);
}

/* 图标动画 */
.confirm-icon {
  width: 48px !important;
  height: 48px !important;
}

.icon-animate {
  animation: iconBounce 0.5s ease;
}

@keyframes iconBounce {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.2);
  }
}

.confirm-title {
  font-size: 18px;
  font-weight: 600;
  color: white;
}

/* 内容区域 */
.confirm-dialog-content {
  padding: 24px;
}

.confirm-message {
  font-size: 15px;
  color: #333;
  line-height: 1.6;
}

.body--dark .confirm-message {
  color: #b0b0b0;
}

/* 现代输入框 */
.modern-input {
  border-radius: 8px;
}

.modern-input :deep(.q-field__control) {
  border-radius: 8px;
  transition: all 0.25s ease;
}

.modern-input :deep(.q-field__control:hover) {
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
}

.modern-input :deep(.q-field--focused .q-field__control) {
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.3);
}

.body--dark .modern-input :deep(.q-field__control) {
  background: #2d2d2d;
}

/* 操作按钮区域 */
.confirm-dialog-actions {
  padding: 16px 24px;
  background: #f8f9fa;
  gap: 12px;
}

.body--dark .confirm-dialog-actions {
  background: #252525;
}

.cancel-btn {
  border-radius: 8px;
  transition: all 0.2s ease;
}

.cancel-btn:hover {
  background: rgba(0, 0, 0, 0.05);
}

.body--dark .cancel-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

/* 确认按钮 */
.confirm-btn {
  border-radius: 8px;
  font-weight: 600;
  transition: all 0.25s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.confirm-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.confirm-btn-negative {
  background: linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%) !important;
}

.confirm-btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
}

.confirm-btn-positive {
  background: linear-gradient(135deg, #51cf66 0%, #40c057 100%) !important;
}

.confirm-btn-warning {
  background: linear-gradient(135deg, #ffc107 0%, #fab005 100%) !important;
  color: #333 !important;
}

/* 响应式 */
@media (max-width: 599px) {
  .confirm-dialog-card {
    min-width: 95vw;
    margin: 16px;
  }

  .confirm-dialog-header {
    padding: 16px 20px;
  }

  .confirm-dialog-content {
    padding: 20px;
  }

  .confirm-dialog-actions {
    padding: 12px 20px;
    flex-direction: column;
  }

  .cancel-btn,
  .confirm-btn {
    width: 100%;
  }
}
</style>
