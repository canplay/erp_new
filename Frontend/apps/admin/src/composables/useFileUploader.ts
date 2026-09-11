/**
 * @file useFileUploader.ts
 * @description 文件上传组件业务逻辑 composable - 支持切片上传、压缩、进度显示
 * @date 2026-07-08
 */

import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { httpClient } from '@/utils/alova';

// ============ 类型定义 ============

/**
 * @brief 文件项接口
 */
export interface UploadFile {
  file: File;
  name: string;
  size: number;
  progress: number;
  status: 'pending' | 'uploading' | 'success' | 'error';
  chunkSize?: number;
  compressed?: boolean;
  compressedSize?: number;
}

/**
 * @brief Props 接口
 */
export interface FileUploaderProps {
  modelValue?: string | string[];
  accept?: string;
  multiple?: boolean;
  maxSize?: number;
  maxFiles?: number;
  label?: string;
  icon?: string;
  uploadUrl?: string;
  chunkSize?: number;
  enableCompress?: boolean;
  compressMaxSize?: number;
  showDragHint?: boolean;
}

// ============ Composable ============

/**
 * @brief 文件上传 composable
 * @description 封装文件上传、切片、压缩、进度等全部业务逻辑
 */
export function useFileUploader(props: FileUploaderProps, emit: {
  (e: 'update:modelValue', value: string | string[]): void;
  (e: 'upload', files: File[]): void;
  (e: 'change', files: File[]): void;
  (e: 'progress', file: File, progress: number): void;
  (e: 'success', file: File, url: string): void;
  (e: 'error', file: File, error: Error): void;
}) {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // ============ 状态 ============
  const fileInputRef = ref<HTMLInputElement | null>(null);
  const uploading = ref(false);
  const files = ref<UploadFile[]>([]);
  const uploadedUrls = ref<string[]>([]);

  // ============ 计算属性 ============

  /** 总上传进度 */
  const totalProgress = computed(() => {
    if (files.value.length === 0) return 0;
    const total = files.value.reduce((sum, f) => sum + f.progress, 0);
    return Math.round(total / files.value.length);
  });

  // ============ 监听 ============
  watch(() => props.modelValue, (newVal) => {
    if (Array.isArray(newVal)) {
      uploadedUrls.value = newVal;
    } else if (newVal) {
      uploadedUrls.value = [newVal];
    } else {
      uploadedUrls.value = [];
    }
  }, { immediate: true });

  // ============ 方法 ============

  /** 触发文件选择对话框 */
  function triggerUpload() {
    fileInputRef.value?.click();
  }

  /** 处理文件选择变更 */
  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const selectedFiles = Array.from(target.files || []);

    if (selectedFiles.length === 0) return;

    // 验证文件数量
    if (props.multiple && files.value.length + selectedFiles.length > (props.maxFiles ?? 10)) {
      $q.notify({
        type: 'warning',
        message: $t('uploader.maxFilesReached', { count: props.maxFiles }),
      });
      return;
    }

    // 验证并添加文件
    for (const file of selectedFiles) {
      if (validateFile(file)) {
        files.value.push({
          file,
          name: file.name,
          size: file.size,
          progress: 0,
          status: 'pending',
        });
      }
    }

    // 触发上传
    if (files.value.length > 0) {
      void uploadFiles();
    }

    // 重置 input
    target.value = '';
  }

  /** 验证文件是否符合要求 */
  function validateFile(file: File): boolean {
    const maxSizeBytes = (props.maxSize ?? 50) * 1024 * 1024;
    if (file.size > maxSizeBytes) {
      $q.notify({
        type: 'negative',
        message: `${file.name}: ${$t('uploader.fileTooLarge')} (${props.maxSize}MB)`,
      });
      return false;
    }
    // 审计修复 (C4): accept 扩展名/MIME 白名单校验, 禁止 HTML/SVG 等危险类型
    const accept = props.accept;
    if (accept && accept.trim() && accept !== '*/*') {
      const allowed = accept
        .split(',')
        .map((a) => a.trim().toLowerCase())
        .filter(Boolean);
      const ext = file.name.includes('.') ? `.${file.name.split('.').pop()?.toLowerCase()}` : '';
      const mime = (file.type || '').toLowerCase();
      const ok = allowed.some((a) => {
        if (a.startsWith('.')) return a === ext;
        if (a.includes('/')) return a === mime || (a.endsWith('/*') && mime.startsWith(a.slice(0, -1)));
        return false;
      });
      if (!ok) {
        $q.notify({
          type: 'negative',
          message: `${file.name}: 文件类型不允许 (仅支持 ${accept})`,
        });
        return false;
      }
    }
    return true;
  }

  /** 压缩图片文件（仅图片且超过阈值时生效） */
  async function compressFile(file: File): Promise<File> {
    if (!file.type.startsWith('image/') || file.size < (props.compressMaxSize ?? 1) * 1024 * 1024) {
      return file;
    }

    return new Promise((resolve) => {
      const img = new Image();
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');

      img.onload = () => {
        let { width, height } = img;
        const maxDim = 1920;
        if (width > maxDim || height > maxDim) {
          if (width > height) {
            height = (height / width) * maxDim;
            width = maxDim;
          } else {
            width = (width / height) * maxDim;
            height = maxDim;
          }
        }

        canvas.width = width;
        canvas.height = height;
        ctx?.drawImage(img, 0, 0, width, height);

        canvas.toBlob(
          (blob) => {
            if (blob) {
              const compressedFile = new File([blob], file.name, { type: file.type });
              logger.info(`【压缩】${file.name}: ${formatFileSize(file.size)} -> ${formatFileSize(blob.size)}`);
              resolve(compressedFile);
            } else {
              resolve(file);
            }
          },
          file.type,
          0.8
        );
      };

      img.onerror = () => resolve(file);
      img.src = URL.createObjectURL(file);
    });
  }

  /** 将文件切片 */
  function chunkFile(file: File, chunkSize: number): Blob[] {
    const chunks: Blob[] = [];
    const size = file.size;
    const totalChunks = Math.ceil(size / chunkSize);

    for (let i = 0; i < totalChunks; i++) {
      const start = i * chunkSize;
      const end = Math.min(start + chunkSize, size);
      chunks.push(file.slice(start, end));
    }

    return chunks;
  }

  /** 上传全部待上传文件（支持切片） */
  async function uploadFiles() {
    if (files.value.length === 0) return;

    uploading.value = true;

    try {
      for (let i = 0; i < files.value.length; i++) {
        const uploadFile = files.value[i];
        if (!uploadFile || uploadFile.status === 'success') continue;

        uploadFile.status = 'uploading';

        // 1. 压缩文件（如启用）
        let fileToUpload = uploadFile.file;
        if (props.enableCompress) {
          fileToUpload = await compressFile(uploadFile.file);
          if (fileToUpload !== uploadFile.file) {
            uploadFile.compressed = true;
            uploadFile.compressedSize = fileToUpload.size;
          }
        }

        // 2. 判断是否需要切片
        const chunkSizeBytes = (props.chunkSize ?? 5) * 1024 * 1024;
        const needsChunking = (props.chunkSize ?? 5) > 0 && fileToUpload.size > chunkSizeBytes;

        let uploadedUrl: string;

        if (needsChunking) {
          uploadedUrl = await uploadInChunks(fileToUpload, uploadFile, chunkSizeBytes);
        } else {
          uploadedUrl = await uploadSingle(fileToUpload, uploadFile);
        }

        // 更新状态
        uploadFile.status = 'success';
        uploadFile.progress = 100;
        uploadedUrls.value.push(uploadedUrl);

        emit('success', uploadFile.file, uploadedUrl);
      }

      // 更新 v-model
      const emitValue = props.multiple ? uploadedUrls.value : uploadedUrls.value[0];
      if (emitValue !== undefined) {
        emit('update:modelValue', emitValue);
      }

      const uploadedFiles = files.value.map((f) => f.file);
      emit('upload', uploadedFiles);
      emit('change', uploadedFiles);

      $q.notify({
        type: 'positive',
        message: $t('uploader.uploadSuccess'),
      });

      // 清空已上传的文件（保留历史记录）
      files.value = files.value.filter((f) => f.status !== 'success');
    } catch (error) {
      logger.error('【上传失败】', error);
      // 审计修复 (C4): 失败项状态置为 error, 不再停留在 uploading
      for (const f of files.value) {
        if (f.status === 'uploading') {
          f.status = 'error';
          f.progress = 0;
        }
      }
      $q.notify({
        type: 'negative',
        message: $t('uploader.uploadFailed'),
      });
    } finally {
      uploading.value = false;
    }
  }

  /** 普通单文件上传 */
  async function uploadSingle(file: File, uploadFile: UploadFile): Promise<string> {
    const formData = new FormData();
    formData.append('file', file);

    const response = await httpClient.post(props.uploadUrl ?? '/files/upload', formData, {
      onUploadProgress: (progressEvent: { loaded: number; total?: number }) => {
        if (progressEvent.total) {
          const progress = Math.round((progressEvent.loaded / progressEvent.total) * 100);
          uploadFile.progress = progress;
          emit('progress', file, progress);
        }
      },
    }) as { data?: { url?: string; data?: { url?: string } } };

    // 审计修复 (C4): 响应无有效 url 即抛错, 禁止本地 blob URL 兜底(假成功)
    const uploadedUrl = response.data?.data?.url || response.data?.url;
    if (!uploadedUrl) {
      uploadFile.status = 'error';
      throw new Error(`上传失败: 响应缺少文件 URL (${file.name})`);
    }
    return uploadedUrl;
  }

  /** 切片上传 */
  async function uploadInChunks(file: File, uploadFile: UploadFile, chunkSize: number): Promise<string> {
    const chunks = chunkFile(file, chunkSize);
    const totalChunks = chunks.length;
    uploadFile.chunkSize = totalChunks;

    logger.info(`【切片上传】${file.name}: ${file.size} bytes, 分为 ${totalChunks} 片`);

    let uploadedUrl = '';

    for (let i = 0; i < chunks.length; i++) {
      const chunk = chunks[i];
      if (!chunk) continue;
      const formData = new FormData();
      formData.append('file', chunk);
      formData.append('chunkIndex', String(i));
      formData.append('totalChunks', String(totalChunks));
      formData.append('fileName', file.name);
      formData.append('fileSize', String(file.size));

      try {
        const response = await httpClient.post(`${props.uploadUrl ?? '/files/upload'}/chunk`, formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
          onUploadProgress: (progressEvent: { loaded: number; total: number }) => {
            if (progressEvent.total) {
              const chunkProgress = Math.round((progressEvent.loaded / progressEvent.total) * 100);
              const overallProgress = ((i + chunkProgress / 100) / totalChunks) * 100;
              uploadFile.progress = Math.round(overallProgress);
              emit('progress', file, Math.round(overallProgress));
            }
          },
        });

        const chunkResponse = response as { data?: { url?: string; data?: { url?: string } } };
        if (chunkResponse.data?.data?.url) {
          uploadedUrl = chunkResponse.data.data.url;
        } else if (i === chunks.length - 1) {
          // 审计修复 (C4): 末片仍无 url 即抛错, 禁止本地 blob URL 兜底(假成功)
          uploadFile.status = 'error';
          throw new Error(`分片上传失败: 末片响应缺少文件 URL (${file.name})`);
        }
      } catch (error) {
        logger.error(`【切片上传失败】第 ${i + 1} 片`, error);
        uploadFile.status = 'error';
        throw error;
      }
    }

    return uploadedUrl;
  }

  /** 移除待上传文件 */
  function removeFile(index: number) {
    files.value.splice(index, 1);
  }

  /** 获取文件类型图标名 */
  function getFileIcon(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    const iconMap: Record<string, string> = {
      jpg: 'image', jpeg: 'image', png: 'image', gif: 'image',
      webp: 'image', svg: 'image', bmp: 'image',
      mp4: 'video_file', avi: 'video_file', mov: 'video_file', wmv: 'video_file',
      mp3: 'audio_file', wav: 'audio_file', flac: 'audio_file',
      pdf: 'picture_as_pdf',
      doc: 'description', docx: 'description',
      xls: 'table_chart', xlsx: 'table_chart',
      ppt: 'slideshow', pptx: 'slideshow',
      zip: 'folder_zip', rar: 'folder_zip', '7z': 'folder_zip',
      tar: 'folder_zip', gz: 'folder_zip',
      txt: 'article', md: 'article',
      json: 'data_object', xml: 'data_object',
      html: 'code', css: 'code', js: 'code', ts: 'code', vue: 'code',
      py: 'code', java: 'code', cpp: 'code', c: 'code',
    };
    return iconMap[ext] || 'insert_drive_file';
  }

  /** 格式化文件大小 */
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  }

  /** 从 URL 提取文件名 */
  function getFileNameFromUrl(url: string): string {
    try {
      const urlObj = new URL(url);
      const pathname = urlObj.pathname;
      return pathname.substring(pathname.lastIndexOf('/') + 1) || url;
    } catch {
      return url;
    }
  }

  /** 复制 URL 到剪贴板 */
  async function copyUrl(url: string) {
    try {
      await navigator.clipboard.writeText(url);
      $q.notify({
        type: 'positive',
        message: $t('uploader.urlCopied'),
      });
    } catch {
      $q.notify({
        type: 'negative',
        message: $t('uploader.copyFailed'),
      });
    }
  }

  return {
    fileInputRef,
    uploading,
    files,
    uploadedUrls,
    totalProgress,
    triggerUpload,
    handleFileChange,
    removeFile,
    getFileIcon,
    formatFileSize,
    getFileNameFromUrl,
    copyUrl,
  };
}
