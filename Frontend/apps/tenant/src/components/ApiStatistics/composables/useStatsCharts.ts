/**
 * @file useStatsCharts.ts
 * @description Composable for stats charts utility functions
 * @date 2026-09-12
 */

export function useStatsCharts() {
  function getMethodColor(method: string): string {
    const colors: Record<string, string> = {
      GET: 'green',
      POST: 'blue',
      PUT: 'orange',
      DELETE: 'red',
      PATCH: 'purple',
    };
    return colors[method] || 'grey';
  }

  function getDistributionColor(max: number): string {
    if (max < 100) return '#4caf50';
    if (max < 200) return '#ff9800';
    return '#f44336';
  }

  function getBaselineStatusColor(status: string): string {
    const colors: Record<string, string> = {
      critical: 'red',
      degraded: 'orange',
      normal: 'green',
    };
    return colors[status] || 'grey';
  }

  function formatTime(timeStr?: string): string {
    if (!timeStr) return '-';
    return new Date(timeStr).toLocaleString('zh-CN');
  }

  return {
    getMethodColor,
    getDistributionColor,
    getBaselineStatusColor,
    formatTime,
  };
}
