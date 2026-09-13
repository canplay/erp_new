<template>
  <div class="row items-center q-gutter-sm q-mb-md">
    <q-input
      :model-value="search"
      :label="i18nT('raw.s2a09b4')"
      dense
      outlined
      clearable
      debounce="300"
      style="min-width: 260px"
      @update:model-value="onSearchChange"
    >
      <template #prepend>
        <q-icon name="search" />
      </template>
    </q-input>
    <q-select
      :model-value="brandFilter"
      :options="brandOptions"
      :label="i18nT('raw.s27ebf0')"
      dense
      outlined
      emit-value
      map-options
      clearable
      style="min-width: 180px"
      @update:model-value="onBrandChange"
    />
    <q-select
      :model-value="categoryFilter"
      :options="categoryOptions"
      :label="i18nT('raw.sc06012')"
      dense
      outlined
      emit-value
      map-options
      clearable
      style="min-width: 180px"
      @update:model-value="onCategoryChange"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t: i18nT } = useI18n();

defineProps<{
  search: string;
  brandFilter: string | null;
  categoryFilter: string | null;
  brandOptions: { label: string; value: string }[];
  categoryOptions: { label: string; value: string }[];
}>();

const emit = defineEmits<{
  (e: 'update:search', val: string): void;
  (e: 'update:brandFilter', val: string | null): void;
  (e: 'update:categoryFilter', val: string | null): void;
  (e: 'search'): void;
}>();

function onSearchChange(val: string | number | null) {
  emit('update:search', String(val ?? ''));
  emit('search');
}

function onBrandChange(val: string | null) {
  emit('update:brandFilter', val);
  emit('search');
}

function onCategoryChange(val: string | null) {
  emit('update:categoryFilter', val);
  emit('search');
}
</script>
