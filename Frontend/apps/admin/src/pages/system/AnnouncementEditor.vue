/**
 * @file system/AnnouncementEditor.vue
 * @description 公告编辑 - 表单、保存、取消
 * @date 2026-08-22
 */

<template>
  <q-dialog v-model="dialogVisible" persistent>
    <q-card style="min-width: 700px; max-width: 900px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ isEdit ? editTitle : addTitle }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="dialogVisible = false" />
      </q-card-section>

      <q-separator />

      <q-card-section>
        <q-form class="q-gutter-md">
          <q-input
            v-model="form.title"
            :label="titleLabel"
            outlined
            :rules="[(val) => !!val || requiredMessage]"
          />

          <div class="row q-col-gutter-md">
            <div class="col-12 col-sm-6">
              <q-select
                v-model="form.type"
                :options="typeOptions"
                :label="typeLabel"
                outlined
                emit-value
                map-options
              />
            </div>
            <div class="col-12 col-sm-6">
              <q-input
                v-model.number="form.priority"
                :label="priorityLabel"
                outlined
                type="number"
                min="0"
              />
            </div>
          </div>

          <div>
            <div class="text-caption q-mb-sm">{{ contentLabel }}</div>
            <q-input
              v-model="form.content"
              outlined
              type="textarea"
              rows="6"
              :placeholder="contentPlaceholder"
            />
          </div>

          <div class="row q-col-gutter-md">
            <div class="col-12 col-sm-6">
              <q-input
                v-model="form.start_time"
                :label="startTimeLabel"
                outlined
                type="datetime-local"
              />
            </div>
            <div class="col-12 col-sm-6">
              <q-input
                v-model="form.end_time"
                :label="endTimeLabel"
                outlined
                type="datetime-local"
              />
            </div>
          </div>

          <div class="row q-col-gutter-md">
            <div class="col-auto">
              <q-toggle v-model="form.isPinned" :label="pinnedLabel" />
            </div>
            <div class="col-auto">
              <q-toggle v-model="form.isActive" :label="activeLabel" />
            </div>
          </div>
        </q-form>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="cancelLabel" color="grey" @click="dialogVisible = false" />
        <q-btn color="primary" :label="saveLabel" @click="$emit('save', form)" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface AnnouncementForm {
  title: string;
  content: string;
  type: string;
  priority: number;
  isPinned: boolean;
  isActive: boolean;
  start_time: string;
  end_time: string;
}

interface Props {
  modelValue: boolean;
  isEdit: boolean;
  form: AnnouncementForm;
  typeOptions: { label: string; value: string }[];
  editTitle?: string;
  addTitle?: string;
  titleLabel?: string;
  requiredMessage?: string;
  typeLabel?: string;
  priorityLabel?: string;
  contentLabel?: string;
  contentPlaceholder?: string;
  startTimeLabel?: string;
  endTimeLabel?: string;
  pinnedLabel?: string;
  activeLabel?: string;
  cancelLabel?: string;
  saveLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  isEdit: false,
  editTitle: '编辑',
  addTitle: '添加',
  titleLabel: '标题',
  requiredMessage: '必填',
  typeLabel: '类型',
  priorityLabel: '优先级',
  contentLabel: '内容',
  contentPlaceholder: '请输入内容',
  startTimeLabel: '开始时间',
  endTimeLabel: '结束时间',
  pinnedLabel: '置顶',
  activeLabel: '激活',
  cancelLabel: '取消',
  saveLabel: '保存',
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'save': [form: AnnouncementForm];
}>();

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
});
</script>
