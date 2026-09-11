/**
 * @file workflow.ts
 * @description 工作流 API 接口
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse } from '@/types/api';

// ============ 类型定义 ============

/** 工作流状态 */
export type WorkflowStatus = 'draft' | 'published' | 'disabled';

/** 节点类型 */
export type NodeType = 'start' | 'end' | 'task' | 'approval' | 'condition' | 'parallel' | 'merge';

/** 节点状态 */
export type NodeStatus = 'pending' | 'running' | 'completed' | 'rejected' | 'skipped';

/** 实例状态 */
export type InstanceStatus = 'pending' | 'running' | 'completed' | 'cancelled' | 'rejected';

/** 动作类型 */
export type WorkflowAction = 'approve' | 'reject' | 'reassign' | 'cancel';

/** 位置坐标 */
export interface Position {
  x: number;
  y: number;
}

/** 工作流定义 */
export interface Workflow {
  id: string;
  name: string;
  description?: string;
  definition: Record<string, unknown>;
  status: WorkflowStatus;
  version: number;
  created_by: string;
  created_at: string;
  updated_at: string;
}

/** 工作流节点 */
export interface WorkflowNode {
  id: string;
  workflowId: string;
  name: string;
  nodeType: NodeType;
  positionX: number;
  positionY: number;
  config: Record<string, unknown>;
  timeout?: number;
  autoComplete: boolean;
  created_at: string;
}

/** 工作流边（连线） */
export interface WorkflowEdge {
  id: string;
  workflowId: string;
  sourceNodeId: string;
  targetNodeId: string;
  edgeType: 'normal' | 'condition' | 'default';
  condition?: string;
  label?: string;
  priority: number;
  created_at: string;
}

/** 工作流实例 */
export interface WorkflowInstance {
  id: string;
  workflowId: string;
  workflowVersion: number;
  status: InstanceStatus;
  currentNodeId?: string;
  variables: Record<string, unknown>;
  startedBy: string;
  startedAt: string;
  completed_at?: string;
}

/** 任务记录 */
export interface TaskRecord {
  id: string;
  instanceId: string;
  nodeId: string;
  nodeName: string;
  assignee: string;
  status: NodeStatus;
  comment?: string;
  formData?: Record<string, unknown>;
  startedAt: string;
  completed_at?: string;
  timeoutAt?: string;
}

// ============ 请求参数 ============

/** 创建工作流参数 */
export interface CreateWorkflowParam {
  name: string;
  description?: string;
}

/** 更新工作流参数 */
export interface UpdateWorkflowParam {
  name?: string;
  description?: string;
  definition?: Record<string, unknown>;
  status?: WorkflowStatus;
}

/** 创建节点参数 */
export interface CreateNodeParam {
  name: string;
  nodeType: NodeType;
  positionX: number;
  positionY: number;
  config?: Record<string, unknown>;
  timeout?: number;
  autoComplete?: boolean;
}

/** 更新节点参数 */
export interface UpdateNodeParam {
  name?: string;
  config?: Record<string, unknown>;
  timeout?: number;
  autoComplete?: boolean;
}

/** 创建边参数 */
export interface CreateEdgeParam {
  sourceNodeId: string;
  targetNodeId: string;
  edgeType?: 'normal' | 'condition' | 'default';
  condition?: string;
  label?: string;
}

/** 启动实例参数 */
export interface StartInstanceParam {
  variables?: Record<string, unknown>;
  startNodeId?: string;
}

/** 执行动作参数 */
export interface ExecuteActionParam {
  action: WorkflowAction;
  comment?: string;
  assignee?: string;
  variables?: Record<string, unknown>;
}

/** 列表查询参数 */
export interface WorkflowListParam {
  page?: number;
  page_size?: number;
  keyword?: string;
  status?: WorkflowStatus;
}

// ============ API 函数 ============

const baseUrl = '/workflows';

