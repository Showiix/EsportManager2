/**
 * Tauri IPC Client
 * Replaces axios HTTP calls with Tauri invoke commands
 */
import { invoke } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'
import { createLogger } from '@/utils/logger'
import { usePerformanceStoreRaw } from '@/stores/usePerformanceStore'

const logger = createLogger('TauriAPI')

// Generic API response from Rust backend
export interface CommandResult<T> {
  success: boolean
  data: T | null
  error: string | null
}

// Helper to invoke Tauri commands with error handling
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const perfStore = usePerformanceStoreRaw()
  const startTime = perfStore.isMonitoring ? performance.now() : 0

  try {
    const result = await invoke<CommandResult<T>>(command, args)
    logger.debug('Tauri命令执行成功', { command, result: JSON.stringify(result) })

    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: result.success,
        error: result.success ? undefined : (result.error || undefined),
        timestamp: Date.now(),
      })
    }

    if (result.success) {
      return result.data as T
    }
    throw new Error(result.error || 'Unknown error')
  } catch (error) {
    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: false,
        error: error instanceof Error ? error.message : String(error),
        timestamp: Date.now(),
      })
    }
    logger.error('Tauri命令执行失败', { command, error })
    throw error
  }
}

// Helper that returns the full CommandResult (for cases where we need to check success)
export async function invokeCommandRaw<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<CommandResult<T>> {
  const perfStore = usePerformanceStoreRaw()
  const startTime = perfStore.isMonitoring ? performance.now() : 0

  try {
    const result = await invoke<CommandResult<T>>(command, args)

    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: result.success,
        error: result.success ? undefined : (result.error || undefined),
        timestamp: Date.now(),
      })
    }

    return result
  } catch (error) {
    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: false,
        error: error instanceof Error ? error.message : String(error),
        timestamp: Date.now(),
      })
    }
    logger.error('Tauri命令执行失败', { command, error })
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : String(error)
    }
  }
}

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
}

// ========================================
// Team Management
// ========================================

export interface Team {
  id: number
  region_id: number
  name: string
  short_name: string | null
  power_rating: number
  total_matches: number
  wins: number
  win_rate: number
  annual_points: number
  cross_year_points: number
  balance: number
}

export interface TeamRoster {
  team: Team
  starters: Player[]
  substitutes: Player[]
}

export const teamApi = {
  getTeamsByRegion: (regionId: number) =>
    invokeCommand<Team[]>('get_teams_by_region', { regionId }),

  getAllTeams: () =>
    invokeCommand<Team[]>('get_all_teams'),

  getAllPlayers: () =>
    invokeCommand<Player[]>('get_all_players'),

  getTeam: (teamId: number) =>
    invokeCommand<Team>('get_team', { teamId }),

  getTeamRoster: (teamId: number) =>
    invokeCommand<TeamRoster>('get_team_roster', { teamId }),

  getTeamStarters: (teamId: number) =>
    invokeCommand<Player[]>('get_team_starters', { teamId }),

  setStarter: (teamId: number, playerId: number, isStarter: boolean) =>
    invokeCommand<void>('set_starter', { teamId, playerId, isStarter }),
}

// ========================================
// Player Management
// ========================================

export interface Player {
  id: number
  game_id: string
  real_name: string | null
  team_id: number | null
  position: string | null
  nationality: string | null
  age: number
  ability: number
  potential: number
  stability: number
  salary: number
  market_value: number
  calculated_market_value: number  // 计算后的身价（含荣誉和赛区系数）
  contract_end_season: number | null
  status: string
  tag: string
  is_starter: boolean
  satisfaction: number  // 满意度 0-100
  loyalty: number  // 忠诚度 0-100
}

// 特性信息
export interface TraitInfo {
  trait_type: string
  name: string
  description: string
  rarity: number  // 1-5
  is_negative: boolean
}

// 状态因子信息（包含计算后的 condition）
export interface PlayerConditionInfo {
  player_id: number
  form_cycle: number
  momentum: number          // -5 ~ +5
  last_performance: number
  last_match_won: boolean
  games_since_rest: number
  condition: number         // -10 ~ +10
  condition_range: [number, number]  // 年龄对应的范围
}

// 选手完整详情
export interface PlayerFullDetail {
  player: Player
  traits: TraitInfo[]
  condition_info: PlayerConditionInfo
}

// 选手属性更新请求
export interface UpdatePlayerRequest {
  player_id: number
  ability?: number
  potential?: number
  stability?: number
  age?: number
}

export const playerApi = {
  getPlayer: (playerId: number) =>
    invokeCommand<Player>('get_player', { playerId }),

  // 获取选手特性列表
  getPlayerTraits: (playerId: number) =>
    invokeCommand<TraitInfo[]>('get_player_traits', { playerId }),

  // 获取选手状态因子和 condition
  getPlayerCondition: (playerId: number) =>
    invokeCommand<PlayerConditionInfo>('get_player_condition', { playerId }),

  // 获取选手完整详情（包含特性和状态）
  getPlayerFullDetail: (playerId: number) =>
    invokeCommand<PlayerFullDetail>('get_player_full_detail', { playerId }),

  // 更新选手属性（能力值、潜力值、稳定性、年龄）
  updatePlayer: (request: UpdatePlayerRequest) =>
    invokeCommand<Player>('update_player', { request }),
}

// ========================================
// Tournament & Match Management
// ========================================

export interface TournamentMatch {
  id: number
  tournament_id: number
  stage: string
  round: number | null
  match_order: number | null
  format: string
  home_team_id: number | null
  away_team_id: number | null
  home_team_name: string | null
  away_team_name: string | null
  home_score: number
  away_score: number
  winner_id: number | null
  status: string
}

export interface Standing {
  team_id: number
  team_name: string
  wins: number
  losses: number
  round_diff: number
  points: number
  rank: number
}

export const tournamentApi = {
  getTournamentMatches: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('get_tournament_matches', { tournamentId }),

  getStandings: (tournamentId: number) =>
    invokeCommand<Standing[]>('get_standings', { tournamentId }),

  simulateNextMatch: (tournamentId: number) =>
    invokeCommand<TournamentMatch | null>('simulate_next_match', { tournamentId }),

  simulateAllMatches: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('simulate_all_matches', { tournamentId }),

  // getSchedule is alias for getTournamentMatches
  getSchedule: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('get_tournament_matches', { tournamentId }),
}

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

// ========================================
// Draft System
// ========================================

export interface DraftPlayer {
  id: number
  game_id: string
  position: string
  nationality: string | null
  age: number
  ability: number
  potential: number
  tag: string
  draft_rank: number
  is_picked: boolean
}

export interface DraftOrder {
  team_id: number
  team_name: string
  original_team_id: number | null
  summer_rank: number
  draft_position: number
  lottery_result: string | null
}

export interface DraftPick {
  pick_number: number
  team_id: number
  team_name: string
  player: DraftPlayer
}

export interface DraftResultItem {
  pick_number: number
  team_id: number
  team_name: string
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
}

export interface DraftRegionStatus {
  status: 'not_started' | 'roster_drawn' | 'lottery_done' | 'completed'
  draft_players: DraftPlayer[]
  draft_results: DraftResultItem[]
  draft_orders: DraftOrder[]
  total_players: number
  picked_count: number
}

export interface DraftPoolPlayer {
  id: number
  game_id: string
  real_name: string | null
  nationality: string | null
  age: number
  ability: number
  potential: number
  position: string
  tag: string
  status: string
}

export interface NewDraftPoolPlayer {
  game_id: string
  real_name?: string | null
  nationality?: string | null
  age: number
  ability: number
  potential: number
  position: string
  tag: string
}

export interface UpdateDraftPoolPlayer {
  game_id: string
  ability: number
  potential: number
  position: string
  tag: string
}

export const draftApi = {
  generateDraftPool: (regionId: number, poolSize?: number) =>
    invokeCommand<DraftPlayer[]>('generate_draft_pool', {
      regionId,
      poolSize: poolSize ?? 14
    }),

  runDraftLottery: (regionId: number) =>
    invokeCommand<DraftOrder[]>('run_draft_lottery', { regionId }),

  getDraftOrder: (regionId: number) =>
    invokeCommand<DraftOrder[]>('get_draft_order', { regionId }),

  getAvailableDraftPlayers: (regionId: number) =>
    invokeCommand<DraftPlayer[]>('get_available_draft_players', { regionId }),

  makeDraftPick: (regionId: number, teamId: number, playerId: number) =>
    invokeCommand<DraftPick>('make_draft_pick', { regionId, teamId, playerId }),

  aiAutoDraft: (regionId: number) =>
    invokeCommand<DraftPick[]>('ai_auto_draft', { regionId }),

  // 选手池管理
  getDraftPoolPlayers: (regionId: number) =>
    invokeCommand<DraftPoolPlayer[]>('get_draft_pool_players', { regionId }),

  addDraftPoolPlayers: (regionId: number, players: NewDraftPoolPlayer[]) =>
    invokeCommand<number>('add_draft_pool_players', { regionId, players }),

  updateDraftPoolPlayer: (playerId: number, data: UpdateDraftPoolPlayer) =>
    invokeCommand<void>('update_draft_pool_player', { playerId, ...data }),

  deleteDraftPoolPlayers: (regionId: number, playerIds?: number[]) =>
    invokeCommand<number>('delete_draft_pool_players', { regionId, playerIds }),

  // 选秀状态查询
  getDraftRegionStatus: (regionId: number, seasonId?: number) =>
    invokeCommand<DraftRegionStatus>('get_draft_region_status', { regionId, seasonId }),
}

