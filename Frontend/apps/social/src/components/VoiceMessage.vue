<template>
  <div class="voice-message" :class="{ 'is-sender': isSender }">
    <div class="voice-row items-center no-wrap">
      <!-- 播放/暂停按钮 -->
      <q-btn
        round
        dense
        :icon="isPlaying ? 'pause' : 'play_arrow'"
        :color="isSender ? 'white' : 'primary'"
        :text-color="isSender ? 'primary' : 'white'"
        size="sm"
        class="play-btn"
        @click="togglePlay"
      />

      <!-- 进度条 -->
      <div class="voice-slider-container col q-mx-sm">
        <q-slider
          :model-value="progress"
          :min="0"
          :max="duration || 1"
          :color="isSender ? 'white' : 'primary'"
          :label="false"
          track-size="4px"
          thumb-size="14px"
          @update:model-value="seekTo"
        />
        <div class="row justify-between text-caption">
          <span :class="isSender ? 'text-white text-opacity-70' : 'text-grey-6'">
            {{ formatTime(currentTime) }}
          </span>
          <span :class="isSender ? 'text-white text-opacity-70' : 'text-grey-6'">
            {{ formatTime(duration) }}
          </span>
        </div>
      </div>

      <!-- 语音图标 -->
      <q-icon
        :name="isPlaying ? 'graphic_eq' : 'mic'"
        :color="isPlaying ? 'green' : (isSender ? 'white' : 'grey-6')"
        size="20px"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

export interface VoiceMessageContent {
  url?: string
  file?: Record<string, unknown>
  info?: {
    duration?: number
    mimetype?: string
    size?: number
  }
}

const props = defineProps<{
  content: VoiceMessageContent
  isSender?: boolean
}>();

const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const progress = ref(0);
const isLoading = ref(true);
const audioError = ref<string | null>(null);

let audioElement: HTMLAudioElement | null = null;
let animationFrameId: number | null = null;

// Computed
const audioUrl = computed(() => {
  if (props.content?.url) {
    return mxcToHttp(props.content.url);
  }
  return '';
});

// MXC to HTTP converter
function mxcToHttp(mxcUrl: string): string {
  if (!mxcUrl) return '';
  const parts = mxcUrl.replace('mxc://', '').split('/');
  if (parts.length !== 2) return mxcUrl;
  const base = import.meta.env.VITE_TUWUNEL_URL || 'http://localhost:8008';
  return `${base}/_matrix/media/v3/download/${parts[0]}/${parts[1]}`;
}

function formatTime(seconds: number): string {
  if (!seconds || !isFinite(seconds)) return '0:00';
  const min = Math.floor(seconds / 60);
  const sec = Math.floor(seconds % 60);
  return `${min}:${sec.toString().padStart(2, '0')}`;
}

// Initialize audio element
function initAudio() {
  if (!audioUrl.value) {
    isLoading.value = false;
    audioError.value = '无效的音频 URL';
    return;
  }

  audioElement = new Audio(audioUrl.value);
  audioElement.preload = 'metadata';

  audioElement.addEventListener('loadedmetadata', () => {
    duration.value = audioElement!.duration || props.content?.info?.duration || 0;
    isLoading.value = false;
  });

  audioElement.addEventListener('error', () => {
    isLoading.value = false;
    audioError.value = '无法加载音频';
  });

  audioElement.addEventListener('ended', () => {
    stopPlayback();
  });

  // 尝试加载
  audioElement.load();
}

// Playback control
function togglePlay() {
  if (!audioElement) return;

  if (isPlaying.value) {
    pausePlayback();
  } else {
    startPlayback();
  }
}

function startPlayback() {
  if (!audioElement) return;

  audioElement.play().then(() => {
    isPlaying.value = true;
    updateProgress();
  }).catch(() => {
    audioError.value = '播放失败';
  });
}

function pausePlayback() {
  if (!audioElement) return;
  audioElement.pause();
  isPlaying.value = false;
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
}

function stopPlayback() {
  if (!audioElement) return;
  audioElement.pause();
  audioElement.currentTime = 0;
  isPlaying.value = false;
  currentTime.value = 0;
  progress.value = 0;
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
}

function seekTo(value: number | null) {
  const v = value ?? 0;
  if (!audioElement) return;
  audioElement.currentTime = v;
  currentTime.value = v;
  progress.value = v;
}

function updateProgress() {
  if (!audioElement || audioElement.paused) return;

  currentTime.value = audioElement.currentTime;
  progress.value = audioElement.currentTime;

  animationFrameId = requestAnimationFrame(updateProgress);
}

// Lifecycle
onMounted(() => {
  initAudio();
});

onUnmounted(() => {
  stopPlayback();
  audioElement = null;
});
</script>

<style scoped>
.voice-message {
  display: inline-block;
  min-width: 200px;
  max-width: 280px;
  padding: 8px 12px;
  border-radius: 16px;
  background: #f0f0f0;
}

.voice-message.is-sender {
  background: var(--q-primary, #1976d2);
}

.voice-row {
  display: flex;
  align-items: center;
}

.play-btn {
  flex-shrink: 0;
}

.voice-slider-container {
  min-width: 120px;
}
</style>
