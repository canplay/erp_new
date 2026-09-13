// @viewpoint platform-only
// 复用共享 alova 封装（packages/api），不再重复定义 class M
import { api, normalize, unwrapArray } from '@erp-new-frontend-monorepo/api';

export interface BrandDto {
  id: string;
  name: string;
  description?: string | null;
  logoUrl?: string | null;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc?: string | null;
}

export interface CategoryDto {
  id: string;
  name: string;
  description?: string | null;
  parentId?: string | null;
  level: number;
  sortOrder: number;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc?: string | null;
  children?: CategoryDto[];
}

export interface MoneyDto {
  amount: number;
  currency: string;
}

export interface ProductDto {
  id: string;
  sku?: string;
  name: string;
  slug?: string;
  description?: string | null;
  brandId?: string;
  categoryId?: string;
  price?: MoneyDto | null;
  stock?: number;
  isActive?: boolean;
  thumbnailUrl?: string | null;
  createdAtUtc?: string;
  updatedAtUtc?: string | null;
}

export interface CreateProductDto {
  sku: string;
  name: string;
  brandId: string;
  categoryId: string;
  priceAmount?: string;
  priceCurrency?: string;
  stock?: string;
  isActive?: boolean;
  description?: string;
  model?: string;
  specifications?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface UpdateProductDto {
  name?: string;
  description?: string;
  brandId?: string;
  categoryId?: string;
  sku?: string;
  model?: string;
  specifications?: Record<string, unknown>;
  priceAmount?: string;
  priceCurrency?: string;
  stock?: string;
  isActive?: boolean;
  [key: string]: unknown;
}

export const catalogApi = {
  brands: (params: { page?: number; pageSize?: number; search?: string } = {}) => {
    const { page, ...rest } = params;
    const q = page !== undefined ? { ...rest, pageNumber: page } : rest;
    return api
      .Get<BrandDto[]>('/api/v1/catalog/brands', q)
      .then((r) => normalize<BrandDto>(r, page ?? 1, params.pageSize ?? 20));
  },
  brand: (id: string) => api.Get<BrandDto>(`/api/v1/catalog/brands/${id}`),
  createBrand: (data: { name: string; description?: string; logoUrl?: string }) =>
    api.Post('/api/v1/catalog/brands', data),
  updateBrand: (id: string, data: { name: string; description?: string; logoUrl?: string }) =>
    api.Put(`/api/v1/catalog/brands/${id}`, data),
  deleteBrand: (id: string) => api.Delete(`/api/v1/catalog/brands/${id}`),
  brandTrash: () =>
    api.Get<BrandDto[]>('/api/v1/catalog/brands/trash').then((r) => unwrapArray<BrandDto>(r)),
  restoreBrand: (id: string) => api.Post(`/api/v1/catalog/brands/${id}/restore`, {}),
  categories: (params: { page?: number; pageSize?: number } = {}) => {
    const { page, ...rest } = params;
    const q = page !== undefined ? { ...rest, pageNumber: page } : rest;
    return api
      .Get<CategoryDto[]>('/api/v1/catalog/categories', q)
      .then((r) => normalize<CategoryDto>(r, page ?? 1, params.pageSize ?? 20));
  },
  categoryTree: () =>
    api
      .Get<CategoryDto[]>('/api/v1/catalog/categories/tree')
      .then((r) => unwrapArray<CategoryDto>(r)),
  category: (id: string) => api.Get<CategoryDto>(`/api/v1/catalog/categories/${id}`),
  createCategory: (data: { name: string; description?: string; parentId?: string }) =>
    api.Post('/api/v1/catalog/categories', data),
  updateCategory: (id: string, data: { name: string; description?: string; parentId?: string }) =>
    api.Put(`/api/v1/catalog/categories/${id}`, data),
  deleteCategory: (id: string) => api.Delete(`/api/v1/catalog/categories/${id}`),
  restoreCategory: (id: string) => api.Post(`/api/v1/catalog/categories/${id}/restore`, {}),
  products: (
    params: {
      page?: number;
      pageSize?: number;
      search?: string;
      brandId?: string;
      categoryId?: string;
    } = {},
  ) => {
    const { page, ...rest } = params;
    const q = page !== undefined ? { ...rest, pageNumber: page } : rest;
    return api
      .Get<ProductDto[]>('/api/v1/catalog/products', q)
      .then((r) => normalize<ProductDto>(r, page ?? 1, params.pageSize ?? 20));
  },
  product: (id: string) => api.Get<ProductDto>(`/api/v1/catalog/products/${id}`),
  createProduct: (data: Record<string, unknown>) => api.Post('/api/v1/catalog/products', data),
  updateProduct: (id: string, data: UpdateProductDto) =>
    api.Put(`/api/v1/catalog/products/${id}`, data),
  deleteProduct: (id: string) => api.Delete(`/api/v1/catalog/products/${id}`),
  restoreProduct: (id: string) => api.Post(`/api/v1/catalog/products/${id}/restore`, {}),
  productImages: (id: string) => api.Post(`/api/v1/catalog/products/${id}/images`, {}),
  productTrash: () =>
    api.Get<ProductDto[]>('/api/v1/catalog/products/trash').then((r) => unwrapArray<ProductDto>(r)),

  // Phase 4: 商品图片/价格/库存操作
  uploadProductImage: (productId: string, file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    return api.Post(`/api/v1/catalog/products/${productId}/images`, fd);
  },
  deleteProductImage: (productId: string, imageId: string) =>
    api.Delete(`/api/v1/catalog/products/${productId}/images/${imageId}`),
  setThumbnail: (productId: string, imageId: string) =>
    api.Put(`/api/v1/catalog/products/${productId}/images/${imageId}/thumbnail`, {}),
  reorderImages: (productId: string, imageIds: string[]) =>
    api.Put(`/api/v1/catalog/products/${productId}/images/order`, { imageIds }),
  adjustPrice: (productId: string, price: string, currency = 'CNY') =>
    api.Patch(`/api/v1/catalog/products/${productId}/price`, { price, currency }),
  adjustStock: (productId: string, stock: number) =>
    api.Patch(`/api/v1/catalog/products/${productId}/stock`, { stock }),
  categoryTrash: () =>
    api
      .Get<CategoryDto[]>('/api/v1/catalog/categories/trash')
      .then((r) => unwrapArray<CategoryDto>(r)),
};
