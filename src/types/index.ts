// 基础数据类型
export interface Team {
  id: string
  name: string
  shortName?: string  // 队伍简写名称
  regionId: string
  strength: number // 0-100
  createdAt?: string
  statistics?: TeamStatistics
}

export interface Region {
  id: string
  name: string
  teams?: Team[]
}

export interface Season {
  id: string | number     // 数据库原始ID (1, 2, 3...)
  seasonCode?: string     // "S1", "S2", "S3" (新增的赛季代码，可选以保持兼容性)
  name: string            // "S1赛季", "S2赛季"
  displayName?: string    // "2024赛季" (用户友好显示)
  year: number            // 2024, 2025 (对应年份)
  status: SeasonStatus    // planned | active | completed
  currentStage: string    // "spring" | "msi" | "summer" | "worlds"
  startDate?: string      // "2024-01-01"
  endDate?: string        // "2024-12-31"
  competitions?: Competition[]
}

export interface Competition {
  id: string | number           // 数据库原始ID (1, 2, 3...)
  competitionCode?: string      // "S1-spring", "S1-summer" (新增的赛事代码，可选)
  seasonId: string | number     // 数据库season_id
  regionId?: string             // "LPL" | "LCK" | "LEC" | "LCS" | "GLOBAL"
  type: CompetitionType         // spring | summer | msi | worlds | intercontinental
  stage?: string                // "regular" | "playoff" | "main"
  name: string                  // "S1 春季赛"
  displayName?: string          // "2024 春季赛" (用户显示)
  format: CompetitionFormat     // regular_season | playoffs | double_elimination
  status: CompetitionStatus     // planned | ongoing | completed
  teams: Team[]
  matches?: Match[]
  stages?: CompetitionStage[]
  startDate?: string
  endDate?: string
}

export interface Match {
  id: string | number           // 数据库原始ID (6271, 6272...)
  matchCode?: string            // "S1-spring-M6271" (新增的比赛代码，可选)
  competitionId: string | number // 数据库competition_id
  competitionCode?: string      // "S1-spring" (新增)
  seasonCode?: string           // "S1" (新增，便于查询)
  homeTeamId?: string           // 保留兼容性
  awayTeamId?: string           // 保留兼容性
  teamAId?: string | number     // 后端使用的字段
  teamBId?: string | number     // 后端使用的字段
  round?: number | string       // 兼容性：number用于常规赛，string用于worlds/MSI淘汰赛轮次
  roundNumber?: number          // 后端使用的字段
  matchNumber?: number          // 比赛编号
  stage?: string
  phase?: string                // 后端使用的字段
  status: MatchStatus
  regionId?: string
  result?: MatchResult
  scheduledAt?: string
  startedAt?: string
  playedAt?: string
  completedAt?: string
  scoreA?: number
  scoreB?: number
  winnerId?: string | number
  format?: string
  teamAName?: string
  teamBName?: string
}

export interface MatchResult {
  homeScore: number
  awayScore: number
  homePoints: number
  awayPoints: number
  winner?: string
}

// 统计数据类型
export interface TeamStatistics {
  totalMatches: number
  wins: number
  losses: number
  winRate: number
  totalPoints: number
  seasonPoints: number
  intercontinentalPoints: number
}

export interface CompetitionStage {
  name: string
  format: string
  teams: Team[]
  brackets?: BracketNode[]
}

export interface BracketNode {
  id: string
  round: number
  position: number
  team?: Team
  match?: Match
  nextNode?: string
}

export type CompetitionType = 'spring' | 'msi' | 'summer' | 'worlds' | 'intercontinental'

export type CompetitionFormat = 'regular_season' | 'playoffs' | 'double_elimination' | 'swiss' | 'single_elimination'

export type CompetitionStatus = 'planned' | 'ongoing' | 'completed'

export type MatchStatus = 'scheduled' | 'ongoing' | 'completed' | 'in_progress'

export type SeasonStatus = 'planned' | 'active' | 'completed'

// API 相关类型
export interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
  errors?: string[]
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  limit: number
  totalPages: number
}

export interface QueryOptions {
  page?: number
  limit?: number
  search?: string
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
  filters?: Record<string, any>
}

// 表单数据类型
export interface CreateTeamForm {
  name: string
  regionId: string
  strength: number
}

export interface UpdateTeamForm extends Partial<CreateTeamForm> {}

export interface CreateCompetitionForm {
  name: string
  type: CompetitionType
  format: CompetitionFormat
  seasonId: string | number     // Support both string ("S1", "S2") and number (1, 2)
  teamIds: string[]
  regionId?: string             // "LPL" | "LCK" | "LEC" | "LCS"
  stage?: string                // "regular" | "playoff" | "main"
}

// 图表数据类型
export interface ChartData {
  labels: string[]
  datasets: ChartDataset[]
}

