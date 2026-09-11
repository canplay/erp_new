import { ref, computed } from 'vue'

interface Rule {
  field: string
  operator: string
  value: string
}

interface UseDataFilterBuilderOptions {
  fieldOptions: Array<{ label: string; value: string; type: string }>
  initialFilters?: Record<string, unknown>
  onChange: (event: 'change', filters: Record<string, unknown>) => void
  onReset: (event: 'reset') => void
}

export function useDataFilterBuilder(options: UseDataFilterBuilderOptions) {
  const rules = ref<Rule[]>([])

  const filters = computed(() => {
    const result: Record<string, unknown> = {}
    rules.value.forEach(rule => {
      if (rule.field && rule.operator && rule.value) {
        result[rule.field] = {
          operator: rule.operator,
          value: rule.value
        }
      }
    })
    return result
  })

  function addRule(): void {
    rules.value.push({
      field: options.fieldOptions[0]?.value || '',
      operator: 'eq',
      value: ''
    })
  }

  function removeRule(index: number): void {
    rules.value.splice(index, 1)
    options.onChange('change', filters.value)
  }

  function updateRule(index: number, rule: Rule): void {
    rules.value[index] = rule
    options.onChange('change', filters.value)
  }

  function applyFilters(): void {
    options.onChange('change', filters.value)
  }

  function resetFilters(): void {
    rules.value = []
    options.onReset('reset')
  }

  return {
    rules,
    filters,
    addRule,
    removeRule,
    updateRule,
    applyFilters,
    resetFilters
  }
}
