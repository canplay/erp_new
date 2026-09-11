export interface AppConfig {
  tuwunelUrl: string
  appName: string
}

export const config: AppConfig = {
  tuwunelUrl: import.meta.env.VITE_TUWUNEL_URL ?? 'http://localhost:8008',
  appName: import.meta.env.VITE_APP_NAME ?? 'Social',
}
