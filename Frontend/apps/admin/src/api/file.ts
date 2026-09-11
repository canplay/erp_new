/**
 * @file file.ts
 * @description 文件管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

/**
 * @brief 文件信息接口
 */
export interface FileInfo {
  id: number;
  name: string;
  path: string;
  size: number;
  mimeType: string;
  extension: string;
  folderId?: number;
  folderPath?: string;
  isFolder: boolean;
  isImage: boolean;
  thumbnail?: string;
  url?: string;
  shareToken?: string;
  shareUrl?: string;
  shareExpiry?: string;
  created_at: string;
  updated_at: string;
  created_by: {
    id: number;
    username: string;
  };
}

/**
 * @brief 文件夹信息接口
 */
export interface FolderInfo {
  id: number;
  name: string;
  parent_id?: number;
  path: string;
  fileCount: number;
  childCount: number;
  created_at: string;
  updated_at: string;
  created_by: {
    id: number;
    username: string;
  };
}

/**
 * @brief 分享信息接口
 */
export interface ShareInfo {
  token: string;
  url: string;
  expiry: string;
  password?: string;
  hasPassword: boolean;
}

/**
 * @brief 获取文件列表
 */
export function listFiles(params: {
  folderId?: number;
  keyword?: string;
  type?: string;
  page?: number;
  page_size?: number;
}) {
  return httpClient.get('/files', { params });
}

/**
 * @brief 获取文件夹列表
 */
export function listFolders(params: {
  parent_id?: number;
  keyword?: string;
}) {
  return httpClient.get('/files/folders', { params });
}

/**
 * @brief 获取文件/文件夹详情
 */
export function getFileInfo(id: number) {
  return httpClient.get(`/files/${id}`);
}

/**
 * @brief 创建文件夹
 */
export function createFolder(data: {
  name: string;
  parent_id?: number;
}) {
  return httpClient.post('/files/folders', data);
}

/**
 * @brief 重命名文件/文件夹
 */
export function renameFile(id: number, name: string) {
  return httpClient.put(`/files/${id}/rename`, { name });
}

/**
 * @brief 移动文件/文件夹
 */
export function moveFile(id: number, targetFolderId?: number) {
  return httpClient.put(`/files/${id}/move`, { folder_id: targetFolderId });
}

/**
 * @brief 删除文件/文件夹
 */
export function deleteFile(id: number) {
  return httpClient.delete(`/files/${id}`);
}

/**
 * @brief 批量删除文件
 */
export function deleteFiles(ids: number[]) {
  return httpClient.post('/files/batch-delete', { ids });
}

/**
 * @brief 获取上传签名
 */
export function getUploadSignature(data: {
  filename: string;
  size: number;
  mimeType: string;
  folderId?: number;
}) {
  return httpClient.post('/files/upload/signature', data);
}

/**
 * @brief 上传文件（带进度回调）
 */
 
export function uploadFile(formData: FormData, onProgress?: (percent: number) => void) {
  return httpClient.post('/files/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    onUploadProgress: (progressEvent: { loaded: number; total?: number }) => {
      if (onProgress && progressEvent.total) {
        const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total);
        onProgress(percent);
      }
    },
  });
}

/**
 * @brief 获取文件下载链接
 */
export function getDownloadUrl(id: number) {
  return httpClient.get(`/files/${id}/download`);
}

/**
 * @brief 创建分享链接
 */
export function createShare(data: {
  fileIds: number[];
  password?: string;
  expiryDays?: number;
}) {
  return httpClient.post('/files/share', data);
}

/**
 * @brief 获取分享信息
 */
export function getShareInfo(token: string, password?: string) {
  return httpClient.get(`/files/share/${token}`, {
    params: password ? { password } : undefined,
  });
}

/**
 * @brief 验证分享密码
 */
export function verifySharePassword(token: string, password: string) {
  return httpClient.post(`/files/share/${token}/verify`, { password });
}

/**
 * @brief 下载分享文件
 */
export function downloadSharedFile(token: string, fileId: number) {
  return httpClient.get(`/files/share/${token}/download/${fileId}`, {
    responseType: 'blob',
  });
}

/**
 * @brief 取消分享
 */
export function cancelShare(token: string) {
  return httpClient.delete(`/files/share/${token}`);
}

/**
 * @brief 获取图片缩略图
 */
export function getThumbnail(id: number, size: 'small' | 'medium' | 'large' = 'medium') {
  return httpClient.get(`/files/${id}/thumbnail`, {
    params: { size },
    responseType: 'blob',
  });
}

/**
 * @brief 复制文件
 */
export function copyFile(id: number, targetFolderId?: number) {
  return httpClient.post(`/files/${id}/copy`, { folder_id: targetFolderId });
}

/**
 * @brief 搜索文件
 */
export function searchFiles(params: {
  keyword: string;
  type?: string;
  dateFrom?: string;
  dateTo?: string;
  page?: number;
  page_size?: number;
}) {
  return httpClient.get('/files/search', { params });
}

/**
 * @brief 获取文件夹面包屑路径
 */
export function getFolderPath(folderId?: number) {
  return httpClient.get('/files/folders/path', {
    params: folderId !== undefined ? { folder_id: folderId } : {},
  });
}

/**
 * @brief 获取存储使用统计
 */
export function getStorageStats() {
  return httpClient.get('/files/stats');
}

/**
 * @brief 下载文件
 */
export function downloadFile(id: number) {
  return httpClient.get(`/files/${id}/download`, {
    responseType: 'blob',
  });
}

/**
 * @brief 获取文件预览URL
 */
export function getPreviewUrl(id: number) {
  return httpClient.get(`/files/${id}/preview`);
}

/**
 * @brief 批量移动文件
 */
export function batchMoveFiles(ids: number[], targetFolderId?: number) {
  return httpClient.post('/files/batch-move', { ids, folder_id: targetFolderId });
}

/**
 * @brief 批量复制文件
 */
export function batchCopyFiles(ids: number[], targetFolderId?: number) {
  return httpClient.post('/files/batch-copy', { ids, folder_id: targetFolderId });
}

/**
 * @brief 获取最近访问文件
 */
export function getRecentFiles(limit: number = 10) {
  return httpClient.get('/files/recent', { params: { limit } });
}

/**
 * @brief 收藏文件
 */
export function favoriteFile(id: number) {
  return httpClient.post(`/files/${id}/favorite`);
}

/**
 * @brief 取消收藏文件
 */
export function unfavoriteFile(id: number) {
  return httpClient.delete(`/files/${id}/favorite`);
}

/**
 * @brief 获取收藏文件列表
 */
export function getFavoriteFiles() {
  return httpClient.get('/files/favorites');
}