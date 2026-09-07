/**
 * @file matrix-types.ts
 * @description Matrix 协议类型定义 — 从 matrix.ts 拆分
 */

import type { IPublicRoomsChunkRoom, MatrixEvent } from 'matrix-js-sdk'

export interface AuthCredentials {
  userId: string
  accessToken: string
  refreshToken?: string | undefined
  deviceId: string
  homeServer?: string | undefined
}

export interface LoginFlows {
  password: boolean
  sso: boolean
}

export interface RoomInfo {
  roomId: string
  name: string
  topic: string | null
  alias: string | null
  memberCount: number
  isDirect: boolean
  lastEvent: MatrixEvent | null
  avatarUrl: string | null
  canonicalAlias: string | null
}

export interface RoomMessage {
  eventId: string
  roomId: string
  sender: string
  content: Record<string, unknown>
  type: string
  timestamp: number
  status?: string | undefined
}

export interface UserInfo {
  userId: string
  displayName?: string | undefined
  avatarUrl?: string | undefined
}

export interface PublicRoomResult {
  chunk: IPublicRoomsChunkRoom[]
  total: number
  nextBatch?: string | undefined
}

export interface PresenceResult {
  presence: string
  statusMsg?: string | undefined
  lastActive?: number | undefined
}

export interface PaginatedResult<T> {
  chunk: T[]
  start?: string | undefined
  end?: string | undefined
}

export interface UploadResult {
  contentUri: string
}
