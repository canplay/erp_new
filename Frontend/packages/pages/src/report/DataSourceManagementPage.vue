<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">{{ $t('report.dataSourceManagement') }}</div>
    <q-card>
      <q-card-section>
        <div class="row q-mb-md">
          <q-space />
          <q-btn color="positive" icon="add" :label="$t('common.add')" @click="showDialog = true" />
        </div>
        <q-table :rows="dataSources" :columns="columns" :loading="loading" row-key="id">
          <template #body-cell-dsType="props">
            <q-td :props="props">
              <q-badge :label="getTypeLabel(props.row.dsType)" color="primary" />
            </q-td>
          </template>
          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense color="primary" icon="edit" @click="handleEdit(props.row)">
                <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
              </q-btn>
              <q-btn flat dense color="negative" icon="delete" @click="handleDelete(props.row)">
                <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
              </q-btn>
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? t('report.editDataSource') : t('report.addDataSource') }}</div>
        </q-card-section>
        <q-card-section>
          <q-form>
            <q-input v-model="formData.name" :label="$t('common.name')" outlined :rules="[v => !!v || $t('common.required')]" />
            <q-select
              v-model="formData.dsType"
              :options="typeOptions"
              emit-value
              map-options
              :label="$t('common.type')"
              outlined
              class="q-mt-md"
              :disable="isEdit"
            />
            <div v-if="formData.dsType === 'mysql' || formData.dsType === 'postgresql'" class="q-mt-md">
              <q-input v-model="configHost" :label="$t('report.host')" outlined />
              <q-input v-model="configPort" :label="$t('common.port')" outlined type="number" class="q-mt-sm" />
              <q-input v-model="configDatabase" :label="$t('report.database')" outlined class="q-mt-sm" />
              <q-input v-model="configUsername" :label="$t('common.username')" outlined class="q-mt-sm" />
            </div>
          </q-form>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.save')" @click="handleSubmit" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import {
  listDataSources, createDataSource, updateDataSource, deleteDataSource,
  type DataSource
} from '@/api/report';

const { t } = useI18n();

/**
 * @brief 数据源管理页面
 */
const $q = useQuasar();

const loading = ref(false);
const dataSources = ref<DataSource[]>([]);
const showDialog = ref(false);
const isEdit = ref(false);
const currentDs = ref<DataSource | null>(null);

// 表单数据
const formData = ref({
  name: '',
  dsType: 'mysql',
});

// 配置字段
const configHost = ref('localhost');
const configPort = ref(3306);
const configDatabase = ref('');
const configUsername = ref('');

const columns = [
  { name: 'name', label: t('common.name'), field: 'name', align: 'left' as const },
  { name: 'dsType', label: t('common.type'), field: 'dsType', align: 'center' as const },
  { name: 'created_at', label: t('common.createdAt'), field: 'created_at', align: 'center' as const },
  { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
];

const typeOptions = [
  { label: 'MySQL', value: 'mysql' },
  { label: 'PostgreSQL', value: 'postgresql' },
  { label: 'MongoDB', value: 'mongodb' },
  { label: 'API', value: 'api' },
];

function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    mysql: 'MySQL',
    postgresql: 'PostgreSQL',
    mongodb: 'MongoDB',
    api: 'API',
  };
  return labels[type] || type;
}

function handleEdit(ds: DataSource) {
  isEdit.value = true;
  currentDs.value = ds;
  formData.value = {
    name: ds.name,
    dsType: ds.dsType,
  };
  // 加载配置
  if (ds.config) {
    const cfg = ds.config;
    configHost.value = (cfg.host as string) || 'localhost';
    configPort.value = (cfg.port as number) || 3306;
    configDatabase.value = (cfg.database as string) || '';
    configUsername.value = (cfg.username as string) || '';
  }
  showDialog.value = true;
}

function handleDelete(ds: DataSource): void {
  $q.dialog({
    title: t('common.confirm'),
    message: t('report.confirmDeleteDataSource', { name: ds.name }),
    cancel: true,
  }).onOk(() => {
    void (async () => {
      try {
        await deleteDataSource(ds.id);
        $q.notify({ type: 'positive', message: t('report.dataSourceDeleted') });
        void loadData();
      } catch {
        $q.notify({ type: 'negative', message: t('report.dataSourceDeleteFailed') });
      }
    })();
  });
}

async function handleSubmit() {
  const config = {
    host: configHost.value,
    port: configPort.value,
    database: configDatabase.value,
    username: configUsername.value,
  };

  try {
    if (isEdit.value && currentDs.value) {
      await updateDataSource(currentDs.value.id, { name: formData.value.name, config });
    } else {
      await createDataSource({ name: formData.value.name, dsType: formData.value.dsType as 'mysql' | 'postgresql' | 'http' | 'file', config });
    }
    $q.notify({ type: 'positive', message: t('common.saved') });
    showDialog.value = false;
    void loadData();
  } catch {
    $q.notify({ type: 'negative', message: t('common.saveFailed') });
  }
}

async function loadData() {
  loading.value = true;
  try {
    const response = await listDataSources();
    // 类型断言：兼容新旧格式（已展开的 list/data 格式）
    const respData = response as { list?: DataSource[]; data?: DataSource[] };
    dataSources.value = respData.list || respData.data || [];
  } catch {
    $q.notify({ type: 'negative', message: t('common.loadFailed') });
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void loadData();
});
</script>