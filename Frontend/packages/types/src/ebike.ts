export interface GpsCoord {
  lng: number | string;
  lat: number | string;
}

export interface CarItem {
  code: string;
  provide?: string;
  status: number;
  speed?: unknown;
  gps: GpsCoord;
  time?: { start?: string; end?: string };
  alert?: string;
  remark?: string;
  check: boolean;
  points?: Array<Array<number>>;
  gps_type?: number;
}

export interface StorageItem {
  code: string;
  provide?: string;
  status: number;
  cur: number;
  sum?: number;
  gps: GpsCoord;
  check: boolean;
  points: Array<Array<number>>;
  alert?: string;
  remark?: string;
  gps_type?: number;
}

export interface AlertItem {
  date: string;
  title: string;
  link: string;
}

export interface OrderItem {
  code: string;
  type?: string;
  provide?: string;
  status?: number;
  speed?: unknown;
  time?: string;
  alert?: string;
  remark?: string;
}

export interface CarInfo {
  code: string;
  provide: string;
  speed: number;
  status: string;
  gps: { lng: string; lat: string };
  time: { start: string; end: string };
  alert: string;
  remark: string;
}

export interface StorageInfo {
  code: string;
  provide: string;
  status: string;
  gps: { lng: number; lat: number };
  sum: number;
  cur: number;
  alert: string;
  remark: string;
  points: Array<Array<number>>;
}

export interface TrackState<T> {
  model: string;
  options: string[];
  list: Map<string, T>;
  time?: { start: string; end: string };
}

export interface MapWebView {
  clear?: () => void;
  setCar?: (...args: unknown[]) => void;
  setCenter?: (lng: number | string, lat: number | string, zoom: number) => void;
  setPolygon?: (data: Array<Record<string, unknown>>) => void;
  setText?: (data: Array<Record<string, unknown>>) => void;
}

export interface SplitterState {
  model: number;
  show: boolean;
}

export interface SearchState {
  code: string;
  provide: string;
  status: { model: string; options: string[] };
  time?: { start: string; end: string };
}

export interface TableState {
  filter?: string;
  pagination?: {
    rowsPerPage: number;
    sortBy?: string;
    descending: boolean;
    options?: Array<number | string>;
  };
  columns: Array<Record<string, unknown>>;
  rows: Array<Record<string, unknown>>;
}

export function getStatusString(
  status: number,
  map: Record<number, string>,
): string {
  return map[status] ?? '未知';
}

export function convertCoord(
  lng: string | number,
  lat: string | number,
  gpsType: number,
): [number, number] {
  const gcoord = (window as Record<string, unknown>).gcoord as {
    transform: (
      pt: [number, number],
      from: unknown,
      to: unknown,
    ) => [number, number];
    WGS84: unknown;
    BD09: unknown;
  };
  if (gpsType === 1) {
    return gcoord.transform(
      [Number(lng), Number(lat)],
      gcoord.WGS84,
      gcoord.BD09,
    );
  }
  return [Number(lng), Number(lat)];
}

export const STATUS_MAP_CAR: Record<number, string> = {
  0: '离线',
  1: '停止中',
  2: '骑行中',
  3: '故障',
  4: '其他',
};

export const STATUS_MAP_STORAGE: Record<number, string> = {
  0: '停用',
  1: '启用',
  2: '维护',
  3: '其他',
};

export const STATUS_MAP_ORDER: Record<number, string> = {
  0: '正常运营',
  1: '关闭',
  2: '离线',
};
