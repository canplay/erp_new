import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import { useI18n } from 'vue-i18n'

interface Announcement {
  id: string | number
  title: string
  content: string
  status: string
  created_at: string
}

export function useAnnouncement() {
  const $q = useQuasar()
  const { t: $t } = useI18n()

  const announcements = ref<Announcement[]>([])
  const loading = ref(false)

  async function loadAnnouncements(): Promise<void> {
    loading.value = true
    try {
      // 模拟获取数据
      await new Promise(resolve => setTimeout(resolve, 500))
      announcements.value = []
    } finally {
      loading.value = false
    }
  }

  function openDialog(announcement?: Announcement): void {
    // 触发打开对话框
  }

  async function handleSave(): Promise<void> {
    $q.notify({ type: 'positive', message: $t('announcement.saveSuccess') })
    await loadAnnouncements()
  }

  async function handleDelete(_announcement: Announcement): Promise<void> {
    $q.notify({ type: 'info', message: $t('announcement.deleteSuccess') })
    await loadAnnouncements()
  }

  onMounted(() => {
    void loadAnnouncements()
  })

  return {
    announcements,
    loading,
    openDialog,
    handleSave,
    handleDelete
  }
}
