import { ref, computed, watch } from 'vue'

interface UseVirtualScrollTableOptions {
  rows: unknown[]
  pageSize: number
  loading: boolean
  onSortChange: (event: 'sort-change', payload: { field: string; order: string }) => void
  onPageChange: (event: 'page-change', page: number) => void
}

export function useVirtualScrollTable(options: UseVirtualScrollTableOptions) {
  const currentPage = ref(1)
  const sortField = ref('')
  const sortOrder = ref('asc')

  const totalPages = computed(() => {
    return Math.ceil(options.rows.length / options.pageSize)
  })

  const totalCount = computed(() => {
    return options.rows.length
  })

  const isLoading = computed(() => {
    return options.loading
  })

  const visibleRows = computed(() => {
    const start = (currentPage.value - 1) * options.pageSize
    const end = start + options.pageSize
    return options.rows.slice(start, end)
  })

  function handleSortChange(fieldName: string): void {
    if (sortField.value === fieldName) {
      sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    } else {
      sortField.value = fieldName
      sortOrder.value = 'asc'
    }

    options.onSortChange('sort-change', {
      field: fieldName,
      order: sortOrder.value
    })
  }

  function handlePageChange(page: number): void {
    currentPage.value = page
    options.onPageChange('page-change', page)
  }

  function renderRow({ row }: { row: unknown }): unknown {
    return row
  }

  watch(() => options.rows, () => {
    currentPage.value = 1
  })

  return {
    currentPage,
    totalPages,
    totalCount,
    isLoading,
    visibleRows,
    handleSortChange,
    handlePageChange,
    renderRow
  }
}