export interface ChartDataset {
  label: string
  data: number[]
  backgroundColor?: string | string[]
  borderColor?: string | string[]
}

// 抽签相关类型
export interface DrawGroup {
  name: string
  teams: Team[]
  maxSize: number
}

export interface DrawResult {
  groups: DrawGroup[]
  matchups: Matchup[]
  timestamp: string
}

export interface Matchup {
  team1: Team
  team2: Team
  round: number
  stage: string
}

// 积分排名类型
export interface RegionalStandingItem {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  matchesPlayed: number
  wins: number
  losses: number
  winRate: number
  regularSeasonPoints: number
  roundDifferential: number
  position: number
  lastUpdated: string
}

export interface RegionalStandingsResponse {
  regionId: string
  regionName: string
  seasonId: string              // "S1", "S2"
  competitionType: 'spring' | 'summer'
  standings: RegionalStandingItem[]
  lastUpdated: string
}

export interface AnnualRankingItem {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  totalPoints: number
  springPoints: number
  summerPoints: number
  playoffPoints: number
  msiPoints: number
  worldsPoints: number
  intercontinentalPoints: number
  achievements: string[]
  position: number
  seasonId: string              // "S1", "S2"
}

export interface AnnualRankingsResponse {
  seasonId: string              // "S1", "S2"
  seasonYear: number            // 2024, 2025 (仅用于显示)
  annualRankings: AnnualRankingItem[]
  lastUpdated: string
}

// 荣誉殿堂类型
export interface SeasonInfo {
  id: string
  name: string
  year: number
  status: string
}

export interface HonorTeam {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  points: number
  competitionId: string
  competitionName: string
  achievementDate: string
  winRate?: number
  specialRecord?: string
}

export interface RegionalHonor {
  regionId: string
  regionName: string
  competitionType: 'spring' | 'summer'
  champion: HonorTeam | null
  runnerUp: HonorTeam | null
  thirdPlace: HonorTeam | null
}

export interface GlobalHonor {
  competitionId: string
  competitionName: string
  competitionType: 'msi' | 'worlds' | 'intercontinental'
  champion: HonorTeam | null
  runnerUp: HonorTeam | null
  thirdPlace: HonorTeam | null
  fourthPlace: HonorTeam | null
  participants: HonorTeam[]
}

export interface SeasonHonorsResponse {
  seasonId: string              // "S1", "S2"
  seasonYear: number            // 2024, 2025 (仅用于显示)
  regionalHonors: {
    spring: RegionalHonor[]
    summer: RegionalHonor[]
  }
  globalHonors: {
    msi: GlobalHonor | null
    worlds: GlobalHonor | null
  }
  intercontinentalHonors: GlobalHonor | null
  annualRankings: {
    topThree: AnnualRankingItem[]
    regionalTop: AnnualRankingItem[][]
  }
  statistics: {
    totalCompetitions: number
    totalMatches: number
    dominantRegion: string
    breakoutTeam: string
  }
}

export interface CreateHonorRecordForm {
  seasonId: string              // "S1", "S2"
  competitionId: string         // "S1-LPL-spring-playoff"
  teamId: string
  position: number
  points: number
  achievementDate?: string
}

export interface UpdateRankingForm {
  regionId: string
  seasonId: string              // "S1", "S2"
  competitionType: 'spring' | 'summer'
}

// 季后赛相关类型
export interface PlayoffQualification {
  teamId: string
  teamName: string
  regionId: string
  seed: number // 1-4: 排名种子位
  regularSeasonRank: number
  regularSeasonPoints: number
  wins: number
  losses: number
}

export interface PlayoffMatch extends Match {
  matchType: 'winners_bracket' | 'losers_bracket' | 'grand_final'
  bestOf: number // BO5 = 5
  nextMatchId?: string // 下一场比赛ID(胜者去向)
  loserNextMatchId?: string // 败者去向
}

export interface PlayoffBracket {
  id: string
  competitionId: string         // "S1-LPL-spring-playoff"
  seasonId?: string             // "S1" (冗余字段，便于查询)
  regionId: string
  regionName: string
  competitionType: 'spring' | 'summer'
  status: 'not_started' | 'in_progress' | 'completed'
  qualifiedTeams: PlayoffQualification[]

  // 对阵信息
  rounds: PlayoffRound[]

  // 最终排名
  champion?: PlayoffQualification
  runnerUp?: PlayoffQualification
  thirdPlace?: PlayoffQualification
  fourthPlace?: PlayoffQualification

  // 积分分配
  pointsDistribution: {
    champion: number // 12
    runnerUp: number // 10
    thirdPlace: number // 8
    fourthPlace: number // 6
  }

  createdAt: string
  updatedAt: string
}

export interface PlayoffRound {
  roundNumber: number
  roundName: string // "胜者组第一轮" | "败者组第一轮" | "败者组决赛" | "总决赛"
  bracketType: 'winners' | 'losers' | 'grand_final'
  matches: PlayoffMatch[]
  startDate?: string
  endDate?: string
  status: 'pending' | 'in_progress' | 'completed'
}

