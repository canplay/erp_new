<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('kg_title') }}</div>
      <q-space />
      <q-btn
        v-if="view === 'graph'"
        flat
        color="primary"
        icon="arrow_back"
        :label="i18nT('kg_back_list')"
        @click="backToList"
      />
    </div>

    <!-- 列表视图（默认，天眼查式） -->
    <q-card v-if="view === 'list'" flat bordered class="erp-card">
      <q-card-section>
        <div class="row items-center q-mb-md">
          <q-input
            v-model="keyword"
            outlined
            dense
            clearable
            :placeholder="i18nT('kg_search_placeholder')"
            style="min-width: 260px"
            class="q-mr-md"
          >
            <template #prepend><q-icon name="search" /></template>
          </q-input>
          <div class="text-caption text-grey-7">
            {{ i18nT('kg_count_hint', { count: filteredNodes.length }) }}
          </div>
        </div>

        <q-table
          :rows="filteredNodes"
          :columns="columns"
          row-key="id"
          :loading="listLoading"
          :pagination="{ sortBy: 'type', descending: false, page: 1, rowsPerPage: 10 }"
          flat
          bordered
        >
          <template #body-cell-type="props">
            <q-td :props="props">
              <q-badge :style="{ background: colorOf(props.row.type) }">{{
                labelOf(props.row.type)
              }}</q-badge>
            </q-td>
          </template>
          <template #body-cell-title="props">
            <q-td :props="props">
              <div
                class="text-weight-medium cursor-pointer text-primary"
                @click="penetrate(props.row)"
              >
                {{ props.row.title }}
              </div>
              <div v-if="props.row.content" class="text-caption text-grey-7 ellipsis-2-lines">
                {{ props.row.content }}
              </div>
            </q-td>
          </template>
          <template #body-cell-category="props">
            <q-td :props="props">
              <span class="text-caption text-grey-6">{{ props.row.category || '—' }}</span>
            </q-td>
          </template>
          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn
                flat
                dense
                round
                color="primary"
                icon="hub"
                :title="i18nT('kg_penetrate')"
                @click="penetrate(props.row)"
              />
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 图谱视图（穿透） -->
    <template v-else>
      <q-card flat bordered class="erp-card q-mb-md">
        <q-card-section class="q-pb-none">
          <div class="row items-center">
            <div class="text-subtitle2">
              关联穿透：<span class="text-primary">{{ centerTitle }}</span>
              <q-badge
                v-if="centerType"
                class="q-ml-sm"
                :style="{ background: colorOf(centerType) }"
                >{{ labelOf(centerType) }}</q-badge
              >
            </div>
            <q-space />
            <div class="text-caption text-grey-7">{{ i18nT('kg_click_penetrate') }}</div>
          </div>
        </q-card-section>
        <q-card-section>
          <EChartBase
            v-if="nodes.length"
            :option="graphOption"
            height="500px"
            @click="onNodeClick"
          />
          <EmptyState
            v-else
            icon="inbox"
            :title="i18nT('kg_no_relation')"
            :hint="i18nT('kg_no_relation_hint')"
          />
        </q-card-section>
      </q-card>

      <q-card v-if="selectedNode" flat bordered class="erp-card q-mb-md">
        <q-card-section>
          <div class="row items-center q-mb-sm">
            <q-badge :style="{ background: colorOf(selectedNode.type) }">{{
              labelOf(selectedNode.type)
            }}</q-badge>
            <div class="text-subtitle1 q-ml-sm">{{ selectedNode.title }}</div>
            <q-space />
            <q-btn
              flat
              dense
              round
              icon="close"
              color="grey"
              :title="i18nT('kg_close_detail')"
              @click="selectedNode = null"
            />
          </div>
          <div v-if="selectedNode.content" class="text-body2 text-grey-8">
            {{ selectedNode.content }}
          </div>
          <div v-if="selectedNode.category" class="text-caption text-grey-6 q-mt-sm">
            {{ i18nT('kg_category') }}：{{ selectedNode.category }}
          </div>
          <div v-if="selectedEdges.length" class="q-mt-md">
            <div class="text-caption text-grey-7 q-mb-xs">
              {{ i18nT('kg_relation_count', { count: selectedEdges.length }) }}
            </div>
            <div v-for="(e, i) in selectedEdges" :key="i" class="text-caption q-mb-xs">
              {{ nodeTitle(e.source) }}
              <q-badge outline color="grey-7">{{ EDGE_LABELS[e.label] ?? e.label }}</q-badge>
              {{ nodeTitle(e.target) }}
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="erp-card">
        <q-card-section class="q-pb-none">
          <div class="text-subtitle2">
            {{ i18nT('kg_related_nodes', { count: nodes.length })
            }}<span class="text-caption text-grey-6">— {{ i18nT('kg_click_continue') }}</span>
          </div>
        </q-card-section>
        <q-card-section>
          <q-list v-if="nodes.length" separator>
            <q-item v-for="n in nodes" :key="n.id" clickable v-ripple @click="penetrate(n)">
              <q-item-section avatar>
                <q-badge :style="{ background: colorOf(n.type) }">{{ labelOf(n.type) }}</q-badge>
              </q-item-section>
              <q-item-section>
                <q-item-label :class="{ 'text-primary': n.id === centerId }">{{
                  n.title
                }}</q-item-label>
                <q-item-label v-if="n.content" caption class="text-grey-7 ellipsis">{{
                  n.content
                }}</q-item-label>
              </q-item-section>
            </q-item>
          </q-list>
          <EmptyState v-else icon="inbox" :title="i18nT('kg_no_node')" />
        </q-card-section>
      </q-card>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar, type QTableColumn } from 'quasar';
