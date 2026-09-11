<template>
  <div class="report-designer">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="analytics" class="q-mr-sm" />
        {{ $t('reportDesigner.pageTitle') }}
      </div>
      <q-space />
      <q-btn flat color="grey" icon="save" :label="$t('common.save')" @click="handleSave" />
      <q-btn color="primary" icon="preview" :label="$t('common.preview')" @click="handlePreview" />
    </div>

    <div class="row q-col-gutter-md">
      <!-- 左侧：组件面板 -->
      <div class="col-12 col-md-3">
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">{{ $t('reportDesigner.reportComponents') }}</div>
            <q-list>
              <q-expansion-item :label="$t('common.chartComponent')" icon="bar_chart" default-opened>
                <q-list class="q-pl-md">
                  <q-item v-for="comp in chartComponents" :key="comp.type" clickable v-ripple draggable="true" @dragstart="handleDragStart($event, comp)">
                    <q-item-section avatar><q-icon :name="comp.icon" :color="comp.color" /></q-item-section>
                    <q-item-section>{{ comp.label }}</q-item-section>
                  </q-item>
                </q-list>
              </q-expansion-item>
              <q-expansion-item :label="$t('common.dataComponent')" icon="table_chart" default-opened>
                <q-list class="q-pl-md">
                  <q-item v-for="comp in dataComponents" :key="comp.type" clickable v-ripple draggable="true" @dragstart="handleDragStart($event, comp)">
                    <q-item-section avatar><q-icon :name="comp.icon" :color="comp.color" /></q-item-section>
                    <q-item-section>{{ comp.label }}</q-item-section>
                  </q-item>
                </q-list>
              </q-expansion-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>

      <!-- 中间：报表设计区 -->
      <div class="col-12 col-md-6">
        <q-card flat bordered class="report-canvas">
          <q-card-section>
            <div class="row items-center q-mb-md">
              <q-input v-model="reportConfig.title" outlined dense :placeholder="$t('common.reportTitle')" class="col" />
              <q-btn flat round icon="more_vert">
                <q-menu>
                  <q-list style="min-width: 150px">
                    <q-item clickable v-close-popup @click="handleCopyConfig">
                      <q-item-section avatar><q-icon name="content_copy" /></q-item-section>
                      <q-item-section>{{ $t('reportDesigner.copyReport') }}</q-item-section>
                    </q-item>
                    <q-item clickable v-close-popup @click="handleExportConfig">
                      <q-item-section avatar><q-icon name="download" /></q-item-section>
                      <q-item-section>{{ $t('reportDesigner.exportConfig') }}</q-item-section>
                    </q-item>
                  </q-list>
                </q-menu>
              </q-btn>
            </div>

            <div class="report-design-area" @dragover.prevent @drop="handleDrop">
              <div v-if="reportConfig.widgets.length === 0" class="empty-state">
                <q-icon name="analytics" size="64px" color="grey-5" />
                <div class="text-h6 q-mt-md text-grey">{{ $t('reportDesigner.dragHereHint') }}</div>
              </div>

              <div v-else class="widget-list">
                <div v-for="(widget, index) in reportConfig.widgets" :key="widget.id" class="widget-item" :class="{ 'widget-item--selected': selectedWidgetId === widget.id }" @click="handleSelectWidget(widget)">
                  <div class="widget-drag-handle"><q-icon name="drag_indicator" size="sm" /></div>
                  <div class="widget-content">
                    <div class="widget-info">
                      <div class="text-subtitle2">{{ widget.title }}</div>
                      <q-badge :color="getWidgetColor(widget.type)" :label="widget.type" class="q-mt-xs" />
                    </div>
                  </div>
                  <div class="widget-actions">
                    <q-btn flat round size="sm" icon="edit" @click.stop="handleEditWidget(index)" />
                    <q-btn flat round size="sm" icon="delete" color="negative" @click.stop="handleDeleteWidget(index)" />
                  </div>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 右侧：属性配置 -->
      <div class="col-12 col-md-3">
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">{{ $t('reportDesigner.propertyConfig') }}</div>
            <div v-if="!selectedWidget" class="text-center q-pa-xl">
              <q-icon name="touch_app" size="48px" color="grey-5" />
              <div class="text-body2 text-grey q-mt-md">{{ $t('reportDesigner.selectComponentHint') }}</div>
            </div>
            <div v-else class="widget-config">
              <q-banner class="q-mb-md" rounded banner-class="bg-blue-1">
                <template v-slot:avatar><q-icon name="info" color="blue" /></template>
                <div class="text-caption">编辑组件：<strong>{{ selectedWidget.title }}</strong></div>
              </q-banner>
              <q-input v-model="selectedWidget.title" outlined dense :label="$t('reportDesigner.titleLabel')" class="q-mb-sm" />
              <q-select v-model="selectedWidget.type" :options="widgetTypeOptions" outlined dense :label="$t('reportDesigner.componentTypeLabel')" emit-value map-options class="q-mb-sm" />
              <q-input v-model.number="selectedWidget.height" outlined dense type="number" :label="$t('reportDesigner.heightLabel')" class="q-mb-sm" />
              <q-checkbox v-model="selectedWidget.showLegend" :label="$t('reportDesigner.showLegendLabel')" class="q-mb-sm" />
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 预览对话框 -->
    <q-dialog v-model="showPreviewDialog" maximized>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('reportDesigner.dialogTitle') }}</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section>
          <div class="text-h4 q-mb-lg">{{ reportConfig.title }}</div>
          <div class="preview-widgets">
            <div v-for="widget in reportConfig.widgets" :key="widget.id" class="preview-widget">
              <div class="text-subtitle1 q-mb-sm">{{ widget.title }}</div>
              <q-icon name="insert_chart" size="48px" color="grey-5" />
            </div>
          </div>
        </q-card-section>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

