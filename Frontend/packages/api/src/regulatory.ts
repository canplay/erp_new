// @viewpoint platform-only
// 复用共享 alova 封装（packages/api），不再重复定义 class M
import { api, normalize } from '@erp-new-frontend-monorepo/api';
import type {
  RegulatoryExportRequest,
  RegulatoryExportResponse,
} from '@erp-new-frontend-monorepo/types';

export interface RegulatoryReceivePayload {
  referenceNo: string;
  title: string;
  content: string;
  source: string;
}

export interface RegulatoryFeedbackPayload {
  referenceNo: string;
  title: string;
  content: string;
  replyToReferenceNo?: string | null;
}

export interface RegulatoryReceiveRecordItem {
  id: string;
  referenceNo: string;
  title: string;
  content: string;
  source: string;
  receivedAt: string;
}

export interface RegulatoryFeedbackRecordItem {
  id: string;
  referenceNo: string;
  title: string;
  content: string;
  replyToReferenceNo?: string | null;
  feedbackAt: string;
}

/** export-records 返回：导出留痕 + 监管反馈（后端已持久化） */
export interface RegulatoryExportRecordItem {
  id: string;
  scope: string;
  month?: string | null;
  standard: string;
  fileUrl: string;
  generatedBy: string;
  generatedAt: string;
}

export interface RegulatoryExportRecordsResult {
  exports: RegulatoryExportRecordItem[];
  feedbacks: RegulatoryFeedbackRecordItem[];
}

export interface RegulatoryReceiveRecordsResponse {
  total: number;
  items: RegulatoryReceiveRecordItem[];
  page: number;
  size: number;
}

export const regulatoryApi = {
  export: (req: RegulatoryExportRequest) =>
    api.Post<RegulatoryExportResponse>('/api/v1/regulatory/export', req),
  /** 导出留痕+监管反馈查询（当前租户） */
  exportRecords: (scope?: string, month?: string) =>
    api
      .Post<RegulatoryExportRecordsResult>('/api/v1/regulatory/export-records', {
        ...(scope ? { scope } : {}),
        ...(month ? { month } : {}),
      })
      .then((r) => r as RegulatoryExportRecordsResult),
  receive: (data: RegulatoryReceivePayload) =>
    api.Post<{ id: string }>('/api/v1/regulatory/receive', { request: data }),
  receiveRecords: (page = 1, pageSize = 20) =>
    api
      .Get<RegulatoryReceiveRecordItem[]>('/api/v1/regulatory/receive-records', {
        pageNumber: page,
        pageSize,
      })
      .then((r) => normalize<RegulatoryReceiveRecordItem>(r, page, pageSize)),
  feedback: (data: RegulatoryFeedbackPayload) =>
    api.Post<{ id: string }>('/api/v1/regulatory/feedback', { request: data }),
};
