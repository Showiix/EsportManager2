import { invokeCommand } from './client'

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
  brand_value: number
  sponsorship: number
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
  season_id: number
  opening_balance: number
  closing_balance: number
  total_income: number
  total_expense: number
  financial_status: string
  salary_expense: number
  prize_money: number
  sponsorship: number
  league_share: number
  transfer_net: number
  operating_cost: number
  weak_team_subsidy: number
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
