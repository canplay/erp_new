<template>
  <div class="row q-col-gutter-md">
    <div class="col-12 col-md-4">
      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('common.connectionStatus') }}</div>
          <div class="text-center q-pa-lg">
            <q-icon name="cable" size="80px" :color="wsStatusColor" />
            <div class="text-h5 q-mt-md" :class="`text-${wsStatusColor}`">{{ wsStatusLabel }}</div>
          </div>
          <q-separator class="q-my-md" />
          <q-list dense>
            <q-item><q-item-section avatar><q-icon name="schedule" /></q-item-section><q-item-section>{{ $t('common.connectionDuration') }}</q-item-section><q-item-section side>{{ connectionDuration }}</q-item-section></q-item>
            <q-item><q-item-section avatar><q-icon name="favorite" /></q-item-section><q-item-section>{{ $t('common.lastHeartbeat') }}</q-item-section><q-item-section side>{{ lastHeartbeatAgo }}</q-item-section></q-item>
            <q-item><q-item-section avatar><q-icon name="replay" /></q-item-section><q-item-section>{{ $t('common.reconnectCount') }}</q-item-section><q-item-section side>{{ wsMetrics.reconnectAttempts }}</q-item-section></q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </div>
    <div class="col-12 col-md-4">
      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('common.connectionQuality') }}</div>
          <div class="text-center">
            <q-circular-progress :value="wsMetrics.qualityScore" size="120px" :color="qualityScoreColor" track-color="grey-3" show-value class="q-mx-auto">
              <div class="text-h4">{{ wsMetrics.qualityScore }}</div>
            </q-circular-progress>
            <div class="text-subtitle2 q-mt-md" :class="`text-${qualityScoreColor}`">{{ qualityLevelLabel }}</div>
          </div>
          <q-separator class="q-my-md" />
          <q-list dense>
            <q-item><q-item-section avatar><q-icon name="upload" /></q-item-section><q-item-section>{{ $t('common.messagesSent') }}</q-item-section><q-item-section side>{{ wsMetrics.messagesSent }}</q-item-section></q-item>
            <q-item><q-item-section avatar><q-icon name="download" /></q-item-section><q-item-section>{{ $t('common.messagesReceived') }}</q-item-section><q-item-section side>{{ wsMetrics.messagesReceived }}</q-item-section></q-item>
            <q-item><q-item-section avatar><q-icon name="error" /></q-item-section><q-item-section>{{ $t('common.errorCount') }}</q-item-section><q-item-section side>{{ wsMetrics.errorCount }}</q-item-section></q-item>
            <q-item><q-item-section avatar><q-icon name="timer" /></q-item-section><q-item-section>{{ $t('common.avgResponse') }}</q-item-section><q-item-section side>{{ wsMetrics.avg_response_time }}ms</q-item-section></q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </div>
    <div class="col-12 col-md-4">
      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('common.qualityLevel') }}</div>
          <q-list>
            <q-item v-for="level in qualityLevels" :key="level.label" dense>
              <q-item-section avatar><q-icon :name="level.icon" :color="level.color" /></q-item-section>
              <q-item-section>
                <q-item-label>{{ level.label }}</q-item-label>
                <q-item-label caption>{{ level.range }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-badge :color="level.color" :label="`${level.value}+`" />
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  wsMetrics: { status: string; connectedAt: number | null; lastHeartbeatAt: number | null; reconnectAttempts: number; messagesSent: number; messagesReceived: number; errorCount: number; avg_response_time: number; qualityScore: number };
  wsStatusColor: string;
  wsStatusLabel: string;
  qualityScoreColor: string;
  qualityLevelLabel: string;
  connectionDuration: string;
  lastHeartbeatAgo: string;
  qualityLevels: { label: string; range: string; value: number; color: string; icon: string }[];
}>();
</script>
