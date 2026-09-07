// Type-only imports — zero bundle cost at runtime
import type {
  MatrixClient,
  MatrixEvent,
  IPublicRoomsChunkRoom,
  AccountDataEvents,
  SetPresence,
} from 'matrix-js-sdk'
import { config } from './config'

// 从拆分文件导入类型和辅助函数
import type { AuthCredentials, LoginFlows, RoomInfo, RoomMessage, UserInfo, PaginatedResult, UploadResult } from './matrix-types'
import { mapRoom, mapEvent } from './matrix-helpers'

interface MatrixSdkExports {
  createClient: (...args: unknown[]) => MatrixClient
  IndexedDBCryptoStore: new (...args: unknown[]) => unknown
  MatrixEvent: new (...args: unknown[]) => unknown
  Direction: { Backward: string }
  Preset: { TrustedPrivateChat: string }
  EventType: { Reaction: string }
  RelationType: { Annotation: string }
  [key: string]: unknown
}

let sdkPromise: Promise<MatrixSdkExports> | null = null

function getSdk(): Promise<MatrixSdkExports> {
  if (!sdkPromise) {
    sdkPromise = import('matrix-js-sdk') as Promise<MatrixSdkExports>
  }
  return sdkPromise
}

// ─── Type Definitions ───────────────────────────────────────────

interface ClientWithExtraMethods {
  sendMessage(roomId: string, content: Record<string, unknown>, threadId?: string): Promise<{ event_id: string }>
  sendReadReceipt(roomId: string, eventId: string): Promise<unknown>
  uploadContent(file: File | Blob | ArrayBuffer, opts?: Record<string, unknown>): Promise<{ content_uri?: string; contentUri?: string }>
  setAccountData(eventType: string, content: Record<string, unknown>): Promise<void>
}

// ─── Mapping Helpers (imported from matrix-helpers.ts) ────────

// ─── MatrixService ──────────────────────────────────────────────

export class MatrixService {
  private static instance: MatrixService | null = null

  private client!: MatrixClient
  private userId_: string | null = null
  private accessToken_: string | null = null
  private refreshToken_: string | null = null
  private deviceId_: string | null = null
  private cryptoEnabled_ = false
  private cryptoStore_: unknown = null
  private initPromise: Promise<void>

  private constructor() {
    this.initPromise = this.init()
  }

  private async init(): Promise<void> {
    const sdk = await getSdk()
    this.cryptoStore_ = new sdk.IndexedDBCryptoStore(globalThis.indexedDB, String(10))
    this.client = sdk.createClient({
      baseUrl: config.tuwunelUrl,
      cryptoStore: this.cryptoStore_,
    })
  }

  private async ensureReady(): Promise<void> {
    await this.initPromise
  }

  static getInstance(): MatrixService {
    if (!MatrixService.instance) {
      MatrixService.instance = new MatrixService()
    }
    return MatrixService.instance
  }

  static resetInstance(): void {
    if (MatrixService.instance) {
      MatrixService.instance.stopClient()
      MatrixService.instance = null
    }
  }

  private async rebuildClient(options?: {
    baseUrl: string
    accessToken?: string
    userId?: string
    deviceId?: string
    refreshToken?: string
  }): Promise<void> {
    const sdk = await getSdk()
    this.client = sdk.createClient(options ?? { baseUrl: config.tuwunelUrl })
  }

  private async setSession(userId: string, accessToken: string, deviceId: string, refreshToken?: string): Promise<void> {
    this.userId_ = userId
    this.accessToken_ = accessToken
    this.deviceId_ = deviceId
    this.refreshToken_ = refreshToken ?? null
    await this.rebuildClient({
      baseUrl: config.tuwunelUrl,
      accessToken,
      userId,
      deviceId,
      ...(refreshToken !== undefined ? { refreshToken } : {}),
    })
  }

  private handleError(error: unknown): never {
    if (error instanceof Error) throw error
    throw new Error(String(error))
  }

  getClient(): MatrixClient {
    return this.client
  }

  getUserId(): string | null {
    return this.userId_
  }

  get isLoggedIn(): boolean {
    return this.accessToken_ !== null && this.userId_ !== null
  }

  // ── Auth ──

