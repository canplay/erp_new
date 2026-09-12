/**
 * @file InheritPermissionPanel.vue
 * @description 继承权限配置面板
 * @date 2026-04-04
 * @features
 * - 配置角色继承关系
 * - 查看继承的权限列表
 * - 支持多继承
 */

<template>
  <div class="inherit-permission-panel">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon name="link" class="q-mr-sm" color="primary" />
          继承权限配置
        </div>
        <div class="text-caption text-grey q-mb-md">
          配置角色继承关系，被继承的角色权限会自动传递给当前角色
        </div>

        <!-- 继承说明 -->
        <q-banner class="q-mb-md" rounded banner-class="bg-blue-1">
          <template v-slot:avatar>
            <q-icon name="info" color="blue" />
          </template>
          <div class="text-body2">
            <strong>继承权限说明：</strong>
            <ul class="q-my-sm" style="padding-left: 20px">
              <li>继承的权限会自动添加到角色的有效权限中</li>
              <li>当前角色的自有权限会与继承权限合并</li>
              <li>如发生冲突（同一权限被授予又被打钩），自有权限优先</li>
              <li>支持多继承，即一个角色可以继承多个角色的权限</li>
              <li>继承关系会递归传递（A继承B，B继承C，则A也继承C的权限）</li>
            </ul>
          </div>
        </q-banner>

        <!-- 当前继承状态 -->
        <div class="q-mb-lg">
          <div class="text-subtitle2 q-mb-md">当前继承配置</div>

          <div v-if="currentInherit?.inherit_from?.length" class="q-gutter-sm">
            <q-chip
              v-for="parentRole in currentInherit!.inherit_from!"
              :key="parentRole"
              removable
              color="primary"
              text-color="white"
              icon="link"
              @remove="removeInherit(parentRole)"
            >
              {{ parentRole }}
            </q-chip>
          </div>

          <div v-else class="text-grey text-center q-pa-md">
            <q-icon name="link_off" size="24px" class="q-mb-sm" />
            <div>当前角色没有配置继承权限</div>
          </div>
        </div>

        <!-- 添加继承 -->
        <div v-if="(availableRoles?.length ?? 0) > 0" class="q-mb-lg">
          <div class="text-subtitle2 q-mb-md">添加继承</div>

          <div class="row items-center q-col-gutter-md">
            <div class="col-12 col-sm-8">
              <q-select
                v-model="selectedInheritRole"
                :options="availableRoles"
                outlined
                dense
                clearable
                :label="$t('common.selectInheritRole')"
              >
                <template v-slot:option="{ itemProps, opt }">
                  <q-item v-bind="itemProps">
                    <q-item-section avatar>
                      <q-icon name="source" color="primary" />
                    </q-item-section>
                    <q-item-section>
                      <q-item-label>{{ opt }}</q-item-label>
                      <q-item-label caption>
                        {{ getRoleDescription(opt) }}
                      </q-item-label>
                    </q-item-section>
                  </q-item>
                </template>
              </q-select>
            </div>
            <div class="col-12 col-sm-4">
              <q-btn
                color="primary"
                icon="add_link"
                :label="$t('common.addInheritance')"
                :disable="!selectedInheritRole"
                @click="addInherit"
              />
            </div>
          </div>
        </div>

        <!-- 继承链可视化 -->
        <div v-if="currentInherit?.inherit_from?.length" class="q-mb-lg">
          <div class="text-subtitle2 q-mb-md">继承链</div>

          <div class="inherit-chain">
            <div class="inherit-node current">
              <q-avatar color="primary" text-color="white" icon="security" />
              <div class="q-mt-sm text-center">当前角色</div>
            </div>

            <div
              v-for="parentRole in currentInherit!.inherit_from!"
              :key="parentRole + '-arrow'"
              class="inherit-arrow"
            >
              <q-icon name="arrow_forward" color="grey" />
            </div>

            <div
              v-for="parentRole in currentInherit!.inherit_from!"
              :key="parentRole + '-node'"
              class="inherit-node"
            >
              <q-avatar color="blue" text-color="white" icon="link" />
              <div class="q-mt-sm text-center">{{ parentRole }}</div>
            </div>
          </div>
        </div>

        <!-- 有效权限预览 -->
        <div v-if="currentInherit?.effective_permissions?.length">
          <div class="text-subtitle2 q-mb-md">
            继承的有效权限
            <q-badge color="grey" :label="`${currentInherit!.effective_permissions!.length} 项`" class="q-ml-sm" />
          </div>

          <q-scroll-area style="height: 200px">
            <div class="row q-col-gutter-sm">
              <div
                v-for="perm in currentInherit!.effective_permissions!.slice(0, 20)"
                :key="perm"
                class="col-12 col-sm-6"
              >
                <q-chip size="sm" color="blue-2" text-color="blue-9">
                  <code>{{ perm }}</code>
                </q-chip>
              </div>
            </div>
          </q-scroll-area>

          <div v-if="currentInherit!.effective_permissions!.length > 20" class="text-caption text-grey q-mt-sm">
            还有 {{ currentInherit!.effective_permissions!.length - 20 }} 项权限...
          </div>
        </div>
      </q-card-section>

      <!-- 保存按钮 -->
      <q-separator />
      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.reset')" @click="resetToDefault" />
        <q-btn
          color="primary"
          :label="$t('common.save')"
          :loading="saving"
          @click="saveInherit"
        />
      </q-card-actions>
    </q-card>
  </div>
