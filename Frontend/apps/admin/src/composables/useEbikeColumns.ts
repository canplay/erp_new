import type { QTableProps } from 'quasar';

/**
 * Common table column definitions for all e-bike pages.
 * Each page imports and uses the columns relevant to its domain.
 */

export const carColumns: QTableProps['columns'] = [
  { name: 'code', label: '识别码', field: 'code' },
  { name: 'status', label: '状态', field: 'status' },
  { name: 'speed', label: '速度', field: 'speed' },
  { name: 'gps', label: '位置', field: 'gps' },
  { name: 'time', label: '时间', field: 'time' },
  { name: 'alert', label: '警告', field: 'alert' },
  { name: 'remark', label: '备注', field: 'remark' },
] as QTableProps['columns'];

export const storageColumns: QTableProps['columns'] = [
  { name: 'code', label: '识别码', field: 'code' },
  { name: 'provide', label: '运营商', field: 'provide' },
  { name: 'status', label: '状态', field: 'status' },
  { name: 'total_capacity', label: '车辆总数', field: 'total_capacity' },
  { name: 'current_count', label: '当前数量', field: 'current_count' },
  { name: 'capacity', label: '容量', field: 'capacity' },
  { name: 'condition', label: '状态', field: 'condition' },
  { name: 'location', label: '位置', field: 'location' },
] as QTableProps['columns'];

export const orderColumns: QTableProps['columns'] = [
  { name: 'code', label: '识别码', field: 'code' },
  { name: 'type', label: '类型', field: 'type' },
  { name: 'provide', label: '运营商', field: 'provide' },
  { name: 'status', label: '状态', field: 'status' },
  { name: 'speed', label: '速度', field: 'speed' },
  { name: 'time', label: '时间', field: 'time' },
  { name: 'alert', label: '警告', field: 'alert' },
  { name: 'remark', label: '备注', field: 'remark' },
] as QTableProps['columns'];
