// C洲际赛相关类型定义

// 比赛类型
export interface ClauchMatch {
  id: string | number
  backendMatchId?: number // 后端数据库中的比赛ID
  teamAId?: string | number
  teamBId?: string | number
  teamAName?: string
  teamBName?: string
  scoreA?: number
  scoreB?: number
  winnerId?: string | number | null
  status: 'scheduled' | 'in_progress' | 'completed'
  bestOf?: number
  stage?: string
  groupName?: string
  roundNumber?: number
  matchType?: string
  completedAt?: Date | string
}

// 小组积分榜类型
export interface ClauchGroupStanding {
  teamId: string | number
  teamName: string
  position: number
  matchesPlayed: number
  wins: number
  losses: number
  points: number
  roundsWon: number
  roundsLost: number
  roundDifferential: number
  qualified?: boolean
}

// 小组类型
export interface ClauchGroup {
  groupName: string
  standings: ClauchGroupStanding[]
  matches: ClauchMatch[]
}

// 淘汰赛对阵图类型
export interface ClauchKnockoutBracket {
  round1?: ClauchMatch[]
  quarterMatches?: ClauchMatch[]
  semiFinals?: ClauchMatch[]
  semiMatches?: ClauchMatch[]
  final?: ClauchMatch[]
  finalMatch?: ClauchMatch
}
