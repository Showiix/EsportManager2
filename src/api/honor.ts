import { invokeCommand } from './client'

// ========================================
// Honor System
// ========================================

export interface HonorRecord {
  id: number
  tournament_id: number
  tournament_name: string
  tournament_type: string
  team_id: number
  team_name: string
  player_id: number | null
  player_name: string | null
  honor_type: string
  season_id: number
  achieved_at: string
}

export interface HonorHall {
  regional_champions: HonorRecord[]
  international_champions: HonorRecord[]
  mvps: HonorRecord[]
}

export interface TeamHonorStats {
  team_id: number
  team_name: string
  regional_titles: number
  msi_titles: number
  worlds_titles: number
  total_titles: number
}

export interface PlayerHonorStats {
  player_id: number
  player_name: string
  regional_titles: number
  international_titles: number
  mvp_awards: number
}

export const honorApi = {
  getHonorHall: () =>
    invokeCommand<HonorHall>('get_honor_hall'),

  getTeamHonors: (teamId: number) =>
    invokeCommand<HonorRecord[]>('get_team_honors', { teamId }),

  getPlayerHonors: (playerId: number) =>
    invokeCommand<HonorRecord[]>('get_player_honors', { playerId }),

  getSeasonHonors: (seasonId: number) =>
    invokeCommand<HonorRecord[]>('get_season_honors', { seasonId }),

  getTournamentHonors: (tournamentId: number) =>
    invokeCommand<HonorRecord[]>('get_tournament_honors', { tournamentId }),

  getTeamChampionCount: (teamId: number) =>
    invokeCommand<number>('get_team_champion_count', { teamId }),

  getPlayerChampionCount: (playerId: number) =>
    invokeCommand<number>('get_player_champion_count', { playerId }),

  getPlayerMvpCount: (playerId: number) =>
    invokeCommand<number>('get_player_mvp_count', { playerId }),

  getTeamHonorStats: (teamId: number) =>
    invokeCommand<TeamHonorStats>('get_team_honor_stats', { teamId }),

  getPlayerHonorStats: (playerId: number) =>
    invokeCommand<PlayerHonorStats>('get_player_honor_stats', { playerId }),

  getChampionsByType: (honorType: string) =>
    invokeCommand<HonorRecord[]>('get_champions_by_type', { honorType }),

  getAllChampions: () =>
    invokeCommand<HonorRecord[]>('get_all_champions'),

  getAllMvps: () =>
    invokeCommand<HonorRecord[]>('get_all_mvps'),

  // ========== 荣誉殿堂新增 API ==========

  /** 获取国际赛事冠军列表（旗帜墙） */
  getInternationalChampions: () =>
    invokeCommand<InternationalChampionCard[]>('get_international_champions'),

  /** 获取冠军详情（展开后显示阵容等） */
  getChampionDetail: (tournamentId: number) =>
    invokeCommand<ChampionDetail>('get_champion_detail', { tournamentId }),

  /** 获取选手荣誉排行榜 */
  getPlayerHonorRankings: (limit?: number) =>
    invokeCommand<PlayerHonorRanking[]>('get_player_honor_rankings', { limit }),

  /** 获取战队荣誉排行榜 */
  getTeamHonorRankings: (limit?: number) =>
    invokeCommand<TeamHonorRanking[]>('get_team_honor_rankings', { limit }),

  /** 获取选手荣誉详情 */
  getPlayerHonorDetail: (playerId: number) =>
    invokeCommand<PlayerHonorDetail>('get_player_honor_detail', { playerId }),

  /** 获取战队荣誉详情 */
  getTeamHonorDetail: (teamId: number) =>
    invokeCommand<TeamHonorDetail>('get_team_honor_detail', { teamId }),

  /** 重新生成赛事荣誉 */
  regenerateTournamentHonors: (tournamentId: number) =>
    invokeCommand<{ deleted_count: number; created_count: number; message: string }>('regenerate_tournament_honors', { tournamentId }),

  /** 重新生成所有已完成赛事的荣誉 */
  regenerateAllHonors: () =>
    invokeCommand<{ deleted_count: number; created_count: number; message: string }>('regenerate_all_honors'),
}

