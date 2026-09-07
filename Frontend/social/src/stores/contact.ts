import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MatrixService } from '@/services'
import type { UserInfo } from '@/services'

export interface Contact extends UserInfo {
  presence?: 'online' | 'offline' | 'unavailable' | undefined
  statusMsg?: string | undefined
  lastActive?: number | undefined
  isDirectRoom?: boolean | undefined
  directRoomId?: string | undefined
}

export const useContactStore = defineStore('contact', () => {
  // State
  const contacts = ref<Contact[]>([])
  const searchResults = ref<UserInfo[]>([])
  const isSearching = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Actions
  function loadContacts(): void {
    isLoading.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const rooms = ms.getRooms()
      const contactMap = new Map<string, Contact>()
      const currentUserId = ms.getUserId()

      for (const room of rooms) {
        if (room.isDirect) {
          const roomObj = ms.getClient().getRoom(room.roomId)
          if (roomObj) {
            const members = roomObj.getJoinedMembers()
            for (const member of members) {
              if (member.userId !== currentUserId) {
                const existing = contactMap.get(member.userId)
                const presence = ms.getPresence(member.userId)
                if (existing) {
                  existing.directRoomId = room.roomId
                } else {
                  contactMap.set(member.userId, {
                    userId: member.userId,
                    displayName: member.name || member.userId,
                    avatarUrl: member.getMxcAvatarUrl() ?? undefined,
                    presence: (presence?.presence ?? 'offline') as 'online' | 'offline' | 'unavailable',
                    statusMsg: presence?.statusMsg,
                    lastActive: presence?.lastActive,
                    isDirectRoom: true,
                    directRoomId: room.roomId,
                  })
                }
              }
            }
          }
        }
      }

      contacts.value = Array.from(contactMap.values())
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '加载联系人失败'
    } finally {
      isLoading.value = false
    }
  }

  async function searchUsers(query: string, limit = 20): Promise<void> {
    if (!query.trim()) {
      searchResults.value = []
      return
    }

    isSearching.value = true
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      const res = await ms.searchUserDirectory(query, limit)
      searchResults.value = res.results
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '搜索用户失败'
    } finally {
      isSearching.value = false
    }
  }

  async function getUserProfile(userId: string): Promise<UserInfo> {
    error.value = null
    try {
      const ms = MatrixService.getInstance()
      return await ms.getUserProfile(userId)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '获取用户信息失败'
      throw e
    }
  }

  function clearSearch(): void {
    searchResults.value = []
    isSearching.value = false
  }

  function clearError(): void {
    error.value = null
  }

  return {
    // State
    contacts,
    searchResults,
    isSearching,
    isLoading,
    error,
    // Actions
    loadContacts,
    searchUsers,
    getUserProfile,
    clearSearch,
    clearError,
  }
})
