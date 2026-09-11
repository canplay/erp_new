import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      { path: '', name: 'Dashboard', component: () => import('@/pages/DashboardPage.vue'), meta: { title: '运营总览', icon: 'dashboard' } },
      { path: 'accounts', name: 'AccountList', component: () => import('@/pages/accounts/AccountListPage.vue'), meta: { title: '社交账号', icon: 'alternate_email' } },
      { path: 'accounts/:id', name: 'AccountDetail', component: () => import('@/pages/accounts/AccountDetailPage.vue'), meta: { title: '账号详情' } },
      { path: 'crawl/sources', name: 'CrawlSources', component: () => import('@/pages/crawl/SourceListPage.vue'), meta: { title: '抓取源管理', icon: 'rss_feed' } },
      { path: 'crawl/history', name: 'CrawlHistory', component: () => import('@/pages/crawl/CrawlHistoryPage.vue'), meta: { title: '抓取历史' } },
      { path: 'contents', name: 'ContentList', component: () => import('@/pages/content/ContentListPage.vue'), meta: { title: '内容库', icon: 'article' } },
      { path: 'contents/:id', name: 'ContentDetail', component: () => import('@/pages/content/ContentDetailPage.vue'), meta: { title: '内容详情' } },
      { path: 'rewrite', name: 'RewriteTasks', component: () => import('@/pages/rewrite/RewriteTaskPage.vue'), meta: { title: '智能洗文', icon: 'auto_awesome' } },
      { path: 'rewrite/:taskId', name: 'RewriteEditor', component: () => import('@/pages/rewrite/RewriteEditorPage.vue'), meta: { title: '洗文编辑' } },
      { path: 'llm-providers', name: 'LLMProviders', component: () => import('@/pages/rewrite/LLMProviderSettings.vue'), meta: { title: 'LLM 配置', icon: 'psychology' } },
      { path: 'publish', name: 'PublishTasks', component: () => import('@/pages/publish/PublishTaskPage.vue'), meta: { title: '发布任务', icon: 'publish' } },
      { path: 'publish/new', name: 'PublishWizard', component: () => import('@/pages/publish/PublishWizard.vue'), meta: { title: '新建发布' } },
      { path: 'schedules', name: 'ScheduleList', component: () => import('@/pages/publish/ScheduleListPage.vue'), meta: { title: '定时计划', icon: 'schedule' } },
      { path: 'stats', name: 'StatsDashboard', component: () => import('@/pages/stats/StatsDashboard.vue'), meta: { title: '数据统计', icon: 'bar_chart' } },
      { path: 'insights', name: 'InsightList', component: () => import('@/pages/insights/InsightListPage.vue'), meta: { title: 'AI 分析报告', icon: 'insights' } },
    ],
  },
];

export default routes;
