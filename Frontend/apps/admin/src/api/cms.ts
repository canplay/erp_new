/**
 * @file cms.ts
 * @description CMS内容管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

/**
 * @brief 文章状态
 */
export type ArticleStatus = 'draft' | 'pending' | 'published' | 'rejected' | 'archived';

/**
 * @brief 内容类型
 */
export type ContentType = 'richText' | 'markdown';

/**
 * @brief 文章分类接口
 */
export interface CmsCategory {
  id: number;
  parent_id: number;
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  sort_order: number;
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
  status: number;
  allowAttachment: boolean;
  created_at: string;
  updated_at: string;
  children?: CmsCategory[];
}

/**
 * @brief 文章附件
 */
export interface ArticleAttachment {
  name: string;
  url: string;
  size?: number;
}

/**
 * @brief 文章接口
 */
export interface CmsArticle {
  id: number;
  categoryId: number;
  categoryName?: string;
  title: string;
  slug?: string;
  summary?: string;
  content: string;
  contentType: ContentType;
  coverImage?: string;
  authorId: number;
  authorName?: string;
  tags?: string[];
  viewCount: number;
  likeCount: number;
  commentCount: number;
  shareCount: number;
  isFeatured: boolean;
  isTop: boolean;
  isDraft: boolean;
  status: ArticleStatus;
  rejectReason?: string;
  publishedAt?: string;
  source?: string;
  sourceUrl?: string;
  attachments?: ArticleAttachment[];
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 文章查询参数
 */
export interface ArticleQueryParams {
  categoryId?: number;
  status?: ArticleStatus;
  authorId?: number;
  keyword?: string;
  tags?: string[];
  isFeatured?: boolean;
  start_date?: string;
  end_date?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief 文章创建参数
 */
export interface ArticleCreateParams {
  categoryId: number;
  title: string;
  slug?: string;
  summary?: string;
  content: string;
  contentType?: ContentType;
  coverImage?: string;
  tags?: string[];
  isFeatured?: boolean;
  isTop?: boolean;
  isDraft?: boolean;
  source?: string;
  sourceUrl?: string;
  attachments?: ArticleAttachment[];
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
}

/**
 * @brief 文章更新参数
 */
export interface ArticleUpdateParams extends ArticleCreateParams {
  id: number;
}

/**
 * @brief 分类查询参数
 */
export interface CategoryQueryParams {
  parent_id?: number;
  status?: number;
  keyword?: string;
}

/**
 * @brief 分类创建参数
 */
export interface CategoryCreateParams {
  parent_id?: number;
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  sort_order?: number;
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
  status?: number;
  allowAttachment?: boolean;
}

/**
 * @brief 分类更新参数
 */
export interface CategoryUpdateParams extends CategoryCreateParams {
  id: number;
}

// ==================== 分类 API ====================

/**
 * @brief 获取分类树
 */
export function getCategoryTree(params?: CategoryQueryParams) {
  return httpClient.get('/cms/categories/tree', { params });
}

/**
 * @brief 获取分类列表
 */
export function getCategoryList(params?: CategoryQueryParams) {
  return httpClient.get('/cms/categories', { params });
}

/**
 * @brief 获取分类详情
 */
export function getCategoryDetail(id: number) {
  return httpClient.get(`/cms/categories/${id}`);
}

/**
 * @brief 创建分类
 */
export function createCategory(data: CategoryCreateParams) {
  return httpClient.post('/cms/categories', data);
}

/**
 * @brief 更新分类
 */
export function updateCategory(id: number, data: CategoryUpdateParams) {
  return httpClient.put(`/cms/categories/${id}`, data);
}

/**
 * @brief 删除分类
 */
export function deleteCategory(id: number) {
  return httpClient.delete(`/cms/categories/${id}`);
}

/**
 * @brief 批量删除分类
 */
export function batchDeleteCategories(ids: number[]) {
  return httpClient.delete('/cms/categories/batch', { data: { ids } });
}

/**
 * @brief 调整分类排序
 */
export function reorderCategories(orders: Array<{ id: number; sort_order: number }>) {
  return httpClient.put('/cms/categories/reorder', { orders });
}

// ==================== 文章 API ====================

/**
 * @brief 获取文章列表
 */
export function getArticleList(params?: ArticleQueryParams) {
  return httpClient.get('/cms/articles', { params });
}

/**
 * @brief 获取文章详情
 */
export function getArticleDetail(id: number) {
  return httpClient.get(`/cms/articles/${id}`);
}

/**
 * @brief 获取文章详情（通过slug）
 */
export function getArticleBySlug(slug: string) {
  return httpClient.get('/cms/articles/slug/' + slug);
}

/**
 * @brief 创建文章
 */
export function createArticle(data: ArticleCreateParams) {
  return httpClient.post('/cms/articles', data);
}

/**
 * @brief 更新文章
 */
export function updateArticle(id: number, data: ArticleUpdateParams) {
  return httpClient.put(`/cms/articles/${id}`, data);
}

/**
 * @brief 删除文章
 */
export function deleteArticle(id: number) {
  return httpClient.delete(`/cms/articles/${id}`);
}

/**
 * @brief 批量删除文章
 */
export function batchDeleteArticles(ids: number[]) {
  return httpClient.delete('/cms/articles/batch', { data: { ids } });
}

/**
 * @brief 发布文章
 */
export function publishArticle(id: number) {
  return httpClient.put(`/cms/articles/${id}/publish`);
}

/**
 * @brief 下架文章
 */
export function unpublishArticle(id: number) {
  return httpClient.put(`/cms/articles/${id}/unpublish`);
}

/**
 * @brief 审核文章
 */
export function reviewArticle(id: number, approved: boolean, reason?: string) {
  return httpClient.put(`/cms/articles/${id}/review`, { approved, reason });
}

/**
 * @brief 置顶文章
 */
export function topArticle(id: number, isTop: boolean) {
  return httpClient.put(`/cms/articles/${id}/top`, { isTop });
}

/**
 * @brief 推荐文章
 */
export function featureArticle(id: number, isFeatured: boolean) {
  return httpClient.put(`/cms/articles/${id}/feature`, { isFeatured });
}

/**
 * @brief 获取我的文章草稿
 */
export function getMyDrafts() {
  return httpClient.get('/cms/articles/my-drafts');
}

/**
 * @brief 获取相关文章
 */
export function getRelatedArticles(id: number, limit: number = 5) {
  return httpClient.get(`/cms/articles/${id}/related`, { params: { limit } });
}

/**
 * @brief 增加浏览次数
 */
export function incrementViewCount(id: number) {
  return httpClient.put(`/cms/articles/${id}/view`);
}

/**
 * @brief 点赞文章
 */
export function likeArticle(id: number) {
  return httpClient.post(`/cms/articles/${id}/like`);
}

/**
 * @brief 获取文章统计
 */
export function getArticleStatistics(params?: { categoryId?: number; start_date?: string; end_date?: string }) {
  return httpClient.get('/cms/articles/statistics', { params });
}

/**
 * @brief 获取热门文章
 */
export function getHotArticles(params?: { limit?: number; categoryId?: number }) {
  return httpClient.get('/cms/articles/hot', { params });
}

/**
 * @brief 获取最新文章
 */
export function getLatestArticles(params?: { limit?: number; categoryId?: number }) {
  return httpClient.get('/cms/articles/latest', { params });
}