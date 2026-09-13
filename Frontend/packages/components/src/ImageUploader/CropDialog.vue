/**
 * @file CropDialog.vue
 * @description 图片裁剪对话框 - 缩放、旋转、应用裁剪
 * @date 2026-04-03
 */

<template>
  <q-dialog v-model="visible" persistent>
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
            v-model="scale"
            :min="0.5"
            :max="2"
            :step="0.1"
            label
            class="col"
            @update:model-value="updateCrop"
          />
          <q-icon name="zoom_in" size="20px" class="q-ml-sm" />
          <span class="q-ml-md text-caption">{{ Math.round(scale * 100) }}%</span>
        </div>

        <!-- 旋转控制 -->
        <div class="row items-center">
          <q-icon name="rotate_left" size="20px" class="q-mr-sm cursor-pointer" @click="rotateCrop(-90)" />
          <q-slider
            v-model="rotate"
            :min="-180"
            :max="180"
            :step="1"
            label
            class="col"
            @update:model-value="updateCrop"
          />
          <q-icon name="rotate_right" size="20px" class="q-ml-sm cursor-pointer" @click="rotateCrop(90)" />
          <span class="q-ml-md text-caption">{{ rotate }}°</span>
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
import { ref } from 'vue';

interface Props {
  visible: boolean;
  sourceUrl: string;
  quality?: number;
}

const props = withDefaults(defineProps<Props>(), {
  quality: 0.8,
});

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'apply', blob: Blob, fileName: string): void;
}>();

const cropImageRef = ref<HTMLImageElement | null>(null);
const scale = ref(1);
const rotate = ref(0);

function updateCrop() {
  if (cropImageRef.value) {
    cropImageRef.value.style.transform = `scale(${scale.value}) rotate(${rotate.value}deg)`;
  }
}

function rotateCrop(degrees: number) {
  rotate.value = (rotate.value + degrees) % 360;
  updateCrop();
}

function close() {
  emit('update:visible', false);
}

function apply() {
  if (!cropImageRef.value) return;

  try {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    const img = cropImageRef.value;

    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;

    ctx?.save();
    ctx?.translate(canvas.width / 2, canvas.height / 2);
    ctx?.rotate((rotate.value * Math.PI) / 180);
    ctx?.scale(scale.value, scale.value);
    ctx?.drawImage(img, -img.naturalWidth / 2, -img.naturalHeight / 2);
    ctx?.restore();

    canvas.toBlob(
      (blob) => {
        if (blob) {
          emit('apply', blob, 'cropped.jpg');
        }
      },
      'image/jpeg',
      props.quality
    );
  } catch (error) {
    console.error('【裁剪失败】', error);
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
