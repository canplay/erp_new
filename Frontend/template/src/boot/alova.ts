import { defineBoot } from '#q-app';
import { createAlova } from 'alova';
import VueHook from 'alova/vue';
import adapterFetch from 'alova/fetch';

const alova = createAlova({
  requestAdapter: adapterFetch(),
  statesHook: VueHook,
});

export default defineBoot(({ app }) => {
  app.config.globalProperties.$alova = alova;
});
