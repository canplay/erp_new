/**
 * @file cms.ts
 * @description CMS内容管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

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
export async function getCategoryTree(params?: CategoryQueryParams) {
  try {
    return await httpClient.get('/cms/categories/tree', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取分类列表
 */
export async function getCategoryList(params?: CategoryQueryParams) {
  try {
    return await httpClient.get('/cms/categories', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取分类详情
 */
export async function getCategoryDetail(id: number) {
  try {
    return await httpClient.get(`/cms/categories/${id}`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 创建分类
 */
export async function createCategory(data: CategoryCreateParams) {
  try {
    return await httpClient.post('/cms/categories', data);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 更新分类
 */
export async function updateCategory(id: number, data: CategoryUpdateParams) {
  try {
    return await httpClient.put(`/cms/categories/${id}`, data);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 删除分类
 */
export async function deleteCategory(id: number) {
  try {
    return await httpClient.delete(`/cms/categories/${id}`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 批量删除分类
 */
export async function batchDeleteCategories(ids: number[]) {
  try {
    return await httpClient.delete('/cms/categories/batch', { data: { ids } });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 调整分类排序
 */
export async function reorderCategories(orders: Array<{ id: number; sort_order: number }>) {
  try {
    return await httpClient.put('/cms/categories/reorder', { orders });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

// ==================== 文章 API ====================

/**
 * @brief 获取文章列表
 */
export async function getArticleList(params?: ArticleQueryParams) {
  try {
    return await httpClient.get('/cms/articles', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取文章详情
 */
export async function getArticleDetail(id: number) {
  try {
    return await httpClient.get(`/cms/articles/${id}`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取文章详情（通过slug）
 */
export async function getArticleBySlug(slug: string) {
  try {
    return await httpClient.get('/cms/articles/slug/' + slug);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 创建文章
 */
export async function createArticle(data: ArticleCreateParams) {
  try {
    return await httpClient.post('/cms/articles', data);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 更新文章
 */
export async function updateArticle(id: number, data: ArticleUpdateParams) {
  try {
    return await httpClient.put(`/cms/articles/${id}`, data);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 删除文章
 */
export async function deleteArticle(id: number) {
  try {
    return await httpClient.delete(`/cms/articles/${id}`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 批量删除文章
 */
export async function batchDeleteArticles(ids: number[]) {
  try {
    return await httpClient.delete('/cms/articles/batch', { data: { ids } });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 发布文章
 */
export async function publishArticle(id: number) {
  try {
    return await httpClient.put(`/cms/articles/${id}/publish`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 下架文章
 */
export async function unpublishArticle(id: number) {
  try {
    return await httpClient.put(`/cms/articles/${id}/unpublish`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 审核文章
 */
export async function reviewArticle(id: number, approved: boolean, reason?: string) {
  try {
    return await httpClient.put(`/cms/articles/${id}/review`, { approved, reason });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 置顶文章
 */
export async function topArticle(id: number, isTop: boolean) {
  try {
    return await httpClient.put(`/cms/articles/${id}/top`, { isTop });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 推荐文章
 */
export async function featureArticle(id: number, isFeatured: boolean) {
  try {
    return await httpClient.put(`/cms/articles/${id}/feature`, { isFeatured });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取我的文章草稿
 */
export async function getMyDrafts() {
  try {
    return await httpClient.get('/cms/articles/my-drafts');
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取相关文章
 */
export async function getRelatedArticles(id: number, limit: number = 5) {
  try {
    return await httpClient.get(`/cms/articles/${id}/related`, { params: { limit } });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 增加浏览次数
 */
export async function incrementViewCount(id: number) {
  try {
    return await httpClient.put(`/cms/articles/${id}/view`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 点赞文章
 */
export async function likeArticle(id: number) {
  try {
    return await httpClient.post(`/cms/articles/${id}/like`);
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取文章统计
 */
export async function getArticleStatistics(params?: { categoryId?: number; start_date?: string; end_date?: string }) {
  try {
    return await httpClient.get('/cms/articles/statistics', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取热门文章
 */
export async function getHotArticles(params?: { limit?: number; categoryId?: number }) {
  try {
    return await httpClient.get('/cms/articles/hot', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}

/**
 * @brief 获取最新文章
 */
export async function getLatestArticles(params?: { limit?: number; categoryId?: number }) {
  try {
    return await httpClient.get('/cms/articles/latest', { params });
  } catch (error) {
    handleApiError(error, 'CMS内容管理');
    throw error;
  }
}