// 季后赛生成请求
export interface GeneratePlayoffRequest {
  competitionId: string         // "S1-LPL-spring-regular"
  regionId: string
  seasonId: string              // "S1", "S2"
  competitionType: 'spring' | 'summer'
}

// 季后赛模拟请求
export interface SimulatePlayoffMatchRequest {
  matchId: string
  competitionId: string
}

// 季后赛模拟响应
export interface SimulatePlayoffMatchResponse {
  match: PlayoffMatch
  winner: PlayoffQualification
  loser: PlayoffQualification
  nextMatch?: PlayoffMatch
  isPlayoffComplete: boolean
  finalStandings?: {
    champion?: PlayoffQualification
    runnerUp?: PlayoffQualification
    thirdPlace?: PlayoffQualification
    fourthPlace?: PlayoffQualification
  }
}

// ========================================
// MSI季中赛相关类型
// ========================================

// MSI参赛队伍资格
export interface MSIQualification {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  seed: number // 1: 冠军(传奇组), 2: 亚军(挑战者组), 3: 季军(资格赛组)
  springPlayoffRank: number // 春季赛季后赛排名
  springPlayoffPoints: number // 春季赛季后赛积分
  group: 'legendary' | 'challenger' | 'qualifier' // 所属分组
}

// MSI比赛类型
export type MSIMatchType =
  | 'qualifier_knockout' // 资格赛组单淘汰
  | 'qualifier' // 资格赛组 (别名)
  | 'challenger_match' // 挑战者组对决
  | 'challenger' // 挑战者组 (别名)
  | 'losers_round_1' // 败者组第一轮
  | 'loser_r1' // 败者组第一轮 (别名)
  | 'losers_round_2' // 败者组第二轮
  | 'loser_r2' // 败者组第二轮 (别名)
  | 'winners_round_1' // 胜者组第一轮(传奇组对决)
  | 'winner_r1' // 胜者组第一轮 (别名)
  | 'losers_round_3' // 败者组第三轮
  | 'loser_r3' // 败者组第三轮 (别名)
  | 'losers_round_4' // 败者组第四轮(攀登者赛)
  | 'loser_r4' // 败者组第四轮 (别名)
  | 'winners_round_2' // 胜者组第二轮
  | 'winner_final' // 胜者组决赛 (别名)
  | 'losers_final' // 败者组决赛
  | 'loser_final' // 败者组决赛 (别名)
  | 'grand_final' // 总决赛

// MSI比赛
export interface MSIMatch extends Match {
  matchType: MSIMatchType
  bestOf: number // BO5 = 5
  bracketType: 'winners' | 'losers' | 'qualifier' | 'challenger' | 'grand_final'
  nextMatchId?: string // 下一场比赛ID(胜者去向)
  loserNextMatchId?: string // 败者去向
  matchNumber?: number // 比赛编号
}

// MSI轮次
export interface MSIRound {
  roundNumber: number
  roundName: string
  stage: 'qualifier' | 'main' // 预选赛阶段或正式阶段
  bracketType: 'winners' | 'losers' | 'qualifier' | 'challenger' | 'grand_final'
  matches: MSIMatch[]
  startDate?: string
  endDate?: string
  status: 'pending' | 'in_progress' | 'completed'
}

// MSI对阵图
export interface MSIBracket {
  id: string
  seasonId: string              // "S1", "S2" (使用S标识)
  seasonYear: number            // 2024, 2025 (仅用于显示)
  status: 'not_started' | 'in_progress' | 'completed'

  // 参赛队伍分组
  qualifiedTeams: MSIQualification[]
  legendaryGroup: MSIQualification[] // 4队: 各赛区春季赛冠军
  challengerGroup: MSIQualification[] // 4队: 各赛区春季赛亚军
  qualifierGroup: MSIQualification[] // 4队: 各赛区春季赛季军

  // 对阵信息
  rounds: MSIRound[]

  // 最终排名
  champion?: MSIQualification
  runnerUp?: MSIQualification
  thirdPlace?: MSIQualification
  fourthPlace?: MSIQualification

  // 其他排名(用于积分分配)
  loserRound2?: MSIQualification[] // 败者组第二轮淘汰(2队)
  loserRound1?: MSIQualification[] // 败者组第一轮淘汰(2队)

  // 积分分配规则
  pointsDistribution: {
    champion: number // 20分
    runnerUp: number // 16分
    thirdPlace: number // 12分
    fourthPlace: number // 8分
    loserRound2: number // 6分
    loserRound1: number // 4分
  }

  createdAt: string
  updatedAt: string
}

// MSI生成请求
export interface GenerateMSIRequest {
  seasonId: string              // "S1", "S2"
}

