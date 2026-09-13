<template>
  <q-dialog v-model="showDialog" seamless>
    <q-card style="min-width: 400px; max-width: 600px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('shortcuts.title') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="showDialog = false" />
      </q-card-section>

      <q-separator />

      <q-card-section class="shortcuts-list">
        <!-- 通用快捷键 -->
        <div class="shortcuts-section">
          <div class="section-title">{{ $t('shortcuts.general') }}</div>
          <div v-for="(shortcut, index) in generalShortcuts" :key="`general-${index}`" class="shortcut-item">
            <div class="shortcut-keys">
              <q-badge
                v-for="(key, index) in shortcut.keys"
                :key="index"
                color="primary"
                class="shortcut-key"
              >
                {{ key }}
              </q-badge>
            </div>
            <div class="shortcut-desc">{{ shortcut.description }}</div>
          </div>
        </div>

        <!-- 表格快捷键 -->
        <div class="shortcuts-section">
          <div class="section-title">{{ $t('shortcuts.table') }}</div>
          <div v-for="(shortcut, index) in tableShortcuts" :key="`table-${index}`" class="shortcut-item">
            <div class="shortcut-keys">
              <q-badge
                v-for="(key, index) in shortcut.keys"
                :key="index"
                color="primary"
                class="shortcut-key"
              >
                {{ key }}
              </q-badge>
            </div>
            <div class="shortcut-desc">{{ shortcut.description }}</div>
          </div>
        </div>

        <!-- 导航快捷键 -->
        <div class="shortcuts-section">
          <div class="section-title">{{ $t('shortcuts.navigation') }}</div>
          <div v-for="(shortcut, index) in navShortcuts" :key="`nav-${index}`" class="shortcut-item">
            <div class="shortcut-keys">
              <q-badge
                v-for="(key, index) in shortcut.keys"
                :key="index"
                color="primary"
                class="shortcut-key"
              >
                {{ key }}
              </q-badge>
            </div>
            <div class="shortcut-desc">{{ shortcut.description }}</div>
          </div>
        </div>
      </q-card-section>

      <q-card-actions align="center" class="q-pb-md">
        <q-btn flat color="grey" :label="$t('common.close')" @click="showDialog = false" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';


const { t } = useI18n();
const showDialog = ref(false);

/**
 * @brief 打开对话框
 */
function open() {
  showDialog.value = true;
}

/**
 * @brief 关闭对话框
 */
function close() {
  showDialog.value = false;
}

// 暴露方法
defineExpose({ open, close });

// 通用快捷键
const generalShortcuts = computed(() => [
  { keys: ['Ctrl', '?'], description: t('shortcuts.showHelp') },
  { keys: ['Ctrl', 'K'], description: t('shortcuts.globalSearch') },
  { keys: ['Ctrl', 'N'], description: t('shortcuts.new') },
  { keys: ['Esc'], description: t('shortcuts.close') },
  { keys: ['Ctrl', 'S'], description: t('shortcuts.save') },
  { keys: ['Ctrl', 'Z'], description: t('shortcuts.undo') },
]);

// 表格快捷键
const tableShortcuts = computed(() => [
  { keys: ['↑', '↓'], description: t('shortcuts.navigateRows') },
  { keys: ['Enter'], description: t('shortcuts.editRow') },
  { keys: ['Delete'], description: t('shortcuts.deleteRow') },
  { keys: ['Space'], description: t('shortcuts.selectRow') },
  { keys: ['Ctrl', 'A'], description: t('shortcuts.selectAll') },
]);

// 导航快捷键
const navShortcuts = computed(() => [
  { keys: ['G', 'H'], description: t('shortcuts.goHome') },
  { keys: ['G', 'P'], description: t('shortcuts.goProfile') },
  { keys: ['G', 'S'], description: t('shortcuts.goSettings') },
  { keys: ['Ctrl', '←'], description: t('shortcuts.prevTab') },
  { keys: ['Ctrl', '→'], description: t('shortcuts.nextTab') },
]);
</script>

<style scoped>
.shortcuts-list {
  max-height: 500px;
  overflow-y: auto;
}

.shortcuts-section {
  margin-bottom: 24px;
}

.shortcuts-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #666;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #eee;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.shortcut-keys {
  display: flex;
  gap: 4px;
}

.shortcut-key {
  font-size: 12px;
  padding: 4px 8px;
  min-width: 28px;
  justify-content: center;
}

.shortcut-desc {
  color: #333;
  font-size: 14px;
}
</style>
