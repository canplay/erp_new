<template>
  <q-item clickable v-ripple class="contact-card-item" @click="$emit('select', contact.userId)">
    <q-item-section avatar>
      <q-avatar :color="avatarColor" text-color="white" size="40px">
        <img v-if="contact.avatarUrl" :src="mxcToHttp(contact.avatarUrl)" />
        <span v-else>{{ initial }}</span>
      </q-avatar>
    </q-item-section>

    <q-item-section>
      <q-item-label class="text-weight-medium">
        {{ (contact?.displayName || contact?.userId?.split(':')[0]?.replace('@', '')) || '未知用户' }}
      </q-item-label>
      <q-item-label caption lines="1">
        <q-badge
          v-if="contact.presence === 'online'"
          color="positive"
          size="xs"
          class="q-mr-xs"
          label="在线"
        />
        <q-badge
          v-else-if="contact.presence === 'offline'"
          color="grey-5"
          size="xs"
          class="q-mr-xs"
          label="离线"
        />
        {{ contact.userId }}
      </q-item-label>
    </q-item-section>

    <q-item-section side>
      <q-btn
        flat
        round
        icon="chat"
        size="sm"
        color="primary"
        @click.stop="$emit('chat', contact)"
      >
        <q-tooltip>发起聊天</q-tooltip>
      </q-btn>
    </q-item-section>
  </q-item>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Contact } from '@/stores/contact';

const props = defineProps<{
  contact: Contact;
}>();

defineEmits<{
  select: [userId: string];
  chat: [contact: Contact];
}>();

const initial = computed(() => {
  const name = (props.contact?.displayName || props.contact?.userId);
  return (name ?? '?').charAt(0).toUpperCase();
});

const avatarColor = computed(() => {
  const colors = ['primary', 'secondary', 'teal', 'orange', 'purple', 'cyan', 'pink', 'indigo'];
  const hash = props.contact.userId.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0);
  return colors[hash % colors.length];
});

function mxcToHttp(mxcUrl: string): string {
  if (!mxcUrl) return '';
  const parts = mxcUrl.replace('mxc://', '').split('/');
  if (parts.length !== 2) return mxcUrl;
  const base = import.meta.env.VITE_TUWUNEL_URL || 'http://localhost:8008';
  return `${base}/_matrix/media/v3/download/${parts[0]}/${parts[1]}`;
}
</script>

<style scoped>
.contact-card-item {
  border-radius: 0;
  transition: background 0.15s;
}
.contact-card-item:hover {
  background: #f5f5f5;
}
</style>