// MSI模拟请求
export interface SimulateMSIMatchRequest {
  matchId: string
  msiId: string
}

// MSI模拟响应
export interface SimulateMSIMatchResponse {
  match: MSIMatch
  winner: MSIQualification
  loser: MSIQualification
  nextMatch?: MSIMatch
  loserNextMatch?: MSIMatch
  isMSIComplete: boolean
  finalStandings?: {
    champion?: MSIQualification
    runnerUp?: MSIQualification
    thirdPlace?: MSIQualification
    fourthPlace?: MSIQualification
    loserRound2?: MSIQualification[]
    loserRound1?: MSIQualification[]
  }
}

// MSI资格检查响应
export interface MSIEligibilityResponse {
  eligible: boolean
  reason?: string
  qualifiedTeams?: MSIQualification[]
  legendaryGroup?: MSIQualification[]
  challengerGroup?: MSIQualification[]
  qualifierGroup?: MSIQualification[]
}

// ========================================
// 世界赛相关类型
// ========================================

// 世界赛参赛队伍资格
export interface WorldsQualification {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  seed: number // 1: 冠军(直通淘汰赛), 2: 亚军(小组赛), 3: 季军(小组赛)
  summerPlayoffRank: number // 夏季赛季后赛排名
  summerPlayoffPoints: number // 夏季赛季后赛积分
  directToKnockout: boolean // 是否直接晋级淘汰赛
  quarterSlot?: number // 1/4决赛半区位置 (1-4)，仅冠军队伍有值
}

// 世界赛阶段类型
export type WorldsStage =
  | 'not_started'
  | 'group_stage' // 小组赛（瑞士轮）
  | 'knockout_stage' // 淘汰赛
  | 'completed'

// 世界赛比赛类型
export type WorldsMatchType =
  | 'swiss_round' // 小组赛（瑞士轮BO1）
  | 'quarter_final' // 1/4决赛（BO5）
  | 'semi_final' // 半决赛（BO5）
  | 'third_place' // 季军赛（BO5）- 世界赛特有！
  | 'grand_final' // 总决赛（BO5）

// 小组赛瑞士轮战绩（2胜晋级、2败淘汰制）
export interface SwissStandings {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  wins: number // 胜场数 (0-2)
  losses: number // 败场数 (0-2)
  record: string // 战绩字符串 "X-Y" 格式，如 "2-0", "1-1", "0-2"
  matchesPlayed: number // 已比赛场次 (最多3场)
  status: 'active' | 'qualified' | 'eliminated' // active: 继续比赛, qualified: 2胜晋级, eliminated: 2败淘汰
  qualified: boolean // 是否晋级淘汰赛 (wins === 2)
  eliminated: boolean // 是否已被淘汰 (losses === 2)
  position: number // 小组赛最终排名
  currentRound: number // 当前所在轮次 (1-3)
}

// 世界赛比赛
export interface WorldsMatch extends Match {
  matchType: WorldsMatchType
  bestOf: number // 小组赛BO1=1, 淘汰赛BO5=5
  stage: 'group' | 'knockout' | 'third_place'
  quarterSlot?: number // 所属半区 (1-4)，仅淘汰赛有值
  nextMatchId?: string // 下一场比赛ID(胜者去向)
  loserNextMatchId?: string // 败者去向(仅半决赛有值，指向季军赛)
  matchNumber?: number // 比赛编号
  swissRound?: number // 瑞士轮轮次 (1-3)，仅小组赛有值
  roundNumber?: number // 轮次编号
  swissGroup?: string // 瑞士轮分组 "1-0", "0-1", "1-1" 等，表示对阵双方的战绩分组
}

// 瑞士轮比赛
export interface WorldsSwissMatch extends WorldsMatch {
  stage: 'group'
  roundNumber: number
  teamAId: string
  teamBId: string
  teamAName: string
  teamBName: string
  winnerId?: string
  scoreA?: number
  scoreB?: number
}

// 淘汰赛比赛
export interface WorldsKnockoutMatch extends WorldsMatch {
  stage: 'knockout' | 'third_place'
  round?: string // 轮次名称：QUARTER_FINAL, SEMI_FINAL, THIRD_PLACE, FINAL
  quarterSlot?: number
  teamAQuarterSlot?: number
  teamBQuarterSlot?: number
}

// 世界赛轮次
export interface WorldsRound {
  roundNumber: number
  roundName: string // "瑞士轮第1轮" | "1/4决赛" | "半决赛" | "季军赛" | "总决赛"
  stage: 'group' | 'knockout' | 'third_place'
  matches: WorldsMatch[]
  startDate?: string
  endDate?: string
  status: 'pending' | 'in_progress' | 'completed'
}

// 淘汰赛对阵结构（4个半区）
export interface KnockoutBracket {
  quarterSlot: number // 半区编号 (1-4)
  champion: WorldsQualification // 直通的冠军队伍
  groupQualifier?: WorldsQualification // 小组赛晋级队伍
  quarterFinalMatch?: WorldsMatch // 1/4决赛
  quarterWinner?: WorldsQualification // 1/4决赛胜者
}

