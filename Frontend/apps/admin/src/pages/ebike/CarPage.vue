<template>
  <q-page class="q-pa-md">
    <q-splitter
      v-model="splitter.model"
      :limits="[50, 99]"
      :disable="!splitter.show"
    >
      <template v-slot:before>
        <q-table
          :rows="table.rows"
          :columns="table.columns"
          row-key="code"
          v-model:pagination="table.pagination"
          :rows-per-page-options="[10, 20, 30, 40, 50, 0]"
        >
          <template v-slot:top>
            <div class="fit">
              <div class="row">
                <q-input class="col" v-model="search.code" :label="$t('common.identifier')" />

                <div class="col-auto" style="width: 5px" />

                <q-input class="col" v-model="search.provide" :label="$t('common.operator')" />

                <div class="col-auto" style="width: 5px" />

                <q-select
                  class="col"
                  v-model="search.status.model"
                  :options="search.status.options"
                  :label="$t('common.status')"
                />
              </div>

              <div style="height: 5px" />

              <div class="row">
                <q-input class="col" filled v-model="track.time!.start" :label="$t('common.startTime')">
                  <template v-slot:append>
                    <q-icon name="event" class="cursor-pointer">
                      <q-popup-proxy transition-show="scale" transition-hide="scale">
                        <q-date v-model="track.time!.start" mask="YYYY-M-D">
                          <div class="row items-center justify-end">
                            <q-btn v-close-popup :label="$t('common.close')" color="primary" flat />
                          </div>
                        </q-date>
                      </q-popup-proxy>
                    </q-icon>
                  </template>
                </q-input>

                <div class="col-auto" style="width: 5px" />

                <q-input class="col" filled v-model="track.time!.end" label="关闭时间">
                  <template v-slot:append>
                    <q-icon name="event" class="cursor-pointer">
                      <q-popup-proxy transition-show="scale" transition-hide="scale">
                        <q-date v-model="track.time!.end" mask="YYYY-M-D">
                          <div class="row items-center justify-end">
                            <q-btn v-close-popup :label="$t('common.close')" color="primary" flat />
                          </div>
                        </q-date>
                      </q-popup-proxy>
                    </q-icon>
                  </template>
                </q-input>

                <div class="col-auto" style="width: 5px" />

                <q-btn class="col-2" flat color="primary" :label="$t('common.export')" @click="onExport" />

                <div class="col-auto" style="width: 5px" />

                <q-btn class="col-2" color="primary" :label="$t('common.query')" @click="onSearch" />
              </div>
            </div>
          </template>

          <template v-slot:body="props">
            <q-tr :props="props">
              <q-td key="code" :props="props">
                <q-btn color="primary" :label="props.row.code" @click="onHistory(props.row.code)" />
              </q-td>
              <q-td key="provide" :props="props">
                {{ props.row.provide }}
              </q-td>
              <q-td key="status" :props="props">
                {{ props.row.status }}
              </q-td>
              <q-td key="speed" :props="props">
                {{ props.row.speed }}
              </q-td>
              <q-td key="time_start" :props="props">
                {{ props.row.time_start }}
              </q-td>
              <q-td key="time_end" :props="props">
                {{ props.row.time_end }}
              </q-td>
              <q-td key="alert" :props="props">
                {{ props.row.alert }}
              </q-td>
              <q-td key="remark" :props="props">
                {{ props.row.remark }}
              </q-td>
            </q-tr>
          </template>
        </q-table>
      </template>

      <template v-slot:after v-if="splitter.show">
        <q-inner-loading :showing="!init">
          <q-spinner-gears size="50px" color="primary" />
        </q-inner-loading>

        <div
          v-if="init"
          class="absolute full-width"
          style="background-color: rgba(255, 255, 255, 0.5); backdrop-filter: blur(2px)"
        >
          <div class="row">
            <q-input class="col" filled v-model="track.time!.start" :label="$t('common.startTime')">
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy transition-show="scale" transition-hide="scale">
                    <q-date v-model="track.time!.start" mask="YYYY-M-D">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup :label="$t('common.close')" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>

            <div class="col-auto" style="width: 5px" />

            <q-input class="col" filled v-model="track.time!.end" :label="$t('common.endTime')">
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy transition-show="scale" transition-hide="scale">
                    <q-date v-model="track.time!.end" mask="YYYY-M-D">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup :label="$t('common.close')" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </div>

          <q-select
            v-model="track.model"
            :options="track.options"
            label="历史记录"
            @input="onTrackChange"
          />
        </div>

        <div class="fit">
          <iframe class="fit" src="/ebike/map.html" frameborder="0" seamless ref="webview" style="width: 100%; height: 100vh; border: none;" />
        </div>
      </template>
    </q-splitter>
  </q-page>