// ========================================
// Draft Pick Auction System (选秀权拍卖)
// ========================================

// 拍卖挂牌信息
export interface AuctionListing {
  id: number
  seller_team_id: number
  seller_team_name: string
  draft_position: number
  position_name: string
  starting_price: number
  current_price: number
  min_increment: number
  status: string  // PENDING/ACTIVE/SOLD/WITHDRAWN/EXPIRED
  buyer_team_id: number | null
  buyer_team_name: string | null
  final_price: number | null
  current_bid_round: number
}

// 拍卖状态信息
export interface AuctionStatus {
  id: number
  status: string  // PREPARING/IN_PROGRESS/COMPLETED
  current_round: number
  total_rounds: number
  total_auctions: number
  successful_auctions: number
  total_revenue: number
  total_commission: number
  listings: AuctionListing[]
}

// 拍卖事件
export interface AuctionEvent {
  id: number
  event_type: string  // AUCTION_START/LISTING_CREATED/BID_PLACED/BID_RAISED/SOLD/WITHDRAWN/EXPIRED/AUCTION_END
  team_id: number | null
  team_name: string | null
  draft_position: number | null
  position_name: string | null
  amount: number | null
  headline: string
  description: string
  importance: string  // BREAKING/MAJOR/NORMAL/MINOR
  round: number
  created_at: string
}

// 签位价格配置
export interface DraftPickPrice {
  position: number
  name: string
  starting_price: number
  min_increment: number
}

export const draftAuctionApi = {
  // 获取签位价格配置
  getDraftPickPrices: () =>
    invokeCommand<DraftPickPrice[]>('get_draft_pick_prices'),

  // 开始拍卖
  startAuction: (regionId: number) =>
    invokeCommand<AuctionStatus>('start_draft_auction', { regionId }),

  // 执行一轮竞拍
  executeRound: (regionId: number) =>
    invokeCommand<AuctionStatus>('execute_auction_round', { regionId }),

  // 快进完成所有轮次
  fastForward: (regionId: number) =>
    invokeCommand<AuctionStatus>('fast_forward_auction', { regionId }),

  // 获取拍卖状态
  getStatus: (regionId: number) =>
    invokeCommand<AuctionStatus | null>('get_auction_status', { regionId }),

  // 获取拍卖事件
  getEvents: (regionId: number) =>
    invokeCommand<AuctionEvent[]>('get_auction_events', { regionId }),

  // 完成拍卖并更新选秀顺位
  finalizeAuction: (regionId: number) =>
    invokeCommand<boolean>('finalize_auction', { regionId }),
}

// ========================================
// Transfer Market
// ========================================

export interface TransferListing {
  id: number
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
  seller_team_id: number
  seller_team_name: string
  asking_price: number
  listing_type: string
  status: string
  listed_at: string
}

export interface FreeAgent {
  id: number
  name: string
  position: string
  nationality: string
  age: number
  ability: number
  potential: number
  tag: string
  expected_salary: number
}

export interface TransferRecord {
  id: number
  player_id: number
  player_name: string
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number
  to_team_name: string
  transfer_type: string
  fee: number
  salary: number
  contract_years: number
  transferred_at: string
}

export const transferApi = {
  getTransferMarket: () =>
    invokeCommand<TransferListing[]>('get_transfer_market'),

  getFreeAgents: () =>
    invokeCommand<FreeAgent[]>('get_free_agents'),

  listPlayerForTransfer: (teamId: number, playerId: number, askingPrice: number) =>
    invokeCommand<TransferListing>('list_player_for_transfer', {
      teamId,
      playerId,
      askingPrice
    }),

  cancelTransferListing: (listingId: number) =>
    invokeCommand<void>('cancel_transfer_listing', { listingId }),

  buyListedPlayer: (listingId: number, buyerTeamId: number, contractYears: number, salary: number) =>
    invokeCommand<TransferRecord>('buy_listed_player', {
      listingId,
      buyerTeamId,
      contractYears,
      salary
    }),

  signFreeAgent: (playerId: number, teamId: number, contractYears: number, salary: number) =>
    invokeCommand<TransferRecord>('sign_free_agent', {
      playerId,
      teamId,
      contractYears,
      salary
    }),

  getTransferHistory: (teamId?: number) =>
    invokeCommand<TransferRecord[]>('get_transfer_history', { teamId }),

  // ========== AI 转会窗口 API ==========

  // 开始转会窗口
  startTransferWindow: () =>
    invokeCommand<TransferWindowInfo>('start_transfer_window'),

  // 执行下一轮转会
  executeTransferRound: () =>
    invokeCommand<TransferRoundInfo>('execute_transfer_round'),

  // 快进完成所有转会
  fastForwardTransfers: () =>
    invokeCommand<TransferWindowInfo>('fast_forward_transfers'),

  // 获取转会窗口状态
  getTransferWindowStatus: () =>
    invokeCommand<TransferWindowInfo>('get_transfer_window_status'),

  // 获取转会事件列表
  getTransferEvents: (round?: number) =>
    invokeCommand<TransferEventInfo[]>('get_transfer_events', { round }),

  // ========== 市场分析和选手市场 API ==========

  // 获取球队转会计划列表
  getTeamTransferPlans: () =>
    invokeCommand<TeamTransferPlanInfo[]>('get_team_transfer_plans'),

  // 获取选手市场列表
  getPlayerMarketList: () =>
    invokeCommand<PlayerMarketInfo[]>('get_player_market_list'),

  // 获取选手合同详情
  getPlayerContractDetail: (playerId: number) =>
    invokeCommand<PlayerContractDetail>('get_player_contract_detail', { playerId }),
}

// AI 转会窗口类型定义
export interface TransferWindowInfo {
  id: number
  season_id: number
  status: string  // 'PREPARING' | 'IN_PROGRESS' | 'COMPLETED'
  current_round: number
  total_rounds: number
  total_transfers: number
  total_fees: number
  free_agents_signed: number
  retirements: number
  contract_expires: number
  started_at: string | null
  completed_at: string | null
}

export interface TransferEventInfo {
  id: number
  round: number
  event_type: string  // 'FREE_AGENT' | 'PURCHASE' | 'RETIREMENT' | 'CONTRACT_EXPIRE' | 'TRANSFER_REQUEST' | 'CONTRACT_RENEWAL' | 'RENEWAL_FAILED' | 'STAR_POACHED' | 'LOYALTY_STAY' | 'REBUILD_SALE'
  status: string
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  market_value: number
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number | null
  to_team_name: string | null
  transfer_fee: number
  new_salary: number | null
  contract_years: number | null
  contract_type: string
  price_ratio: number | null
  headline: string
  description: string
  importance: string  // 'BREAKING' | 'MAJOR' | 'NORMAL' | 'MINOR'
  competing_teams: number[]
  was_bidding_war: boolean
  created_at: string | null
}

export interface TransferRoundInfo {
  round: number
  round_name: string
  events_count: number
  transfers_count: number
  total_fees: number
  summary: string
  events: TransferEventInfo[]
}

// 选手满意度/忠诚度状态信息
export interface PlayerStatusInfo {
  player_id: number
  player_name: string
  satisfaction: number  // 0-100
  satisfaction_trend: number  // 相比上赛季的变化
  loyalty: number  // 0-100
  loyalty_type: string  // '忠心耿耿' | '忠诚' | '中立' | '机会主义' | '雇佣兵'
  wants_to_leave: boolean
  departure_reasons: string[]
}

// 想离队选手信息 (用于旧版选手市场)
export interface PlayerStatusDepartureInfo {
  player: PlayerStatusInfo
  team_id: number
  team_name: string
  market_value: number
  primary_reason: string
}

// ========================================
// 市场分析和选手市场类型定义
// ========================================

// 球队转会计划信息
export interface TeamTransferPlanInfo {
  team_id: number
  team_name: string
  region_code: string
  // 财务
  balance: number
  financial_status: string  // 'Wealthy' | 'Healthy' | 'Struggling' | 'Bankrupt'
  transfer_budget: number
  salary_space: number
  current_total_salary: number
  // 阵容
  roster_count: number
  avg_ability: number
  avg_age: number
  // 位置需求 (0-100)
  position_needs: Record<string, number>
  // 策略
  strategy: string  // 'AggressiveBuy' | 'Passive' | 'MustSell' | 'ForceClear' | 'FullRebuild' | 'StarHunting'
  ambition: string  // 'Championship' | 'Playoff' | 'Rebuild'
  // 标记
  must_sign: boolean
  must_clear: boolean
}

// 选手市场信息
export interface PlayerMarketInfo {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  // 战队信息
  team_id: number | null
  team_name: string | null
  region_code: string | null
  // 合同信息
  salary: number
  contract_end_season: number | null
  join_season: number | null
  // 身价信息
  base_market_value: number
  calculated_market_value: number
  // 状态信息
  satisfaction: number
  loyalty: number
  is_starter: boolean
  status: string
}

// 忠诚度变化记录
export interface LoyaltyChangeInfo {
  season_id: number
  change_amount: number
  reason: string
}

