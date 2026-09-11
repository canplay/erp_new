/**
 * @file permission.ts
 * @description 权限指令 boot 文件
 * @date 2026-04-03
 */

import { defineBoot } from '#q-app';
import PermissionDirective from '@/directives/permission';

export default defineBoot(({ app }) => {
  (app as { directive: (name: string, directive: object) => void }).directive('permission', PermissionDirective);
});