</template>

<script setup lang="ts">
import type { QTableProps } from 'quasar';
import { date, exportFile, useQuasar } from 'quasar';
import gcoord from 'gcoord';
import { useEbikeStore as useStore } from '@/stores/ebike';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { httpClient as api } from '@/utils/alova';
import { parseCarResponse } from '@/utils/ebike';
import { STATUS_MAP_CAR, type CarInfo, type MapWebView, type TrackState } from '@/types/ebike';

const $q = useQuasar();
const store = useStore();

const init = ref(false);
const splitter = ref({
  model: 100,
  show: false,
});
const search = ref({
  code: '',
  provide: '',
  status: {
    model: '',
    options: ['', '离线', '停止中', '骑行中', '故障', '其他'],
  },
  time: {
    start: '',
    end: '',
  },
});
const track = ref<TrackState<CarInfo>>({
  model: '',
  options: [],
  list: new Map(),
  time: {
    start: '',
    end: '',
  },
});
const table = ref<{
  pagination: { rowsPerPage: number; sortBy: string; descending: boolean; options: Array<number | string> };
  columns: QTableProps['columns'];
  rows: Array<Record<string, unknown>>;
}>({
  pagination: {
    rowsPerPage: 10,
    sortBy: 'time_start',
    descending: false,
    options: [10, 20, 30, 40, 50, '全部'],
  },
  columns: [
    { name: 'code', align: 'center', label: '识别码', field: 'code', sortable: true },
    { name: 'provide', align: 'center', label: '运营商', field: 'provide', sortable: true },
    { name: 'status', align: 'center', label: '状态', field: 'status', sortable: true },
    { name: 'speed', align: 'center', label: '速度', field: 'speed', sortable: true },
    { name: 'time_start', align: 'center', label: '启动时间', field: 'time_start', sortable: true },
    { name: 'time_end', align: 'center', label: '关闭时间', field: 'time_end', sortable: true },
    { name: 'alert', align: 'center', label: '警告', field: 'alert' },
    { name: 'remark', align: 'center', label: '备注', field: 'remark' },
  ] as QTableProps['columns'],
  rows: [],
});

const webview = ref<MapWebView | null>(null);

function onMessage(event: MessageEvent): void {
  if (event.data.method === 'init') {
    init.value = true;
  }
}

function wrapCsvValue(val: unknown, formatFn?: (val: unknown, row?: unknown) => unknown): string {
  const formatted = formatFn !== undefined ? formatFn(val) : val;
  const result = formatted === undefined || formatted === null ? '' : JSON.stringify(formatted);
  return result.split('"').join('""');
}

function onExport(): void {
  let loadingTimer: ReturnType<typeof setTimeout> | null = null;
  loadingTimer = setTimeout(() => {
    $q.loading.hide();
    if (loadingTimer !== null) {
      clearTimeout(loadingTimer);
      loadingTimer = null;
    }
  }, 120000);

  $q.loading.show();

  const content = [table.value.columns!.map((col) => wrapCsvValue(col.label))]
    .concat(
      table.value.rows.map((row: Record<string, unknown>) =>
        table.value
          .columns!.map((col) =>
            wrapCsvValue(
              typeof col.field === 'function'
                ? col.field(row)
                : row[col.field === void 0 ? col.name : col.field],
              col.format,
            ),
          )
          .join(','),
      ),
    )
    .join('\r\n');

  const status = exportFile('车辆列表.xlsx', content, 'text/xlsx');

  if (status !== true) {
    $q.notify('正在准备下载，请稍后...');
  }

  $q.loading.hide();
  if (loadingTimer !== null) {
    clearTimeout(loadingTimer);
    loadingTimer = null;
  }
}

