export interface ProductItem {
  id: string;
  sku?: string;
  name: string;
  slug?: string;
  description?: string | null;
  brandId?: string;
  categoryId?: string;
  price?: { amount: number; currency: string };
  stock?: number;
  isActive?: boolean;
  thumbnailUrl?: string | null;
  createdAtUtc?: string;
  updatedAtUtc?: string | null;
}

export function formatMoney(amount?: number, currency?: string) {
  if (amount === undefined || amount === null || Number.isNaN(Number(amount))) return '—';
  try {
    return new Intl.NumberFormat('zh-CN', {
      style: 'currency',
      currency: currency || 'USD',
    }).format(Number(amount));
  } catch {
    return `${currency || ''} ${amount}`.trim();
  }
}

export function initial(name?: string) {
  const n = (name ?? '').trim();
  return n ? n.charAt(0).toUpperCase() : '·';
}

export function stockColor(stock?: number) {
  if (stock === undefined || stock === null) return 'grey';
  if (stock === 0) return 'red';
  if (stock < 10) return 'orange';
  return 'green';
}

export function brandNameOf(id?: string, brandOptions: { label: string; value: string }[] = []) {
  if (!id) return '—';
  return brandOptions.find((o) => o.value === id)?.label ?? '—';
}

export function categoryNameOf(
  id?: string,
  categoryOptions: { label: string; value: string }[] = [],
) {
  if (!id) return '—';
  return categoryOptions.find((o) => o.value === id)?.label ?? '—';
}