/**
 * @file ReportDesigner.vue
 * @description 报表设计器组件
 * @date 2026-04-04
 */

import { ref, reactive, computed } from 'vue';
import { useQuasar } from 'quasar';

const { t } = useI18n();
const $q = useQuasar();

const chartComponents = [
  { type: 'bar', label: t('reportDesigner.barChart'), icon: 'bar_chart', color: 'primary' },
  { type: 'line', label: t('reportDesigner.lineChart'), icon: 'show_chart', color: 'primary' },
  { type: 'pie', label: t('reportDesigner.pieChart'), icon: 'pie_chart', color: 'primary' },
];

const dataComponents = [
  { type: 'table', label: t('reportDesigner.dataTable'), icon: 'table_chart', color: 'info' },
  { type: 'stat', label: t('reportDesigner.statCard'), icon: 'numbers', color: 'info' },
];

const widgetTypeOptions = [
  { label: t('reportDesigner.barChart'), value: 'bar' },
  { label: t('reportDesigner.lineChart'), value: 'line' },
  { label: t('reportDesigner.pieChart'), value: 'pie' },
  { label: t('reportDesigner.dataTable'), value: 'table' },
  { label: t('reportDesigner.statCard'), value: 'stat' },
];

const WIDGET_COLORS: Record<string, string> = {
  bar: 'primary', line: 'primary', pie: 'primary',
  table: 'info', stat: 'info',
};

interface ReportWidget {
  id: string;
  type: string;
  title: string;
  height?: number;
  showLegend?: boolean;
  showDataLabel?: boolean;
  backgroundColor?: string;
  padding?: number;
  borderRadius?: number;
}

interface ReportConfig {
  id: string;
  title: string;
  widgets: ReportWidget[];
  created_at: string;
  updated_at: string;
}

const showPreviewDialog = ref(false);
const selectedWidgetId = ref<string | null>(null);