import { EChartBase, EmptyState } from '@erp-new-frontend-monorepo/components';
import { searchApi, knowledgeGraphApi } from '@erp-new-frontend-monorepo/api';
import type {
  KnowledgeGraphNode,
  KnowledgeGraphEdge,
  EChartEventParams,
} from '@erp-new-frontend-monorepo/types';

const $q = useQuasar();

const view = ref<'list' | 'graph'>('list');
const keyword = ref('');
const listLoading = ref(false);
const allNodes = ref<KnowledgeGraphNode[]>([]);

const nodes = ref<KnowledgeGraphNode[]>([]);
const edges = ref<KnowledgeGraphEdge[]>([]);
const selectedNode = ref<KnowledgeGraphNode | null>(null);
const centerId = ref('');
const centerTitle = ref('');
const centerType = ref('');

const TYPE_LABELS: Record<string, string> = {
  Regulation: '法规',
  DeviceType: '设备类型',
  Fault: '故障',
  AccidentCase: '事故案例',
  OperationRule: '操作规范',
  Device: '设备实例',
  HiddenDanger: '隐患实例',
};

const TYPE_COLORS: Record<string, string> = {
  Regulation: '#f59e0b',
  DeviceType: '#3b82f6',
  Fault: '#ef4444',
  AccidentCase: '#8b5cf6',
  OperationRule: '#10b981',
  Device: '#0ea5e9',
  HiddenDanger: '#f97316',
};

const EDGE_LABELS: Record<string, string> = {
  regulates: '规定',
  has_fault: '常见故障',
  caused_by: '导致',
  requires_rule: '需规范',
  applies_to: '适用于',
  instance_of: '属于',
  located_on: '位于',
};

const columns: QTableColumn[] = [
  { name: 'type', label: '类型', field: 'type', align: 'left', sortable: true },
  { name: 'title', label: '标题', field: 'title', align: 'left' },
  { name: 'category', label: '分类', field: 'category', align: 'left', sortable: true },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' },
];

function labelOf(t: string) {
  return TYPE_LABELS[t] ?? t;
}
function colorOf(t: string) {
  return TYPE_COLORS[t] ?? '#9ca3af';
}

const filteredNodes = computed(() => {
  const k = keyword.value.trim().toLowerCase();
  if (!k) return allNodes.value;
  return allNodes.value.filter((n) =>
    [n.title, n.content, n.category, labelOf(n.type)].some((f) =>
      String(f ?? '')
        .toLowerCase()
        .includes(k),
    ),
  );
});

