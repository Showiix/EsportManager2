// ICP四赛区洲际对抗赛相关类型定义

// 参赛队伍信息
export interface ICPTeam {
  id: string | number
  name: string
  region: 'LPL' | 'LCK' | 'LEC' | 'LCS'
  seed: number // 1-4号种子
  badges: number // 获得的徽章数量
}

// 比赛类型
export interface ICPMatch {
  id: string | number
  backendMatchId?: number // 后端数据库中的比赛ID，用于加载比赛详情
  teamAId?: string | number
  teamBId?: string | number
  teamAName?: string
  teamBName?: string
  teamARegion?: string
  teamBRegion?: string
  scoreA?: number
  scoreB?: number
  winnerId?: string | number | null
  status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled'
  bestOf: number // BO3或BO5
  stage: 'group' | 'semifinal' | 'final'
  groupName?: string // A/B/C/D组
  roundNumber?: number
  matchType?: string
  completedAt?: Date | string
}

// 小组积分榜类型（种子组）
export interface ICPGroupStanding {
  teamId: string | number
  teamName: string
  region: string
  seed: number
  position: number
  matchesPlayed: number
  wins: number
  losses: number
  points: number
  roundsWon: number
  roundsLost: number
  roundDifferential: number
  hasBadge: boolean // 是否获得徽章（前两名）
}

// 种子组类型（ABCD四组）
export interface ICPSeedGroup {
  groupName: 'A' | 'B' | 'C' | 'D' // A=一号种子组，B=二号种子组...
  seedNumber: number // 1/2/3/4
  teams?: { id: string, name: string, region: string }[] // 参赛队伍列表
  standings: ICPGroupStanding[]
  matches: ICPMatch[]
  isComplete: boolean
}

// 赛区统计
export interface ICPRegionStats {
  region: 'LPL' | 'LCK' | 'LEC' | 'LCS'
  regionName: string
  totalBadges: number // 获得的徽章总数
  teams: ICPTeam[]
  ranking?: number // 赛区排名
}

// 淘汰赛对阵（赛区对决）
export interface ICPRegionMatch {
  id: string
  regionA: string
  regionB: string
  regionAName: string
  regionBName: string
  matches: ICPMatch[] // 四场BO5对决（一号种子vs一号种子，以此类推）
  tiebreakerMatch?: ICPMatch // 加赛（2:2平局时使用）
  regionAWins: number
  regionBWins: number
  winnerId?: string | null
  status: 'scheduled' | 'in_progress' | 'completed' | 'tiebreaker' // 添加 tiebreaker 状态
  stage: 'semifinal' | 'final'
  _needsFillTeams?: boolean // 标记是否需要填充队伍信息
}

// ICP赛事整体数据
export interface ICPTournament {
  id: string | number
  seasonYear: number
  status: 'not_started' | 'group_stage' | 'region_battle' | 'tiebreaker' | 'completed'
  seedGroups: ICPSeedGroup[] // A/B/C/D四个种子组
  regionStats: ICPRegionStats[] // 四个赛区统计
  semifinal?: ICPRegionMatch // 半决赛（如果需要）
  final?: ICPRegionMatch // 决赛
  champion?: ICPRegionStats // 冠军赛区
  runnerUp?: ICPRegionStats // 亚军赛区
  thirdPlace?: ICPRegionStats // 季军赛区
  fourthPlace?: ICPRegionStats // 第四名赛区
}

// 积分规则
export interface ICPPointsRule {
  championRegionParticipant: number // 最强赛区参赛队伍积分
  championRegionNonParticipant: number // 最强赛区未参赛队伍积分
  secondRegionParticipant: number
  secondRegionNonParticipant: number
  thirdRegionParticipant: number
  thirdRegionNonParticipant: number
  fourthRegionParticipant: number
  fourthRegionNonParticipant: number
}
