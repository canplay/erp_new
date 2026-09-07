import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

interface Rule {
  conditionField: string
  conditionOperator: string
  conditionValue: string
}

interface Permissions {
  read: string[]
  write: string[]
}

interface UseRowPermissionOptions {
  dataSources: Array<{ label: string; value: string }>
  conditionOptions: Array<{ label: string; value: string }>
  operatorOptions: Array<{ label: string; value: string }>
  userOptions: Array<{ label: string; value: string }>
  initialConfig: Record<string, unknown>
}

export function useRowPermission(options: UseRowPermissionOptions) {
  const { t: $t } = useI18n()

  const currentSource = ref<string>('')
  const currentRules = ref<Rule[]>([])
  const currentPermissions = ref<Permissions>({ read: [], write: [] })

  const sourceOptions = computed(() => options.dataSources.length > 0 ? options.dataSources : [
    { label: $t('permission.user'), value: 'user' },
    { label: $t('permission.department'), value: 'department' },
    { label: $t('permission.role'), value: 'role' }
  ])

  const conditionOptions = computed(() => options.conditionOptions.length > 0 ? options.conditionOptions : [
    { label: $t('permission.createdBy'), value: 'created_by' },
    { label: $t('permission.department'), value: 'department' },
    { label: $t('permission.status'), value: 'status' }
  ])

  const operatorOptions = computed(() => options.operatorOptions.length > 0 ? options.operatorOptions : [
    { label: $t('permission.equals'), value: 'eq' },
    { label: $t('permission.notEqual'), value: 'ne' },
    { label: $t('permission.contains'), value: 'contains' }
  ])

  function onSourceChange(): void {
    const key = `row_permission_${currentSource.value}`
    const saved = options.initialConfig[key] as Record<string, unknown> | undefined
    if (saved) {
      currentRules.value = (saved.rules as Rule[]) || []
      currentPermissions.value = {
        read: (saved.read as string[]) || [],
        write: (saved.write as string[]) || []
      }
    } else {
      currentRules.value = []
      currentPermissions.value = { read: [], write: [] }
    }
  }

  function addRule(): void {
    currentRules.value.push({
      conditionField: '',
      conditionOperator: '',
      conditionValue: ''
    })
  }

  function removeRule(index: number): void {
    currentRules.value.splice(index, 1)
  }

  function editRule(index: number): void {
    // 可以在这里添加编辑逻辑，如打开对话框
    console.log('Edit rule:', index)
  }

  function saveConfig(): void {
    const config = {
      source: currentSource.value,
      rules: currentRules.value,
      permissions: currentPermissions.value
    }
    // 这里应该触发保存事件或调用API
    return config as any
  }

  function resetConfig(): void {
    currentSource.value = ''
    currentRules.value = []
    currentPermissions.value = { read: [], write: [] }
  }

  return {
    currentSource,
    currentRules,
    currentPermissions,
    sourceOptions,
    conditionOptions,
    operatorOptions,
    addRule,
    removeRule,
    editRule,
    onSourceChange,
    saveConfig,
    resetConfig
  }
}
