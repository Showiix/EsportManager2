/**
 * Tauri IPC Client
 * Replaces axios HTTP calls with Tauri invoke commands
 */
import { invoke } from '@tauri-apps/api/core'

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
  try {
    const result = await invoke<CommandResult<T>>(command, args)
    if (result.success && result.data !== null) {
      return result.data
    }
    throw new Error(result.error || 'Unknown error')
  } catch (error) {
    console.error(`Command ${command} failed:`, error)
    throw error
  }
}

// Helper that returns the full CommandResult (for cases where we need to check success)
export async function invokeCommandRaw<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<CommandResult<T>> {
  try {
    return await invoke<CommandResult<T>>(command, args)
  } catch (error) {
    console.error(`Command ${command} failed:`, error)
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
  save_id: string
  current_season: number
  current_phase: string
  phase_display: string
}

export const saveApi = {
  initDatabase: () => invokeCommand<void>('init_database'),

  createSave: (name: string) =>
    invokeCommand<string>('create_save', { name }),

  getSaves: () =>
    invokeCommand<SaveInfo[]>('get_saves'),

  loadSave: (saveId: string) =>
    invokeCommand<SaveInfo>('load_save', { saveId }),

  deleteSave: (saveId: string) =>
    invokeCommand<void>('delete_save', { saveId }),

  getCurrentSaveId: () =>
    invokeCommand<string | null>('get_current_save_id'),

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
  team_id: number | null
  name: string
  position: string
  nationality: string
  age: number
  ability: number
  potential: number
  form: number
  salary: number
  contract_end_season: number
  status: string
  tag: string
}

export const playerApi = {
  getPlayer: (playerId: number) =>
    invokeCommand<Player>('get_player', { playerId }),
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
}

// ========================================
// Draft System
// ========================================

export interface DraftPlayer {
  id: number
  name: string
  position: string
  nationality: string
  age: number
  ability: number
  potential: number
  tag: string
}

export interface DraftOrder {
  pick_number: number
  team_id: number
  team_name: string
}

export interface DraftPick {
  pick_number: number
  team_id: number
  team_name: string
  player: DraftPlayer
}

export const draftApi = {
  generateDraftPool: (regionId: number, poolSize?: number) =>
    invokeCommand<DraftPlayer[]>('generate_draft_pool', {
      regionId,
      poolSize: poolSize ?? 30
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
}

// ========================================
// Finance System
// ========================================

export interface TeamFinanceSummary {
  team_id: number
  team_name: string
  balance: number
  total_salary: number
  salary_cap: number
  salary_cap_remaining: number
  recent_income: number
  recent_expense: number
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

export const financeApi = {
  getTeamFinanceSummary: (teamId: number) =>
    invokeCommand<TeamFinanceSummary>('get_team_finance_summary', { teamId }),

  getAllTeamsFinance: () =>
    invokeCommand<TeamFinanceSummary[]>('get_all_teams_finance'),

  getTeamTransactions: (teamId: number, limit?: number) =>
    invokeCommand<FinanceTransaction[]>('get_team_transactions', {
      teamId,
      limit: limit ?? 50
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
  champion_team_id: number | null
  champion_team_name: string | null
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
}

// ========================================
// Match Simulation
// ========================================

export interface DetailedMatchResult {
  match_id: number
  tournament_id: number
  home_team_id: number
  away_team_id: number
  home_team_name: string
  away_team_name: string
  home_score: number
  away_score: number
  winner_id: number
  mvp: MvpInfo
  games: GameDetailInfo[]
  player_stats: PlayerMatchStats[]
}

export interface MvpInfo {
  player_id: number
  player_name: string
  team_id: number
  position: string
  mvp_score: number
}

export interface GameDetailInfo {
  game_number: number
  winner_id: number
  duration_minutes: number
  home_performance: number
  away_performance: number
  home_player_performances: PlayerGamePerformance[]
  away_player_performances: PlayerGamePerformance[]
}

export interface PlayerGamePerformance {
  player_id: number
  player_name: string
  position: string
  kills: number
  deaths: number
  assists: number
  cs: number
  damage: number
  gold: number
  vision_score: number
  performance_rating: number
}

export interface PlayerMatchStats {
  player_id: number
  player_name: string
  team_id: number
  position: string
  games_played: number
  total_kills: number
  total_deaths: number
  total_assists: number
  avg_cs: number
  avg_damage: number
  avg_gold: number
  kda: number
  mvp_score: number
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
  team_id: number
  team_name: string
  age: number
  career_years: number
}

export interface ExpiringContract {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  current_salary: number
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

  getExpiringContracts: () =>
    invokeCommand<ExpiringContract[]>('get_expiring_contracts'),
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
  transfer: transferApi,
  finance: financeApi,
  query: queryApi,
  international: internationalApi,
  match: matchApi,
  event: eventApi,
  test: testApi,
}

export default tauriApi
