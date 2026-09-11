import { defineStore } from 'pinia';
import { ref } from 'vue';
import { llmApi, type LLMProvider } from '@/api/llm';

export const useLlmStore = defineStore('llm', () => {
  const providers = ref<LLMProvider[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchProviders() {
    loading.value = true;
    error.value = null;
    try {
      providers.value = await llmApi.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载LLM提供商失败';
    } finally {
      loading.value = false;
    }
  }

  async function addProvider(data: Partial<LLMProvider>) {
    error.value = null;
    try {
      const result = await llmApi.add(data);
      providers.value.push(result);
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '添加LLM提供商失败';
      throw e;
    }
  }

  return { providers, loading, error, fetchProviders, addProvider };
});
