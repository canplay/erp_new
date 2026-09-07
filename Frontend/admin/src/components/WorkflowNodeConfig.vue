<template>
  <q-list dense bordered separator class="q-mb-md">
    <q-item-label header>节点属性</q-item-label>
    <q-item v-if="!node">
      <q-item-section class="text-grey">请选择节点进行配置</q-item-section>
    </q-item>
    <q-item v-else>
      <q-item-section>
        <q-input :model-value="node.name" :label="$t('workflow.nodeName')" outlined dense class="q-mb-sm" @update:model-value="(v: unknown) => $emit('update:name', v as string)" />
        <q-select :model-value="node.type" :options="typeOptions" :label="$t('workflow.nodeType')" outlined dense emit-value map-options class="q-mb-sm" disable />
        <div v-if="node.type === 'approval'" class="q-mb-sm">
          <q-input :model-value="approvalConfig.assignee" :label="$t('workflow.approver')" outlined dense :hint="$t('workflow.approverHint')" class="q-mb-sm" @update:model-value="(v: unknown) => $emit('update:approvalAssignee', v as string)" />
          <q-select :model-value="approvalConfig.approvalType" :options="['单人审批', '会签', '或签']" :label="$t('workflow.approvalMethod')" outlined dense @update:model-value="(v: unknown) => $emit('update:approvalType', v as string)" />
        </div>
        <div v-if="node.type === 'condition'">
          <q-input :model-value="conditionConfig.expression" :label="$t('workflow.conditionExpr')" outlined dense :hint="$t('workflow.conditionExample')" class="q-mb-sm" @update:model-value="(v: unknown) => $emit('update:expression', v as string)" />
        </div>
        <q-input :model-value="node.timeout" :label="$t('workflow.timeout')" type="number" outlined dense :hint="$t('workflow.timeoutHint')" class="q-mb-sm" @update:model-value="(v: unknown) => $emit('update:timeout', Number(v ?? 0))" />
      </q-item-section>
    </q-item>
  </q-list>
</template>

<script setup lang="ts">
interface WfNode { name: string; type: string; timeout?: number; }
interface ApprovalCfg { assignee: string; approvalType: string; }
interface ConditionCfg { expression: string; }

defineProps<{ node: WfNode | null; typeOptions: { label: string; value: string }[]; approvalConfig: ApprovalCfg; conditionConfig: ConditionCfg; }>();
defineEmits<{ 'update:name': [v: string]; 'update:approvalAssignee': [v: string]; 'update:approvalType': [v: string]; 'update:expression': [v: string]; 'update:timeout': [v: number]; }>();
</script>
