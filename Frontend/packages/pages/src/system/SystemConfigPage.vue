<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('systemConfig.title') }}</div>

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row q-col-gutter-md items-center">
        <div class="col-12 col-sm-4">
          <q-btn-toggle
            v-model="currentCategory"
            spread
            no-caps
            toggle-color="primary"
            :options="categoryOptions"
            @update:model-value="handleCategoryChange"
          />
        </div>
        <div class="col-12 col-sm-8 text-right">
          <q-btn
            flat
            color="primary"
            :label="$t('systemConfig.resetAll')"
            icon="restore"
            @click="handleResetAll"
          />
          <q-btn
            color="primary"
            :label="$t('common.save')"
            icon="save"
            :loading="saving"
            @click="handleSaveAll"
          />
        </div>
      </q-card-section>
    </q-card>

    <!-- 配置卡片 -->
    <q-card v-for="config in currentConfigs" :key="config.key" class="q-mb-md" bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon :name="getConfigIcon(config.type)" class="q-mr-sm" />
          {{ config.label }}
        </div>
        <div v-if="config.description" class="text-caption text-grey-6 q-mb-md">
          {{ config.description }}
        </div>

        <!-- 字符串输入 -->
        <q-input
          v-if="config.type === 'string'"
          v-model="configForm[config.key]"
          outlined
          dense
          :type="config.key.includes('password') ? 'password' : 'text'"
          @update:model-value="handleFieldChange(config.key, $event)"
        />

        <!-- 数字输入 -->
        <q-input
          v-else-if="config.type === 'number'"
          v-model.number="configForm[config.key]"
          outlined
          dense
          type="number"
          @update:model-value="handleFieldChange(config.key, $event)"
        />

        <!-- 布尔切换 -->
        <q-toggle
          v-else-if="config.type === 'boolean'"
          v-model="configForm[config.key]"
          :label="configForm[config.key] ? $t('systemConfig.enabled') : $t('systemConfig.disabled')"
          @update:model-value="handleFieldChange(config.key, $event)"
        />

        <!-- JSON 编辑 -->
        <q-input
          v-else-if="config.type === 'json'"
          v-model="configForm[config.key]"
          outlined
          dense
          type="textarea"
          rows="4"
          @update:model-value="handleFieldChange(config.key, $event)"
        />

        <div class="text-caption text-grey-7 q-mt-sm">
          {{ $t('systemConfig.configKey') }}: {{ config.key }}
        </div>
      </q-card-section>
    </q-card>

    <!-- 空状态 -->
    <q-card v-if="currentConfigs.length === 0" class="q-mb-md" bordered>
      <q-card-section class="text-center q-pa-xl">
        <q-icon name="settings" size="64px" color="grey" />
        <div class="text-grey-6 q-mt-md">{{ $t('systemConfig.noConfigs') }}</div>
      </q-card-section>
    </q-card>

    <!-- 提示信息 -->
    <q-banner v-if="hasChanges" class="q-mt-md" rounded style="background: #fff3e0;">
      <template v-slot:avatar>
        <q-icon name="info" color="warning" />
      </template>
      {{ $t('systemConfig.unsavedChanges') }}
      <template v-slot:action>
        <q-btn flat color="warning" :label="$t('common.save')" @click="handleSaveAll" />
        <q-btn flat color="grey" :label="$t('common.cancel')" @click="handleDiscardChanges" />
      </template>
    </q-banner>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import {
  getSystemConfigs,
  getConfigCategories,
  batchUpdateConfigs,
  type SystemConfig,
} from '@/api/system';

const { t } = useI18n();
const $q = useQuasar();

// ============ 状态 ============

const configs = ref<SystemConfig[]>([]);
const currentCategory = ref('basic');
const configForm = reactive<Record<string, string | number | null>>({});
const originalValues = ref<Record<string, string | number | null>>({});
const saving = ref(false);

// 配置分类
const categories = getConfigCategories();

