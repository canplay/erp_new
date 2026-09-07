/**
 * @file matrix-helpers.ts
 * @description Matrix 类型映射辅助函数 — 从 matrix.ts 拆分
 */

import type { Room, MatrixEvent } from 'matrix-js-sdk'
import type { RoomInfo, RoomMessage } from './matrix-types'

/** 获取房间 Topic */
export function getRoomTopic(room: Room): string | null {
  try {
    const state = room.currentState
    if (!state) return null
    const event = state.getStateEvents('m.room.topic', '')
    if (!event) return null
    const content = event.getContent()
    return content?.topic ?? null
  } catch {
    return null
  }
}

/** 将 Matrix Room 映射为 RoomInfo */
export function mapRoom(room: Room): RoomInfo {
  const name = room.name ?? room.getDefaultRoomName(room.getMyMembership() ?? '')
  return {
    roomId: room.roomId,
    name,
    topic: getRoomTopic(room),
    alias: room.getCanonicalAlias() ?? null,
    memberCount: room.getJoinedMemberCount(),
    isDirect: room.isElementVideoRoom(),
    lastEvent: room.getLastLiveEvent() ?? null,
    avatarUrl: room.getMxcAvatarUrl(),
    canonicalAlias: room.getCanonicalAlias() ?? null,
  }
}

/** 将 MatrixEvent 映射为 RoomMessage */
export function mapEvent(event: MatrixEvent): RoomMessage {
  return {
    eventId: event.getId()!,
    roomId: event.getRoomId()!,
    sender: event.getSender()!,
    content: event.getContent<Record<string, unknown>>(),
    type: event.getType(),
    timestamp: event.getTs(),
    status: (event as unknown as { status?: string }).status,
  }
}
