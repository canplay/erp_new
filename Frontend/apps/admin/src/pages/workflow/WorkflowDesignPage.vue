<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">{{ $t('workflow.title') }}</div>
    <q-card class="full-height">
      <q-card-section>
        <div class="row q-col-gutter-md">
          <div class="col-3">
            <q-list dense bordered separator>
              <q-item-label header>{{ $t('workflow.componentList') }}</q-item-label>
              <q-item v-for="node in nodeTypes" :key="node.type" clickable v-ripple draggable @dragstart="onDragStart($event, node)">
                <q-item-section avatar><q-icon :name="node.icon" :color="node.color" /></q-item-section>
                <q-item-section>{{ node.label }}</q-item-section>
              </q-item>
            </q-list>
            <q-list dense bordered separator class="q-mt-md">
              <q-item-label header>{{ $t('workflow.addedNodes') }}</q-item-label>
              <q-item v-if="nodes.length === 0"><q-item-section class="text-grey">{{ $t('workflow.noNodes') }}</q-item-section></q-item>
              <q-item v-for="node in nodes" :key="node.id" clickable v-ripple @click="selectNode(node)">
                <q-item-section avatar><q-icon :name="getNodeIcon(node.type)" :color="getNodeColor(node.type)" /></q-item-section>
                <q-item-section><q-item-label>{{ node.name }}</q-item-label><q-item-label caption>{{ node.type }}</q-item-label></q-item-section>
                <q-item-section side><q-btn flat dense round icon="delete" size="sm" @click.stop="removeNode(node.id)" /></q-item-section>
              </q-item>
            </q-list>
          </div>

          <div class="col-6">
            <div class="workflow-canvas" :class="{ 'drag-over': isDragOver }" style="min-height:500px;border:1px solid #ccc;border-radius:8px;position:relative" @dragover.prevent="onDragOver" @dragleave="onDragLeave" @drop="onDrop">
              <div v-for="node in nodes" :key="node.id" class="workflow-node" :class="{ selected: selectedNode?.id === node.id }" :style="{ left: node.x + 'px', top: node.y + 'px' }" @click="selectNode(node)">
                <div class="node-header"><q-icon :name="getNodeIcon(node.type)" :color="getNodeColor(node.type)" /><span class="q-ml-sm">{{ node.name }}</span></div>
                <div class="node-body"><q-btn v-if="getOutputPorts(node).length > 0" class="output-port" flat dense icon="arrow_forward" @click.stop="createEdge(node, null)" /></div>
              </div>
              <svg class="edges-layer" v-if="edges.length > 0">
                <defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="#666" /></marker></defs>
                <line v-for="edge in edges" :key="edge.id" :x1="edge.sourceX" :y1="edge.sourceY" :x2="edge.targetX" :y2="edge.targetY" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)" @click.stop="selectEdge(edge)" style="cursor:pointer" />
              </svg>
              <div v-if="nodes.length === 0" class="empty-state text-center text-grey q-pa-xl"><q-icon name="account_tree" size="xl" /><p>{{ $t('workflow.dragHere') }}</p></div>
            </div>
          </div>

          <div class="col-3">
            <q-list dense bordered separator class="q-mb-md">
              <q-item-label header>{{ $t('workflow.workflowInfo') }}</q-item-label>
              <q-item><q-item-section><q-input v-model="workflowName" :label="$t('workflow.name')" outlined dense class="q-mb-sm" /></q-item-section></q-item>
            </q-list>
            <q-list dense bordered separator>
              <q-item-label header>{{ $t('workflow.nodeProperties') }}</q-item-label>
              <q-item v-if="!selectedNode"><q-item-section class="text-grey">{{ $t('workflow.selectNodeHint') }}</q-item-section></q-item>
              <q-item v-else>
                <q-item-section>
                  <q-input v-model="selectedNode.name" :label="$t('workflow.nodeName')" outlined dense class="q-mb-sm" />
                  <q-select v-model="selectedNode.type" :options="nodeTypeOptions" :label="$t('workflow.nodeType')" outlined dense emit-value map-options class="q-mb-sm" disable />
                  <div v-if="selectedNode.type === 'approval'" class="q-mb-sm">
                    <q-input v-model="approvalConfig.assignee" :label="$t('workflow.approver')" outlined dense :hint="$t('workflow.approverHint')" class="q-mb-sm" />
                    <q-select v-model="approvalConfig.approvalType" :options="[$t('workflow.singleApproval'), $t('workflow.countersign'), $t('workflow.orSign')]" :label="$t('workflow.approvalMethod')" outlined dense />
                  </div>
                  <div v-if="selectedNode.type === 'condition'"><q-input v-model="conditionConfig.expression" :label="$t('workflow.conditionExpr')" outlined dense :hint="$t('workflow.conditionExample')" class="q-mb-sm" /></div>
                  <q-input v-model.number="selectedNode.timeout" :label="$t('workflow.timeout')" type="number" outlined dense :hint="$t('workflow.timeoutHint')" class="q-mb-sm" />
                </q-item-section>
              </q-item>
            </q-list>
            <q-list v-if="selectedEdge" dense bordered separator class="q-mt-md">
              <q-item-label header>连线配置</q-item-label>
              <q-item><q-item-section><q-input v-model="selectedEdge.label" :label="$t('workflow.edgeLabel')" outlined dense class="q-mb-sm" /><q-input v-model="selectedEdge.condition" :label="$t('workflow.conditionExpr')" outlined dense :hint="$t('workflow.conditionExample')" /></q-item-section></q-item>
              <q-item><q-item-section side><q-btn flat dense icon="delete" color="negative" size="sm" @click="removeEdge(selectedEdge.id)" /></q-item-section></q-item>
            </q-list>
            <q-list dense bordered separator class="q-mt-md">
              <q-item-label header>{{ $t('workflow.operations') }}</q-item-label>
              <q-item><q-item-section><q-btn-group spread flat><q-btn flat color="grey" icon="undo" :label="$t('common.undo')" @click="handleUndo" /><q-btn flat color="grey" icon="redo" :label="$t('common.redo')" @click="handleRedo" /></q-btn-group></q-item-section></q-item>
              <q-item><q-item-section><q-btn flat color="info" icon="check_circle" :label="$t('workflow.validate')" @click="handleValidate" /></q-item-section></q-item>
            </q-list>
          </div>
        </div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('common.reset')" @click="handleReset" />
        <q-btn color="primary" :label="$t('common.save')" @click="handleSave" />
        <q-btn color="positive" :label="$t('common.publish')" @click="handlePublish" />
      </q-card-actions>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { useWorkflowDesign } from '@/composables/useWorkflowDesign';

