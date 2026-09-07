// HTTP client shared across admin/ops/social apps
// Architecture review 2026-08-05: F11 fix

export interface ApiResponse<T> {
  code: number;
  data: T;
  message?: string;
}

export const httpClient = {
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  headers: {
    'Content-Type': 'application/json',
  },
};