// 世界赛主结构
export interface WorldsBracket {
  id: string
  seasonId: string // "S1", "S2"
  seasonYear: number // 2024, 2025
  status: WorldsStage
  currentSwissRound?: number // 当前瑞士轮轮次 (0-3)

  // 参赛队伍（后端字段名）
  playInTeams?: WorldsQualification[] // 入围赛队伍（8支）
  qualified_teams?: WorldsQualification[] // 兼容字段
  qualifiedTeams?: WorldsQualification[] // 前端使用
  directTeams?: WorldsQualification[] // 4队: 各赛区夏季赛冠军（直通淘汰赛）
  groupStageTeams?: WorldsQualification[] // 8队: 各赛区夏季赛亚军+季军（打小组赛）

  // 瑞士轮比赛和积分榜（后端字段名）
  swissMatches?: WorldsSwissMatch[] // 所有瑞士轮比赛
  swissStandings?: SwissStandings[]
  swiss_standings?: SwissStandings[] // 兼容字段

  // 小组赛数据（前端使用）
  groupStage?: {
    rounds?: WorldsRound[] // 瑞士轮轮次
    standings?: SwissStandings[] // 实时积分榜
    qualifiedTeams?: WorldsQualification[] // 晋级的4支队伍
    eliminatedTeams?: WorldsQualification[] // 淘汰的4支队伍
  }

  // 淘汰赛比赛（后端字段名）
  knockoutMatches?: WorldsKnockoutMatch[]
  knockout_matches?: WorldsKnockoutMatch[] // 兼容字段

  // 淘汰赛数据（前端使用）
  knockoutStage?: {
    brackets?: KnockoutBracket[] // 4个半区
    rounds?: WorldsRound[] // 1/4决赛、半决赛、季军赛、总决赛
    semiFinalMatches?: WorldsMatch[] // 半决赛2场
    thirdPlaceMatch?: WorldsMatch // 季军赛
    grandFinalMatch?: WorldsMatch // 总决赛
  }

  // 最终排名
  champion?: WorldsQualification
  runnerUp?: WorldsQualification
  thirdPlace?: WorldsQualification // 季军赛胜者
  fourthPlace?: WorldsQualification // 季军赛败者

  // 其他排名（用于积分分配）
  quarterFinalists?: WorldsQualification[] // 1/4决赛淘汰的4队
  groupStageEliminated?: WorldsQualification[] // 小组赛淘汰的4队

  // 积分分配规则
  pointsDistribution: {
    champion: number // 20分
    runnerUp: number // 16分
    thirdPlace: number // 12分
    fourthPlace: number // 8分
    quarterFinalist: number // 6分（1/4决赛淘汰）
    groupStageEliminated: number // 4分（小组赛淘汰）
  }

  createdAt?: string
  updatedAt?: string
}

// 世界赛生成请求
export interface GenerateWorldsRequest {
  seasonId?: string // 可选，后端会自动检测当前赛季
}

// 世界赛模拟请求
export interface SimulateWorldsMatchRequest {
  matchId: string
  matchType: 'swiss' | 'knockout'
}

// 世界赛模拟响应
export interface SimulateWorldsMatchResponse {
  match: WorldsMatch
  winner: WorldsQualification
  loser: WorldsQualification
  nextMatch?: WorldsMatch // 胜者下一场
  loserNextMatch?: WorldsMatch // 败者下一场（仅半决赛有，指向季军赛）
  isWorldsComplete: boolean
  isGroupStageComplete: boolean // 小组赛是否完成
  isKnockoutStageComplete: boolean // 淘汰赛是否完成
  updatedStandings?: SwissStandings[] // 更新后的小组赛积分榜
  finalStandings?: {
    champion?: WorldsQualification
    runnerUp?: WorldsQualification
    thirdPlace?: WorldsQualification
    fourthPlace?: WorldsQualification
    quarterFinalists?: WorldsQualification[]
    groupStageEliminated?: WorldsQualification[]
  }
}

// 世界赛资格检查响应
export interface WorldsEligibilityResponse {
  eligible: boolean
  reason?: string
  qualifiedTeams?: WorldsQualification[]
  directTeams?: WorldsQualification[] // 直通队伍（冠军）
  groupStageTeams?: WorldsQualification[] // 小组赛队伍（亚军+季军）
}

// ========================================
// Super洲际赛相关类型
// ========================================

// Super洲际赛参赛队伍资格（基于两年积分总和）
export interface SuperQualification {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  // 两年积分数据
  season1Year: number // S1年份，如2024
  season2Year: number // S2年份，如2025
  season1Points: number // S1全年积分
  season2Points: number // S2全年积分
  totalPoints: number // 两年积分总和
  rank: number // 根据两年积分总和排名 (1-16)
  // 分组信息
  group: 'legendary' | 'challenger' | 'fighter' // 传奇组(1-4) | 挑战者组(5-8) | Fighter组(9-16)
  fighterSubGroup?: 'A' | 'B' // Fighter组的A/B小组
}

