import { ref, computed } from 'vue'
import { useQuasar } from 'quasar'

interface ImageItem {
  file: File
  url: string
  name: string
  size: number
}

interface UseImageUploaderOptions {
  maxSize?: number | undefined
  maxCount?: number | undefined
  onUpload: (event: 'upload', images: ImageItem[]) => void
  onError: (event: 'error', message: string) => void
}

export function useImageUploader(options: UseImageUploaderOptions) {
  const $q = useQuasar()

  const fileInput = ref<HTMLInputElement | null>(null)
  const images = ref<ImageItem[]>([])
  const isUploading = ref(false)
  const uploadProgress = ref(0)

  const maxSize = options.maxSize || 5 * 1024 * 1024
  const maxCount = options.maxCount || 10

  function triggerFileInput(): void {
    fileInput.value?.click()
  }

  function handleFileSelect(event: Event): void {
    const target = event.target as HTMLInputElement
    const files = target.files
    if (!files) return

    const newImages: ImageItem[] = []

    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      if (!file) continue

      if (!file.type.startsWith('image/')) {
        $q.notify({ type: 'warning', message: '请选择图片文件' })
        continue
      }

      if (file.size > maxSize) {
        $q.notify({ type: 'warning', message: '图片大小不能超过 5MB' })
        continue
      }

      if (images.value.length + newImages.length >= maxCount) {
        $q.notify({ type: 'warning', message: `最多上传 ${maxCount} 张图片` })
        break
      }

      newImages.push({
        file,
        url: URL.createObjectURL(file),
        name: file.name,
        size: file.size
      })
    }

    images.value.push(...newImages)
    target.value = ''
  }

  function removeImage(index: number): void {
    images.value.splice(index, 1)
  }

  function clearAll(): void {
    images.value.forEach(img => URL.revokeObjectURL(img.url))
    images.value = []
    uploadProgress.value = 0
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
  }

  async function uploadImages(): Promise<void> {
    if (images.value.length === 0 || isUploading.value) return

    isUploading.value = true
    uploadProgress.value = 0

    const interval = setInterval(() => {
      uploadProgress.value += 10
      if (uploadProgress.value >= 100) {
        clearInterval(interval)
        isUploading.value = false
        options.onUpload('upload', [...images.value])
        images.value = []
        uploadProgress.value = 0
      }
    }, 200)
  }

  function openPreview(index: number): void {
    console.log('Open preview:', index)
  }

  return {
    fileInput,
    images,
    isUploading,
    uploadProgress,
    triggerFileInput,
    handleFileSelect,
    removeImage,
    clearAll,
    formatFileSize,
    uploadImages,
    openPreview
  }
}
