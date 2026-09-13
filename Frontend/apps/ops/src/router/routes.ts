import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      { path: '', name: 'Dashboard', component: () => import('@erp-new-frontend-monorepo/pages/src/DashboardPage.vue'), meta: { title: '运营总览', icon: 'dashboard' ,
      cap: 'ops:dashboard:view'
    } },
      { path: 'accounts', name: 'AccountList', component: () => import('@erp-new-frontend-monorepo/pages/src/accounts/AccountListPage.vue'), meta: { title: '社交账号', icon: 'alternate_email' ,
      cap: 'account:list'
    } },
      { path: 'accounts/:id', name: 'AccountDetail', component: () => import('@erp-new-frontend-monorepo/pages/src/accounts/AccountDetailPage.vue'), meta: { title: '账号详情' ,
      cap: 'account:detail'
    } },
      { path: 'crawl/sources', name: 'CrawlSources', component: () => import('@erp-new-frontend-monorepo/pages/src/crawl/SourceListPage.vue'), meta: { title: '抓取源管理', icon: 'rss_feed' ,
      cap: 'crawl:source:list'
    } },
      { path: 'crawl/history', name: 'CrawlHistory', component: () => import('@erp-new-frontend-monorepo/pages/src/crawl/CrawlHistoryPage.vue'), meta: { title: '抓取历史' ,
      cap: 'crawl:history:list'
    } },
      { path: 'contents', name: 'ContentList', component: () => import('@erp-new-frontend-monorepo/pages/src/content/ContentListPage.vue'), meta: { title: '内容库', icon: 'article' ,
      cap: 'content:list'
    } },
      { path: 'contents/:id', name: 'ContentDetail', component: () => import('@erp-new-frontend-monorepo/pages/src/content/ContentDetailPage.vue'), meta: { title: '内容详情' ,
      cap: 'content:detail'
    } },
      { path: 'rewrite', name: 'RewriteTasks', component: () => import('@erp-new-frontend-monorepo/pages/src/rewrite/RewriteTaskPage.vue'), meta: { title: '智能洗文', icon: 'auto_awesome' ,
      cap: 'rewrite:task:list'
    } },
      { path: 'rewrite/:taskId', name: 'RewriteEditor', component: () => import('@erp-new-frontend-monorepo/pages/src/rewrite/RewriteEditorPage.vue'), meta: { title: '洗文编辑' ,
      cap: 'rewrite:editor:view'
    } },
      { path: 'llm-providers', name: 'LLMProviders', component: () => import('@erp-new-frontend-monorepo/pages/src/rewrite/LLMProviderSettings.vue'), meta: { title: 'LLM 配置', icon: 'psychology' ,
      cap: 'llm-provider:list'
    } },
      { path: 'publish', name: 'PublishTasks', component: () => import('@erp-new-frontend-monorepo/pages/src/publish/PublishTaskPage.vue'), meta: { title: '发布任务', icon: 'publish' ,
      cap: 'publish:task:list'
    } },
      { path: 'publish/new', name: 'PublishWizard', component: () => import('@erp-new-frontend-monorepo/pages/src/publish/PublishWizard.vue'), meta: { title: '新建发布' ,
      cap: 'publish:task:create'
    } },
      { path: 'schedules', name: 'ScheduleList', component: () => import('@erp-new-frontend-monorepo/pages/src/publish/ScheduleListPage.vue'), meta: { title: '定时计划', icon: 'schedule' ,
      cap: 'publish:schedule:list'
    } },
      { path: 'stats', name: 'StatsDashboard', component: () => import('@erp-new-frontend-monorepo/pages/src/stats/StatsDashboard.vue'), meta: { title: '数据统计', icon: 'bar_chart' ,
      cap: 'stats:dashboard:view'
    } },
      { path: 'insights', name: 'InsightList', component: () => import('@erp-new-frontend-monorepo/pages/src/insights/InsightListPage.vue'), meta: { title: 'AI 分析报告', icon: 'insights' ,
      cap: 'insight:list'
    } },
    ],
  },
];

export default routes;