// 身价变化记录
export interface MarketValueChangeInfo {
  season_id: number
  old_value: number
  new_value: number
  change_amount: number
  change_percent: number
  reason: string
}

// 选手合同详情
export interface PlayerContractDetail {
  // 基础信息
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  potential: number
  stability: number
  // 战队
  team_id: number | null
  team_name: string | null
  region_code: string | null
  // 合同
  salary: number
  contract_end_season: number | null
  join_season: number
  years_in_team: number
  // 身价详情
  base_market_value: number
  honor_factor: number
  region_factor: number
  calculated_market_value: number
  // 满意度详情
  satisfaction: number
  // 忠诚度详情
  loyalty: number
  loyalty_type: string
  departure_threshold: number
  loyalty_price_factor: number
  wants_to_leave: boolean
  departure_reasons: string[]
  // 历史
  market_value_history: MarketValueChangeInfo[]
  loyalty_changes: LoyaltyChangeInfo[]
}

// ========================================
// Finance System
// ========================================

export interface TeamFinanceSummary {
  team_id: number
  team_name: string
  short_name: string | null
  region_id: number
  region_code: string
  balance: number
  total_income: number
  total_expense: number
  financial_status: 'Wealthy' | 'Healthy' | 'Tight' | 'Deficit' | 'Bankrupt'
  is_crisis: boolean
  transfer_budget: number
  max_new_salary: number
  projected_season_profit: number
  total_salary: number
}

export interface FinanceTransaction {
  id: number
  team_id: number
  amount: number
  transaction_type: string
  description: string
  created_at: string
}

export interface SeasonFinanceReport {
  team_id: number
  team_name: string
  season_id: number
  total_income: number
  total_expense: number
  net_profit: number
  breakdown: {
    prize_money: number
    league_share: number
    transfers_in: number
    salaries_paid: number
    transfers_out: number
    other: number
  }
}

export interface PrizePoolInfo {
  tournament_id: number
  tournament_name: string
  total_pool: number
  distribution: {
    position: number
    amount: number
  }[]
}

export interface TournamentPrizeDetail {
  tournament_id: number
  tournament_name: string
  tournament_type: string  // "international" or "regional"
  season_id: number
  position: string
  amount: number
}

export const financeApi = {
  getTeamFinanceSummary: (teamId: number) =>
    invokeCommand<TeamFinanceSummary>('get_team_finance_summary', { teamId }),

  getAllTeamsFinance: (regionId?: number) =>
    invokeCommand<TeamFinanceSummary[]>('get_all_teams_finance', { regionId }),

  getTeamTransactions: (teamId: number, seasonId?: number) =>
    invokeCommand<FinanceTransaction[]>('get_team_transactions', {
      teamId,
      seasonId
    }),

  recordTransaction: (teamId: number, amount: number, transactionType: string, description: string) =>
    invokeCommand<FinanceTransaction>('record_transaction', {
      teamId,
      amount,
      transactionType,
      description
    }),

  getSeasonFinanceReport: (teamId: number, seasonId?: number) =>
    invokeCommand<SeasonFinanceReport>('get_season_finance_report', {
      teamId,
      seasonId
    }),

  payTeamSalaries: (teamId: number) =>
    invokeCommand<FinanceTransaction>('pay_team_salaries', { teamId }),

  distributeLeagueShare: (regionId: number) =>
    invokeCommand<void>('distribute_league_share', { regionId }),

  getPrizePoolInfo: (tournamentId: number) =>
    invokeCommand<PrizePoolInfo>('get_prize_pool_info', { tournamentId }),

  distributeTournamentPrizes: (tournamentId: number) =>
    invokeCommand<void>('distribute_tournament_prizes', { tournamentId }),

  getTeamPrizeDetails: (teamId: number, seasonId?: number) =>
    invokeCommand<TournamentPrizeDetail[]>('get_team_prize_details', { teamId, seasonId }),
}

// ========================================
// Query System
// ========================================

export interface Region {
  id: number
  code: string
  name: string
  team_count: number
}

export interface RegionDetail {
  region: Region
  teams: Team[]
  current_tournament: TournamentInfo | null
}

export interface TournamentInfo {
  id: number
  name: string
  tournament_type: string
  season_id: number
  region_id: number | null
  status: string
  champion_team_id?: number | null
  champion_team_name?: string | null
  match_count: number
  completed_matches: number
}

export interface SeasonOverview {
  season_id: number
  tournaments: TournamentInfo[]
  top_teams: Team[]
  recent_champions: HonorRecord[]
}

export interface SearchResult {
  teams: Team[]
  players: Player[]
}

export const queryApi = {
  getAllRegions: () =>
    invokeCommand<Region[]>('get_all_regions'),

  getRegionDetail: (regionId: number) =>
    invokeCommand<RegionDetail>('get_region_detail', { regionId }),

  getSeasonTournaments: (seasonId: number) =>
    invokeCommand<TournamentInfo[]>('get_season_tournaments', { seasonId }),

  getRegionTournaments: (regionId: number, seasonId?: number) =>
    invokeCommand<TournamentInfo[]>('get_region_tournaments', { regionId, seasonId }),

  getTournamentDetail: (tournamentId: number) =>
    invokeCommand<TournamentInfo>('get_tournament_detail', { tournamentId }),

  getInternationalTournaments: (seasonId?: number) =>
    invokeCommand<TournamentInfo[]>('get_international_tournaments', { seasonId }),

  getSeasonOverview: (seasonId: number) =>
    invokeCommand<SeasonOverview>('get_season_overview', { seasonId }),

  searchTeams: (query: string) =>
    invokeCommand<Team[]>('search_teams', { query }),

  searchPlayers: (query: string) =>
    invokeCommand<Player[]>('search_players', { query }),

  getTeamsByRegion: (regionId: number) =>
    invokeCommand<Team[]>('get_teams_by_region', { regionId }),
}

// ========================================
// International Tournaments
// ========================================

export interface BracketInfo {
  tournament_id: number
  tournament_name: string
  tournament_type: string
  stages: StageInfo[]
  matches: MatchBracketInfo[]
}

export interface StageInfo {
  name: string
  display_name: string
  order: number
  total_matches: number
  completed_matches: number
}

export interface MatchBracketInfo {
  match_id: number
  stage: string
  match_order: number
  format: string
  home_team: TeamBracketInfo | null
  away_team: TeamBracketInfo | null
  home_score: number
  away_score: number
  winner_id: number | null
  status: string
}

export interface TeamBracketInfo {
  id: number
  name: string
  short_name: string | null
  region_code: string
}

export interface SwissRoundStatus {
  current_round: number
  teams: SwissTeamStatus[]
  completed: boolean
  qualified_teams: number[]
  eliminated_teams: number[]
}

export interface SwissTeamStatus {
  team_id: number
  team_name: string
  wins: number
  losses: number
  is_qualified: boolean
  is_eliminated: boolean
}

export interface GroupStandingInfo {
  group_name: string
  teams: TeamGroupStats[]
}

export interface TeamGroupStats {
  team_id: number
  team_name: string
  region_code: string
  wins: number
  losses: number
  games_won: number
  games_lost: number
  points: number
}

export const internationalApi = {
  createMsiTournament: (
    legendaryTeamIds: number[],
    challengerTeamIds: number[],
    qualifierTeamIds: number[]
  ) => invokeCommand<number>('create_msi_tournament', {
    legendaryTeamIds,
    challengerTeamIds,
    qualifierTeamIds
  }),

  createWorldsTournament: (directTeamIds: number[], groupTeamIds: number[]) =>
    invokeCommand<number>('create_worlds_tournament', {
      directTeamIds,
      groupTeamIds
    }),

  createMastersTournament: (tournamentType: string, teamIds: number[]) =>
    invokeCommand<number>('create_masters_tournament', {
      tournamentType,
      teamIds
    }),

  createSuperTournament: (
    legendaryTeamIds: number[],
    challengerTeamIds: number[],
    fighterTeamIds: number[]
  ) => invokeCommand<number>('create_super_tournament', {
    legendaryTeamIds,
    challengerTeamIds,
    fighterTeamIds
  }),

  getTournamentBracket: (tournamentId: number) =>
    invokeCommand<BracketInfo>('get_tournament_bracket', { tournamentId }),

  advanceBracket: (tournamentId: number, completedMatchId: number, winnerId: number) =>
    invokeCommand<number[]>('advance_bracket', {
      tournamentId,
      completedMatchId,
      winnerId
    }),

  getSwissRoundStatus: (tournamentId: number) =>
    invokeCommand<SwissRoundStatus>('get_swiss_round_status', { tournamentId }),

  generateNextSwissRound: (tournamentId: number) =>
    invokeCommand<number[]>('generate_next_swiss_round', { tournamentId }),

  // 填充世界赛淘汰赛对阵（瑞士轮完成后调用）
  fillWorldsKnockoutBracket: (tournamentId: number, qualifiedTeamIds: number[]) =>
    invokeCommand<number[]>('fill_worlds_knockout_bracket', { tournamentId, qualifiedTeamIds }),

  // 清理重复赛事
  cleanupDuplicateTournaments: (tournamentType: string) =>
    invokeCommand<number>('cleanup_duplicate_tournaments', { tournamentType }),

  // ICP洲际对抗赛
  createIcpTournament: (regionTeams: number[][]) =>
    invokeCommand<number>('create_icp_tournament', { regionTeams }),

  // 小组赛积分榜
  getGroupStandings: (tournamentId: number) =>
    invokeCommand<GroupStandingInfo[]>('get_group_standings', { tournamentId }),

  // 生成淘汰赛对阵
  generateKnockoutBracket: (tournamentId: number) =>
    invokeCommand<number[]>('generate_knockout_bracket', { tournamentId }),

  // 生成Super赛事第三阶段（冠军预备战）
  generateChampionPrepStage: (tournamentId: number) =>
    invokeCommand<number[]>('generate_champion_prep_stage', { tournamentId }),

  // 生成Super赛事第四阶段（终极冠军赛）
  generateFinalStage: (tournamentId: number) =>
    invokeCommand<number[]>('generate_final_stage', { tournamentId }),

  // 完成赛事 - 处理荣誉殿堂和年度积分
  completeTournament: (tournamentId: number) =>
    invokeCommand<TournamentCompletionResult>('complete_tournament', { tournamentId }),

  // 获取MSI参赛队伍分组（基于春季季后赛结果）
  getMsiQualifiedTeams: (seasonId: number) =>
    invokeCommand<MsiTeamGroups>('get_msi_qualified_teams', { seasonId }),

  // 重新生成MSI对阵（当队伍就绪但比赛未生成时使用）
  regenerateMsiBracket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_msi_bracket', { tournamentId }),

  // 根据类型获取赛事列表
  getTournamentsByType: (tournamentType: string, seasonId: number) =>
    invokeCommand<TournamentInfo[]>('get_tournaments_by_type', { tournamentType, seasonId }),

  // 获取上海大师赛参赛队伍分组（基于夏季季后赛结果）
  getShanghaiQualifiedTeams: (seasonId: number) =>
    invokeCommand<MsiTeamGroups>('get_shanghai_qualified_teams', { seasonId }),

  // 重新生成上海大师赛对阵（删除现有比赛并重新初始化）
  regenerateShanghairacket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_shanghai_bracket', { tournamentId }),

  // 重新生成ICP洲际对抗赛对阵（删除现有比赛并重新初始化）
  regenerateIcpBracket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_icp_bracket', { tournamentId }),
}