// Super洲际赛阶段类型
export type SuperStage =
  | 'not_started'
  | 'fighter_group' // 第一阶段：Fighter组预选赛
  | 'challenger_stage' // 第二阶段：挑战者组定位赛与晋级赛
  | 'preparation_stage' // 第三阶段：冠军赛预备战
  | 'championship_stage' // 第四阶段：终极冠军赛
  | 'completed'

// Super洲际赛比赛类型
export type SuperMatchType =
  // 第一阶段：Fighter组预选赛 (BO1单循环)
  | 'fighter_group_a' // Fighter A组比赛
  | 'fighter_group_b' // Fighter B组比赛
  // 第二阶段：挑战者组
  | 'challenger_positioning' // 挑战者组定位赛 (BO5)
  | 'advancement_match' // 晋级赛 (BO5)
  // 第三阶段：冠军赛预备战
  | 'prep_winners' // 胜者组 (BO5)
  | 'prep_losers' // 败者组 (BO5)
  | 'prep_losers_final' // 败者组决赛 (BO5)
  // 第四阶段：终极冠军赛
  | 'championship_round1' // 首轮对阵 (BO5)
  | 'championship_round2' // 次轮对阵 (BO5)
  | 'third_place_match' // 季军加赛 (BO5)
  | 'grand_final' // 总决赛 (BO5)

// Super洲际赛比赛
export interface SuperMatch extends Match {
  matchType: SuperMatchType
  stage: SuperStage
  bestOf: number // BO1=1, BO5=5
  bracketType?: 'winners' | 'losers' | 'championship' | 'fighter'
  nextMatchId?: string // 胜者去向
  loserNextMatchId?: string // 败者去向
  matchNumber?: number // 比赛编号
  groupName?: string // 分组名称（如"Fighter A组"）
}

// Super洲际赛轮次
export interface SuperRound {
  roundNumber: number
  roundName: string // 如"Fighter组预选赛"、"挑战者组定位赛"
  stage: SuperStage
  matches: SuperMatch[]
  startDate?: string
  endDate?: string
  status: 'pending' | 'in_progress' | 'completed'
}

// Fighter组积分榜
export interface FighterStanding {
  teamId: string
  teamName: string
  regionName: string
  group: 'A' | 'B'
  matchesPlayed: number
  wins: number
  losses: number
  record: string // "2-1" 格式
  position: number // 组内排名
  qualified: boolean // 是否晋级（每组第1名）
}

// Super洲际赛主结构
export interface SuperBracket {
  id: string
  // 赛季周期信息
  season1Code: string // "S1"
  season2Code: string // "S2"
  season1Year: number // 2024
  season2Year: number // 2025
  superYear: number // Super赛举办年份（通常等于season2Year）
  status: SuperStage
  
  // 参赛队伍分组（16支）
  qualifiedTeams: SuperQualification[] // 所有16支队伍
  legendaryGroup: SuperQualification[] // 4队: 第1-4名（传奇组）
  challengerGroup: SuperQualification[] // 4队: 第5-8名（挑战者组）
  fighterGroup: SuperQualification[] // 8队: 第9-16名（Fighter组）
  fighterGroupA: SuperQualification[] // Fighter A组 (4队)
  fighterGroupB: SuperQualification[] // Fighter B组 (4队)
  
  // 各阶段数据
  rounds: SuperRound[]
  
  // Fighter组积分榜
  fighterStandings?: FighterStanding[]
  
  // 第二阶段晋级情况
  challengerWinners?: SuperQualification[] // 定位赛胜者(2队) -> 第三阶段胜者组
  challengerLosers?: SuperQualification[] // 定位赛败者(2队) -> 晋级赛
  fighterQualifiers?: SuperQualification[] // Fighter组晋级者(2队) -> 晋级赛
  advancementWinners?: SuperQualification[] // 晋级赛胜者(2队) -> 第三阶段败者组
  
  // 第三阶段晋级情况
  prepWinnersChampion?: SuperQualification // 胜者组冠军 -> 第四阶段
  prepLosersFinalWinner?: SuperQualification // 败者组决赛胜者 -> 第四阶段
  
  // 第四阶段参赛队伍（6支）
  championshipTeams?: SuperQualification[] // 传奇组4队 + 第三阶段晋级2队
  
  // 最终排名
  champion?: SuperQualification
  runnerUp?: SuperQualification
  thirdPlace?: SuperQualification // 季军加赛胜者
  fourthPlace?: SuperQualification // 季军加赛败者
  
