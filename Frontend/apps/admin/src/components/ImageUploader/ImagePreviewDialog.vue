<template>
  <div class="image-preview-dialog">
    <q-dialog v-model="showDialog" persistent>
      <q-card class="image-preview-dialog__card">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">{{ $t('image.preview') }}</div>
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section class="image-preview-dialog__content">
          <q-img
            :src="props.images[currentIndex]?.url"
            contain
            class="image-preview-dialog__image"
          />
        </q-card-section>

        <q-card-actions align="center">
          <q-btn
            flat
            icon="chevron_left"
            :disable="currentIndex === 0"
            @click="prevImage"
          />
          <span class="text-caption">{{ currentIndex + 1 }} / {{ images.length }}</span>
          <q-btn
            flat
            icon="chevron_right"
            :disable="currentIndex === images.length - 1"
            @click="nextImage"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface ImageItem {
  url: string
}

interface Props {
  images: ImageItem[]
  modelValue: boolean
  initialIndex?: number
}

const props = withDefaults(defineProps<Props>(), {
  initialIndex: 0
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'close'): void
}>()

const showDialog = ref(false)
const currentIndex = ref(props.initialIndex)

watch(() => props.modelValue, (val) => {
  showDialog.value = val
  if (val) {
    currentIndex.value = props.initialIndex
  }
})

watch(showDialog, (val) => {
  if (!val) {
    emit('close')
  }
})

function prevImage(): void {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

function nextImage(): void {
  if (currentIndex.value < props.images.length - 1) {
    currentIndex.value++
  }
}
</script>

<style scoped>
.image-preview-dialog__card {
  width: 80vw;
  max-width: 800px;
}

.image-preview-dialog__content {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.image-preview-dialog__image {
  max-height: 70vh;
  border-radius: 8px;
}
</style>