const {
  nodeTypes, nodeTypeOptions, workflowName, nodes, edges,
  selectedNode, selectedEdge, isDragOver, approvalConfig, conditionConfig,
  getNodeIcon, getNodeColor, getOutputPorts,
  onDragStart, onDragOver, onDragLeave, onDrop,
  selectNode, selectEdge, removeNode, removeEdge, createEdge,
  handleUndo, handleRedo, handleValidate, handleReset, handleSave, handlePublish,
} = useWorkflowDesign();
</script>

<style scoped>
.workflow-canvas { background: #fafafa; position: relative; }
.workflow-canvas.drag-over { background: #e3f2fd; border-color: #2196f3 !important; }
.workflow-node { position: absolute; width: 100px; height: 50px; background: white; border: 2px solid #ccc; border-radius: 8px; cursor: move; display: flex; flex-direction: column; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
.workflow-node:hover { border-color: #2196f3; }
.workflow-node.selected { border-color: #2196f3; box-shadow: 0 0 0 2px rgba(33,150,243,0.3); }
.node-header { display: flex; align-items: center; padding: 4px 8px; font-size: 12px; background: #f5f5f5; border-radius: 6px 6px 0 0; }
.node-body { flex: 1; display: flex; align-items: center; justify-content: center; padding: 4px; }
.output-port { width: 20px; height: 20px; }
.edges-layer { position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
.edges-layer line { pointer-events: stroke; }
.empty-state { position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); }
</style>