const graphOption = computed(() => {
  const categories = Object.keys(TYPE_LABELS).map((name) => ({
    name,
    itemStyle: { color: TYPE_COLORS[name] },
    label: { show: true },
  }));
  return {
    tooltip: {
      formatter: (p: EChartEventParams) => {
        if (p.dataType === 'edge') return EDGE_LABELS[p.data?.label ?? ''] ?? p.data?.label ?? '';
        const n = nodes.value.find((x) => x.id === p.data?.id);
        return n ? `${labelOf(n.type)}：${n.title}` : p.name;
      },
    },
    legend: [
      {
        data: categories.map((c) => c.name),
        formatter: (name: string) => TYPE_LABELS[name] ?? name,
      },
    ],
    series: [
      {
        type: 'graph',
        layout: 'force',
        roam: true,
        draggable: true,
        label: { show: true, position: 'right', fontSize: 12 },
        edgeLabel: {
          show: true,
          fontSize: 10,
          formatter: (p: EChartEventParams) => EDGE_LABELS[p.data?.label ?? ''] ?? '',
        },
        data: nodes.value.map((n) => ({
          id: n.id,
          name: n.title,
          category: n.type,
          symbolSize: n.id === centerId.value ? 46 : 30,
          itemStyle:
            n.id === centerId.value ? { borderColor: '#e11d48', borderWidth: 3 } : undefined,
        })),
        links: edges.value.map((e) => ({ source: e.source, target: e.target, label: e.label })),
        categories,
        force: { repulsion: 300, edgeLength: [80, 160], gravity: 0.1 },
        emphasis: { focus: 'adjacency' },
        lineStyle: { color: 'source', curveness: 0.15 },
      },
    ],
  };
});

const selectedEdges = computed(() => {
  if (!selectedNode.value) return [];
  return edges.value.filter(
    (e) => e.source === selectedNode.value!.id || e.target === selectedNode.value!.id,
  );
});

function nodeTitle(id: string) {
  return nodes.value.find((x) => x.id === id)?.title ?? '(未命名)';
}

async function loadList() {
  listLoading.value = true;
  try {
    // 使用 Meilisearch 搜索知识图谱节点（解决 Dgraph 中文分词差的问题）
    const r = await searchApi.search({
      query: keyword.value || '',
      page: 1,
      pageSize: 100,
      types: ['knowledge'],
    });
    allNodes.value = r.items.map((item): KnowledgeGraphNode => ({
      id: item.id,
      type: item.category ?? 'knowledge',
      title: item.title,
      content: item.content ?? '',
      category: item.category ?? '',
    }));
  } catch {
    allNodes.value = [];
    $q.notify({
      type: 'warning',
      message: '知识图谱服务暂时不可用，请稍后重试',
      position: 'top',
      timeout: 3000,
    });
  } finally {
    listLoading.value = false;
  }
}

/** 穿透：点击节点 → 展开其 1-hop 关联，切换到图谱视图 */
async function penetrate(node: KnowledgeGraphNode) {
  view.value = 'graph';
  centerId.value = node.id;
  centerTitle.value = node.title;
  centerType.value = node.type;
  selectedNode.value = node;
  await loadExpand(node.id);
}

async function loadExpand(uid: string) {
  try {
    const r = await knowledgeGraphApi.expand(uid);
    nodes.value = r.nodes ?? [];
    edges.value = r.edges ?? [];
  } catch {
    nodes.value = [];
    edges.value = [];
    $q.notify({
      type: 'warning',
      message: '关联穿透失败，请稍后重试',
      position: 'top',
      timeout: 3000,
    });
  }
}

function onNodeClick(params: EChartEventParams) {
  if (params?.dataType === 'node' && params.data?.id) {
    const n = nodes.value.find((x) => x.id === params.data?.id);
    if (n) {
      selectedNode.value = n;
      // 点击非中心节点 → 继续穿透
      if (n.id !== centerId.value) {
        void penetrate(n);
      }
    }
  }
}

function backToList() {
  view.value = 'list';
  selectedNode.value = null;
  centerId.value = '';
  nodes.value = [];
  edges.value = [];
}

onMounted(() => {
  void loadList();
});
</script>
