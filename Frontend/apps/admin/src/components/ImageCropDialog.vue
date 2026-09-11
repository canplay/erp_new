/**
 * @file ImageCropDialog.vue
 * @description 图片裁剪弹窗组件 - 支持缩放、旋转
 * @date 2026-08-22
 */

<template>
  <q-dialog v-model="props.modelValue" persistent>
    <q-card style="min-width: 600px; max-width: 90vw">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('uploader.cropImage') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="close" />
      </q-card-section>

      <q-separator />

      <q-card-section class="crop-container">
        <img ref="cropImageRef" :src="sourceUrl" class="crop-image" />
      </q-card-section>

      <q-separator />

      <q-card-section>
        <!-- 缩放控制 -->
        <div class="row items-center q-mb-md">
          <q-icon name="zoom_out" size="20px" class="q-mr-sm" />
          <q-slider
            v-model="localScale"
            :min="0.5"
            :max="2"
            :step="0.1"
            label
            class="col"
            @update:model-value="onScaleChange"
          />
          <q-icon name="zoom_in" size="20px" class="q-ml-sm" />
          <span class="q-ml-md text-caption">{{ Math.round(localScale * 100) }}%</span>
        </div>

        <!-- 旋转控制 -->
        <div class="row items-center">
          <q-icon name="rotate_left" size="20px" class="q-mr-sm cursor-pointer" @click="rotate(-90)" />
          <q-slider
            v-model="localRotate"
            :min="-180"
            :max="180"
            :step="1"
            label
            class="col"
            @update:model-value="onRotateChange"
          />
          <q-icon name="rotate_right" size="20px" class="q-ml-sm cursor-pointer" @click="rotate(90)" />
          <span class="q-ml-md text-caption">{{ localRotate }}°</span>
        </div>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" color="grey" @click="close" />
        <q-btn color="primary" :label="$t('common.confirm')" @click="apply" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

interface Props {
  /** 是否显示 */
  modelValue: boolean;
  /** 裁剪源图片 URL */
  sourceUrl: string;
  /** 缩放比例 */
  scale: number;
  /** 旋转角度 */
  rotate: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'update:scale', value: number): void;
  (e: 'update:rotate', value: number): void;
  (e: 'apply'): void;
}>();

const cropImageRef = ref<HTMLImageElement | null>(null);
const localScale = ref(props.scale);
const localRotate = ref(props.rotate);

watch(() => props.scale, (val) => { localScale.value = val; });
watch(() => props.rotate, (val) => { localRotate.value = val; });

function close() {
  emit('update:modelValue', false);
}

function apply() {
  emit('apply');
}

function onScaleChange() {
  updateCrop();
  emit('update:scale', localScale.value);
}

function onRotateChange() {
  updateCrop();
  emit('update:rotate', localRotate.value);
}

function rotate(degrees: number) {
  localRotate.value = (localRotate.value + degrees) % 360;
  updateCrop();
  emit('update:rotate', localRotate.value);
}

function updateCrop() {
  if (cropImageRef.value) {
    cropImageRef.value.style.transform = `scale(${localScale.value}) rotate(${localRotate.value}deg)`;
  }
}
</script>

<style scoped>
.crop-container {
  display: flex;
  justify-content: center;
  max-height: 400px;
  overflow: hidden;
}

.crop-image {
  max-width: 100%;
  max-height: 400px;
  transition: transform 0.2s ease;
}
</style>