  async login(username: string, password: string): Promise<AuthCredentials> {
    await this.ensureReady()
    try {
      const res = await this.client.login('m.login.password', {
        user: username,
        password,
        initial_device_display_name: config.appName,
      })
      await this.setSession(res.user_id, res.access_token, res.device_id, res.refresh_token)
      return {
        userId: res.user_id,
        accessToken: res.access_token,
        refreshToken: res.refresh_token,
        deviceId: res.device_id,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  async register(
    username: string,
    password: string,
    options?: { displayName?: string },
  ): Promise<AuthCredentials> {
    await this.ensureReady()
    try {
      const res = await this.client.register(username, password, null, {
        initial_device_display_name: config.appName,
      } as unknown as Parameters<MatrixClient['register']>[3])
      await this.setSession(res.user_id, res.access_token!, res.device_id!, res.refresh_token)
      if (options?.displayName) {
        await this.client.setDisplayName(options.displayName).catch(() => {})
      }
      return {
        userId: res.user_id,
        accessToken: res.access_token!,
        refreshToken: res.refresh_token,
        deviceId: res.device_id!,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  async loginWithToken(token: string): Promise<AuthCredentials> {
    await this.ensureReady()
    try {
      const res = await this.client.login('m.login.token', {
        token,
        initial_device_display_name: config.appName,
      })
      await this.setSession(res.user_id, res.access_token, res.device_id, res.refresh_token)
      return {
        userId: res.user_id,
        accessToken: res.access_token,
        deviceId: res.device_id,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  async loginWithSession(accessToken: string, userId: string, deviceId: string, refreshToken?: string): Promise<void> {
    await this.ensureReady()
    await this.setSession(userId, accessToken, deviceId, refreshToken)
  }

  async refreshAuthToken(): Promise<string> {
    if (!this.refreshToken_) {
      throw new Error('No refresh token available')
    }
    await this.ensureReady()
    try {
      const res = await this.client.refreshToken(this.refreshToken_)
      this.accessToken_ = res.access_token
      if (res.refresh_token) {
        this.refreshToken_ = res.refresh_token
      }
      // Recreate client with new token
      await this.rebuildClient({
        baseUrl: config.tuwunelUrl,
        ...(this.accessToken_ !== null ? { accessToken: this.accessToken_ } : {}),
        ...(this.userId_ !== null ? { userId: this.userId_ } : {}),
        ...(this.deviceId_ !== null ? { deviceId: this.deviceId_ } : {}),
        ...(this.refreshToken_ !== null ? { refreshToken: this.refreshToken_ } : {}),
      })
      return this.accessToken_
    } catch (error) {
      this.handleError(error)
    }
  }

  async logout(): Promise<void> {
    await this.ensureReady()
    try {
      if (this.client) {
        await this.client.logout(true)
      }
    } catch {
      // ignore
    } finally {
      this.accessToken_ = null
      this.userId_ = null
      this.deviceId_ = null
      await this.rebuildClient({ baseUrl: config.tuwunelUrl })
    }
  }

  async getAvailableLoginFlows(): Promise<LoginFlows> {
    try {
      const flows = await this.client.loginFlows()
      return {
        password: flows.flows.some((f: { type: string }) => f.type === 'm.login.password'),
        sso: flows.flows.some((f: { type: string }) => f.type === 'm.login.sso' || f.type === 'm.login.cas'),
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  /** Get SSO login redirect URL */
  getSsoLoginUrl(redirectPath: string = '/'): string {
    const base = config.tuwunelUrl.replace(/\/+$/, '')
    const redirectUrl = `${window.location.origin}/#/login?sso_callback=1&redirect=${encodeURIComponent(redirectPath)}`
    return `${base}/_matrix/client/v3/login/sso/redirect?redirectUrl=${encodeURIComponent(redirectUrl)}`
  }

  // ── Client Lifecycle ──

  startClient(opts?: Record<string, unknown>): Promise<void> {
    return this.client.startClient({
      ...opts,
      initialSyncLimit: (opts?.initialSyncLimit as number) ?? 10,
    })
  }

  /**
   * Enable end-to-end encryption.
   * Must be called BEFORE startClient.
   */
  enableCrypto(): void {
    this.cryptoEnabled_ = true
  }

  get isCryptoEnabled(): boolean {
    return this.cryptoEnabled_
  }

  getCrypto(): unknown {
    return this.client.getCrypto()
  }

  stopClient(): void {
    this.client.stopClient()
  }

  // ── Rooms ──

  getRooms(): RoomInfo[] {
    try {
      return this.client.getRooms().map(mapRoom)
    } catch (error) {
      this.handleError(error)
    }
  }

  getRoom(roomId: string): RoomInfo | null {
    try {
      const room = this.client.getRoom(roomId)
      return room ? mapRoom(room) : null
    } catch (error) {
      this.handleError(error)
    }
  }

  async createRoom(options: Record<string, unknown>): Promise<string> {
    try {
      const res = await this.client.createRoom(options)
      return res.room_id
    } catch (error) {
      this.handleError(error)
    }
  }

  async createDirectRoom(userId: string): Promise<string> {
    const sdk = await getSdk()
    try {
      const res = await this.client.createRoom({
        is_direct: true,
        invite: [userId],
        preset: sdk.Preset.TrustedPrivateChat as never,
      })
      return res.room_id
    } catch (error) {
      this.handleError(error)
    }
  }

  async joinRoom(roomIdOrAlias: string, opts?: Record<string, unknown>): Promise<string> {
    try {
      const room = await this.client.joinRoom(roomIdOrAlias, opts)
      return room.roomId
    } catch (error) {
      this.handleError(error)
    }
  }

  async leaveRoom(roomId: string): Promise<void> {
    try {
      await this.client.leave(roomId)
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── Messages ──

  async getMessages(
    roomId: string,
    from?: string,
    limit: number = 30,
    dir?: string,
  ): Promise<PaginatedResult<RoomMessage>> {
    const sdk = await getSdk()
    const direction = dir ?? sdk.Direction.Backward
    try {
      const res = await this.client.createMessagesRequest(roomId, from ?? null, limit, direction as never)
      return {
        chunk: (res.chunk ?? []).map((event: unknown) => {
          return mapEvent(event as MatrixEvent)
        }),
        start: res.start,
        end: res.end,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  async sendMessage(roomId: string, content: Record<string, unknown>, threadId?: string): Promise<string> {
    try {
      let eventId: string
      const extClient = this.client as unknown as ClientWithExtraMethods
      if (threadId) {
        const res = await extClient.sendMessage(roomId, content, threadId)
        eventId = res.event_id
      } else {
        const res = await extClient.sendMessage(roomId, content)
        eventId = res.event_id
      }
      return eventId
    } catch (error) {
      this.handleError(error)
    }
  }

  async sendReadReceipt(roomId: string, eventId: string): Promise<void> {
    try {
      await (this.client as unknown as ClientWithExtraMethods).sendReadReceipt(roomId, eventId)
    } catch (error) {
      this.handleError(error)
    }
  }

  async sendTyping(roomId: string, isTyping: boolean, timeoutMs = 20000): Promise<void> {
    try {
      await this.client.sendTyping(roomId, isTyping, timeoutMs)
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── Reactions / Edit / Redact / Threads ──

  /** 发送 emoji reaction（m.annotation） */
  async sendReaction(roomId: string, eventId: string, emoji: string): Promise<string> {
    const sdk = await getSdk()
    try {
      const content = {
        'm.relates_to': {
          rel_type: sdk.RelationType.Annotation,
          event_id: eventId,
          key: emoji,
        },
      }
      // @ts-expect-error - Matrix SDK type mismatch for reaction events
      const sender = await this.client.sendEvent(roomId, sdk.EventType.Reaction, content)
      return (sender as { event_id?: string }).event_id ?? ''
    } catch (error) {
      this.handleError(error)
    }
  }

  /** 编辑消息（m.replace）*/
  async editMessage(roomId: string, eventId: string, newBody: string, htmlBody?: string): Promise<string> {
    try {
      const content: Record<string, unknown> = {
        body: newBody,
        msgtype: 'm.text',
        'm.new_content': {
          body: newBody,
          msgtype: 'm.text',
        },
        'm.relates_to': {
          rel_type: 'm.replace',
          event_id: eventId,
        },
      }
      if (htmlBody) {
        content.format = 'org.matrix.custom.html'
        content.formatted_body = htmlBody
        const newContent = content['m.new_content'] as Record<string, unknown>
        newContent.format = 'org.matrix.custom.html'
        newContent.formatted_body = htmlBody
      }
      const extClient = this.client as unknown as ClientWithExtraMethods
      const res = await extClient.sendMessage(roomId, content)
      return res.event_id
    } catch (error) {
      this.handleError(error)
    }
  }

  /** 删除/撤回消息 */
  async redactEvent(roomId: string, eventId: string, reason?: string): Promise<string> {
    try {
      const res = await this.client.redactEvent(roomId, eventId, undefined, reason !== undefined ? { reason } : undefined)
      return res.event_id
    } catch (error) {
      this.handleError(error)
    }
  }

  /** 获取消息的关系事件（reactions/edits/threads） */
  async fetchRelations(
    roomId: string,
    eventId: string,
    relationType: string | null,
    eventType?: string | null,
  ): Promise<MatrixEvent[]> {
    try {
      const res = await this.client.relations(roomId, eventId, relationType, eventType)
      return res.events ?? []
    } catch {
      return []
    }
  }

  // ── 成员管理 ──

  /** 邀请用户加入房间 */
  async inviteUser(roomId: string, userId: string): Promise<void> {
    try {
      await this.client.invite(roomId, userId)
    } catch (error) {
      this.handleError(error)
    }
  }

  /** 踢出房间成员 */
  async kickUser(roomId: string, userId: string, reason?: string): Promise<void> {
    try {
      await this.client.kick(roomId, userId, reason)
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── 公开房间搜索 ──

  /** 获取公开房间列表 */
  async getPublicRooms(limit = 50, since?: string): Promise<{ chunk: IPublicRoomsChunkRoom[]; total: number; nextBatch?: string | undefined }> {
    try {
      const res = await this.client.publicRooms({ limit, ...(since !== undefined ? { since } : {}) })
      return {
        chunk: res.chunk ?? [],
        total: res.total_room_count_estimate ?? 0,
        nextBatch: res.next_batch,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  /** 搜索公开房间 */
  async searchPublicRooms(term: string, limit = 20): Promise<{ chunk: IPublicRoomsChunkRoom[]; total: number }> {
    try {
      const res = await this.client.publicRooms({ limit, filter: { generic_search_term: term } })
      return {
        chunk: res.chunk ?? [],
        total: res.total_room_count_estimate ?? 0,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── 消息搜索 ──

  /** 搜索房间内的消息 */
  async searchMessages(roomId: string, query: string, limit = 20): Promise<Record<string, unknown>[]> {
    try {
      const res = await this.client.searchRoomEvents({
        term: query,
        filter: { rooms: [roomId], limit },
      })
      return (res as unknown as { results: Record<string, unknown>[] }).results ?? []
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── User / Profile ──

  async searchUserDirectory(
    query: string,
    limit: number = 20,
  ): Promise<{ results: UserInfo[]; limited: boolean }> {
    try {
      const res = await this.client.searchUserDirectory({ term: query, limit })
      return {
        results: (res.results ?? []).map((u: { user_id: string; display_name?: string; avatar_url?: string }) => ({
          userId: u.user_id,
          displayName: u.display_name,
          avatarUrl: u.avatar_url,
        })),
        limited: res.limited ?? false,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  async getUserProfile(userId: string): Promise<UserInfo> {
    try {
      const info = await this.client.getProfileInfo(userId)
      return {
        userId,
        displayName: info.displayname,
        avatarUrl: info.avatar_url,
      }
    } catch (error) {
      this.handleError(error)
    }
  }

  // ── Account Data ──

  async setAccountData(eventType: string, content: Record<string, unknown>): Promise<void> {
    try {
      await (this.client as unknown as ClientWithExtraMethods).setAccountData(eventType, content)
    } catch (error) {
      this.handleError(error)
    }
  }

  getAccountData(eventType: string): Record<string, unknown> | null {
    try {
      const event = this.client.getAccountData(eventType as keyof AccountDataEvents)
      return event?.getContent() ?? null
    } catch {
      return null
    }
  }

  async getAccountDataFromServer(eventType: string): Promise<Record<string, unknown> | null> {
    try {
      const data = await this.client.getAccountDataFromServer(eventType as keyof AccountDataEvents)
      return data as Record<string, unknown> | null
    } catch {
      return null
    }
  }

  // ── Presence ──

  async setPresence(presence: 'online' | 'offline' | 'unavailable'): Promise<void> {
    try {
      await this.client.setPresence({ presence })
    } catch {
      // ignore
    }
  }

  /** Get presence info for a user (requires sync to have populated it) */
  getPresence(userId: string): { presence: string; statusMsg?: string | undefined; lastActive?: number | undefined } | null {
    try {
      const user = this.client.getUser(userId)
      if (!user) return null
      return {
        presence: user.presence,
        statusMsg: (user as unknown as { statusMsg?: string }).statusMsg,
        lastActive: user.lastActiveAgo,
      }
    } catch {
      return null
    }
  }

  async setSyncPresence(presence: 'online' | 'offline' | 'unavailable'): Promise<void> {
    try {
      await this.client.setSyncPresence(presence as SetPresence)
    } catch {
      // ignore
    }
  }

  // ── Files ──

  async uploadContent(
    file: File | Blob | ArrayBuffer,
    opts?: Record<string, unknown>,
  ): Promise<UploadResult> {
    try {
      const extClient = this.client as unknown as ClientWithExtraMethods
      const res = await extClient.uploadContent(file, opts)
      return { contentUri: res.content_uri ?? res.contentUri ?? '' }
    } catch (error) {
      this.handleError(error)
    }
  }
}
