import { invokeCommand } from './client'
import { appDataDir, join } from '@tauri-apps/api/path'
import type { GameInitConfig } from '@/types/initConfig'

// ========================================
// Save Management (Saves)
// ========================================

export interface SaveInfo {
  id: string
  name: string
  created_at: string
  updated_at: string
  current_season: number
  current_phase: string
}

export interface GameState {
  current_season: number
  current_phase: string
  phase_name: string  // 后端返回的字段名是 phase_name
  progress?: [number, number]
  available_actions?: string[]
}

export const saveApi = {
  initDatabase: async () => {
    const dataDir = await appDataDir()
    const dbPath = await join(dataDir, 'esport_manager.db')
    return invokeCommand<void>('init_database', { dbPath })
  },

  createSave: (name: string) =>
    invokeCommand<SaveInfo>('create_save', { name }),

  getSaves: () =>
    invokeCommand<SaveInfo[]>('get_saves'),

  loadSave: (saveId: string) =>
    invokeCommand<SaveInfo>('load_save', { saveId }),

  deleteSave: (saveId: string) =>
    invokeCommand<void>('delete_save', { saveId }),

  getCurrentSaveId: () =>
    invokeCommand<string | null>('get_current_save_id'),

  deleteDatabase: async () => {
    const dataDir = await appDataDir()
    const dbPath = await join(dataDir, 'esport_manager.db')
    return invokeCommand<void>('delete_database', { dbPath })
  },

  getGameState: () =>
    invokeCommand<GameState>('get_game_state'),

  advancePhase: () =>
    invokeCommand<GameState>('advance_phase'),

  getDefaultGameConfig: () =>
    invokeCommand<GameInitConfig>('get_default_game_config'),

  createSaveWithConfig: (name: string, config: GameInitConfig) =>
    invokeCommand<SaveInfo>('create_save_with_config', { name, config }),
}
