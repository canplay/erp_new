import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { MatrixService } from '@/services'
import type { RoomInfo } from '@/services'
import { useAuthStore } from './auth'

export const useRoomStore = defineStore('room', () => {
  // State
  const rooms = ref<RoomInfo[]>([])
  const currentRoomId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const currentRoom = computed(() =>
    currentRoomId.value
      ? rooms.value.find((r) => r.roomId === currentRoomId.value) ?? null
      : null,
  )

  const joinedRooms = computed(() => rooms.value)

  const sortedRooms = computed(() =>
    [...rooms.value].sort((a, b) => {
      const timeA = a.lastEvent?.getTs() ?? 0
      const timeB = b.lastEvent?.getTs() ?? 0
      return timeB - timeA
    }),
  )

  // Actions
  function loadRooms(): void {
    const auth = useAuthStore()
    if (!auth.isLoggedIn) return

    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      rooms.value = ms.getRooms()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '加载房间列表失败'
    } finally {
      isLoading.value = false
    }
  }

  async function createRoom(name: string, topic?: string, isDirect = false): Promise<string> {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const roomId = await ms.createRoom({
        name,
        topic,
        preset: isDirect ? 'trusted_private_chat' : 'public_chat',
        room_alias_name: name.toLowerCase().replace(/\s+/g, '-'),
        initial_state: [
          {
            type: 'm.room.history_visibility',
            state_key: '',
            content: { history_visibility: 'shared' },
          },
        ],
      })
      loadRooms()
      return roomId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '创建房间失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function createDirectRoom(userId: string): Promise<string> {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const roomId = await ms.createDirectRoom(userId)
      loadRooms()
      return roomId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '创建私聊失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function joinRoom(roomIdOrAlias: string): Promise<string> {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const roomId = await ms.joinRoom(roomIdOrAlias)
      loadRooms()
      return roomId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '加入房间失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function leaveRoom(roomId: string): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      await ms.leaveRoom(roomId)
      rooms.value = rooms.value.filter((r) => r.roomId !== roomId)
      if (currentRoomId.value === roomId) {
        currentRoomId.value = null
      }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '离开房间失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  function setCurrentRoom(roomId: string | null): void {
    currentRoomId.value = roomId
  }

  function clearError(): void {
    error.value = null
  }

  return {
    // State
    rooms,
    currentRoomId,
    isLoading,
    error,
    // Getters
    currentRoom,
    joinedRooms,
    sortedRooms,
    // Actions
    loadRooms,
    createRoom,
    createDirectRoom,
    joinRoom,
    leaveRoom,
    setCurrentRoom,
    clearError,
  }
})
