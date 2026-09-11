<template>
  <div class="row q-mb-md">
    <div class="col-12 col-sm-4">
      <q-select
        v-model="selectedEntityType"
        :options="entityTypeOptions"
        outlined
        dense
        :label="$t('common.selectEntityType')"
        emit-value
        map-options
      />
    </div>
    <div class="col-12 col-sm-4 q-px-sm">
      <q-select
        v-model="selectedRole"
        :options="roleOptions"
        outlined
        dense
        :label="$t('common.selectRole')"
        emit-value
        map-options
        clearable
      />
    </div>
    <div class="col-12 col-sm-4">
      <q-input
        v-model="searchQuery"
        outlined
        dense
        placeholder="搜索字段..."
      >
        <template v-slot:prepend>
          <q-icon name="search" />
        </template>
      </q-input>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  selectedEntityType: string;
  selectedRole: string | null;
  searchQuery: string;
  entityTypeOptions: Array<{ label: string; value: string }>;
  roleOptions: Array<{ label: string; value: string }>;
}>();

const emit = defineEmits<{
  'update:selectedEntityType': [value: string];
  'update:selectedRole': [value: string | null];
  'update:searchQuery': [value: string];
}>();
</script>