  // 其他排名（仅用于显示）
  championshipRound2Eliminated?: SuperQualification[] // 次轮淘汰2队（未进季军赛）
  championshipRound1Eliminated?: SuperQualification[] // 首轮淘汰2队
  prepStageEliminated?: SuperQualification[] // 第三阶段淘汰2队
  advancementEliminated?: SuperQualification[] // 第二阶段晋级赛淘汰2队
  fighterEliminated?: SuperQualification[] // Fighter组未晋级6队
  
  createdAt: string
  updatedAt: string
}

// Super洲际赛生成请求
export interface GenerateSuperRequest {
  season1Code: string // "S1"
  season2Code: string // "S2"
}

// Super洲际赛模拟请求
export interface SimulateSuperMatchRequest {
  matchId: string
  superId: string
}

// Super洲际赛模拟响应
export interface SimulateSuperMatchResponse {
  match: SuperMatch
  winner: SuperQualification
  loser: SuperQualification
  nextMatch?: SuperMatch // 胜者下一场
  loserNextMatch?: SuperMatch // 败者下一场
  isSuperComplete: boolean
  isStageComplete: boolean // 当前阶段是否完成
  updatedStandings?: FighterStanding[] // 更新后的Fighter组积分榜
  finalStandings?: {
    champion?: SuperQualification
    runnerUp?: SuperQualification
    thirdPlace?: SuperQualification
    fourthPlace?: SuperQualification
    championshipRound2Eliminated?: SuperQualification[]
    championshipRound1Eliminated?: SuperQualification[]
    prepStageEliminated?: SuperQualification[]
    advancementEliminated?: SuperQualification[]
    fighterEliminated?: SuperQualification[]
  }
}

// Super洲际赛资格检查响应
export interface SuperEligibilityResponse {
  eligible: boolean
  reason?: string
  season1Code: string
  season2Code: string
  season1Completed: boolean
  season2Completed: boolean
  qualifiedTeams?: SuperQualification[]
  legendaryGroup?: SuperQualification[]
  challengerGroup?: SuperQualification[]
  fighterGroup?: SuperQualification[]
}

// ========================================
// C洲际赛（Clauch Intercontinental Cup）相关类型
// ========================================

// C洲际赛参赛队伍资格（基于夏季赛常规赛排名前8）
export interface ClauchQualification {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  summerRegularRank: number // 夏季赛常规赛排名 (1-8)
  summerRegularPoints: number // 夏季赛常规赛积分
  groupName: string // 所属小组 'A'-'H'
}

// C洲际赛阶段类型
export type ClauchStage =
  | 'not_started'
  | 'group_stage' // 小组赛阶段
  | 'knockout_stage' // 淘汰赛阶段
  | 'completed'

// C洲际赛比赛类型
export type ClauchMatchType =
  | 'group_stage' // 小组赛 (BO3)
  | 'east_quarter' // 东半区淘汰赛第一轮 (BO5)
  | 'east_semi' // 东半区淘汰赛半决赛 (BO5)
  | 'east_final' // 东半区淘汰赛决赛 (BO5)
  | 'west_quarter' // 西半区淘汰赛第一轮 (BO5)
  | 'west_semi' // 西半区淘汰赛半决赛 (BO5)
  | 'west_final' // 西半区淘汰赛决赛 (BO5)
  | 'third_place' // 季军赛 (BO5)
  | 'grand_final' // 总决赛 (BO5)

// C洲际赛小组积分榜
export interface ClauchGroupStanding {
  teamId: string
  teamName: string
  regionName: string
  groupName: string // 'A'-'H'
  matchesPlayed: number
  wins: number
  losses: number
  points: number // 小组赛积分（2:0=3分，2:1=2分，1:2=1分，0:2=0分）
  roundDifferential: number // 局差（胜局-败局）
  position: number // 组内排名
  qualified: boolean // 是否晋级（前2名）
}

// C洲际赛比赛
export interface ClauchMatch extends Match {
  matchType: ClauchMatchType
  stage: 'group' | 'knockout'
  bestOf: number // 小组赛BO3=3, 淘汰赛BO5=5
  groupName?: string // 所属小组 'A'-'H'（仅小组赛）
  bracket?: 'east' | 'west' // 所属半区（仅淘汰赛）
  nextMatchId?: string // 胜者去向
  loserNextMatchId?: string // 败者去向（仅半决赛有值，指向季军赛）
  matchNumber?: number // 比赛编号
  roundNumber?: number // 轮次编号
}

// C洲际赛小组
export interface ClauchGroup {
  groupName: string // 'A'-'H'
  teams: ClauchQualification[] // 4支队伍
  standings: ClauchGroupStanding[] // 积分榜
  matches: ClauchMatch[] // 小组内比赛
  qualified: ClauchQualification[] // 晋级队伍（前2名）
}

