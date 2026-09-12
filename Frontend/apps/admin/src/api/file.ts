/**
 * @file file.ts
 * @description 文件管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

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
export async function listFiles(params: {
  folderId?: number;
  keyword?: string;
  type?: string;
  page?: number;
  page_size?: number;
}) {
  try {
    return await httpClient.get('/files', { params });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取文件夹列表
 */
export async function listFolders(params: {
  parent_id?: number;
  keyword?: string;
}) {
  try {
    return await httpClient.get('/files/folders', { params });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取文件/文件夹详情
 */
export async function getFileInfo(id: number) {
  try {
    return await httpClient.get(`/files/${id}`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 创建文件夹
 */
export async function createFolder(data: {
  name: string;
  parent_id?: number;
}) {
  try {
    return await httpClient.post('/files/folders', data);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 重命名文件/文件夹
 */
export async function renameFile(id: number, name: string) {
  try {
    return await httpClient.put(`/files/${id}/rename`, { name });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 移动文件/文件夹
 */
export async function moveFile(id: number, targetFolderId?: number) {
  try {
    return await httpClient.put(`/files/${id}/move`, { folder_id: targetFolderId });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 删除文件/文件夹
 */
export async function deleteFile(id: number) {
  try {
    return await httpClient.delete(`/files/${id}`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 批量删除文件
 */
export async function deleteFiles(ids: number[]) {
  try {
    return await httpClient.post('/files/batch-delete', { ids });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取上传签名
 */
export async function getUploadSignature(data: {
  filename: string;
  size: number;
  mimeType: string;
  folderId?: number;
}) {
  try {
    return await httpClient.post('/files/upload/signature', data);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 上传文件（带进度回调）
 */
 
export async function uploadFile(formData: FormData, onProgress?: (percent: number) => void) {
  try {
    return await httpClient.post('/files/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    onUploadProgress: (progressEvent: { loaded: number; total?: number }) => {
      if (onProgress && progressEvent.total) {
        const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total);
        onProgress(percent);
      }
    },  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取文件下载链接
 */
export async function getDownloadUrl(id: number) {
  try {
    return await httpClient.get(`/files/${id}/download`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 创建分享链接
 */
export async function createShare(data: {
  fileIds: number[];
  password?: string;
  expiryDays?: number;
}) {
  try {
    return await httpClient.post('/files/share', data);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取分享信息
 */
export async function getShareInfo(token: string, password?: string) {
  try {
    return await httpClient.get(`/files/share/${token}`, {
    params: password ? { password } : undefined,  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 验证分享密码
 */
export async function verifySharePassword(token: string, password: string) {
  try {
    return await httpClient.post(`/files/share/${token}/verify`, { password });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 下载分享文件
 */
export async function downloadSharedFile(token: string, fileId: number) {
  try {
    return await httpClient.get(`/files/share/${token}/download/${fileId}`, {
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 取消分享
 */
export async function cancelShare(token: string) {
  try {
    return await httpClient.delete(`/files/share/${token}`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取图片缩略图
 */
export async function getThumbnail(id: number, size: 'small' | 'medium' | 'large' = 'medium') {
  try {
    return await httpClient.get(`/files/${id}/thumbnail`, {
    params: { size },
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 复制文件
 */
export async function copyFile(id: number, targetFolderId?: number) {
  try {
    return await httpClient.post(`/files/${id}/copy`, { folder_id: targetFolderId });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 搜索文件
 */
export async function searchFiles(params: {
  keyword: string;
  type?: string;
  dateFrom?: string;
  dateTo?: string;
  page?: number;
  page_size?: number;
}) {
  try {
    return await httpClient.get('/files/search', { params });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取文件夹面包屑路径
 */
export async function getFolderPath(folderId?: number) {
  try {
    return await httpClient.get('/files/folders/path', {
    params: folderId !== undefined ? { folder_id: folderId } : {},  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取存储使用统计
 */
export async function getStorageStats() {
  try {
    return await httpClient.get('/files/stats');
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 下载文件
 */
export async function downloadFile(id: number) {
  try {
    return await httpClient.get(`/files/${id}/download`, {
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取文件预览URL
 */
export async function getPreviewUrl(id: number) {
  try {
    return await httpClient.get(`/files/${id}/preview`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 批量移动文件
 */
export async function batchMoveFiles(ids: number[], targetFolderId?: number) {
  try {
    return await httpClient.post('/files/batch-move', { ids, folder_id: targetFolderId });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 批量复制文件
 */
export async function batchCopyFiles(ids: number[], targetFolderId?: number) {
  try {
    return await httpClient.post('/files/batch-copy', { ids, folder_id: targetFolderId });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取最近访问文件
 */
export async function getRecentFiles(limit: number = 10) {
  try {
    return await httpClient.get('/files/recent', { params: { limit } });
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 收藏文件
 */
export async function favoriteFile(id: number) {
  try {
    return await httpClient.post(`/files/${id}/favorite`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 取消收藏文件
 */
export async function unfavoriteFile(id: number) {
  try {
    return await httpClient.delete(`/files/${id}/favorite`);
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}

/**
 * @brief 获取收藏文件列表
 */
export async function getFavoriteFiles() {
  try {
    return await httpClient.get('/files/favorites');
  } catch (error) {
    handleApiError(error, '文件管理');
    throw error;
  }
}