const reportConfig = reactive<ReportConfig>({
  id: `report_${Date.now()}`,
  title: '数据分析报表',
  widgets: [],
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
});

const selectedWidget = computed(() => {
  if (!selectedWidgetId.value) return null;
  return reportConfig.widgets.find((w) => w.id === selectedWidgetId.value) || null;
});

function getWidgetColor(type: string): string {
  return WIDGET_COLORS[type] || 'grey';
}

function handleDragStart(event: DragEvent, comp: { type: string; label: string }) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('widgetType', comp.type);
    event.dataTransfer.setData('widgetLabel', comp.label);
  }
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  const widgetType = event.dataTransfer?.getData('widgetType');
  const widgetLabel = event.dataTransfer?.getData('widgetLabel');
  if (widgetType) {
    reportConfig.widgets.push({
    id: `widget_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    type: widgetType,
    title: widgetLabel || t('reportDesigner.unnamedComponent'),
      height: 300,
      showLegend: true,
      showDataLabel: false,
      backgroundColor: '#ffffff',
      padding: 16,
      borderRadius: 8,
    });
  }
}

function handleSelectWidget(widget: ReportWidget) {
  selectedWidgetId.value = widget.id;
}

function handleEditWidget(index: number) {
  const widget = reportConfig.widgets[index];
  if (!widget) return;
  void $q.dialog({
    title: t('reportDesigner.editWidget'),
    message: t('reportDesigner.editWidgetMessage'),
    prompt: { model: widget.title, type: 'text' },
    cancel: true,
  }).onOk((title: string) => {
    widget.title = title;
  });
}

function handleDeleteWidget(index: number) {
  void $q.dialog({
    title: t('reportDesigner.confirmDelete'),
    message: t('reportDesigner.deleteConfirmMessage'),
    cancel: true,
  }).onOk(() => {
    reportConfig.widgets.splice(index, 1);
  });
}

function handleSave() {
  reportConfig.updated_at = new Date().toISOString();
  $q.notify({ type: 'positive', message: t('reportDesigner.configSaved') });
}

function handlePreview() {
  showPreviewDialog.value = true;
}

function handleCopyConfig() {
  void navigator.clipboard.writeText(JSON.stringify(reportConfig, null, 2));
  $q.notify({ type: 'positive', message: t('reportDesigner.copiedToClipboard') });
}

function handleExportConfig() {
  const blob = new Blob([JSON.stringify(reportConfig, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `report_config_${Date.now()}.json`;
  a.click();
  $q.notify({ type: 'positive', message: t('reportDesigner.exported') });
}
</script>

<style scoped>
.report-designer { padding: 16px; }
.report-canvas { min-height: 600px; }
.report-design-area {
  min-height: 400px;
  border: 2px dashed #e0e0e0;
  border-radius: 8px;
  padding: 16px;
  background: #fafafa;
}
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}
.widget-list { display: flex; flex-direction: column; gap: 12px; }
.widget-item {
  display: flex;
  align-items: center;
  padding: 16px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}
.widget-item:hover {
  border-color: #1976d2;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
.widget-item--selected {
  border-color: #1976d2;
  border-width: 2px;
  background: #e3f2fd;
}
.widget-drag-handle { cursor: grab; padding: 0 8px; color: #9e9e9e; }
.widget-content { flex: 1; display: flex; align-items: center; gap: 16px; }
.widget-info { flex: 1; }
.widget-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}
.widget-item:hover .widget-actions { opacity: 1; }
.widget-config { max-height: calc(100vh - 200px); overflow-y: auto; }
.preview-widgets {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 16px;
}
.preview-widget {
  padding: 16px;
  min-height: 200px;
  background: #f5f5f5;
  border-radius: 8px;
}
.body--dark .report-design-area,
.body--dark .widget-item,
.body--dark .report-canvas { background: #1e1e1e; }
.body--dark .widget-item { background: #2d2d2d; }
.body--dark .widget-item--selected { background: #1a237e; }
</style>
