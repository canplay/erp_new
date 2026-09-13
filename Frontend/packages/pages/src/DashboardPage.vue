<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">运营总览</div>
    <div class="row q-col-gutter-md q-mb-lg">
      <div class="col-12 col-sm-6 col-md-3" v-for="s in stats" :key="s.label">
        <q-card bordered :style="`border-top: 3px solid ${s.color}`">
          <q-card-section class="row items-center">
            <div class="col">
              <div class="text-grey text-subtitle2">{{ s.label }}</div>
              <div class="text-h4 text-weight-bold">{{ s.value }}</div>
            </div>
            <q-icon :name="s.icon" :color="s.color" size="40px" />
          </q-card-section>
        </q-card>
      </div>
    </div>
    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section><div class="text-h6 q-mb-sm">快捷操作</div>
            <q-list dense>
              <q-item clickable to="/crawl/sources" v-ripple><q-item-section avatar><q-icon name="rss_feed" color="primary" /></q-item-section><q-item-section>管理抓取源</q-item-section><q-item-section side><q-icon name="chevron_right" /></q-item-section></q-item>
              <q-item clickable to="/contents" v-ripple><q-item-section avatar><q-icon name="article" color="green" /></q-item-section><q-item-section>内容库</q-item-section><q-item-section side><q-icon name="chevron_right" /></q-item-section></q-item>
              <q-item clickable to="/publish/new" v-ripple><q-item-section avatar><q-icon name="publish" color="orange" /></q-item-section><q-item-section>新建发布</q-item-section><q-item-section side><q-icon name="chevron_right" /></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>
<script setup lang="ts">
import { ref } from 'vue';
const stats = ref([
  { label: '已接入账号', value: '0', icon: 'alternate_email', color: 'blue' },
  { label: '内容库', value: '0', icon: 'article', color: 'green' },
  { label: '抓取源', value: '0', icon: 'rss_feed', color: 'orange' },
  { label: '已发布', value: '0', icon: 'publish', color: 'purple' },
]);
</script>
