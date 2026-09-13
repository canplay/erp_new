// @erp-new-frontend-monorepo/capabilities
// 企业级 SaaS 能力分治单一可信源
// 每个路由/菜单通过本文件判定是否渲染入口，避免越权导航与重复实现。

export type Viewpoint = 'admin' | 'ops' | 'social';

/**
 * 能力 → 作用域映射。
 * - shared:    admin/ops/social 三端都有
 * - admin:     仅管理后台
 * - ops:       仅运营平台
 * - social:    仅社交端
 */
export const CAP_SCOPE: Record<string, string> = {
  // ==================== 共享能力（三端都有）====================
  'auth:login': 'shared',
  'auth:logout': 'shared',
  'profile:view': 'shared',
  'profile:edit': 'shared',
  'settings:view': 'shared',
  'notifications:view': 'shared',
  'messages:view': 'shared',
  'files:view': 'shared',
  'files:upload': 'shared',
  'files:download': 'shared',
  'theme:switch': 'shared',

  // ==================== 仅Admin（管理后台）====================
  'dashboard:view': 'admin',
  'data-dashboard:view': 'admin',
  'user:list': 'admin',
  'user:create': 'admin',
  'user:edit': 'admin',
  'user:delete': 'admin',
  'user:detail': 'admin',
  'user:reset-password': 'admin',
  'department:list': 'admin',
  'department:edit': 'admin',
  'role:list': 'admin',
  'role:create': 'admin',
  'role:edit': 'admin',
  'role:delete': 'admin',
  'role:assign': 'admin',
  'permission:view': 'admin',
  'permission:edit': 'admin',
  'permission:matrix': 'admin',
  'permission:row-level': 'admin',
  'permission:field-mask': 'admin',
  'log:login': 'admin',
  'log:operation': 'admin',
  'log:export': 'admin',
  'system-config:view': 'admin',
  'system-config:edit': 'admin',
  'announcement:list': 'admin',
  'announcement:create': 'admin',
  'announcement:edit': 'admin',
  'announcement:delete': 'admin',
  'notification-template:list': 'admin',
  'notification-template:create': 'admin',
  'notification-template:edit': 'admin',
  'notification-template:delete': 'admin',
  'dictionary:type:list': 'admin',
  'dictionary:type:edit': 'admin',
  'dictionary:item:list': 'admin',
  'dictionary:item:edit': 'admin',
  'tenant:list': 'admin',
  'tenant:create': 'admin',
  'tenant:edit': 'admin',
  'tenant:delete': 'admin',
  'tenant:status': 'admin',
  'permission-log:view': 'admin',
  'api-key:list': 'admin',
  'api-key:create': 'admin',
  'api-key:edit': 'admin',
  'api-key:delete': 'admin',
  'api-log:view': 'admin',
  'api-log:export': 'admin',
  'api-statistics:view': 'admin',
  'api-governance:view': 'admin',
  'ip-whitelist:view': 'admin',
  'ip-whitelist:edit': 'admin',
  'sensitive-audit:view': 'admin',
  'monitor:view': 'admin',
  'monitor:websocket': 'admin',
  'task-schedule:view': 'admin',
  'task-schedule:edit': 'admin',
  'task-schedule:delete': 'admin',
  'cms:article:list': 'admin',
  'cms:article:create': 'admin',
  'cms:article:edit': 'admin',
  'cms:article:delete': 'admin',
  'cms:article:preview': 'admin',
  'cms:category:list': 'admin',
  'cms:category:edit': 'admin',
  'cms:category:delete': 'admin',
  'workflow:list': 'admin',
  'workflow:create': 'admin',
  'workflow:edit': 'admin',
  'workflow:delete': 'admin',
  'workflow:design': 'admin',
  'workflow:instance:list': 'admin',
  'workflow:instance:detail': 'admin',
  'report:list': 'admin',
  'report:create': 'admin',
  'report:edit': 'admin',
  'report:delete': 'admin',
  'report:view': 'admin',
  'report:template:list': 'admin',
  'report:template:create': 'admin',
  'report:datasource:list': 'admin',
  'report:datasource:edit': 'admin',
  'device:list': 'admin',
  'device:detail': 'admin',
  'device:edit': 'admin',
  'device:delete': 'admin',
  'device:abnormal-login': 'admin',
  'device:login-history': 'admin',
  'ctp:device:list': 'admin',
  'ctp:device:detail': 'admin',
  'lpr:records:list': 'admin',
  'lpr:monitor:view': 'admin',
  'lpr:devices:list': 'admin',
  'lpr:devices:edit': 'admin',
  'xlt:vehicles:list': 'admin',
  'xlt:vehicles:edit': 'admin',
  'xlt:records:list': 'admin',
  'xlt/billing:rules:list': 'admin',
  'xlt/billing:rules:edit': 'admin',
  'xlt/parking:list': 'admin',
  'xlt/parking:edit': 'admin',
  'tow:cars:list': 'admin',
  'tow:cars:edit': 'admin',
  'tow:tasks:list': 'admin',
  'tow:config:edit': 'admin',
  'ebike:cars:list': 'admin',
  'ebike:cars:edit': 'admin',
  'ebike/storages:list': 'admin',
  'ebike/orders:list': 'admin',
  'ebike/options:edit': 'admin',
  'ebike/reports:view': 'admin',
  'feedback:list': 'admin',
  'feedback:status': 'admin',
  'feedback:statistics:view': 'admin',
  'export-tasks:list': 'admin',
  'export-tasks:status': 'admin',
  'export-tasks:download': 'admin',

  // ==================== 仅Ops（运营平台）====================
  'ops:dashboard:view': 'ops',
  'account:list': 'ops',
  'account:detail': 'ops',
  'account:status': 'ops',
  'content:list': 'ops',
  'content:detail': 'ops',
  'content:edit': 'ops',
  'content:delete': 'ops',
  'crawl:source:list': 'ops',
  'crawl:source:edit': 'ops',
  'crawl:source:delete': 'ops',
  'crawl:history:list': 'ops',
  'crawl:history:detail': 'ops',
  'rewrite:task:list': 'ops',
  'rewrite:task:edit': 'ops',
  'rewrite:task:delete': 'ops',
  'rewrite:editor:view': 'ops',
  'llm-provider:list': 'ops',
  'llm-provider:edit': 'ops',
  'llm-provider:delete': 'ops',
  'publish:task:list': 'ops',
  'publish:task:create': 'ops',
  'publish:task:edit': 'ops',
  'publish:task:delete': 'ops',
  'publish:wizard:view': 'ops',
  'publish:schedule:list': 'ops',
  'publish:schedule:edit': 'ops',
  'publish:schedule:delete': 'ops',
  'stats:dashboard:view': 'ops',
  'insight:list': 'ops',
  'insight:detail': 'ops',

  // ==================== 仅Social（社交端）====================
  'social:chat:view': 'social',
  'social:chat:send': 'social',
  'social:chat:receive': 'social',
  'social:chat:upload': 'social',
  'social:room:list': 'social',
  'social:room:join': 'social',
  'social:room:leave': 'social',
  'social:room:edit': 'social',
  'social:room:delete': 'social',
  'social:contact:list': 'social',
  'social:contact:add': 'social',
  'social:contact:delete': 'social',
  'social:moments:view': 'social',
  'social:moments:post': 'social',
  'social:moments:like': 'social',
  'social:moments:comment': 'social',
  'social:video-call:make': 'social',
  'social:voice-message:send': 'social',
};

/** 判定某视角是否拥有某能力。 */
export function hasCap(cap: string, viewpoint: Viewpoint): boolean {
  const scope = CAP_SCOPE[cap];
  if (!scope) return false;
  if (scope === 'shared') return true;
  return scope === viewpoint;
}

/** 获取所有能力列表。 */
export function getAllCaps(): string[] {
  return Object.keys(CAP_SCOPE);
}

/** 获取某视角可用的能力列表。 */
export function getAvailableCaps(viewpoint: Viewpoint): string[] {
  return Object.entries(CAP_SCOPE)
    .filter(([, scope]) => scope === 'shared' || scope === viewpoint)
    .map(([cap]) => cap);
}

/** 兼容别名导出。 */
export const CAPABILITIES = CAP_SCOPE;