</template>

<script setup lang="ts">
/**
 * @file InheritPermissionPanel.vue
 * @description 继承权限配置面板组件
 */

import { ref, watch } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import type { InheritPermissionInfo } from '@/types/permission';
import { setRoleInherit } from '@/api/permission';

const $q = useQuasar();
const { t: $t } = useI18n();

// ============ Props & Emits ============

const props = defineProps<{
  /** 角色名称 */
  role_name: string;
  /** 当前继承信息 */
  currentInherit?: InheritPermissionInfo | null;
  /** 可选的继承角色列表 */
  availableRoles?: string[];
}>();

const emit = defineEmits<{
  /** 保存成功 */
  saved: [inheritFrom: string[]];
  /** 变更 */
  changed: [inheritFrom: string[]];
}>();

// ============ 状态 ============

/** 选中的要继承的角色 */
const selectedInheritRole = ref<string | null>(null);

/** 是否正在保存 */
const saving = ref(false);

/** 当前的继承角色列表（本地状态） */
const localInheritFrom = ref<string[]>([]);

// ============ 方法 ============

/**
 * @brief 获取角色描述
 */
function getRoleDescription(role: string): string {
  const descriptions: Record<string, string> = {
    admin: '系统管理员，拥有所有权限',
    user: '普通用户，拥有基础权限',
    vip: 'VIP用户，拥有高级权限',
  };
  return descriptions[role] || '';
}

/**
 * @brief 添加继承
 */
function addInherit() {
  if (!selectedInheritRole.value) return;
  if (localInheritFrom.value.includes(selectedInheritRole.value)) {
    $q.notify({
      type: 'warning',
      message: '该角色已经被继承',
    });
    return;
  }
  localInheritFrom.value.push(selectedInheritRole.value);
  selectedInheritRole.value = null;
  emit('changed', localInheritFrom.value);
}

/**
 * @brief 移除继承
 */
function removeInherit(role: string) {
  const index = localInheritFrom.value.indexOf(role);
  if (index > -1) {
    localInheritFrom.value.splice(index, 1);
    emit('changed', localInheritFrom.value);
  }
}

/**
 * @brief 重置为默认
 */
function resetToDefault() {
  localInheritFrom.value = [...(props.currentInherit?.inherit_from || [])];
  emit('changed', localInheritFrom.value);
}

/**
 * @brief 保存继承配置
 */
async function saveInherit() {
  saving.value = true;
  try {
    await setRoleInherit(props.role_name, localInheritFrom.value);
    $q.notify({
      type: 'positive',
      message: '继承权限保存成功',
    });
    emit('saved', localInheritFrom.value);
  } catch (error) {
    logger.error('【保存继承权限失败】', error);
    $q.notify({
      type: 'negative',
      message: '保存失败，请重试',
    });
  } finally {
    saving.value = false;
  }
}

// ============ 监听器 ============

watch(
  () => props.currentInherit,
  (newVal) => {
    localInheritFrom.value = [...(newVal?.inherit_from || [])];
  },
  { immediate: true, deep: true }
);
</script>

<style scoped>
.inherit-permission-panel {
  width: 100%;
}

.inherit-chain {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  padding: 16px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 8px;
  overflow-x: auto;
}

.body--dark .inherit-chain {
  background: rgba(255, 255, 255, 0.05);
}

.inherit-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 80px;
}

.inherit-node.current {
  border: 2px dashed #1976d2;
  border-radius: 8px;
  padding: 8px;
}

.inherit-arrow {
  flex-shrink: 0;
}
</style>