// MSI参赛队伍分组信息
export interface MsiTeamGroups {
  legendary: MsiTeamInfo[]
  challenger: MsiTeamInfo[]
  qualifier: MsiTeamInfo[]
}

// MSI队伍信息
export interface MsiTeamInfo {
  team_id: number
  team_name: string
  short_name: string
  region_id: number
  region_name: string
}

// 赛事完成结果
export interface TournamentCompletionResult {
  tournament_id: number
  tournament_name: string
  honors_awarded: HonorAwardedInfo[]
  points_awarded: PointsAwardedInfo[]
  message: string
}

// 颁发的荣誉信息
export interface HonorAwardedInfo {
  honor_type: string
  recipient_name: string
  recipient_type: string // "team" or "player"
}

// 颁发的积分信息
export interface PointsAwardedInfo {
  team_id: number
  team_name: string
  points: number
  position: string
}

// ========================================
// Match Simulation
// ========================================

export interface DetailedMatchResult {
  match_id: number
  tournament_id: number
  home_team_id: number
  away_team_id: number
  home_score: number
  away_score: number
  winner_id: number
  games: DetailedGameResult[]
  match_mvp: MvpInfo | null
  home_team_stats: TeamMatchStats
  away_team_stats: TeamMatchStats
}

export interface MvpInfo {
  player_id: number
  player_name: string
  team_id: number
  position: string
  mvp_score: number
}

export interface DetailedGameResult {
  game_number: number
  winner_id: number
  duration_minutes: number
  home_performance: number
  away_performance: number
  game_mvp: MvpInfo
  home_players: PlayerGameStats[]
  away_players: PlayerGameStats[]
  key_events: GameEvent[]
}

export interface PlayerGameStats {
  player_id: number
  player_name: string
  position: string
  base_ability: number
  condition_bonus: number
  stability_noise: number
  actual_ability: number
  kills: number
  deaths: number
  assists: number
  cs: number
  gold: number
  damage_dealt: number
  damage_taken: number
  vision_score: number
  mvp_score: number
  impact_score: number
  // 特性系统
  traits: string[]                          // 选手拥有的特性列表
  activated_traits: ActivatedTraitInfo[]    // 本局激活的特性效果
}

// 激活的特性效果信息
export interface ActivatedTraitInfo {
  trait_type: string
  name: string           // 特性显示名称
  effect: string         // 效果描述
  value: number          // 效果数值
  is_positive: boolean   // 是否为正面效果
}

export interface GameEvent {
  time_minutes: number
  event_type: string
  description: string
  team_id: number
}

export interface TeamMatchStats {
  team_id: number
  total_kills: number
  total_deaths: number
  total_assists: number
  total_gold: number
  total_damage: number
}

export interface PlayerSeasonStats {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  matches_played: number
  games_played: number
  wins: number
  losses: number
  total_kills: number
  total_deaths: number
  total_assists: number
  avg_kills: number
  avg_deaths: number
  avg_assists: number
  kda: number
  avg_cs: number
  avg_damage: number
  avg_gold: number
  mvp_count: number
}

export interface MatchPrediction {
  home_team_id: number
  home_team_name: string
  away_team_id: number
  away_team_name: string
  home_win_probability: number
  away_win_probability: number
  predicted_winner_id: number
  predicted_score: string
  key_factors: string[]
}

export const matchApi = {
  simulateMatchDetailed: (matchId: number) =>
    invokeCommand<DetailedMatchResult>('simulate_match_detailed', { matchId }),

  getPlayerSeasonStats: (playerId: number, seasonId?: number) =>
    invokeCommand<PlayerSeasonStats>('get_player_season_stats', { playerId, seasonId }),

  getMatchPrediction: (homeTeamId: number, awayTeamId: number) =>
    invokeCommand<MatchPrediction>('get_match_prediction', { homeTeamId, awayTeamId }),

  /** 更新比赛结果（用于本地模拟后同步数据库） */
  updateMatchResult: (matchId: number, homeScore: number, awayScore: number, winnerId: number) =>
    invokeCommand<boolean>('update_match_result', { matchId, homeScore, awayScore, winnerId }),

  /** 更新比赛队伍（用于填充淘汰赛待定队伍） */
  updateMatchTeams: (matchId: number, homeTeamId: number, awayTeamId: number) =>
    invokeCommand<boolean>('update_match_teams', { matchId, homeTeamId, awayTeamId }),

  /** 取消比赛（标记为 CANCELLED） */
  cancelMatch: (matchId: number) =>
    invokeCommand<boolean>('cancel_match', { matchId }),
}

// ========================================
// Event System
// ========================================

export interface SeasonSettlementPreview {
  retiring_players: RetiringPlayer[]
  expiring_contracts: ExpiringContract[]
  age_updates: AgeUpdate[]
}

export interface RetiringPlayer {
  player_id: number
  player_name: string
  team_id: number | null
  age: number
  ability: number
  reason: string
  reason_description: string
}

export interface ExpiringContract {
  player_id: number
  player_name: string
  team_id: number | null
  age: number
  ability: number
  contract_end_season: number | null
  salary: number
}

export interface AgeUpdate {
  player_id: number
  player_name: string
  old_age: number
  new_age: number
}

export interface SeasonEvent {
  id: number
  event_type: string
  player_id: number | null
  player_name: string | null
  team_id: number | null
  team_name: string | null
  description: string
  season_id: number
  created_at: string
}

export const eventApi = {
  previewSeasonSettlement: () =>
    invokeCommand<SeasonSettlementPreview>('preview_season_settlement'),

  executeSeasonSettlement: () =>
    invokeCommand<SeasonEvent[]>('execute_season_settlement'),

  getSeasonEvents: (seasonId: number) =>
    invokeCommand<SeasonEvent[]>('get_season_events', { seasonId }),

  getPlayerEvents: (playerId: number) =>
    invokeCommand<SeasonEvent[]>('get_player_events', { playerId }),

  getEventsByType: (eventType: string) =>
    invokeCommand<SeasonEvent[]>('get_events_by_type', { eventType }),

  updatePlayersAge: () =>
    invokeCommand<AgeUpdate[]>('update_players_age'),

  getRetiringCandidates: () =>
    invokeCommand<RetiringPlayer[]>('get_retiring_candidates'),

  getExpiringContracts: (currentSeason: number) =>
    invokeCommand<ExpiringContract[]>('get_expiring_contracts', { currentSeason }),
}

// ========================================
// Player Stats (Data Center)
// ========================================

export interface PlayerSeasonStatistics {
  id: number | null
  save_id: string
  player_id: number
  player_name: string
  season_id: number
  team_id: number | null
  region_id: string | null
  position: string
  matches_played: number
  games_played: number
  total_impact: number
  avg_impact: number
  avg_performance: number
  best_performance: number
  worst_performance: number
  consistency_score: number
  international_titles: number
  regional_titles: number
  champion_bonus: number
  yearly_top_score: number
}

export interface PlayerRankingItem {
  player_id: number
  player_name: string
  team_id: number | null
  position: string
  region_id: string | null
  games_played: number
  avg_impact: number
  avg_performance: number
  consistency_score: number
  champion_bonus: number
  yearly_top_score: number
}

