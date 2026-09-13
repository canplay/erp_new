/**
 * @file useWorkflowDesign.ts
 * @description WorkflowDesignPage 业务逻辑 composable
 */

import { ref, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useQuasar } from 'quasar';
import { getWorkflow, updateWorkflow, publishWorkflow } from '@/api/workflow';

interface WorkflowNode {
  id: string; name: string; type: 'start' | 'end' | 'task' | 'approval' | 'condition';
  x: number; y: number; timeout?: number; config?: Record<string, unknown>;
}

interface WorkflowEdge {
  id: string; sourceNodeId: string; targetNodeId: string;
  sourceX: number; sourceY: number; targetX: number; targetY: number;
  label?: string; condition?: string;
}

export function useWorkflowDesign() {
  const route = useRoute();
  const $q = useQuasar();
  const workflowId = route.params.id as string;

  const nodeTypes = [
    { type: 'start', label: '开始节点', icon: 'play_circle', color: 'positive' },
    { type: 'end', label: '结束节点', icon: 'stop_circle', color: 'negative' },
    { type: 'task', label: '任务节点', icon: 'task', color: 'primary' },
    { type: 'approval', label: '审批节点', icon: 'approval', color: 'warning' },
    { type: 'condition', label: '条件节点', icon: 'alt_route', color: 'info' },
  ];

  const nodeTypeOptions = nodeTypes.map((n) => ({ label: n.label, value: n.type }));

  const workflowName = ref('');
  const nodes = ref<WorkflowNode[]>([]);
  const edges = ref<WorkflowEdge[]>([]);
  const selectedNode = ref<WorkflowNode | null>(null);
  const selectedEdge = ref<WorkflowEdge | null>(null);
  const isDragOver = ref(false);
  let nodeIdCounter = 0;
  let edgeIdCounter = 0;

  const approvalConfig = ref({ assignee: '', approvalType: '单人审批' });
  const conditionConfig = ref({ expression: '' });

  watch(selectedNode, (node) => {
    if (node?.type === 'approval' && node.config) {
      approvalConfig.value.assignee = (node.config.assignee as string) || '';
      approvalConfig.value.approvalType = (node.config.approvalType as string) || '单人审批';
    } else if (node?.type === 'condition' && node.config) {
      conditionConfig.value.expression = (node.config.expression as string) || '';
    }
  }, { immediate: true });

  const history = ref<{ nodes: WorkflowNode[]; edges: WorkflowEdge[] }[]>([]);
  const historyIndex = ref(-1);

  function saveHistory() {
    history.value = history.value.slice(0, historyIndex.value + 1);
    history.value.push({ nodes: JSON.parse(JSON.stringify(nodes.value)), edges: JSON.parse(JSON.stringify(edges.value)) });
    historyIndex.value = history.value.length - 1;
  }

  function handleUndo() {
    if (historyIndex.value > 0) {
      historyIndex.value--;
      const state = history.value[historyIndex.value];
      if (!state) return;
      nodes.value = JSON.parse(JSON.stringify(state.nodes));
      edges.value = JSON.parse(JSON.stringify(state.edges));
      selectedNode.value = null; selectedEdge.value = null;
      $q.notify({ type: 'info', message: '已撤销' });
    } else { $q.notify({ type: 'warning', message: '没有可撤销的操作' }); }
  }

  function handleRedo() {
    if (historyIndex.value < history.value.length - 1) {
      historyIndex.value++;
      const state = history.value[historyIndex.value];
      if (!state) return;
      nodes.value = JSON.parse(JSON.stringify(state.nodes));
      edges.value = JSON.parse(JSON.stringify(state.edges));
      selectedNode.value = null; selectedEdge.value = null;
      $q.notify({ type: 'info', message: '已重做' });
    } else { $q.notify({ type: 'warning', message: '没有可重做的操作' }); }
  }

  function handleValidate() {
    const errors: string[] = [];
    const startNodes = nodes.value.filter((n) => n.type === 'start');
    const endNodes = nodes.value.filter((n) => n.type === 'end');
    if (startNodes.length === 0) errors.push('缺少开始节点');
    if (endNodes.length === 0) errors.push('缺少结束节点');
    if (nodes.value.length > 0 && edges.value.length === 0) errors.push('节点之间没有连线');
    const connected = new Set<string>();
    edges.value.forEach((e) => { connected.add(e.sourceNodeId); connected.add(e.targetNodeId); });
    const isolated = nodes.value.filter((n) => !connected.has(n.id));
    if (isolated.length > 0 && nodes.value.length > 1) errors.push(`孤立节点: ${isolated.map((n) => n.name).join(', ')}`);
    nodes.value.forEach((n) => { if (n.type === 'approval' && (!n.config?.assignee)) errors.push(`${n.name} 未设置审批人`); });
    if (errors.length === 0) $q.notify({ type: 'positive', message: '验证通过！' });
    else $q.notify({ type: 'negative', message: `验证失败: ${errors.join('; ')}`, timeout: 5000 });
  }

  function generateNodeId(): string { return `node_${++nodeIdCounter}`; }
  function generateEdgeId(): string { return `edge_${++edgeIdCounter}`; }
  function getNodeIcon(type: string): string { return nodeTypes.find((n) => n.type === type)?.icon || 'circle'; }
  function getNodeColor(type: string): string { return nodeTypes.find((n) => n.type === type)?.color || 'grey'; }
  function getOutputPorts(node: WorkflowNode): string[] { return node.type === 'end' ? [] : ['output']; }

  function onDragStart(event: DragEvent, node: { type: string; label: string }) {
    event.dataTransfer?.setData('nodeType', node.type);
    event.dataTransfer?.setData('nodeLabel', node.label);
  }

  function onDragOver() { isDragOver.value = true; }
  function onDragLeave() { isDragOver.value = false; }

  function onDrop(event: DragEvent) {
    isDragOver.value = false;
    const nodeType = event.dataTransfer?.getData('nodeType');
    const nodeLabel = event.dataTransfer?.getData('nodeLabel');
    if (nodeType) {
      const rect = (event.target as HTMLElement).getBoundingClientRect();
      const newNode: WorkflowNode = {
        id: generateNodeId(), name: nodeLabel || '新节点',
        type: nodeType as WorkflowNode['type'],
        x: Math.max(0, event.clientX - rect.left - 50),
        y: Math.max(0, event.clientY - rect.top - 20),
      };
      nodes.value.push(newNode);
      selectedNode.value = newNode;
      saveHistory();
    }
  }

  function selectNode(node: WorkflowNode) { selectedNode.value = node; selectedEdge.value = null; }
  function selectEdge(edge: WorkflowEdge) { selectedEdge.value = edge; selectedNode.value = null; }

  function removeNode(nodeId: string) {
    nodes.value = nodes.value.filter((n) => n.id !== nodeId);
    edges.value = edges.value.filter((e) => e.sourceNodeId !== nodeId && e.targetNodeId !== nodeId);
    if (selectedNode.value?.id === nodeId) selectedNode.value = null;
    saveHistory();
  }

  function removeEdge(edgeId: string) {
    edges.value = edges.value.filter((e) => e.id !== edgeId);
    if (selectedEdge.value?.id === edgeId) selectedEdge.value = null;
    saveHistory();
  }

  function createEdge(sourceNode: WorkflowNode, targetNode: WorkflowNode | null) {
    if (!targetNode) { $q.notify({ type: 'info', message: '点击目标节点完成连线' }); return; }
    const src = nodes.value.find((n) => n.id === sourceNode.id);
    if (!src) return;
    edges.value.push({
      id: generateEdgeId(), sourceNodeId: sourceNode.id, targetNodeId: targetNode.id,
      sourceX: src.x + 100, sourceY: src.y + 25, targetX: targetNode.x, targetY: targetNode.y + 25,
    });
    saveHistory();
  }

  function handleReset() {
    nodes.value = []; edges.value = []; selectedNode.value = null; selectedEdge.value = null;
    nodeIdCounter = 0; edgeIdCounter = 0;
    $q.notify({ type: 'info', message: '已重置' });
  }

  async function handleSave() {
    try {
      await updateWorkflow(workflowId, { name: workflowName.value, definition: { nodes: nodes.value, edges: edges.value } });
      $q.notify({ type: 'positive', message: '保存成功' });
    } catch { $q.notify({ type: 'negative', message: '保存失败' }); }
  }

  async function handlePublish() {
    try { await publishWorkflow(workflowId); $q.notify({ type: 'positive', message: '发布成功' }); }
    catch { $q.notify({ type: 'negative', message: '发布失败' }); }
  }

  async function loadWorkflow() {
    if (!workflowId) return;
    try {
      const response = await getWorkflow(workflowId);
      const data = (response.data as { data?: { name?: string; definition?: { nodes?: WorkflowNode[]; edges?: WorkflowEdge[] } } })?.data;
      if (data) {
        workflowName.value = data.name || '';
        if (data.definition?.nodes) {
          nodes.value = data.definition.nodes;
          nodeIdCounter = Math.max(0, ...nodes.value.map((n) => { const m = n.id.match(/node_(\d+)/); return m ? parseInt(m[1]!) : 0; }));
        }
        if (data.definition?.edges) {
          edges.value = data.definition.edges;
          edgeIdCounter = Math.max(0, ...edges.value.map((e) => { const m = e.id.match(/edge_(\d+)/); return m ? parseInt(m[1]!) : 0; }));
        }
      }
    } catch { $q.notify({ type: 'negative', message: '加载工作流失败' }); }
  }

  onMounted(() => { void loadWorkflow(); });

  return {
    workflowId, nodeTypes, nodeTypeOptions, workflowName, nodes, edges,
    selectedNode, selectedEdge, isDragOver, approvalConfig, conditionConfig,
    getNodeIcon, getNodeColor, getOutputPorts,
    onDragStart, onDragOver, onDragLeave, onDrop,
    selectNode, selectEdge, removeNode, removeEdge, createEdge,
    handleUndo, handleRedo, handleValidate, handleReset, handleSave, handlePublish, loadWorkflow,
  };
}
