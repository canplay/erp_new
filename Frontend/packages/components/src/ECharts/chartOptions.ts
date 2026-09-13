import { computed } from 'vue'

export const chartOptions: Array<{ label: string; value: string }> = [
  { label: '柱状图', value: 'bar' },
  { label: '折线图', value: 'line' },
  { label: '饼图', value: 'pie' },
  { label: '默认', value: 'default' }
]

export const chartConfig = {
  bar: computed(() => ({
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: []
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        type: 'bar',
        data: []
      }
    ]
  })),
  line: computed(() => ({
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: []
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        type: 'line',
        data: []
      }
    ]
  })),
  pie: computed(() => ({
    tooltip: {
      trigger: 'item'
    },
    series: [
      {
        type: 'pie',
        radius: ['40%', '70%'],
        data: []
      }
    ]
  })),
  default: computed(() => ({
    tooltip: {
      trigger: 'axis'
    },
    legend: {},
    xAxis: {
      type: 'category',
      data: []
    },
    yAxis: {
      type: 'value'
    },
    series: []
  }))
}
