<template>
  <q-dialog v-model="visible" persistent>
    <q-card style="min-width: 500px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ title }}</div>
        <q-space />
        <q-btn icon="close" flat round dense @click="close" />
      </q-card-section>
      <q-card-section>
        <q-input v-model="form.title" :label="$t('announcement.titleField', '标题')" outlined class="q-mb-sm" />
        <q-input v-model="form.content" type="textarea" :label="$t('announcement.content', '内容')" outlined class="q-mb-sm" />
        <q-select v-model="form.status" :options="statusOptions" :label="$t('announcement.status', '状态')" outlined />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel', '取消')" @click="close" />
        <q-btn color="primary" :label="$t('common.save', '保存')" :loading="saving" @click="onSave" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
defineOptions({ name: 'AnnouncementFormDialog' });

import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

interface Announcement {
  id: string | number;
  title: string;
  content: string;
  status: string;
  created_at: string;
}

const props = defineProps<{
  modelValue: boolean;
  editData?: Announcement | null;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void;
  (e: 'submit'): void;
}>();

const $q = useQuasar();
const { t: $t } = useI18n();
const saving = ref(false);

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
});

const title = computed(() =>
  props.editData ? $t('announcement.edit', '编辑公告') : $t('announcement.add', '新增公告')
);

const form = ref({
  title: '',
  content: '',
  status: 'draft',
});

const statusOptions = [
  { label: $t('common.draft', '草稿'), value: 'draft' },
  { label: $t('common.published', '发布'), value: 'published' },
  { label: $t('common.archived', '归档'), value: 'archived' },
];

watch(() => props.editData, (val) => {
  if (val) {
    form.value = { title: val.title, content: val.content, status: val.status };
  } else {
    form.value = { title: '', content: '', status: 'draft' };
  }
}, { immediate: true });

function close() {
  visible.value = false;
}

async function onSave() {
  saving.value = true;
  try {
    // TODO: API call
    $q.notify({ type: 'positive', message: $t('common.saveSuccess', '保存成功') });
    emit('submit');
    close();
  } catch {
    $q.notify({ type: 'negative', message: $t('common.saveFailed', '保存失败') });
  } finally {
    saving.value = false;
  }
}
</script>