// 荣誉类型中文转换
export const HONOR_TYPE_MAP: Record<string, string> = {
  'TEAM_CHAMPION': '冠军',
  'TEAM_RUNNER_UP': '亚军',
  'TEAM_THIRD': '季军',
  'TEAM_FOURTH': '殿军',
  'REGULAR_SEASON_FIRST': '常规赛第一',
  'TOURNAMENT_MVP': '赛事MVP',
  'FINALS_MVP': '决赛MVP',
  'REGULAR_SEASON_MVP': '常规赛MVP',
  'PLAYOFFS_FMVP': '季后赛FMVP',
  'PLAYER_CHAMPION': '冠军成员',
  'PLAYER_RUNNER_UP': '亚军成员',
  'PLAYER_THIRD': '季军成员',
  'PLAYER_FOURTH': '殿军成员',
  // 年度荣誉
  'ANNUAL_MVP': '年度MVP',
  'ANNUAL_TOP20': '年度Top20',
  'ANNUAL_ALL_PRO1ST': '年度最佳阵容一阵',
  'ANNUAL_ALL_PRO2ND': '年度最佳阵容二阵',
  'ANNUAL_ALL_PRO3RD': '年度最佳阵容三阵',
  'ANNUAL_MOST_CONSISTENT': '年度最稳定选手',
  'ANNUAL_MOST_DOMINANT': '年度最具统治力选手',
  'ANNUAL_BEST_TOP': '年度最佳上单',
  'ANNUAL_BEST_JUNGLE': '年度最佳打野',
  'ANNUAL_BEST_MID': '年度最佳中单',
  'ANNUAL_BEST_ADC': '年度最佳ADC',
  'ANNUAL_BEST_SUPPORT': '年度最佳辅助',
  'ANNUAL_ROOKIE': '年度最佳新秀',
}

// 荣誉类型对应的标签颜色
export const HONOR_TAG_TYPE: Record<string, string> = {
  'TEAM_CHAMPION': 'warning',      // 金色
  'TEAM_RUNNER_UP': '',            // 银色（默认）
  'TEAM_THIRD': 'success',         // 绿色（铜色）
  'TEAM_FOURTH': 'info',           // 蓝色
  'PLAYER_CHAMPION': 'warning',    // 金色
  'PLAYER_RUNNER_UP': '',          // 银色
  'PLAYER_THIRD': 'success',       // 绿色
  'PLAYER_FOURTH': 'info',         // 蓝色
  'TOURNAMENT_MVP': 'danger',      // 红色
  'FINALS_MVP': 'danger',          // 红色
  'REGULAR_SEASON_MVP': 'danger',  // 红色
  'PLAYOFFS_FMVP': 'danger',       // 红色
  'REGULAR_SEASON_FIRST': 'primary', // 主色
  // 年度荣誉
  'ANNUAL_MVP': 'danger',          // 红色（最高荣誉）
  'ANNUAL_TOP20': 'warning',       // 金色
  'ANNUAL_ALL_PRO1ST': 'warning', // 金色（一阵）
  'ANNUAL_ALL_PRO2ND': '',        // 银色（二阵）
  'ANNUAL_ALL_PRO3RD': 'success', // 绿色（三阵）
  'ANNUAL_MOST_CONSISTENT': 'primary', // 蓝色
  'ANNUAL_MOST_DOMINANT': 'danger',    // 红色
  'ANNUAL_BEST_TOP': 'success',    // 绿色
  'ANNUAL_BEST_JUNGLE': 'success', // 绿色
  'ANNUAL_BEST_MID': 'success',    // 绿色
  'ANNUAL_BEST_ADC': 'success',    // 绿色
  'ANNUAL_BEST_SUPPORT': 'success',// 绿色
  'ANNUAL_ROOKIE': 'primary',      // 主色（新秀）
}

export function getHonorTagType(honorType: string): string {
  return HONOR_TAG_TYPE[honorType] || 'info'
}

export function formatHonorType(honorType: string): string {
  return HONOR_TYPE_MAP[honorType] || honorType
}

// 荣誉殿堂新增类型定义
export interface InternationalChampionCard {
  season_id: number
  tournament_id: number
  tournament_name: string
  tournament_type: string
  champion_team_id: number
  champion_team_name: string
  final_score: string | null
}

export interface RosterMember {
  player_id: number
  player_name: string
  position: string
}

export interface ChampionDetail {
  season_id: number
  tournament_id: number
  tournament_name: string
  tournament_type: string
  champion_team_id: number
  champion_team_name: string
  champion_roster: RosterMember[]
  runner_up_team_id: number
  runner_up_team_name: string
  third_team_id: number | null
  third_team_name: string | null
  fourth_team_id: number | null
  fourth_team_name: string | null
  final_score: string | null
}

export interface PlayerHonorRanking {
  rank: number
  player_id: number
  player_name: string
  team_id: number | null
  team_name: string | null
  position: string | null
  champion_count: number
  mvp_count: number
  international_champion_count: number
}

export interface TeamHonorRanking {
  rank: number
  team_id: number
  team_name: string
  champion_count: number
  international_champion_count: number
  runner_up_count: number
}

export interface PlayerHonorDetail {
  player_id: number
  player_name: string
  team_id: number | null
  team_name: string | null
  position: string | null
  champion_count: number
  mvp_count: number
  international_champion_count: number
  honors: HonorRecord[]
}

export interface TeamHonorDetail {
  team_id: number
  team_name: string
  champion_count: number
  international_champion_count: number
  runner_up_count: number
  third_count: number
  honors: HonorRecord[]
}
