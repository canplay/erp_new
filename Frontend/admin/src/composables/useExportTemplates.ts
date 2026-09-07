/**
 * @file useExportTemplates.ts
 * @description Export template management
 */

import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

export function useExportTemplates() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  async function downloadTemplate(filename: string = 'export_template') {
    try {
      const response = await fetch(`/templates/${filename}.csv`);
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${filename}.csv`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
      $q.notify({ type: 'positive', message: $t('user.templateDownloaded') });
    } catch (error) {
      console.error('【下载模板失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  return {
    downloadTemplate,
  };
}