async function onSearch(): Promise<void> {
  table.value.rows = [];

  let statusValue = -1;
  switch (search.value.status.model) {
    case '离线': statusValue = 0; break;
    case '停止中': statusValue = 1; break;
    case '骑行中': statusValue = 2; break;
    case '故障': statusValue = 3; break;
    case '其他': statusValue = 4; break;
  }

  try {
    const resp = await api.post(store.backend.private + '/car', {
      method: 'query',
      code: search.value.code,
      provide: search.value.provide,
      status: statusValue,
      time: { start: search.value.time.start, end: search.value.time.end },
    });

    const cars = parseCarResponse(resp);

    if (!Array.isArray(cars)) return;

    for (const el of cars) {
      const elObj = el as Record<string, unknown>;
      if (!elObj || !elObj.code || elObj.code === '') break;

      const t = elObj.time as { start?: string; end?: string };
      table.value.rows.push({
        code: elObj.code,
        provide: elObj.provide,
        speed: Number(elObj.speed) || 0,
        status: STATUS_MAP_CAR[Number(elObj.status)] ?? '未知',
        time_start: t?.start,
        time_end: t?.end,
        alert: elObj.alert,
        remark: elObj.remark,
      });
    }
  } catch {
    // silent
  }
}

async function onHistory(code: string): Promise<void> {
  splitter.value.show = true;
  splitter.value.model = 50;

  track.value.time = { start: '', end: '' };
  track.value.model = '';
  track.value.options = [];
  track.value.list = new Map();

  webview.value?.clear?.();

  try {
    const resp = await api.post(store.backend.private + '/car', {
      method: 'history',
      code,
    });

    const cars = parseCarResponse(resp);

    if (!Array.isArray(cars)) return;

    for (let i = 0; i < cars.length; i++) {
      const el = cars[i] as Record<string, unknown>;
      if (!el || !el.code || el.code === '') break;

      const elGps = el.gps as { lng?: string; lat?: string };
      let pt: [number, number];
      if (el.gps_type === 1) {
        pt = gcoord.transform(
          [parseFloat(elGps.lng ?? '0'), parseFloat(elGps.lat ?? '0')],
          gcoord.WGS84,
          gcoord.BD09,
        );
      } else {
        pt = [parseFloat(elGps.lng ?? '0'), parseFloat(elGps.lat ?? '0')];
      }

      const label =
        date.formatDate(el.create_date as string, 'YYYY-MM-DD HH:mm:ss') + ' (' + i + ')';

      track.value.options.push(label);

      track.value.list.set(label, {
        code: el.code as string,
        provide: el.provide as string,
        speed: Number(el.speed) || 0,
        status: STATUS_MAP_CAR[Number(el.status)] ?? '未知',
        gps: { lng: String(pt[0]), lat: String(pt[1]) },
        time: el.time as { start: string; end: string },
        alert: el.alert as string,
        remark: el.remark as string,
      });
    }
  } catch {
    // silent
  }
}

function onTrackChange(val: string): void {
  const v = track.value.list.get(val);
  if (!v) return;

  webview.value?.clear?.();
  webview.value?.setCar?.(
    [
      {
        geometry: { type: 'Point', coordinates: [v.gps.lng, v.gps.lat] },
        properties: {
          code: v.code,
          info:
            '识别码：' + v.code +
            '<br />状态：' + v.status +
            '<br />运营商：' + v.provide +
            '<br />速度：' + v.speed +
            '<br />最近启动时间：' + v.time.start +
            '<br />最近关闭时间：' + v.time.end +
            '<br />警告：' + v.alert +
            '<br />备注：' + v.remark,
        },
      },
    ],
    [],
    [],
    [],
  );
  webview.value?.setCenter?.(v.gps.lng, v.gps.lat, 18);
}

onMounted(() => {
  window.addEventListener('message', onMessage);
  void onSearch();
});

onBeforeUnmount(() => {
  window.removeEventListener('message', onMessage);
});
</script>