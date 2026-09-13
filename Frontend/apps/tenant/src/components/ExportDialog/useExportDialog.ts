import { ref, computed } from 'vue'
import { useQuasar } from 'quasar'
import { useI18n } from 'vue-i18n'

interface UseExportDialogOptions {
  formatOptions: Array<{ label: string; value: string }>
  onExport: (event: 'export', payload: { format: string; fileName: string }) => void
  onClose: (event: 'close') => void
}

export function useExportDialog(options: UseExportDialogOptions) {
  const $q = useQuasar()
  const { t: $t } = useI18n()

  const showDialog = ref(false)
  const selectedFormat = ref('')
  const fileName = ref('')
  const isExporting = ref(false)
  const exportProgress = ref(0)

  const formatOptions = computed(() => options.formatOptions)

  function startExport(): void {
    if (!selectedFormat.value || !fileName.value) return

    isExporting.value = true
    exportProgress.value = 0

    const interval = setInterval(() => {
      exportProgress.value += 10
      if (exportProgress.value >= 100) {
        clearInterval(interval)
        isExporting.value = false
        exportProgress.value = 0

        options.onExport('export', {
          format: selectedFormat.value,
          fileName: fileName.value
        })

        $q.notify({ type: 'positive', message: $t('export.exportSuccess') })
        showDialog.value = false
      }
    }, 200)
  }

  return {
    showDialog,
    selectedFormat,
    fileName,
    isExporting,
    exportProgress,
    formatOptions,
    startExport
  }
}
