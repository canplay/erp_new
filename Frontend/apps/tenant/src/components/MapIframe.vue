<template>
  <div class="map-container fit">
    <iframe
      ref="mapIframe"
      class="fit"
      src="/map.html"
      frameborder="0"
      seamless
      @message="onMessage"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * @file MapIframe.vue
 * @description 封装 map.html 的 iframe 组件，提供 window.postMessage 通信
 */

import { ref, defineExpose as vueDefineExpose } from 'vue';

interface MapApi {
  clear?: () => void;
  setCar?: (...args: unknown[]) => void;
  setCenter?: (...args: unknown[]) => void;
  setPolygon?: (data: Array<Record<string, unknown>>) => void;
  setText?: (data: Array<Record<string, unknown>>) => void;
}

const mapIframe = ref<MapApi | null>(null);

const onMessage = (event: MessageEvent) => {
  // 透传消息到父组件
  emit('message', event);
};

// 暴露方法供父组件调用
const clear = () => {
  mapIframe.value?.clear?.();
};

const setCar = (...args: unknown[]) => {
  mapIframe.value?.setCar?.(...args);
};

const setCenter = (lng: number, lat: number, zoom: number) => {
  mapIframe.value?.setCenter?.(lng, lat, zoom);
};

const setPolygon = (data: Array<Record<string, unknown>>) => {
  mapIframe.value?.setPolygon?.(data);
};

const setText = (data: Array<Record<string, unknown>>) => {
  mapIframe.value?.setText?.(data);
};

vueDefineExpose({
  clear,
  setCar,
  setCenter,
  setPolygon,
  setText,
});

const emit = defineEmits<{
  message: [event: MessageEvent];
}>();
</script>

<style scoped>
.map-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.map-container iframe {
  width: 100%;
  height: 100%;
}
</style>
