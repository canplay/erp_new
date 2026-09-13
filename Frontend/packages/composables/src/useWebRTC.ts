/**
 * @file useWebRTC.ts
 * @description WebRTC 音视频通话 composable — 供 VoiceMessage / VideoCall 组件复用
 */

import { ref, onUnmounted } from 'vue';

export type CallState = 'idle' | 'calling' | 'ringing' | 'connected' | 'ended' | 'failed';

export function useWebRTC() {
  const callState = ref<CallState>('idle');
  const localStream = ref<MediaStream | null>(null);
  const remoteStream = ref<MediaStream | null>(null);
  const peerConnection = ref<RTCPeerConnection | null>(null);

  const iceServers: RTCIceServer[] = [
    { urls: 'stun:stun.l.google.com:19302' },
  ];

  /** 获取本地音视频流 */
  async function startLocalStream(video = true, audio = true): Promise<MediaStream | null> {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video, audio });
      localStream.value = stream;
      return stream;
    } catch {
      console.warn('无法获取媒体设备权限');
      return null;
    }
  }

  /** 停止本地流 */
  function stopLocalStream() {
    if (localStream.value) {
      localStream.value.getTracks().forEach((t) => t.stop());
      localStream.value = null;
    }
  }

  /** 创建 RTCPeerConnection */
  function createPeerConnection() {
    const pc = new RTCPeerConnection({ iceServers });
    peerConnection.value = pc;

    pc.ontrack = (event) => {
      remoteStream.value = event.streams[0] ?? null;
    };

    pc.oniceconnectionstatechange = () => {
      if (pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'failed') {
        endCall();
      }
    };

    // 添加本地流
    if (localStream.value) {
      localStream.value.getTracks().forEach((track) => {
        pc.addTrack(track, localStream.value!);
      });
    }

    return pc;
  }

  /** 发起呼叫 */
  async function startCall() {
    callState.value = 'calling';
    await startLocalStream();
    const pc = createPeerConnection();

    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    return offer;
  }

  /** 接听 */
  async function acceptCall(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit | null> {
    callState.value = 'connected';
    await startLocalStream();
    const pc = createPeerConnection();

    await pc.setRemoteDescription(new RTCSessionDescription(offer));
    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);
    return answer;
  }

  /** 挂断 */
  function endCall() {
    stopLocalStream();
    if (peerConnection.value) {
      peerConnection.value.close();
      peerConnection.value = null;
    }
    remoteStream.value = null;
    callState.value = 'idle';
  }

  onUnmounted(() => {
    endCall();
  });

  return {
    callState,
    localStream,
    remoteStream,
    startCall,
    acceptCall,
    endCall,
  };
}
