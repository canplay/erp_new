/**
 * @file CommandPalette/PaletteShortcut.vue
 * @description 命令面板 - 底部快捷键提示
 * @date 2026-08-22
 */

<template>
  <div class="palette-shortcut">
    <div class="row justify-between text-caption text-grey">
      <span>
        <kbd>↑</kbd><kbd>↓</kbd> 导航
      </span>
      <span>
        <kbd>Enter</kbd> 执行
      </span>
      <span>
        <kbd>Esc</kbd> 关闭
      </span>
    </div>
  </div>
</template>

<style scoped>
.palette-shortcut {
  padding: 8px 0;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
}
</style>
