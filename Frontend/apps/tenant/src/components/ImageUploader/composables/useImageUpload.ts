/**
 * @file useImageUpload.ts
 * @description Composable for image upload logic with compression
 * @date 2026-09-12
 */

import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { httpClient } from '@/utils/alova';
import { logger } from '@/utils/logger';
import { handleApiError } from '@/utils/apiErrorHandler';

export interface ImageUploadOptions {
  uploadUrl?: string;
  maxSize?: number;
  maxWidth?: number;
  maxHeight?: number;
  enableCompress?: boolean;
  compressQuality?: number;
}

export function useImageUpload(options: ImageUploadOptions = {}) {
  const { t } = useI18n();
  const $q = useQuasar();

  const uploading = ref(false);
  const uploadProgress = ref(0);
  const compressedSize = ref<number | null>(null);

  const uploadUrl = options.uploadUrl || '/files/upload';
  const maxSize = options.maxSize || 5;
  const maxWidth = options.maxWidth || 1920;
  const maxHeight = options.maxHeight || 1920;
  const enableCompress = options.enableCompress ?? true;
  const compressQuality = options.compressQuality || 0.8;

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  }

  async function compressImage(file: File): Promise<File> {
    return new Promise((resolve) => {
      const img = new Image();
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');

      img.onload = () => {
        let { width, height } = img;
        if (width > maxWidth || height > maxHeight) {
          const ratio = Math.min(maxWidth / width, maxHeight / height);
          width = Math.round(width * ratio);
          height = Math.round(height * ratio);
        }

        canvas.width = width;
        canvas.height = height;
        ctx?.drawImage(img, 0, 0, width, height);

        canvas.toBlob(
          (blob) => {
            if (blob) {
              const compressedFile = new File([blob], file.name, { type: 'image/jpeg' });
              logger.info(`【图片压缩】${file.name}: ${formatFileSize(file.size)} -> ${formatFileSize(blob.size)}`);
              URL.revokeObjectURL(img.src);
              resolve(compressedFile);
            } else {
              resolve(file);
            }
          },
          'image/jpeg',
          compressQuality
        );
      };

      img.onerror = () => {
        URL.revokeObjectURL(img.src);
        resolve(file);
      };

      img.src = URL.createObjectURL(file);
    });
  }

  async function uploadImage(
    file: File,
    onProgress?: (progress: number) => void
  ): Promise<string> {
    uploading.value = true;
    uploadProgress.value = 0;

    try {
      let fileToUpload = file;
      if (enableCompress) {
        fileToUpload = await compressImage(file);
        if (fileToUpload !== file) {
          compressedSize.value = fileToUpload.size;
        }
      }

      const formData = new FormData();
      formData.append('file', fileToUpload);

      const response = await httpClient.post(uploadUrl, formData, {
        onUploadProgress: (progressEvent: { loaded: number; total?: number }) => {
          if (progressEvent.total) {
            uploadProgress.value = Math.round((progressEvent.loaded / progressEvent.total) * 100);
            onProgress?.(uploadProgress.value);
          }
        },
      }) as { data?: { url?: string; data?: { url?: string } } };

      const uploadedUrl = response.data?.data?.url || response.data?.url;
      return uploadedUrl || '';
    } catch (error) {
      handleApiError(error, '图片上传');
      throw error;
    } finally {
      uploading.value = false;
    }
  }

  function validateFile(file: File): { valid: boolean; message?: string } {
    if (!file.type.startsWith('image/')) {
      return { valid: false, message: t('uploader.imageOnly') || '请上传图片文件' };
    }

    const maxSizeBytes = maxSize * 1024 * 1024;
    if (file.size > maxSizeBytes) {
      return {
        valid: false,
        message: `${file.name}: ${t('uploader.fileTooLarge')} (${maxSize}MB)`,
      };
    }

    return { valid: true };
  }

  function notifyUploadResult(success: boolean) {
    $q.notify({
      type: success ? 'positive' : 'negative',
      message: success ? t('uploader.uploadSuccess') : t('uploader.uploadFailed'),
    });
  }

  return {
    uploading,
    uploadProgress,
    compressedSize,
    formatFileSize,
    compressImage,
    uploadImage,
    validateFile,
    notifyUploadResult,
  };
}
