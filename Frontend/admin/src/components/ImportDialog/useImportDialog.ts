import { ref, computed } from 'vue'
import { useQuasar } from 'quasar'
import { useI18n } from 'vue-i18n'

interface UseImportDialogOptions {
  onImport: (event: 'import', data: unknown[]) => void
  onClose: (event: 'close') => void
}

export function useImportDialog(options: UseImportDialogOptions) {
  const $q = useQuasar()
  const { t: $t } = useI18n()

  const showDialog = ref(false)
  const currentStep = ref(0)
  const selectedFile = ref<File | null>(null)
  const isUploading = ref(false)
  const uploadProgress = ref(0)
  const validateResult = ref<unknown>(null)
  const previewData = ref<unknown[]>([])

  const canNext = computed(() => {
    if (currentStep.value === 0) return !!selectedFile.value && !isUploading.value
    if (currentStep.value === 1) return !!validateResult.value
    return false
  })

  const canImport = computed(() => {
    return previewData.value.length > 0
  })

  function handleFileSelect(file: File): void {
    selectedFile.value = file
  }

  function startUpload(): void {
    if (!selectedFile.value) return

    isUploading.value = true
    uploadProgress.value = 0

    const interval = setInterval(() => {
      uploadProgress.value += 10
      if (uploadProgress.value >= 100) {
        clearInterval(interval)
        isUploading.value = false
        validateResult.value = { success: true, message: '验证成功' }
        previewData.value = [{ id: 1, name: '示例数据' }]
      }
    }, 200)
  }

  function nextStep(): void {
    if (currentStep.value < 2) {
      currentStep.value++
    }
  }

  function prevStep(): void {
    if (currentStep.value > 0) {
      currentStep.value--
    }
  }

  function importData(): void {
    if (!canImport.value) return

    options.onImport('import', [...previewData.value])
    $q.notify({ type: 'positive', message: $t('import.importSuccess') })
    reset()
  }

  function reset(): void {
    currentStep.value = 0
    selectedFile.value = null
    isUploading.value = false
    uploadProgress.value = 0
    validateResult.value = null
    previewData.value = []
    showDialog.value = false
    options.onClose('close')
  }

  return {
    showDialog,
    currentStep,
    selectedFile,
    isUploading,
    uploadProgress,
    validateResult,
    previewData,
    canNext,
    canImport,
    handleFileSelect,
    startUpload,
    nextStep,
    prevStep,
    importData,
    reset
  }
}
