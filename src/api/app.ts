import { invokeCommand } from './client'

// ========================================
// App Info
// ========================================

export interface AppInfo {
  name: string
  version: string
  description: string
}

export const appApi = {
  getAppInfo: () => invokeCommand<AppInfo>('get_app_info'),
}
