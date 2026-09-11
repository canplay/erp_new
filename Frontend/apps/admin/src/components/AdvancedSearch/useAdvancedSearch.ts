import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import { useI18n } from 'vue-i18n'
import { getStorageItem, setStorageItem } from '@/utils/storage'
import { logger } from '@/utils/logger'

interface SearchCondition {
  field: string
  operator: string
  value: unknown
}

interface SavedSearch {
  name: string
  conditions: SearchCondition[]
  created_at: number
}

interface UseAdvancedSearchOptions {
  fieldOptions: Array<{ label: string; value: string; type: string; options?: Array<{ label: string; value: string }> }>
  initialConditions?: SearchCondition[]
  storageKey: string
  onSearch: (event: 'search', conditions: SearchCondition[]) => void
  onReset: (event: 'reset') => void
}

export function useAdvancedSearch(options: UseAdvancedSearchOptions) {
  const $q = useQuasar()
  const { t: _t } = useI18n()

  const conditions = ref<SearchCondition[]>(options.initialConditions?.length ? [...options.initialConditions] : [])
  const savedSearches = ref<SavedSearch[]>(loadSavedSearches())
  const showSaveDialog = ref(false)
  const savedSearchName = ref('')

  function addCondition(): void {
    const firstField = options.fieldOptions[0]
    if (firstField) {
      const operators = getOperatorsForField(firstField.value)
      conditions.value.push({ field: firstField.value, operator: operators[0]?.value || 'eq', value: '' })
    }
  }

  function removeCondition(index: number): void {
    conditions.value.splice(index, 1)
  }

  function updateCondition(index: number, condition: SearchCondition): void {
    conditions.value[index] = condition
  }

  function handleSearch(): void {
    options.onSearch('search', [...conditions.value])
  }

  function handleReset(): void {
    conditions.value = []
    options.onReset('reset')
  }

  function openSaveDialog(): void {
    showSaveDialog.value = true
  }

  function handleSaveSearch(name: string): void {
    if (!name.trim()) {
      $q.notify({ type: 'warning', message: '请输入搜索名称' })
      return
    }
    const search: SavedSearch = { name, conditions: JSON.parse(JSON.stringify(conditions.value)), created_at: Date.now() }
    savedSearches.value.push(search)
    saveSavedSearches()
    showSaveDialog.value = false
    savedSearchName.value = ''
    $q.notify({ type: 'positive', message: '搜索条件已保存' })
  }

  function loadSearch(search: SavedSearch): void {
    conditions.value = JSON.parse(JSON.stringify(search.conditions))
    handleSearch()
  }

  function removeSearch(index: number): void {
    savedSearches.value.splice(index, 1)
    saveSavedSearches()
  }

  function getOperatorsForField(field: string): Array<{ label: string; value: string }> {
    const fieldInfo = options.fieldOptions.find(f => f.value === field)
    const type = fieldInfo?.type || 'string'
    const operatorMap: Record<string, Array<{ label: string; value: string }>> = {
      string: [{ label: '等于', value: 'eq' }, { label: '包含', value: 'contains' }],
      number: [{ label: '等于', value: 'eq' }, { label: '大于', value: 'gt' }],
      date: [{ label: '等于', value: 'eq' }, { label: '早于', value: 'lt' }],
      select: [{ label: '等于', value: 'eq' }],
      boolean: [{ label: '是', value: 'eq' }]
    }
    const ops = operatorMap[type];
    return (ops ?? operatorMap.string) as Array<{ label: string; value: string }>
  }

  function loadSavedSearches(): SavedSearch[] {
    try {
      const saved = getStorageItem<string>(options.storageKey, '')
      return saved ? JSON.parse(saved) : []
    } catch (error) {
      logger.warn('【加载保存的搜索失败】', error)
      return []
    }
  }

  function saveSavedSearches(): void {
    try {
      setStorageItem(options.storageKey, JSON.stringify(savedSearches.value))
    } catch (e) {
      console.error('Failed to save searches:', e)
    }
  }

  return {
    conditions,
    savedSearches,
    showSaveDialog,
    savedSearchName,
    addCondition,
    removeCondition,
    updateCondition,
    handleSearch,
    handleReset,
    openSaveDialog,
    handleSaveSearch,
    loadSearch,
    removeSearch
  }
}
