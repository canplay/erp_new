import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { MatrixService } from '@/services'
import type { AuthCredentials } from '@/services'

const LS_KEYS = {
  ACCESS_TOKEN: 'matrix_access_token',
  REFRESH_TOKEN: 'matrix_refresh_token',
  USER_ID: 'matrix_user_id',
  DEVICE_ID: 'matrix_device_id',
} as const

export const useAuthStore = defineStore('auth', () => {
  // State
  const userId = ref<string | null>(null)
  const accessToken = ref<string | null>(null)
  const refreshToken = ref<string | null>(null)
  const deviceId = ref<string | null>(null)
  const isLoggedIn = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const matrixService = computed(() => MatrixService.getInstance())

  // Internal: persist session to localStorage
  function persistSession(creds: AuthCredentials): void {
    localStorage.setItem(LS_KEYS.ACCESS_TOKEN, creds.accessToken)
    localStorage.setItem(LS_KEYS.USER_ID, creds.userId)
    localStorage.setItem(LS_KEYS.DEVICE_ID, creds.deviceId)
    if (creds.refreshToken) {
      localStorage.setItem(LS_KEYS.REFRESH_TOKEN, creds.refreshToken)
    }
  }

  // Internal: apply session to store state
  function applySession(creds: AuthCredentials): void {
    userId.value = creds.userId
    accessToken.value = creds.accessToken
    refreshToken.value = creds.refreshToken ?? null
    deviceId.value = creds.deviceId
    isLoggedIn.value = true
  }

  // Actions
  async function login(username: string, password: string): Promise<AuthCredentials> {
    isLoading.value = true
    error.value = null
    try {
      const ms = matrixService.value
      const creds = await ms.login(username, password)
      applySession(creds)
      persistSession(creds)
      await ms.startClient({ initialSyncLimit: 10 })
      // Set online presence
      ms.setSyncPresence('online').catch(() => {})
      return creds
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '登录失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function register(
    username: string,
    password: string,
    displayName?: string,
  ): Promise<AuthCredentials> {
    isLoading.value = true
    error.value = null
    try {
      const ms = matrixService.value
      const opts: { displayName?: string } = {}
      if (displayName) opts.displayName = displayName
      const creds = await ms.register(username, password, opts)
      applySession(creds)
      persistSession(creds)
      await ms.startClient({ initialSyncLimit: 10 })
      // Set online presence
      ms.setSyncPresence('online').catch(() => {})
      return creds
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '注册失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function loginWithToken(token: string): Promise<AuthCredentials> {
    isLoading.value = true
    error.value = null
    try {
      const ms = matrixService.value
      const creds = await ms.loginWithToken(token)
      applySession(creds)
      persistSession(creds)
      await ms.startClient({ initialSyncLimit: 10 })
      // Set online presence
      ms.setSyncPresence('online').catch(() => {})
      return creds
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Token 登录失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function restoreSession(
    token: string,
    uid: string,
    did: string,
    rToken?: string,
  ): Promise<void> {
    isLoading.value = true
    try {
      const ms = matrixService.value
      await ms.loginWithSession(token, uid, did, rToken)
      applySession({ userId: uid, accessToken: token, deviceId: did, refreshToken: rToken })
      await ms.startClient({ initialSyncLimit: 10 })
      ms.setSyncPresence('online').catch(() => {})
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '会话恢复失败'
      clearSession()
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Try to refresh the access token using the stored refresh token.
   * Returns true if refresh succeeded, false otherwise.
   */
  async function tryRefreshToken(): Promise<boolean> {
    const savedRefreshToken = localStorage.getItem(LS_KEYS.REFRESH_TOKEN)
    if (!savedRefreshToken || !userId.value) return false

    try {
      const ms = matrixService.value
      const newToken = await ms.refreshAuthToken()
      accessToken.value = newToken
      localStorage.setItem(LS_KEYS.ACCESS_TOKEN, newToken)
      return true
    } catch {
      // Refresh failed — clear expired session
      clearSession()
      return false
    }
  }

  async function logout(): Promise<void> {
    try {
      const ms = matrixService.value
      ms.stopClient()
      await ms.logout()
    } catch {
      // ignore
    } finally {
      clearSession()
    }
  }

  function clearSession(): void {
    userId.value = null
    accessToken.value = null
    refreshToken.value = null
    deviceId.value = null
    isLoggedIn.value = false
    ;[LS_KEYS.ACCESS_TOKEN, LS_KEYS.REFRESH_TOKEN, LS_KEYS.USER_ID, LS_KEYS.DEVICE_ID].forEach(
      (k) => localStorage.removeItem(k),
    )
    MatrixService.resetInstance()
  }

  function clearError(): void {
    error.value = null
  }

  return {
    // State
    userId,
    accessToken,
    refreshToken,
    deviceId,
    isLoggedIn,
    isLoading,
    error,
    // Actions
    login,
    register,
    loginWithToken,
    restoreSession,
    tryRefreshToken,
    logout,
    clearSession,
    clearError,
  }
})