// 分类选项
const categoryOptions = computed(() =>
  categories.map((c) => ({ label: c.label, value: c.name }))
);

// 当前分类的配置
const currentConfigs = computed(() =>
  configs.value.filter((c) => c.category === currentCategory.value)
);

// 是否有未保存的更改
const hasChanges = computed(() => {
  return Object.keys(configForm).some((key) => {
    const formVal = configForm[key];
    const origVal = originalValues.value[key];
    return formVal !== origVal;
  });
});

// ============ 方法 ============

/**
 * @brief 获取配置图标
 */
function getConfigIcon(type: string): string {
  const icons: Record<string, string> = {
    string: 'text_fields',
    number: 'numbers',
    boolean: 'toggle_on',
    json: 'code',
  };
  return icons[type] || 'settings';
}

/**
 * @brief 处理字段变化
 */
function handleFieldChange(key: string, value: unknown) {
  if (typeof value === 'boolean') {
    configForm[key] = value ? 1 : 0;
  } else {
    configForm[key] = value as string | number | null;
  }
}

/**
 * @brief 处理分类切换
 */
function handleCategoryChange() {
  if (hasChanges.value) {
    void handleSaveAll();
  }
}

/**
 * @brief 加载配置
 */
  async function loadConfigs() {
    try {
      const response = await getSystemConfigs();
      // 类型断言：兼容新旧格式（已展开的 list/data 格式）
      const respData = response as { data?: { list?: SystemConfig[] } };
      const configList = respData.data?.list || [];
      configs.value = configList;

      configList.forEach((config: SystemConfig) => {
      let value: string | number | null = config.value;

      if (config.type === 'number') {
        value = Number(config.value) || 0;
      } else if (config.type === 'boolean') {
        value = config.value === 'true' || config.value === '1' ? 1 : 0;
      }

      configForm[config.key] = value;
      originalValues.value[config.key] = value;
    });
  } catch (error) {
    console.error('【加载配置失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
  }
}

/**
 * @brief 保存所有更改
 */
async function handleSaveAll() {
  if (!hasChanges.value) return;

  saving.value = true;
  try {
    const changedConfigs = Object.keys(configForm)
      .filter((key) => {
        const formVal = configForm[key];
        const origVal = originalValues.value[key];
        return formVal !== origVal;
      })
      .map((key) => ({
        key,
        value: String(configForm[key] ?? ''),
      }));

    if (changedConfigs.length > 0) {
      await batchUpdateConfigs(changedConfigs);
    }

    Object.assign(originalValues, configForm);
    $q.notify({ type: 'positive', message: t('systemConfig.saveSuccess') });
  } catch (error) {
    console.error('【保存配置失败】', error);
    $q.notify({ type: 'negative', message: t('common.error') });
  } finally {
    saving.value = false;
  }
}

/**
 * @brief 放弃更改
 */
function handleDiscardChanges() {
  Object.keys(configForm).forEach((key) => {
    const origVal = originalValues.value[key];
    configForm[key] = origVal !== undefined ? origVal : null;
  });
}

/**
 * @brief 重置所有配置
 */
function handleResetAll() {
  $q.dialog({
    title: t('common.confirm'),
    message: t('systemConfig.resetConfirm'),
    cancel: true,
    persistent: true,
  }).onOk(() => {
    // 使用 Promise 链式处理，避免 async/await 在回调中的问题
    const resetConfig = (config: SystemConfig) =>
      batchUpdateConfigs([{ key: config.key, value: '' }]);

    void Promise.all(currentConfigs.value.map(resetConfig))
      .then(() => {
        $q.notify({ type: 'positive', message: t('systemConfig.resetSuccess') });
        void loadConfigs();
      })
      .catch((error) => {
        console.error('【重置配置失败】', error);
        $q.notify({ type: 'negative', message: t('common.error') });
      });
  });
}

// ============ 生命周期 ============

onMounted(() => {
  void loadConfigs();
});
</script>
