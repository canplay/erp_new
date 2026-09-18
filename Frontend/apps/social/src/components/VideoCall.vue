<template>
  <q-dialog v-model="visible" persistent maximized transition-show="slide-up" transition-hide="slide-down">
    <q-card class="video-call-card bg-black text-white">
      <!-- 顶部信息 -->
      <q-bar class="bg-transparent text-white q-pa-sm">
        <q-space />
        <div class="text-center">
          <div class="text-weight-bold">{{ callerName || '通话中...' }}</div>
          <div class="text-caption text-grey-4">{{ callDuration }}</div>
        </div>
        <q-space />
        <q-btn flat round dense icon="close" color="white" size="sm" @click="closeCall" />
      </q-bar>

      <q-card-section class="column items-center justify-center full-height q-pa-none" style="min-height: 60vh">
        <!-- 远程视频（主画面） -->
        <video
          ref="remoteVideoRef"
          autoplay
          playsinline
          class="remote-video full-width full-height"
          :class="{ hidden: !hasRemoteStream }"
        />

        <!-- 等待中 -->
        <div v-if="!hasRemoteStream" class="absolute-center text-center">
          <q-spinner-dots color="white" size="48px" />
          <div class="q-mt-md text-h6">{{ isCaller ? '等待对方接听...' : '来电...' }}</div>
          <div class="text-caption text-grey-4">{{ callerName }}</div>
        </div>

        <!-- 本地视频（小窗口） -->
        <video
          ref="localVideoRef"
          autoplay
          playsinline
          muted
          class="local-video"
          :class="{ hidden: !hasLocalStream }"
        />
      </q-card-section>

      <!-- 底部控制栏 -->
      <q-bar class="bg-transparent text-white q-pa-md" style="justify-content: center; gap: 16px">
        <q-btn
          round
          :color="isMicMuted ? 'red' : 'white'"
          :text-color="isMicMuted ? 'white' : 'black'"
          :icon="isMicMuted ? 'mic_off' : 'mic'"
          size="md"
          @click="toggleMic"
        />

        <q-btn
          round
          color="red"
          text-color="white"
          icon="call_end"
          size="lg"
          @click="endCall"
        />

        <q-btn
          round
          :color="isVideoMuted ? 'red' : 'white'"
          :text-color="isVideoMuted ? 'white' : 'black'"
          :icon="isVideoMuted ? 'videocam_off' : 'videocam'"
          size="md"
          @click="toggleVideo"
        />
      </q-bar>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { MatrixService } from '@/services';

const props = defineProps<{
  modelValue: boolean
  callerName?: string
  callerId?: string
  isCaller?: boolean
  roomId?: string
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'call-ended': []
}>();

// Video refs
const localVideoRef = ref<HTMLVideoElement | null>(null);
const remoteVideoRef = ref<HTMLVideoElement | null>(null);

// Streams
const localStream = ref<MediaStream | null>(null);
const remoteStream = ref<MediaStream | null>(null);

// State
const visible = ref(props.modelValue);
const isMicMuted = ref(false);
const isVideoMuted = ref(false);
const hasLocalStream = ref(false);
const hasRemoteStream = ref(false);
const callStartTime = ref<number>(0);
const callDuration = ref('00:00');
let durationInterval: ReturnType<typeof setInterval> | null = null;
let peerConnection: RTCPeerConnection | null = null;

// Watch modelValue
watch(() => props.modelValue, (val) => {
  visible.value = val;
  if (val) void initCall();
  else cleanup();
});

watch(visible, (val) => {
  emit('update:modelValue', val);
  if (!val) cleanup();
});

// Lifecycle
onMounted(() => {
  if (props.modelValue) void initCall();
});

onUnmounted(() => {
  cleanup();
});

async function initCall() {
  try {
    localStream.value = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
    hasLocalStream.value = true;
    if (localVideoRef.value) localVideoRef.value.srcObject = localStream.value;

    const ms = MatrixService.getInstance();
    const client = ms.getClient();
    let turnServers: RTCIceServer[] = [
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' },
    ];

    try {
      const turnResult = await Promise.resolve(client.getTurnServers()).catch(() => null);
      if (turnResult?.length) {
        turnServers = turnResult.map((s: { urls: string[]; username?: string; credential?: string }) => ({
          urls: s.urls,
          ...(s.username !== undefined ? { username: s.username } : {}),
          ...(s.credential !== undefined ? { credential: s.credential } : {}),
        }));
      }
    } catch { /* use default STUN */ }

    peerConnection = new RTCPeerConnection({ iceServers: turnServers });
    localStream.value.getTracks().forEach((track) => peerConnection?.addTrack(track, localStream.value!));

    peerConnection.ontrack = (event: RTCTrackEvent) => {
      remoteStream.value = event.streams[0] ?? null;
      hasRemoteStream.value = true;
      if (remoteVideoRef.value) remoteVideoRef.value.srcObject = event.streams[0] ?? null;
    };

    peerConnection.oniceconnectionstatechange = () => {
      if (peerConnection?.iceConnectionState === 'disconnected' || peerConnection?.iceConnectionState === 'failed') endCall();
    };

    callStartTime.value = Date.now();
    durationInterval = setInterval(updateDuration, 1000);
  } catch (err) {
    if (import.meta.env.DEV) console.error('WebRTC error:', err);
    hasLocalStream.value = false;
  }
}

function updateDuration() {
  const elapsed = Math.floor((Date.now() - callStartTime.value) / 1000);
  callDuration.value = `${String(Math.floor(elapsed / 60)).padStart(2, '0')}:${String(elapsed % 60).padStart(2, '0')}`;
}

function toggleMic() {
  isMicMuted.value = !isMicMuted.value;
  localStream.value?.getAudioTracks().forEach((t) => { t.enabled = !isMicMuted.value; });
}

function toggleVideo() {
  isVideoMuted.value = !isVideoMuted.value;
  localStream.value?.getVideoTracks().forEach((t) => { t.enabled = !isVideoMuted.value; });
}

function endCall() {
  cleanup();
  emit('call-ended');
}

function closeCall() {
  endCall();
  visible.value = false;
}

function cleanup() {
  localStream.value?.getTracks().forEach((t) => t.stop());
  localStream.value = null;
  if (peerConnection) { peerConnection.close(); peerConnection = null; }
  if (localVideoRef.value) localVideoRef.value.srcObject = null;
  if (remoteVideoRef.value) remoteVideoRef.value.srcObject = null;
  hasLocalStream.value = false;
  hasRemoteStream.value = false;
  callDuration.value = '00:00';
  if (durationInterval) { clearInterval(durationInterval); durationInterval = null; }
}
</script>
