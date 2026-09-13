<template>
  <div class="ebike-map-container">
    <div ref="mapContainer" class="map-wrapper"></div>
    <div v-if="mapLoadError" class="map-error">
      <q-icon name="warning" size="48px" color="orange" />
      <p>地图加载失败，请检查网络连接</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';

interface VehicleItem {
  code?: string;
  status?: number;
  info?: string;
  gps?: { lat?: number | string; lng?: number | string };
}
interface PolygonItem {
  code?: string;
  info?: string;
  gps?: {
    lat?: number | string;
    lng?: number | string;
    points?: Array<Array<number | string> | { lat?: number | string; lng?: number | string }>;
  };
}
interface AlertItem {
  code?: string;
  alert?: string;
  gps?: { lat?: number | string; lng?: number | string };
  [key: string]: unknown;
}

const props = defineProps<{
  vehicles?: VehicleItem[];
  polygons?: PolygonItem[];
  alerts?: AlertItem[];
}>();

interface MapMarker {
  position: [number, number];
  title?: string;
  content?: string;
}

interface MapPolygon {
  path: [number, number][];
}

interface MapInstance {
  add: (item: MapMarker | MapPolygon) => void;
  remove: (item: MapMarker | MapPolygon) => void;
  openInfoWindow: (content: string, position: [number, number]) => void;
  setCenter: (position: [number, number]) => void;
  setZoom: (zoom: number) => void;
  destroy: () => void;
}

let mapInstance: MapInstance | null = null;
const vehicleMarkers: MapMarker[] = [];
const polygonPolygons: MapPolygon[] = [];
const alertMarkers: MapMarker[] = [];

const mapContainer = ref<HTMLDivElement | null>(null);
const mapLoadError = ref(false);

const defaultCenter: [number, number] = [23.3833, 104.2500];
const defaultZoom = 13;

const AMap_KEY = 'CHANGE_ME_AMAP_KEY';
const AMap_SECRET = 'CHANGE_ME_AMAP_SECRET';

let AMapLoaded = false;
let scriptLoaded = false;

function loadAMapScript(): Promise<void> {
  return new Promise<void>((resolve) => {
    if (scriptLoaded) {
      resolve();
      return;
    }

    if (AMapLoaded) {
      scriptLoaded = true;
      resolve();
      return;
    }

    if ((window as unknown as Record<string, unknown>).AMap) {
      AMapLoaded = true;
      scriptLoaded = true;
      resolve();
      return;
    }

    const script = document.createElement('script');
    script.src = `https://webapi.amap.com/maps/v2.0?key=${AMap_KEY}&secret=${AMap_SECRET}`;
    script.onload = () => {
      AMapLoaded = true;
      scriptLoaded = true;
      resolve();
    };
    script.onerror = () => {
      mapLoadError.value = true;
      AMapLoaded = true;
      scriptLoaded = true;
      resolve();
    };
    document.head.appendChild(script);
  });
}

function initMap() {
  if (!mapContainer.value) return;

  const AMap = (window as unknown as Record<string, unknown>).AMap as {
    Map: new (el: HTMLDivElement, opts: Record<string, unknown>) => MapInstance;
  };

  if (!AMap?.Map) return;

  mapInstance = new AMap.Map(mapContainer.value, {
    center: defaultCenter,
    zoom: defaultZoom,
    zoomControl: true,
    mapStyle: 'amap://styles/dark',
  });

  renderVehicles(props.vehicles || []);
  renderPolygons(props.polygons || []);
  renderAlerts(props.alerts || []);
}

function renderVehicles(v: VehicleItem[]) {
  const inst = mapInstance;
  if (!inst) return;

  vehicleMarkers.forEach((m) => {
    inst.remove(m);
  });
  vehicleMarkers.length = 0;

  v.forEach((veh) => {
    const lat = Number(veh?.gps?.lat);
    const lng = Number(veh?.gps?.lng);
    if (!isFinite(lat) || !isFinite(lng)) return;

    const marker: MapMarker = {
      position: [lng, lat],
      title: veh.code || '',
    };

    vehicleMarkers.push(marker);
  });
}

function renderPolygons(p: PolygonItem[]) {
  const inst = mapInstance;
  if (!inst) return;

  polygonPolygons.forEach((poly) => {
    inst.remove(poly);
  });
  polygonPolygons.length = 0;

  p.forEach((poly) => {
    const rawPoints = poly?.gps?.points;
    const points: [number, number][] = Array.isArray(rawPoints) && rawPoints.length
      ? rawPoints.map((pt) => {
          if (Array.isArray(pt)) {
            return [Number(pt[0]) || 0, Number(pt[1]) || 0];
          }
          return [Number(pt.lat ?? 0) || 0, Number(pt.lng ?? 0) || 0];
        })
      : [[Number(poly?.gps?.lng ?? 0) || 0, Number(poly?.gps?.lat ?? 0) || 0]];

    if (points.length === 0 || points.some((pt) => pt.some((n) => !isFinite(n)))) return;

    const polygon: MapPolygon = {
      path: points,
    };

    polygonPolygons.push(polygon);
  });
}

function renderAlerts(a: AlertItem[]) {
  const inst = mapInstance;
  if (!inst) return;

  alertMarkers.forEach((m) => {
    inst.remove(m);
  });
  alertMarkers.length = 0;

  a.forEach((item) => {
    const lat = Number(item?.gps?.lat ?? 23.3833);
    const lng = Number(item?.gps?.lng ?? 104.2500);

    const marker: MapMarker = {
      position: [lng, lat],
      content: `<div style="width:10px;height:10px;background:#e74c3c;border-radius:50%;border:2px solid white;"></div>`,
    };

    alertMarkers.push(marker);
  });
}

function setCenter(gps: { lat?: number | string; lng?: number | string } | undefined, zoom = 19) {
  if (!mapInstance) return;
  const lat = Number(gps?.lat);
  const lng = Number(gps?.lng);
  if (!isFinite(lat) || !isFinite(lng)) return;
  mapInstance.setCenter([lng, lat]);
  mapInstance.setZoom(zoom);
}

watch(
  () => props.vehicles,
  (newV) => {
    if (mapInstance) renderVehicles(newV || []);
  },
  { deep: true },
);

watch(
  () => props.polygons,
  (newP) => {
    if (mapInstance) renderPolygons(newP || []);
  },
  { deep: true },
);

watch(
  () => props.alerts,
  (newA) => {
    if (mapInstance) renderAlerts(newA || []);
  },
  { deep: true },
);

defineExpose({
  setCenter,
});

onMounted(async () => {
  await loadAMapScript();
  initMap();
});

onBeforeUnmount(() => {
  if (mapInstance) {
    mapInstance.destroy();
    mapInstance = null;
  }
  vehicleMarkers.length = 0;
  polygonPolygons.length = 0;
  alertMarkers.length = 0;
});
</script>

<style scoped>
.ebike-map-container {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 400px;
}

.map-wrapper {
  width: 100%;
  height: 100%;
  background-color: #1a1a2e;
}

.map-error {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.7);
  color: #ff9800;
  z-index: 1000;
}

.map-error p {
  margin-top: 12px;
  font-size: 16px;
}
</style>