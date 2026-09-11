import { ref, computed, watch } from 'vue'

interface TreeNode {
  id: string | number
  label: string
  icon?: string
  iconColor?: string
  extra?: string
  children?: TreeNode[]
  disabled?: boolean
  [key: string]: unknown
}

interface UseTreeSelectorOptions {
  data: TreeNode[]
  multiple: boolean
  defaultExpandAll: boolean
  onChange: (event: 'update:value' | 'update:modelValue' | 'change', value: string | number | (string | number)[]) => void
}

export function useTreeSelector(options: UseTreeSelectorOptions) {
  const showDialog = ref(false)
  const internalValue = ref<string | number | (string | number)[]>(options.multiple ? [] : '')

  const selectedLabel = computed(() => {
    if (!internalValue.value) return ''
    if (options.multiple && Array.isArray(internalValue.value)) {
      if (internalValue.value.length === 0) return ''
      if (internalValue.value.length === 1) {
        const node = findNode(options.data, internalValue.value[0] as string | number)
        return node?.label || ''
      }
      return `已选择 ${internalValue.value.length} 项`
    }
    const node = findNode(options.data, internalValue.value as string | number)
    return node?.label || ''
  })

  function findNode(nodes: TreeNode[], id: string | number): TreeNode | null {
    for (const node of nodes) {
      if (node.id === id) return node
      if (node.children) {
        const found = findNode(node.children, id)
        if (found) return found
      }
    }
    return null
  }

  function openDialog(): void {
    showDialog.value = true
  }

  function handleRefresh(): void {
    // 触发刷新
  }

  function handleConfirm(): void {
    showDialog.value = false
  }

  function emitValue(val: string | number | (string | number)[] | undefined): void {
    const finalVal = (val === undefined || val === null) ? (options.multiple ? [] : '') : val
    options.onChange('update:modelValue', finalVal)
    options.onChange('update:value', finalVal)
    options.onChange('change', finalVal)
  }

  watch(internalValue, (val) => {
    emitValue(val)
  })

  return {
    showDialog,
    internalValue,
    selectedLabel,
    openDialog,
    handleRefresh,
    handleConfirm
  }
}