// C洲际赛淘汰赛半区
export interface ClauchKnockoutBracket {
  bracket: 'east' | 'west'
  bracketName?: string // '东半区' | '西半区'
  teams?: ClauchQualification[] // 8支队伍
  // 后端返回的字段名
  round1?: ClauchMatch[] // 第一轮 (4场) - 后端字段
  semiFinals?: ClauchMatch[] // 半决赛 (2场) - 后端字段
  final?: ClauchMatch[] // 决赛 (1场数组) - 后端字段
  // 前端兼容字段（保持向后兼容）
  quarterMatches?: ClauchMatch[] // 第一轮 (4场) - 前端字段
  semiMatches?: ClauchMatch[] // 半决赛 (2场) - 前端字段
  finalMatch?: ClauchMatch // 决赛 (1场对象) - 前端字段
  champion?: ClauchQualification // 半区冠军
  runnerUp?: ClauchQualification // 半区亚军
}

// C洲际赛轮次
export interface ClauchRound {
  roundNumber: number
  roundName: string // 如"小组赛"、"东半区第一轮"
  stage: ClauchStage
  matches: ClauchMatch[]
  startDate?: string
  endDate?: string
  status: 'pending' | 'in_progress' | 'completed'
}

// C洲际赛主结构
export interface ClauchBracket {
  id: string
  seasonId: string // "S1", "S2"
  seasonYear: number // 2024, 2025
  status: ClauchStage

  // 参赛队伍（32支）
  qualifiedTeams: ClauchQualification[]

  // 小组赛数据（8个小组，每组4队）
  groups: ClauchGroup[] // A-H共8个小组
  groupStandings: ClauchGroupStanding[] // 所有小组积分榜

  // 淘汰赛数据（东西半区，各8队）
  knockoutEast?: ClauchKnockoutBracket // 东半区
  knockoutWest?: ClauchKnockoutBracket // 西半区

  // 决赛数据
  thirdPlaceMatch?: ClauchMatch // 季军赛
  grandFinal?: ClauchMatch // 总决赛

  // 所有轮次
  rounds: ClauchRound[]

  // 最终排名
  champion?: ClauchQualification
  runnerUp?: ClauchQualification
  thirdPlace?: ClauchQualification // 季军赛胜者
  fourthPlace?: ClauchQualification // 季军赛败者

  // 其他排名（用于积分分配）
  eastFinalLosers?: ClauchQualification[] // 东半区决赛败者（2队，包括进季军赛的）
  westFinalLosers?: ClauchQualification[] // 西半区决赛败者（2队，包括进季军赛的）
  eastSemiLosers?: ClauchQualification[] // 东半区半决赛败者（2队）
  westSemiLosers?: ClauchQualification[] // 西半区半决赛败者（2队）
  eastQuarterLosers?: ClauchQualification[] // 东半区第一轮败者（4队）
  westQuarterLosers?: ClauchQualification[] // 西半区第一轮败者（4队）

  // 积分分配规则
  pointsDistribution: {
    champion: number // 20分
    runnerUp: number // 16分
    thirdPlace: number // 12分
    fourthPlace: number // 8分
    eastWestFinalLosers: number // 6分（东西半区决赛败者）
    eastWestSemiLosers: number // 4分（东西半区半决赛败者）
    eastWestQuarterLosers: number // 2分（东西半区第一轮败者）
  }

  createdAt: string
  updatedAt: string
}

// C洲际赛生成请求
export interface GenerateClauchRequest {
  seasonId: string // "S1", "S2"
}

// C洲际赛模拟请求
export interface SimulateClauchMatchRequest {
  matchId: string
  clauchId: string
}

// C洲际赛模拟响应
export interface SimulateClauchMatchResponse {
  match: ClauchMatch
  winner: ClauchQualification
  loser: ClauchQualification
  nextMatch?: ClauchMatch // 胜者下一场
  loserNextMatch?: ClauchMatch // 败者下一场（仅半决赛有，指向季军赛）
  isClauchComplete: boolean
  isGroupStageComplete: boolean // 小组赛是否完成
  isKnockoutStageComplete: boolean // 淘汰赛是否完成
  updatedGroupStandings?: ClauchGroupStanding[] // 更新后的小组积分榜
  finalStandings?: {
    champion?: ClauchQualification
    runnerUp?: ClauchQualification
    thirdPlace?: ClauchQualification
    fourthPlace?: ClauchQualification
    eastFinalLosers?: ClauchQualification[]
    westFinalLosers?: ClauchQualification[]
    eastSemiLosers?: ClauchQualification[]
    westSemiLosers?: ClauchQualification[]
    eastQuarterLosers?: ClauchQualification[]
    westQuarterLosers?: ClauchQualification[]
  }
}

// C洲际赛资格检查响应
export interface ClauchEligibilityResponse {
  eligible: boolean
  reason?: string
  qualifiedTeams?: ClauchQualification[]
  groupAssignments?: { [key: string]: ClauchQualification[] } // 各小组分配
}