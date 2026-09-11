/**
 * @file report.ts
 * @description 报表 Store
 */

import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  listReports,
  createReport,
  updateReport,
  deleteReport,
  executeReport,
  getReport,
  generateReport,
  type Report,
  type ReportListParam,
  type CreateReportParam,
  type UpdateReportParam,
} from '@/api/report';

export const useReportStore = defineStore('report', () => {
  const reports = ref<Report[]>([]);
  const total = ref(0);
  const loading = ref(false);
  const currentReport = ref<Report | null>(null);

  async function fetchReports(params?: ReportListParam) {
    loading.value = true;
    try {
      const response = await listReports(params);
      const resp = response as unknown as { data?: { list: Report[]; total: number } };
      if (resp?.data) {
        reports.value = resp.data.list;
        total.value = resp.data.total;
      }
    } finally {
      loading.value = false;
    }
  }

  async function addReport(data: CreateReportParam) {
    await createReport(data);
    void fetchReports();
  }

  async function editReport(id: string, data: UpdateReportParam) {
    await updateReport(id, data);
    void fetchReports();
  }

  async function removeReport(id: string) {
    await deleteReport(id);
    void fetchReports();
  }

  async function runReport(id: string) {
    await executeReport(id);
  }

  async function getReportDetail(id: string) {
    const res = await getReport(id);
    const resp = res as unknown as { data?: Report };
    if (resp?.data) {
      currentReport.value = resp.data;
    }
    return resp?.data;
  }

  async function genReport(id: string) {
    await generateReport(id);
  }

  return {
    reports,
    total,
    loading,
    currentReport,
    fetchReports,
    addReport,
    editReport,
    removeReport,
    runReport,
    getReportDetail,
    genReport,
  };
});
