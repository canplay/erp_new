<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">发布向导</div>
    <q-stepper v-model="step" color="primary" animated>
      <q-step :name="1" title="选择内容" icon="article" :done="step > 1">
        <q-select v-model="selContent" :options="contents" option-label="title" option-value="id" label="内容" outlined use-input @filter="filterContent" />
      </q-step>
      <q-step :name="2" title="选择账号" icon="alternate_email" :done="step > 2">
        <q-select v-model="selAccount" :options="accounts" option-label="account_name" option-value="id" label="发布账号" outlined />
      </q-step>
      <q-step :name="3" title="确认发布" icon="publish">
        <div>即将发布: {{ selContent?.title }} → {{ selAccount?.account_name }}</div>
      </q-step>
      <template v-slot:navigation>
        <q-stepper-navigation>
          <q-btn v-if="step < 3" color="primary" label="下一步" @click="step++" :disable="step === 1 ? !selContent : !selAccount" />
          <q-btn v-if="step === 3" color="green" label="立即发布" @click="doPublish" :loading="loading" />
          <q-btn v-if="step > 1" flat color="primary" label="上一步" @click="step--" class="q-ml-sm" />
        </q-stepper-navigation>
      </template>
    </q-stepper>
  </q-page>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { contentApi } from '@/api/contents';
import { accountApi, type SocialAccount } from '@/api/accounts';
import { publishApi } from '@/api/publish';
const $q = useQuasar();
const step = ref(1);
const contents = ref<Array<Record<string, unknown>>>([]);
const allContents = ref<Array<Record<string, unknown>>>([]);
const selContent = ref<Record<string, unknown> | null>(null);
const accounts = ref<SocialAccount[]>([]);
const selAccount = ref<Record<string, unknown> | null>(null);
const loading = ref(false);
function filterContent(val: string, update: (fn: () => void) => void) { update(() => { contents.value = allContents.value.filter((c) => String(c.title).includes(val)); }); }
async function doPublish() {
  loading.value = true;
  try {
    await publishApi.publish({ content_id: selContent.value?.id, account_id: selAccount.value?.id });
    $q.notify({ type: 'positive', message: '发布成功！' });
    step.value = 1; selContent.value = null; selAccount.value = null;
  } catch {
    $q.notify({ type: 'negative', message: '发布失败' });
  }
  loading.value = false;
}
onMounted(async () => { try { allContents.value = (await contentApi.list()) as unknown as Array<Record<string, unknown>>; contents.value = [...allContents.value]; accounts.value = await accountApi.list(); } catch { void 0; } });
</script>