export interface PlayerTournamentStats {
  id: number
  save_id: string
  season_id: number
  tournament_id: number
  tournament_type: string
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  games_played: number
  games_won: number
  total_impact: number
  avg_impact: number
  max_impact: number
  avg_performance: number
  best_performance: number
  game_mvp_count: number
  created_at?: string
  updated_at?: string
}

export interface RecordPerformanceParams {
  player_id: number
  player_name: string
  team_id: number
  position: string
  impact_score: number
  actual_ability: number
  season_id: number
  region_id?: string
}

export const statsApi = {
  // Record a single player performance
  recordPerformance: (params: RecordPerformanceParams) =>
    invokeCommand<PlayerSeasonStatistics>('record_player_performance', { params }),

  // Batch record player performances
  batchRecordPerformance: (performances: RecordPerformanceParams[]) =>
    invokeCommand<number>('batch_record_player_performance', { performances }),

  // Record championship (for all players in a team)
  recordChampionship: (teamId: number, isInternational: boolean, seasonId: number) =>
    invokeCommand<number>('record_championship', { teamId, isInternational, seasonId }),

  // Get season impact ranking
  getSeasonImpactRanking: (seasonId: number, limit?: number) =>
    invokeCommand<PlayerRankingItem[]>('get_season_impact_ranking', { seasonId, limit }),

  // Get position ranking
  getPositionRanking: (seasonId: number, position: string, limit?: number) =>
    invokeCommand<PlayerRankingItem[]>('get_position_ranking', { seasonId, position, limit }),

  // Get player stats
  getPlayerStats: (playerId: number, seasonId?: number) =>
    invokeCommand<PlayerSeasonStatistics[]>('get_player_stats', { playerId, seasonId }),

  // Get team player stats
  getTeamPlayerStats: (teamId: number, seasonId: number) =>
    invokeCommand<PlayerSeasonStatistics[]>('get_team_player_stats', { teamId, seasonId }),

  // Clear season stats
  clearSeasonStats: (seasonId: number) =>
    invokeCommand<boolean>('clear_season_stats', { seasonId }),

  // Get player impact history (real data from match performances)
  getPlayerImpactHistory: (playerId: number, seasonId?: number) =>
    invokeCommand<number[]>('get_player_impact_history', { playerId, seasonId }),

  // Get tournament MVP ranking (sorted by MVP count)
  getTournamentMvpRanking: (tournamentId: number, limit?: number) =>
    invokeCommand<PlayerTournamentStats[]>('get_tournament_mvp_ranking', { tournamentId, limit }),

  // Recalculate yearly scores for all players (use new formula: 50% impact + 50% champion bonus)
  recalculateYearlyScores: (seasonId: number) =>
    invokeCommand<number>('recalculate_yearly_scores', { seasonId }),

  // Get player market value changes history
  getPlayerMarketValueChanges: (playerId: number) =>
    invokeCommand<MarketValueChange[]>('get_player_market_value_changes', { playerId }),
}

// Market value change record
export interface MarketValueChange {
  id: number
  season_id: number
  player_id: number
  player_name: string
  old_value: number
  new_value: number
  change_amount: number
  change_percent: number
  reason: string
  created_at: string
}

// ========================================
// Test Functions
// ========================================

export interface TestMatchResult {
  home_score: number
  away_score: number
  winner: string
  games: {
    game_number: number
    home_performance: number
    away_performance: number
    winner: string
  }[]
  win_probability: number
}

export const testApi = {
  simulateTestMatch: (homePower: number, awayPower: number, format: string) =>
    invokeCommand<TestMatchResult>('simulate_test_match', {
      homePower,
      awayPower,
      format
    }),
}

// ========================================
// Time Progression System API
// ========================================

/** 阶段状态 */
export type PhaseStatus = 'NOT_INITIALIZED' | 'IN_PROGRESS' | 'COMPLETED'

/** 可用的时间操作 */
export type TimeAction =
  | 'INITIALIZE_PHASE'
  | 'SIMULATE_NEXT_MATCH'
  | 'SIMULATE_ALL_MATCHES'
  | 'COMPLETE_AND_ADVANCE'
  | 'FAST_FORWARD_PHASE'
  | 'FAST_FORWARD_TO_SUMMER'
  | 'FAST_FORWARD_TO_WORLDS'
  | 'FAST_FORWARD_TO_SEASON_END'
  | 'START_TRANSFER_WINDOW'
  | 'EXECUTE_TRANSFER_ROUND'
  | 'START_DRAFT'
  | 'START_NEW_SEASON'

/** 赛事进度 */
export interface TournamentProgress {
  tournament_id: number
  tournament_name: string
  region: string | null
  total_matches: number
  completed_matches: number
  status: string
}

/** 阶段进度 */
export interface PhaseProgress {
  tournaments: TournamentProgress[]
  total_matches: number
  completed_matches: number
  percentage: number
}

/** 阶段信息 */
export interface PhaseInfo {
  phase: string
  display_name: string
  status: string // "completed" | "current" | "upcoming"
  index: number
}

/** 赛季进度 */
export interface SeasonProgress {
  phases: PhaseInfo[]
  current_phase_index: number
  total_phases: number
  percentage: number
}

/** 荣誉信息 */
export interface HonorInfo {
  honor_type: string
  recipient_name: string
  tournament_name: string
}

/** 游戏时间状态 - 统一的时间状态返回结构 */
export interface GameTimeState {
  save_id: string
  current_season: number
  current_phase: string
  phase_display_name: string
  phase_status: PhaseStatus
  phase_progress: PhaseProgress
  season_progress: SeasonProgress
  available_actions: TimeAction[]
  can_advance: boolean
  next_phase: string | null
}

/** 完成并推进结果 */
export interface CompleteAndAdvanceResult {
  success: boolean
  completed_phase: string
  new_phase: string | null
  honors_awarded: HonorInfo[]
  message: string
  new_time_state: GameTimeState
}

/** 快进结果 */
export interface FastForwardResult {
  success: boolean
  start_phase: string
  end_phase: string
  phases_advanced: number
  matches_simulated: number
  message: string
  skipped_phases?: string[]
}

/** 新赛季初始化结果 */
export interface NewSeasonResult {
  new_season: number
  starters_confirmed: number
  message: string
}

/** 单场模拟结果 */
export interface SimulateNextResult {
  match_id: number
  tournament_name: string
  home_team_name: string
  away_team_name: string
  home_score: number
  away_score: number
  winner_name: string
  remaining_matches: number
  phase_completed: boolean
}

export const timeApi = {
  /** 获取完整的游戏时间状态 */
  getTimeState: () =>
    invokeCommand<GameTimeState>('get_time_state'),

  /** 初始化当前阶段（创建赛事） */
  initPhase: () =>
    invokeCommand<string>('time_init_phase'),

  /** 完成当前阶段并推进到下一阶段 */
  completeAndAdvance: () =>
    invokeCommand<CompleteAndAdvanceResult>('complete_and_advance'),

  /** 快进到指定目标 */
  fastForwardTo: (target: string) =>
    invokeCommand<FastForwardResult>('fast_forward_to', { target }),

  /** 模拟所有当前阶段的比赛 */
  simulateAll: () =>
    invokeCommand<number>('time_simulate_all'),

  /** 模拟下一场比赛 */
  simulateNext: () =>
    invokeCommand<SimulateNextResult>('time_simulate_next'),

  /** 开始新赛季 */
  startNewSeason: () =>
    invokeCommand<NewSeasonResult>('time_start_new_season'),

  /** 修复赛事状态 - 将已完成的赛事状态更新为 Completed */
  fixTournamentStatus: () =>
    invokeCommand<FixTournamentStatusResult>('fix_tournament_status'),
}

/** 修复赛事状态结果 */
export interface FixTournamentStatusResult {
  fixed_count: number
  fixed_tournaments: string[]
  message: string
}

// ========================================
// 年度积分 API
// ========================================

/** 队伍年度积分 */
export interface TeamAnnualPoints {
  rank: number
  team_id: number
  team_name: string
  team_short_name: string | null
  region_id: number
  region_code: string
  total_points: number
  tournaments_count: number
}

/** 积分明细 */
export interface AnnualPointsDetail {
  id: number
  save_id: string
  season_id: number
  team_id: number
  tournament_id: number
  tournament_name?: string
  tournament_type?: string
  points: number
  final_rank: number | null
}

export const pointsApi = {
  /** 获取年度积分排名 */
  getRankings: (seasonId?: number) =>
    invokeCommand<TeamAnnualPoints[]>('get_annual_points_ranking', seasonId != null ? { seasonId } : undefined),

  /** 获取队伍的积分明细 */
  getTeamPoints: (teamId: number, seasonId?: number) =>
    invokeCommand<AnnualPointsDetail[]>('get_team_points_detail', { teamId, ...(seasonId != null ? { seasonId } : {}) }),

  /** 获取赛事的积分发放记录 */
  getTournamentPoints: (tournamentId: number) =>
    invokeCommand<AnnualPointsDetail[]>('get_tournament_points', { tournamentId }),

  /** 获取Super资格队伍（Top16） */
  getSuperQualifiedTeams: () =>
    invokeCommand<TeamAnnualPoints[]>('get_super_qualified_teams'),
}

// ========================================
// 年度颁奖 API
// ========================================

