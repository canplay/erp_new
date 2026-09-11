<template>
  <div>
    <div class="text-subtitle1 q-mb-md">系统健康检查</div>
    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-list bordered separator>
          <q-item v-for="check in healthChecks" :key="check.name">
            <q-item-section avatar>
              <q-icon :name="getHealthIcon(check.status)" :color="getHealthColor(check.status)" size="32px" />
            </q-item-section>
            <q-item-section>
              <q-item-label>{{ check.name }}</q-item-label>
              <q-item-label caption>{{ check.message || check.details || check.status }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-badge :color="getHealthColor(check.status)" :label="check.status" />
            </q-item-section>
          </q-item>
        </q-list>
      </div>
      <div class="col-12 col-md-6">
        <div v-if="activeAlerts.length > 0">
          <div class="text-subtitle1 q-mb-md">活跃告警</div>
          <q-list bordered separator>
            <q-item v-for="alert in activeAlerts" :key="alert.id">
              <q-item-section avatar><q-icon :name="getAlertIcon(alert.level)" :color="getAlertColor(alert.level)" size="28px" /></q-item-section>
              <q-item-section>
                <q-item-label>{{ alert.message }}</q-item-label>
                <q-item-label caption>{{ alert.triggeredAt }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-btn flat dense icon="check" color="positive" size="sm" @click="$emit('resolveAlert', alert.id)" />
              </q-item-section>
            </q-item>
          </q-list>
        </div>
        <div v-else class="text-center q-pa-xl text-grey-5">
          <q-icon name="check_circle" size="64px" color="positive" />
          <div class="text-h6 q-mt-md">所有系统运行正常</div>
          <div class="text-caption">暂无告警信息</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { HealthCheckResult, PerformanceAlert } from '@/types/monitor';

defineProps<{
  healthChecks: HealthCheckResult[];
  activeAlerts: PerformanceAlert[];
  getHealthIcon: (status: string) => string;
  getHealthColor: (status: string) => string;
  getAlertIcon: (level: string) => string;
  getAlertColor: (level: string) => string;
}>();

defineEmits<{ resolveAlert: [alertId: string] }>();
</script>
