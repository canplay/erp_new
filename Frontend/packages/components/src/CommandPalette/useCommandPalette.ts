import { ref, computed } from 'vue'

interface Command {
  id: string
  label: string
  icon: string
  category: string
  action: () => void
}

interface UseCommandPaletteOptions {
  commands: Command[]
  onClose: (event: 'close') => void
}

export function useCommandPalette(options: UseCommandPaletteOptions) {
  const showDialog = ref(false)
  const searchQuery = ref('')
  const activeIndex = ref(0)

  const filteredCommands = computed(() => {
    if (!searchQuery.value) {
      return options.commands
    }

    const query = searchQuery.value.toLowerCase()
    return options.commands.filter(cmd =>
      cmd.label.toLowerCase().includes(query) ||
      cmd.category.toLowerCase().includes(query)
    )
  })

  function handleSearch(): void {
    activeIndex.value = 0
  }

  function executeCommand(command: Command): void {
    command.action()
    showDialog.value = false
    searchQuery.value = ''
    options.onClose('close')
  }

  function openPalette(): void {
    showDialog.value = true
    searchQuery.value = ''
  }

  return {
    showDialog,
    searchQuery,
    activeIndex,
    filteredCommands,
    handleSearch,
    executeCommand,
    openPalette
  }
}