/** 年度最佳阵容选手信息 */
export interface AllProPlayer {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  yearly_score: number
  avg_impact: number
  games_played: number
}

/** 年度Top20选手信息 */
export interface Top20Player {
  rank: number
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  yearly_score: number
  avg_impact: number
  games_played: number
}

/** 年度最佳新秀信息 */
export interface RookiePlayer {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  age: number
  yearly_score: number
  avg_impact: number
  games_played: number
}

/** 年度颁奖数据 */
export interface AnnualAwardsData {
  season_id: number
  top20: Top20Player[]
  all_pro_team: AllProPlayer[]
  rookie_of_the_year: RookiePlayer | null
  already_awarded: boolean
}

export const awardsApi = {
  /** 获取年度颁奖数据 */
  getAnnualAwardsData: (seasonId?: number) =>
    invokeCommand<AnnualAwardsData>('get_annual_awards_data', { seasonId }),
}

// ========================================
// 比赛详情持久化 API
// ========================================

/** 单局比赛详情 */
export interface MatchGameDetail {
  id: string
  save_id: string
  match_id: number
  game_number: number
  winner_team_id: number
  loser_team_id: number
  duration_minutes: number | null
  mvp_player_id: number | null
  key_player_id: number | null
  created_at: string | null
}

/** 选手单局表现 */
export interface GamePlayerPerformance {
  id: string
  save_id: string
  game_id: string
  player_id: number
  player_name: string           // 选手名称（快照）
  team_id: number
  team_name: string             // 队伍名称（快照）
  position: string
  base_ability: number          // 基础能力值
  condition_bonus: number       // 状态加成
  stability_noise: number       // 稳定性波动
  actual_ability: number        // 实际发挥值
  impact_score: number          // 影响力得分
  mvp_score: number             // MVP 得分
  is_mvp: boolean
  is_key_player: boolean
  // 详细战斗数据
  kills: number | null
  deaths: number | null
  assists: number | null
  cs: number | null
  gold: number | null
  damage_dealt: number | null
  damage_taken: number | null
  vision_score: number | null
  // 特性系统
  traits_json: string | null              // 选手拥有的特性 (JSON数组)
  activated_traits_json: string | null    // 本局激活的特性效果 (JSON数组)
  created_at: string | null
}

/** 单局详情（包含选手表现） */
export interface GameDetailWithPerformances {
  game: MatchGameDetail
  performances: GamePlayerPerformance[]
}

/** 完整比赛详情 */
export interface MatchFullDetails {
  match_id: number
  games: GameDetailWithPerformances[]
}

/** 保存选手表现输入 */
export interface SavePerformanceInput {
  player_id: number
  player_name: string           // 选手名称（快照）
  team_id: number
  team_name: string             // 队伍名称（快照）
  position: string
  base_ability: number          // 基础能力值
  condition_bonus: number       // 状态加成
  stability_noise: number       // 稳定性波动
  actual_ability: number        // 实际发挥值
  impact_score: number          // 影响力得分
  mvp_score: number             // MVP 得分
  is_mvp: boolean
  is_key_player: boolean
  // 详细战斗数据
  kills: number | null
  deaths: number | null
  assists: number | null
  cs: number | null
  gold: number | null
  damage_dealt: number | null
  damage_taken: number | null
  vision_score: number | null
  // 特性系统
  traits_json: string | null              // 选手拥有的特性 (JSON数组)
  activated_traits_json: string | null    // 本局激活的特性效果 (JSON数组)
}

/** 保存单局输入 */
export interface SaveGameInput {
  game_number: number
  winner_team_id: number
  loser_team_id: number
  duration_minutes: number | null
  mvp_player_id: number | null
  key_player_id: number | null
  performances: SavePerformanceInput[]
}

/** 保存比赛详情输入 */
export interface SaveMatchDetailsInput {
  match_id: number
  games: SaveGameInput[]
}

export const matchDetailsApi = {
  /** 保存比赛详情 */
  saveMatchDetails: (saveId: string, input: SaveMatchDetailsInput) =>
    invokeCommand<void>('save_match_details', { saveId, input }),

  /** 获取比赛详情 */
  getMatchDetails: (saveId: string, matchId: number) =>
    invokeCommand<MatchFullDetails | null>('get_match_details', { saveId, matchId }),

  /** 删除比赛详情 */
  deleteMatchDetails: (saveId: string, matchId: number) =>
    invokeCommand<void>('delete_match_details', { saveId, matchId }),
}

// ========================================
// 开发工具 API (Development Tools)
// ========================================

/** 数据一致性检查结果 */
export interface ConsistencyCheckResult {
  total_checks: number
  passed: number
  failed: number
  issues: ConsistencyIssue[]
}

export interface ConsistencyIssue {
  category: string
  description: string
  severity: 'warning' | 'error'
}

/** 同步结果 */
export interface SyncResult {
  updated_count: number
  details: string[]
}

/** 游戏状态摘要 */
export interface GameStatusSummary {
  current_season: number
  current_phase: string
  phase_completed: boolean
  team_count: number
  player_count: number
  tournament_count: number
  total_matches: number
  completed_matches: number
  scheduled_matches: number
  honor_count: number
}

/** 开发命令结果 */
export interface DevCommandResult<T> {
  success: boolean
  data: T | null
  message: string
  error: string | null
}

