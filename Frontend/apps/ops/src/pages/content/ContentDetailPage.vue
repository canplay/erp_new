<template><q-page class="q-pa-md"><q-btn flat icon="arrow_back" label="返回" @click="$router.back()" class="q-mb-md" /><div v-if="item"><div class="text-h4">{{ item.title }}</div><q-separator class="q-my-md" /><div style="white-space:pre-wrap">{{ item.body }}</div></div><div v-else class="text-center q-py-xl"><q-spinner size="40px" /></div></q-page></template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useContentStore } from '@/stores/content';
import type { ContentItem } from '@/api/contents';
const route = useRoute();
const contentStore = useContentStore();
const item = ref<ContentItem | null>(null);
onMounted(async () => {
  const id = (route.params as Record<string, string>).id ?? '';
  const found = contentStore.contents.find((c) => c.id === id);
  if (found) { item.value = found; return; }
  try { await contentStore.fetchDetail(id); item.value = contentStore.current; } catch { void 0; }
});
</script>
