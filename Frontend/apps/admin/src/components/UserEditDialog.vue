<template>
  <q-dialog :model-value="modelValue" persistent @update:model-value="$emit('update:modelValue', $event)">
    <q-card style="min-width: 500px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ isEdit ? $t('user.editUser') : $t('user.addUser') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="$emit('update:modelValue', false)" />
      </q-card-section>

      <q-separator />

      <q-card-section>
        <q-form class="q-gutter-md">
          <q-input
            :model-value="form.username"
            :label="$t('user.username')"
            outlined
            :disable="isEdit"
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('user.username') })]"
            @update:model-value="(val: unknown) => $emit('update:username', (val ?? '') as string)"
          />
          <q-input
            v-if="!isEdit"
            :model-value="form.password"
            :label="$t('user.password')"
            outlined
            type="password"
            :rules="[(val: string) => !!val || $t('validation.required', { field: $t('user.password') })]"
            @update:model-value="(val: unknown) => $emit('update:password', (val ?? '') as string)"
          />
          <q-input
            :model-value="form.nickname"
            :label="$t('user.nickname')"
            outlined
            @update:model-value="(val: unknown) => $emit('update:nickname', (val ?? '') as string)"
          />
          <q-input
            :model-value="form.email"
            :label="$t('user.email')"
            outlined
            type="email"
            :rules="[(val: string) => !val || /.+@.+\..+/.test(val) || $t('validation.email')]"
            @update:model-value="(val: unknown) => $emit('update:email', (val ?? '') as string)"
          />
          <q-input
            :model-value="form.phone"
            :label="$t('user.phone')"
            outlined
            type="tel"
            @update:model-value="(val: unknown) => $emit('update:phone', (val ?? '') as string)"
          />
          <q-select
            :model-value="form.role"
            :options="roleOptions"
            :label="$t('user.role')"
            outlined
            emit-value
            map-options
            @update:model-value="(val: unknown) => $emit('update:role', (val ?? 'user') as string)"
          />
          <q-select
            :model-value="form.status"
            :options="statusOptions"
            :label="$t('user.status')"
            outlined
            emit-value
            map-options
            @update:model-value="(val: unknown) => $emit('update:status', (val ?? 1) as number)"
          />
        </q-form>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" @click="$emit('update:modelValue', false)" />
        <q-btn color="primary" :label="$t('common.save')" @click="$emit('save')" data-testid="save-user-button" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';


interface UserEditForm {
  username: string;
  password: string;
  nickname: string;
  email: string;
  phone: string;
  role: string;
  status: number;
}

defineProps<{
  modelValue: boolean;
  isEdit: boolean;
  form: UserEditForm;
}>();

defineEmits<{
  'update:modelValue': [value: boolean];
  'update:username': [value: string];
  'update:password': [value: string];
  'update:nickname': [value: string];
  'update:email': [value: string];
  'update:phone': [value: string];
  'update:role': [value: string];
  'update:status': [value: number];
  save: [];
}>();

const { t } = useI18n();
const roleOptions = computed(() => [
  { label: t('user.admin'), value: 'admin' },
  { label: t('user.vip'), value: 'vip' },
  { label: t('user.normalUser'), value: 'user' },
]);

const statusOptions = computed(() => [
  { label: t('user.active'), value: 1 },
  { label: t('user.inactive'), value: 0 },
  { label: t('user.locked'), value: 2 },
]);
</script>
