<template>
  <q-layout view="lHh Lpr lFf">
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-btn dense flat round icon="menu" @click="drawer = !drawer" />
        <q-toolbar-title>SocialOps 社交运维</q-toolbar-title>
      </q-toolbar>
    </q-header>

    <q-drawer v-model="drawer" show-if-above bordered>
      <q-list>
        <q-item v-for="item in menuItems" :key="item.path" clickable :to="item.path" exact>
          <q-item-section avatar>
            <q-icon :name="item.icon" />
          </q-item-section>
          <q-item-section>{{ item.label }}</q-item-section>
        </q-item>
      </q-list>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const drawer = ref(true);

const menuItems = [
  { path: '/', label: '运营总览', icon: 'dashboard' },
  { path: '/accounts', label: '社交账号', icon: 'alternate_email' },
  { path: '/crawl/sources', label: '抓取源管理', icon: 'rss_feed' },
  { path: '/contents', label: '内容库', icon: 'article' },
  { path: '/rewrite', label: '智能洗文', icon: 'auto_awesome' },
  { path: '/llm-providers', label: 'LLM 配置', icon: 'psychology' },
  { path: '/publish', label: '发布任务', icon: 'publish' },
  { path: '/schedules', label: '定时计划', icon: 'schedule' },
  { path: '/stats', label: '数据统计', icon: 'bar_chart' },
  { path: '/insights', label: 'AI 分析报告', icon: 'insights' },
];
</script>