export const devApi = {
  // 荣誉系统
  /** 重新颁发荣誉 */
  reassignHonors: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_reassign_honors', { seasonId }),

  /** 重新计算年度积分 */
  recalculateAnnualPoints: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_annual_points', { seasonId }),

  // 数据修复
  /** 同步选手场次统计 */
  syncPlayerGamesPlayed: (seasonId?: number) =>
    invokeCommand<DevCommandResult<SyncResult>>('dev_sync_player_games_played', { seasonId }),

  /** 重新计算积分榜 */
  recalculateStandings: (tournamentId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_standings', { tournamentId }),

  /** 数据一致性检查 */
  checkDataConsistency: () =>
    invokeCommand<DevCommandResult<ConsistencyCheckResult>>('dev_check_data_consistency', {}),

  // 赛事管理
  /** 重置阶段状态 */
  resetPhase: () =>
    invokeCommand<DevCommandResult<void>>('dev_reset_phase', {}),

  /** 获取待模拟比赛数量 */
  simulateAllMatches: (tournamentId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_simulate_all_matches', { tournamentId }),

  // 财务系统
  /** 重新发放赛事奖金 */
  redistributePrizes: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_redistribute_prizes', { seasonId }),

  /** 给所有战队发放测试资金 */
  grantFunds: (amount: number) =>
    invokeCommand<DevCommandResult<number>>('dev_grant_funds', { amount }),

  // 快速测试
  /** 重置存档 */
  resetSave: (keepTeams: boolean) =>
    invokeCommand<DevCommandResult<void>>('dev_reset_save', { keepTeams }),

  /** 获取游戏状态摘要 */
  getGameStatus: () =>
    invokeCommand<DevCommandResult<GameStatusSummary>>('dev_get_game_status', {}),

  /** 检查未完成的比赛 */
  checkIncompleteMatches: (tournamentType?: string) =>
    invokeCommand<DevCommandResult<IncompleteMatchInfo[]>>('dev_check_incomplete_matches', { tournamentType }),

  /** 强制完成比赛 */
  forceCompleteMatch: (matchId: number) =>
    invokeCommand<DevCommandResult<void>>('dev_force_complete_match', { matchId }),

  /** 迁移选手忠诚度和满意度（根据选手属性重新计算） */
  migrateLoyaltySatisfaction: () =>
    invokeCommand<DevCommandResult<number>>('dev_migrate_loyalty_satisfaction'),

  /** 重新计算所有选手身价（使用新公式） */
  recalculateMarketValues: () =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_market_values'),

  /** 自动修复队伍首发阵容 */
  fixStarters: () =>
    invokeCommand<DevCommandResult<FixStartersResult>>('dev_fix_starters'),
}

export interface IncompleteMatchInfo {
  match_id: number
  stage: string
  status: string
  tournament_name: string
  tournament_type: string
  home_team: string | null
  away_team: string | null
}

export interface FixStartersResult {
  teams_fixed: number
  players_fixed: number
  details: TeamFixInfo[]
}

export interface TeamFixInfo {
  team_name: string
  fixes: string[]
}

// ========================================
// AI Transfer System (GM 配置和 AI 策略)
// ========================================

/** GM 人格类型 */
export type GMPersonalityType =
  | 'CHAMPIONSHIP'
  | 'YOUTH_DEVELOPMENT'
  | 'BALANCED'
  | 'SPECULATOR'
  | 'REBUILDING'
  | 'CUSTOM'

/** 出售策略激进度 */
export type SellAggressivenessType =
  | 'CONSERVATIVE'
  | 'NORMAL'
  | 'AGGRESSIVE'

/** GM 人格类型信息 */
export interface PersonalityTypeInfo {
  value: string
  name: string
  description: string
  icon: string
}

/** 球队 GM 配置信息 */
export interface TeamGMProfileInfo {
  team_id: number
  team_name: string
  team_short_name: string | null
  region_id: number
  region_name: string
  personality: GMPersonalityType
  personality_name: string
  personality_description: string
  custom_prompt: string | null
  risk_tolerance: number
  // 转会配置
  budget_ratio: number
  sell_aggressiveness: SellAggressivenessType
  preferred_age_min: number
  preferred_age_max: number
  min_ability_threshold: number
  price_premium_max: number
  position_priorities: Record<string, number>
  // 选秀配置
  draft_pick_sell_threshold: number
  draft_pick_bid_aggressiveness: number
  draft_preference_ability_weight: number
  draft_young_bias: number
}

/** 转会目标 */
export interface TransferTarget {
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
  age: number
  current_team_id: number | null
  current_team_name: string | null
  market_value: number
  max_offer: number
  priority: number
  reasoning: string
}

/** 出售候选 */
export interface SellCandidate {
  player_id: number
  player_name: string
  position: string
  ability: number
  age: number
  salary: number
  market_value: number
  min_price: number
  urgency: number
  reasoning: string
}

/** 预算分配 */
export interface BudgetAllocation {
  total_budget: number
  transfer_spend: number
  salary_spend: number
  reserve: number
}

/** AI 策略信息（用于展示） */
export interface AIStrategyInfo {
  team_id: number
  team_name: string
  season_id: number
  overall_strategy: string
  strategy_description: string
  targets_count: number
  sell_count: number
  priority_positions: string[]
  budget: BudgetAllocation  // 与后端匹配
  reasoning: string
  is_mock: boolean
  generated_at: string
}

/** AI 转会策略（完整） */
export interface AITransferStrategy {
  id: number
  team_id: number
  save_id: string
  season_id: number
  overall_strategy: string
  strategy_description: string
  targets: TransferTarget[]
  willing_to_sell: SellCandidate[]
  priority_positions: string[]
  budget_allocation: BudgetAllocation
  reasoning: string
  analysis_steps?: TeamAnalysisStep[]
  is_mock: boolean
  generated_at: string
}

/** 战队分析步骤 */
export interface TeamAnalysisStep {
  step_name: string
  data_used: string
  threshold: string
  result: string
  impact: string
}

export const aiTransferApi = {
  /** 获取所有 GM 人格类型 */
  getPersonalityTypes: () =>
    invokeCommand<PersonalityTypeInfo[]>('get_gm_personality_types'),

  /** 获取所有球队的 GM 配置 */
  getAllGMProfiles: () =>
    invokeCommand<TeamGMProfileInfo[]>('get_all_gm_profiles'),

  /** 获取单个球队的 GM 配置 */
  getTeamGMProfile: (teamId: number) =>
    invokeCommand<TeamGMProfileInfo>('get_team_gm_profile', { teamId }),

  /** 更新球队 GM 配置 */
  updateTeamGMProfile: (
    teamId: number,
    personality: string,
    customPrompt: string | null,
    riskTolerance: number,
    budgetRatio: number,
    sellAggressiveness: string,
    preferredAgeMin: number,
    preferredAgeMax: number,
    minAbilityThreshold: number,
    pricePremiumMax: number,
    positionPriorities: Record<string, number>,
    draftPickSellThreshold: number,
    draftPickBidAggressiveness: number,
    draftPreferenceAbilityWeight: number,
    draftYoungBias: number
  ) =>
    invokeCommand<void>('update_team_gm_profile', {
      teamId,
      personality,
      customPrompt,
      riskTolerance,
      budgetRatio,
      sellAggressiveness,
      preferredAgeMin,
      preferredAgeMax,
      minAbilityThreshold,
      pricePremiumMax,
      positionPriorities,
      draftPickSellThreshold,
      draftPickBidAggressiveness,
      draftPreferenceAbilityWeight,
      draftYoungBias,
    }),

  /** 批量更新 GM 配置 */
  batchUpdateGMProfiles: (
    profiles: [number, string, string | null, number, boolean][]
  ) =>
    invokeCommand<number>('batch_update_gm_profiles', { profiles }),

  /** 生成所有球队的 AI 策略 */
  generateAIStrategies: () =>
    invokeCommand<AIStrategyInfo[]>('generate_ai_strategies'),

  /** 获取单个球队的 AI 策略 */
  getTeamAIStrategy: (teamId: number) =>
    invokeCommand<AITransferStrategy>('get_team_ai_strategy', { teamId }),

  /** 初始化 AI 转会相关表 */
  initAITransferTables: () =>
    invokeCommand<void>('init_ai_transfer_tables'),

  // ========== 选手转会策略相关 ==========

  /** 为选手生成转会策略 */
  generatePlayerTransferStrategy: (playerId: number) =>
    invokeCommand<PlayerTransferStrategy>('generate_player_transfer_strategy', { playerId }),

  /** 获取选手的转会策略 */
  getPlayerTransferStrategy: (playerId: number) =>
    invokeCommand<PlayerTransferStrategy | null>('get_player_transfer_strategy', { playerId }),

  /** 获取所有想离队选手的策略列表 */
  getAllPlayerStrategies: () =>
    invokeCommand<PlayerTransferStrategyInfo[]>('get_all_player_strategies'),

  /** 初始化选手策略数据库表 */
  initPlayerStrategyTables: () =>
    invokeCommand<void>('init_player_strategy_tables'),
}

// ========== 选手转会策略类型 ==========

/** 偏好球队信息 */
export interface PreferredTeamInfo {
  team_id: number
  team_name: string
  priority: number
  reason: string
  reason_detail: string
  attractiveness_score: number
}

/** AI分析数据快照 */
export interface AnalysisDataSnapshot {
  player_name: string
  position: string
  age: number
  ability: number
  potential: number
  satisfaction: number
  loyalty: number
  is_starter: boolean
  current_salary: number  // 万/年
  contract_end_season: number | null
  team_name: string
  team_avg_ability: number
  loyalty_type: string       // 忠心耿耿/忠诚/中立/机会主义/雇佣兵
  departure_threshold: number // 离队阈值
}

/** AI分析步骤 */
export interface AnalysisStep {
  step_name: string      // 步骤名称，如"满意度分析"
  data_used: string      // 使用的数据，如"满意度: 35"
  threshold?: string     // 阈值说明，如"离队阈值: 50"
  result: string         // 结论，如"低于阈值，判断想离队"
  impact?: string        // 影响，如"基础离队概率 +70%"
}

/** 选手转会策略（完整） */
export interface PlayerTransferStrategy {
  id: number
  player_id: number
  save_id: string
  season_id: number
  wants_to_leave: boolean
  decision_confidence: number
  departure_reasons: string[]
  leave_reasoning: string
  preferred_teams: PreferredTeamInfo[]
  team_preference_reasoning: string
  expected_salary: number
  expected_min_salary: number
  expected_years: number
  requires_starter: boolean
  analysis_data: AnalysisDataSnapshot | null
  analysis_steps: AnalysisStep[]
  is_mock: boolean
  generated_at: string
}

/** 选手转会策略展示信息 */
export interface PlayerTransferStrategyInfo {
  player_id: number
  player_name: string
  position: string
  ability: number
  age: number
  team_id: number
  team_name: string
  wants_to_leave: boolean
  decision_confidence: number
  departure_reasons: string[]
  leave_reasoning: string
  preferred_teams_count: number
  preferred_teams: PreferredTeamInfo[]
  team_preference_reasoning: string
  expected_salary: number
  expected_min_salary: number
  expected_years: number
  requires_starter: boolean
  is_mock: boolean
  generated_at: string
}

// ========================================
// Export all APIs
// ========================================

export const tauriApi = {
  app: appApi,
  save: saveApi,
  team: teamApi,
  player: playerApi,
  tournament: tournamentApi,
  honor: honorApi,
  draft: draftApi,
  draftAuction: draftAuctionApi,
  transfer: transferApi,
  finance: financeApi,
  query: queryApi,
  international: internationalApi,
  match: matchApi,
  event: eventApi,
  stats: statsApi,
  test: testApi,
  time: timeApi,
  points: pointsApi,
  awards: awardsApi,
  matchDetails: matchDetailsApi,
  dev: devApi,
  aiTransfer: aiTransferApi,
}

// ========================================
// Transfer Window System API (新转会系统)
// ========================================

/** 转会期状态 */
export type TransferWindowStatus = 'PENDING' | 'IN_PROGRESS' | 'COMPLETED' | 'CANCELLED'

/** 转会事件类型 */
export type TransferEventType =
  | 'SEASON_SETTLEMENT'
  | 'CONTRACT_RENEWAL'
  | 'CONTRACT_TERMINATION'
  | 'FREE_AGENT_SIGNING'
  | 'TRANSFER_PURCHASE'
  | 'PLAYER_RETIREMENT'
  | 'PLAYER_LISTED'
  | 'EMERGENCY_SIGNING'
  | 'DRAFT_PICK_AUCTION'
  | 'FINANCIAL_ADJUSTMENT'

/** 事件等级 */
export type TransferEventLevel = 'S' | 'A' | 'B' | 'C'

/** AI球队性格类型 */
export type AITeamPersonality = 'AGGRESSIVE' | 'CONSERVATIVE' | 'BALANCED' | 'DEVELOPMENT' | 'WIN_NOW'

/** 转会期响应 */
export interface TransferWindowResponse {
  window_id: number
  current_round: number
  status: string
  season_id: number
}

/** 转会窗口关闭验证结果 */
export interface TransferWindowCloseValidation {
  is_valid: boolean
  window_id: number
  issues: TransferCloseIssue[]
  message: string
}

/** 转会窗口关闭问题 */
export interface TransferCloseIssue {
  team_id: number
  team_name: string
  issue_type: string
  detail: string
}

/** 转会事件 */
export interface TransferEvent {
  id: number
  window_id: number
  round: number
  event_type: string
  level: string
  player_id: number
  player_name: string
  player_ability: number
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number | null
  to_team_name: string | null
  transfer_fee: number
  salary: number
  contract_years: number
  reason: string | null
  created_at: string
}

/** 轮次结果 */
export interface RoundResult {
  round: number
  round_name: string
  events: TransferEvent[]
  summary: string
}

/** 轮次执行响应 */
export interface RoundExecutionResponse {
  round: number
  round_name: string
  events: TransferEvent[]
  event_count: number
  next_round: number | null
  summary: string
}

/** 快进响应 */
export interface FastForwardResponse {
  completed_rounds: number
  total_events: number
  rounds: RoundResult[]
}

/** 球队转会摘要 */
export interface TeamTransferSummary {
  team_id: number
  team_name: string
  players_in: number
  players_out: number
  money_spent: number
  money_earned: number
  net_spend: number
}

/** 转会报告 */
export interface TransferReport {
  window_id: number
  season_id: number
  total_events: number
  total_transfer_fee: number
  events_by_type: Record<string, number>
  events_by_level: Record<string, number>
  team_summaries: TeamTransferSummary[]
  top_events: TransferEvent[]
}

/** 球队性格配置 */
export interface TeamPersonalityConfig {
  id: number
  team_id: number
  save_id: string
  personality: string
  short_term_focus: number
  long_term_focus: number
  risk_tolerance: number
  youth_preference: number
  star_chasing: number
  bargain_hunting: number
  updated_at: string
}

/** 球队声望 */
export interface TeamReputation {
  team_id: number
  overall: number
  historical: number
  recent: number
  international: number
}

/** 更新性格请求 */
export interface UpdatePersonalityRequest {
  personality: string
  short_term_focus?: number
  long_term_focus?: number
  risk_tolerance?: number
  youth_preference?: number
  star_chasing?: number
  bargain_hunting?: number
}

// ========== 评估系统类型 ==========

/** 挂牌选手完整信息 */
export interface TransferMarketListingInfo {
  listing_id: number
  window_id: number
  listing_price: number | null
  min_accept_price: number | null
  listing_status: string
  listed_at: string
  sold_at: string | null
  actual_price: number | null
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  calculated_market_value: number
  listed_by_team_id: number
  listed_by_team_name: string
  listed_by_region_code: string | null
  sold_to_team_id: number | null
  sold_to_team_name: string | null
  sold_to_region_code: string | null
}

/** 自由球员信息 */
export interface FreeAgentMarketInfo {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  calculated_market_value: number
  salary: number
}

/** 转会挂牌市场综合数据 */
export interface TransferMarketData {
  listings: TransferMarketListingInfo[]
  free_agents: FreeAgentMarketInfo[]
  window_status: string | null
  window_id: number | null
  current_round: number | null
  season_id: number
}

// ========== 竞价分析类型 ==========

/** 单条竞价记录 */
export interface TransferBid {
  id: number
  window_id: number
  round: number
  player_id: number
  player_name: string
  player_ability: number
  player_age: number
  player_position: string | null
  from_team_id: number | null
  from_team_name: string | null
  bid_team_id: number
  bid_team_name: string
  bid_team_region_id: number | null
  offered_salary: number
  contract_years: number
  transfer_fee: number
  signing_bonus: number
  match_score: number
  willingness: number
  is_winner: boolean
  reject_reason: string | null
}

/** 单个选手的竞价分析 */
export interface PlayerBidAnalysis {
  player_id: number
  player_name: string
  player_ability: number
  player_age: number
  player_position: string | null
  from_team_id: number | null
  from_team_name: string | null
  round: number
  total_bids: number
  bids: TransferBid[]
  winner_team_name: string | null
  outcome: string
}

/** 竞价总览 */
export interface BidOverview {
  window_id: number
  round: number | null
  total_players: number
  total_bids: number
  successful_signings: number
  failed_signings: number
  avg_bids_per_player: number
  player_analyses: PlayerBidAnalysis[]
}

// ========== 评估系统类型 ==========

/** 战队赛季评估信息 */
export interface TeamSeasonEvaluationInfo {
  evaluation_id: number
  team_id: number
  team_name: string
  team_short_name: string
  region_code: string
  season_id: number
  current_rank: number
  last_rank: number
  stability_score: number
  strategy: string
  urgency_level: string
  roster_power: number
  roster_count: number
  avg_age: number
  avg_ability: number
  budget_remaining: number
  evaluation_reason: string
  created_at: string
}

/** 位置需求信息 */
export interface PositionNeedInfo {
  position: string
  current_starter_name: string | null
  current_starter_ability: number | null
  current_starter_age: number | null
  need_level: string
  min_ability_target: number | null
  reason: string | null
}

/** 选手挂牌评估信息 */
export interface PlayerListingEvaluationInfo {
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  team_id: number
  team_name: string
  should_list: boolean
  list_reason: string
  is_protected: boolean
  protect_reason: string
  estimated_value: number
}

/** 选手留队评估信息 */
export interface PlayerStayEvaluationInfo {
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  team_id: number
  team_name: string
  stay_score: number
  wants_to_leave: boolean
  leave_reason: string
  salary: number
  satisfaction: number
  loyalty: number
}

/** 转会系统 API */
export const transferWindowApi = {
  // 开始转会期
  startTransferWindow: () =>
    invokeCommand<TransferWindowResponse>('start_transfer_window'),

  // 执行单轮转会
  executeTransferRound: (windowId: number, round: number) =>
    invokeCommand<RoundExecutionResponse>('execute_transfer_round', { windowId, round }),

  // 快进转会期
  fastForwardTransfer: (windowId: number, fromRound?: number) =>
    invokeCommand<FastForwardResponse>('fast_forward_transfer', { windowId, fromRound }),

  // 获取转会事件
  getTransferEvents: (windowId: number, round?: number, level?: string) =>
    invokeCommand<TransferEvent[]>('get_transfer_events', { windowId, round, level }),

  // 获取转会报告
  getTransferReport: (windowId: number) =>
    invokeCommand<TransferReport>('get_transfer_report', { windowId }),

  // 获取转会期状态
  getTransferWindowStatus: (windowId: number) =>
    invokeCommand<TransferWindowResponse>('get_transfer_window_status', { windowId }),

  // 查询当前赛季的转会窗口（纯查询，不创建）
  getCurrentTransferWindow: () =>
    invokeCommand<TransferWindowResponse | null>('get_current_transfer_window'),

  // 查询指定赛季的转会窗口
  getTransferWindowBySeason: (seasonId: number) =>
    invokeCommand<TransferWindowResponse | null>('get_transfer_window_by_season', { seasonId }),

  // 获取球队AI性格
  getTeamPersonality: (teamId: number) =>
    invokeCommand<TeamPersonalityConfig | null>('get_team_personality', { teamId }),

  // 更新球队AI性格
  updateTeamPersonality: (teamId: number, request: UpdatePersonalityRequest) =>
    invokeCommand<boolean>('update_team_personality', { teamId, request }),

  // 获取球队声望
  getTeamReputation: (teamId: number) =>
    invokeCommand<TeamReputation>('get_team_reputation', { teamId }),

  // ========== 评估系统 ==========

  // 获取战队评估列表
  getTeamEvaluations: (seasonId?: number) =>
    invokeCommand<TeamSeasonEvaluationInfo[]>('get_team_evaluations', { seasonId }),

  // 获取战队位置需求
  getTeamPositionNeeds: (teamId: number, seasonId?: number) =>
    invokeCommand<PositionNeedInfo[]>('get_team_position_needs', { teamId, seasonId }),

  // 获取选手挂牌评估
  getPlayerListingEvaluations: (teamId?: number, seasonId?: number) =>
    invokeCommand<PlayerListingEvaluationInfo[]>('get_player_listing_evaluations', { teamId, seasonId }),

  // 获取选手留队评估
  getPlayerStayEvaluations: (teamId?: number, seasonId?: number) =>
    invokeCommand<PlayerStayEvaluationInfo[]>('get_player_stay_evaluations', { teamId, seasonId }),

  // 清除评估数据（用于重新生成）
  clearEvaluationData: (seasonId?: number) =>
    invokeCommand<number>('clear_evaluation_data', { seasonId }),

  // 获取转会挂牌市场数据
  getTransferMarketListings: () =>
    invokeCommand<TransferMarketData>('get_transfer_market_listings'),

  // ========== 转会窗口关闭 ==========

  // 确认关闭转会窗口（含验证）
  confirmCloseTransferWindow: (windowId: number, force?: boolean) =>
    invokeCommand<TransferWindowCloseValidation>('confirm_close_transfer_window', { windowId, force }),

  // ========== 竞价分析 ==========

  // 获取竞价总览
  getTransferBidsOverview: (windowId?: number, round?: number, seasonId?: number) =>
    invokeCommand<BidOverview>('get_transfer_bids_overview', { windowId, round, seasonId }),

  // 获取单个选手的竞价记录
  getPlayerBids: (windowId: number, playerId: number) =>
    invokeCommand<PlayerBidAnalysis>('get_player_bids', { windowId, playerId }),
}

export default tauriApi
