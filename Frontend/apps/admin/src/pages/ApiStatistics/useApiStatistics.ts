import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'

interface ApiStats {
  totalRequests: number
  successRequests: number
  failedRequests: number
  avgResponseTime: number
  trendData: Record<string, unknown>
  statusData: Record<string, unknown>
}

export function useApiStatistics() {
  const $q = useQuasar()

  const stats = ref<ApiStats>({
    totalRequests: 0,
    successRequests: 0,
    failedRequests: 0,
    avgResponseTime: 0,
    trendData: {},
    statusData: {}
  })

  async function loadStats(): Promise<void> {
    try {
      // 模拟获取数据
      await new Promise(resolve => setTimeout(resolve, 500))
      stats.value = {
        totalRequests: 10000,
        successRequests: 9500,
        failedRequests: 500,
        avgResponseTime: 120,
        trendData: {
          xAxis: { data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'] },
          series: [{ type: 'line', data: [100, 200, 150, 300, 250] }]
        },
        statusData: {
          series: [{
            type: 'pie',
            data: [
              { name: '200', value: 9500 },
              { name: '404', value: 300 },
              { name: '500', value: 200 }
            ]
          }]
        }
      }
    } catch {
      $q.notify({ type: 'negative', message: 'Failed to load statistics' })
    }
  }

  onMounted(() => {
    loadStats()
  })

  return {
    stats,
    loadStats
  }
}
