/**
 * @file matrix-sdk.d.ts
 * @description Module augmentation for matrix-js-sdk
 * Adds convenience method signatures used by the social app's MatrixService.
 */

import 'matrix-js-sdk';

declare module 'matrix-js-sdk' {
  interface MatrixClient {
    /**
     * Send a message to a room with optional threadId.
     * Convenience overload accepting a plain content object.
     */
    sendMessage(
      roomId: string,
      content: Record<string, unknown>,
      threadId?: string
    ): Promise<{ event_id: string }>;

    /**
     * Send a read receipt for an event.
     * Convenience overload accepting roomId and eventId directly.
     */
    sendReadReceipt(
      roomId: string,
      eventId: string
    ): Promise<unknown>;

    /**
     * Upload content to the media repository.
     * Convenience overload accepting File, Blob, or ArrayBuffer.
     */
    uploadContent(
      file: File | Blob | ArrayBuffer,
      opts?: Record<string, unknown>
    ): Promise<{ content_uri?: string; contentUri?: string }>;

    /**
     * Set account data for the current user.
     * Convenience overload accepting plain string event type.
     */
    setAccountData(
      eventType: string,
      content: Record<string, unknown>
    ): Promise<void>;
  }
}