/**
 * @brief 获取工作流列表
 */
export function listWorkflows(params?: WorkflowListParam) {
  return httpClient.get<ApiResponse<{ list: Workflow[]; total: number }>>(`${baseUrl}`, { params });
}

/**
 * @brief 获取工作流详情
 */
export function getWorkflow(id: string) {
  return httpClient.get<ApiResponse<Workflow>>(`${baseUrl}/${id}`);
}

/**
 * @brief 创建工作流
 */
export function createWorkflow(data: CreateWorkflowParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}`, data);
}

/**
 * @brief 更新工作流
 */
export function updateWorkflow(id: string, data: UpdateWorkflowParam) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}`, data);
}

/**
 * @brief 删除工作流
 */
export function deleteWorkflow(id: string) {
  return httpClient.delete<ApiResponse<void>>(`${baseUrl}/${id}`);
}

/**
 * @brief 发布工作流
 */
export function publishWorkflow(id: string) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}/publish`);
}

/**
 * @brief 获取工作流实例列表
 */
export function listInstances(workflowId: string, params?: WorkflowListParam) {
  return httpClient.get<ApiResponse<{ list: WorkflowInstance[]; total: number }>>(`${baseUrl}/${workflowId}/instances`, { params });
}

/**
 * @brief 获取实例详情
 */
export function getInstance(instanceId: string) {
  return httpClient.get<ApiResponse<WorkflowInstance>>(`${baseUrl}/instances/${instanceId}`);
}

/**
 * @brief 启动工作流实例
 */
export function startInstance(workflowId: string, data?: StartInstanceParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}/${workflowId}/instances`, data);
}

/**
 * @brief 执行实例动作
 */
export function executeAction(instanceId: string, data: ExecuteActionParam) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/instances/${instanceId}`, data);
}

/**
 * @brief 获取实例任务列表
 */
export function listTasks(instanceId: string) {
  return httpClient.get<ApiResponse<TaskRecord[]>>(`${baseUrl}/instances/${instanceId}/tasks`);
}

/**
 * @brief 完成任务
 */
export function completeTask(taskId: string) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/tasks/${taskId}/complete`);
}

/**
 * @brief 拒绝任务
 */
export function rejectTask(taskId: string, comment?: string) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/tasks/${taskId}/reject`, { comment });
}

/**
 * @brief 获取工作流节点
 */
export function listNodes(workflowId: string) {
  return httpClient.get<ApiResponse<WorkflowNode[]>>(`${baseUrl}/${workflowId}/nodes`);
}

/**
 * @brief 创建节点
 */
export function createNode(workflowId: string, data: CreateNodeParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}/${workflowId}/nodes`, data);
}

/**
 * @brief 更新节点
 */
export function updateNode(workflowId: string, nodeId: string, data: UpdateNodeParam) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${workflowId}/nodes/${nodeId}`, data);
}

/**
 * @brief 删除节点
 */
export function deleteNode(workflowId: string, nodeId: string) {
  return httpClient.delete<ApiResponse<void>>(`${baseUrl}/${workflowId}/nodes/${nodeId}`);
}

/**
 * @brief 获取工作流边
 */
export function listEdges(workflowId: string) {
  return httpClient.get<ApiResponse<WorkflowEdge[]>>(`${baseUrl}/${workflowId}/edges`);
}

/**
 * @brief 创建边
 */
export function createEdge(workflowId: string, data: CreateEdgeParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}/${workflowId}/edges`, data);
}

/**
 * @brief 更新边
 */
export function updateEdge(workflowId: string, edgeId: string, data: Partial<CreateEdgeParam>) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${workflowId}/edges/${edgeId}`, data);
}

/**
 * @brief 删除边
 */
export function deleteEdge(workflowId: string, edgeId: string) {
  return httpClient.delete<ApiResponse<void>>(`${baseUrl}/${workflowId}/edges/${edgeId}`);
}