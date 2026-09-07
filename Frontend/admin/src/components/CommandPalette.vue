/**
 * @file CommandPalette.vue
 * @description 命令面板 - 组合 PaletteInput + PaletteResults + PaletteShortcut
 * @date 2026-04-04
 */

<template>
  <q-dialog
    v-model="isOpen"
    seamless
    position="top"
    @hide="handleClose"
  >
    <q-card class="command-palette" @click.stop>
      <q-card-section class="search-section">
        <PaletteInput
          :search-query="searchQuery"
          :selected-category="selectedCategory"
          :categories="categories"
          @update:search-query="searchQuery = $event"
          @select-category="selectCategory"
          @clear-category="clearCategory"
          @keydown="handleKeydown"
        />
      </q-card-section>

      <q-card-section class="results-section">
        <PaletteResults
          :commands="filteredCommands"
          :selected-index="selectedIndex"
          :search-query="searchQuery"
          @execute-command="executeCommand"
          @update:selected-index="selectedIndex = $event"
        />
      </q-card-section>

      <q-card-section class="footer-section">
        <PaletteShortcut />
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useQuasar } from 'quasar';
import { useRouter } from 'vue-router';
import PaletteInput from './CommandPalette/PaletteInput.vue';
import PaletteResults from './CommandPalette/PaletteResults.vue';
import PaletteShortcut from './CommandPalette/PaletteShortcut.vue';

interface Command {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  category: string;
  shortcut?: string;
  action: () => void | Promise<void>;
  keywords?: string[];
}

interface Category {
  id: string;
  label: string;
  icon?: string;
}

interface Props {
  modelValue?: boolean;
  commands?: Command[];
  categories?: Category[];
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  commands: () => [],
  categories: () => [
    { id: 'all', label: '全部' },
    { id: 'navigation', label: '导航' },
    { id: 'action', label: '操作' },
    { id: 'settings', label: '设置' }
  ]
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'execute', command: Command): void;
}>();

const $q = useQuasar();
const router = useRouter();

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
});

const searchQuery = ref('');
const selectedIndex = ref(0);
const selectedCategory = ref<Category | null>(null);

const defaultCommands: Command[] = [
  { id: 'nav-home', name: '首页', description: '跳转到首页', icon: 'home', category: 'navigation', shortcut: 'G H', action: () => navigateTo('/'), keywords: ['首页', 'home', 'dashboard'] },
  { id: 'nav-ctp', name: '地锁设备', description: 'CTP 地锁设备管理', icon: 'lock', category: 'navigation', action: () => navigateTo('/ctp'), keywords: ['地锁', 'ctp', '锁', 'lock'] },
  { id: 'nav-lpr-records', name: '通行记录', description: '车牌识别通行记录', icon: 'directions_car', category: 'navigation', action: () => navigateTo('/lpr/records'), keywords: ['通行', '记录', 'lpr', '车牌'] },
  { id: 'nav-lpr-monitor', name: '实时监控', description: '实时通行监控', icon: 'videocam', category: 'navigation', action: () => navigateTo('/lpr/monitor'), keywords: ['监控', '实时', 'monitor'] },
  { id: 'nav-xlt', name: '停车看板', description: '信路通停车管理看板', icon: 'local_parking', category: 'navigation', action: () => navigateTo('/xlt'), keywords: ['停车', '看板', 'xlt', 'park'] },
  { id: 'nav-tow', name: '拖车任务', description: '拖车任务管理', icon: 'local_shipping', category: 'navigation', action: () => navigateTo('/tow'), keywords: ['拖车', '任务', 'tow'] },
  { id: 'nav-users', name: '用户管理', description: '跳转到用户管理页面', icon: 'people', category: 'navigation', shortcut: 'G U', action: () => navigateTo('/system/user'), keywords: ['用户', 'user'] },
  { id: 'nav-settings', name: '系统设置', description: '跳转到系统设置页面', icon: 'settings', category: 'navigation', action: () => navigateTo('/system/settings'), keywords: ['设置', 'settings', 'config'] },
  { id: 'action-refresh', name: '刷新页面', description: '刷新当前页面', icon: 'refresh', category: 'action', shortcut: 'R', action: () => window.location.reload(), keywords: ['刷新', 'refresh', 'reload'] },
  { id: 'action-export', name: '导出数据', description: '导出当前页面的数据', icon: 'download', category: 'action', action: () => $q.notify({ message: '导出功能开发中' }), keywords: ['导出', 'export', 'download'] },
  { id: 'action-import', name: '导入数据', description: '导入数据到当前页面', icon: 'upload', category: 'action', action: () => $q.notify({ message: '导入功能开发中' }), keywords: ['导入', 'import', 'upload'] },
  { id: 'settings-theme', name: '切换主题', description: '切换明暗主题', icon: 'palette', category: 'settings', action: () => toggleTheme(), keywords: ['主题', 'theme', 'dark', 'light'] },
  { id: 'settings-language', name: '切换语言', description: '切换系统语言', icon: 'language', category: 'settings', action: () => $q.notify({ message: '语言切换功能开发中' }), keywords: ['语言', 'language', 'i18n'] }
];

const allCommands = computed(() => {
  return [...defaultCommands, ...props.commands];
});

const filteredCommands = computed(() => {
  let filtered = allCommands.value;
  if (selectedCategory.value && selectedCategory.value.id !== 'all') {
    filtered = filtered.filter(cmd => cmd.category === selectedCategory.value?.id);
  }
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    filtered = filtered.filter(cmd => {
      return (
        cmd.name.toLowerCase().includes(query) ||
        cmd.description?.toLowerCase().includes(query) ||
        cmd.keywords?.some(k => k.toLowerCase().includes(query))
      );
    });
  }
  return filtered;
});

function selectCategory(category: Category) {
  if (selectedCategory.value?.id === category.id) {
    selectedCategory.value = null;
  } else {
    selectedCategory.value = category;
  }
  selectedIndex.value = 0;
}

function clearCategory() {
  selectedCategory.value = null;
  selectedIndex.value = 0;
}

function handleKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.value.length - 1);
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
      break;
    case 'Enter':
      event.preventDefault();
      const command = filteredCommands.value[selectedIndex.value];
      if (command) executeCommand(command);
      break;
    case 'Escape':
      event.preventDefault();
      handleClose();
      break;
  }
}

function executeCommand(command: Command) {
  emit('execute', command);
  void command.action();
  handleClose();
}

function navigateTo(path: string) {
  router.push(path);
}

function toggleTheme() {
  const currentTheme = $q.dark.isActive ? 'dark' : 'light';
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
  $q.dark.set(newTheme === 'dark');
  localStorage.setItem('theme', newTheme);
}

function handleClose() {
  searchQuery.value = '';
  selectedIndex.value = 0;
  selectedCategory.value = null;
  isOpen.value = false;
}

watch(filteredCommands, () => {
  selectedIndex.value = 0;
});

function handleGlobalKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
    event.preventDefault();
    isOpen.value = true;
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
});

defineExpose({
  open: () => { isOpen.value = true; },
  close: handleClose
});
</script>

<style scoped>
.command-palette {
  width: 600px;
  max-width: 90vw;
}
.search-section {
  padding-bottom: 8px;
}
.results-section {
  padding-top: 8px;
  padding-bottom: 8px;
}
.footer-section {
  padding-top: 0;
}
</style>
