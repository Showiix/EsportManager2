import { invokeCommand } from './client'

// ========================================
// Meta 版本系统
// ========================================

/** Meta 权重信息 */
export interface MetaWeightsInfo {
  top: number
  jug: number
  mid: number
  adc: number
  sup: number
}

/** 当前赛季 Meta 信息 */
export interface MetaInfo {
  season_id: number
  meta_type: string
  meta_name: string
  description: string
  weights: MetaWeightsInfo
}

/** Meta 历史记录 */
export interface MetaHistoryEntry {
  season_id: number
  meta_type: string
  meta_name: string
  weight_top: number
  weight_jug: number
  weight_mid: number
  weight_adc: number
  weight_sup: number
}

/** Meta 类型信息 */
export interface MetaTypeInfo {
  id: string
  name: string
  description: string
  weights: MetaWeightsInfo
}

/** 获取当前赛季的 Meta 版本信息 */
export async function getCurrentMeta(saveId: string) {
  return invokeCommand<MetaInfo>('get_current_meta', { saveId })
}

/** 获取 Meta 历史版本列表 */
export async function getMetaHistory(saveId: string) {
  return invokeCommand<MetaHistoryEntry[]>('get_meta_history', { saveId })
}

/** 获取全部 20 种 Meta 类型配置 */
export async function getAllMetaTypes() {
  return invokeCommand<MetaTypeInfo[]>('get_all_meta_types')
}

/** 获取指定 Meta 类型的详细信息 */
export async function getMetaDetail(metaType: string) {
  return invokeCommand<MetaTypeInfo>('get_meta_detail', { metaType })
}
