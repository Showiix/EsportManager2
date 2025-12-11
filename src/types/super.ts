// Super洲际年度邀请赛相关类型定义

// 比赛类型
export interface SuperMatch {
  id: string | number
  teamAId?: string | number
  teamBId?: string | number
  teamAName?: string
  teamBName?: string
  teamARegion?: string
  teamBRegion?: string
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

// Fighter组小组积分榜类型
export interface SuperGroupStanding {
  teamId: string | number
  teamName: string
  regionName: string
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

// Fighter组小组类型
export interface SuperGroup {
  groupName: string
  standings: SuperGroupStanding[]
  matches: SuperMatch[]
}

// 挑战者组定位赛结构
export interface ChallengerStage {
  positioningMatches: SuperMatch[]  // 定位赛（5vs8, 6vs7）
  promotionMatches: SuperMatch[]     // 晋级赛（Fighter胜者 vs 定位赛败者）
}

// 第三阶段结构
export interface ChampionPrepStage {
  winnersMatch?: SuperMatch      // 胜者组对决
  losersMatch?: SuperMatch       // 败者组对决
  losersFinal?: SuperMatch       // 败者组决赛
}

// 第四阶段终极冠军赛结构
export interface FinalStage {
  round1: SuperMatch[]           // 首轮对阵
  round2: SuperMatch[]           // 次轮对阵
  thirdPlaceMatch?: SuperMatch   // 季军赛
  grandFinal?: SuperMatch        // 总决赛
}

// Super洲际赛整体数据结构
export interface SuperBracket {
  id: string
  seasonYear: number
  status: 'not_started' | 'fighter_stage' | 'challenger_stage' | 'champion_prep_stage' | 'final_stage' | 'completed'

  // 参赛队伍（按年度积分排名）
  qualifiedTeams: {
    legendGroup: QualifiedTeam[]    // 传奇组（1-4名）
    challengerGroup: QualifiedTeam[] // 挑战者组（5-8名）
    fighterGroup: QualifiedTeam[]    // Fighter组（9-16名）
  }

  // 第一阶段：Fighter组预选赛
  fighterGroups: SuperGroup[]

  // 第二阶段：挑战者组
  challengerStage: ChallengerStage | null

  // 第三阶段：冠军赛预备战
  championPrepStage: ChampionPrepStage | null

  // 第四阶段：终极冠军赛
  finalStage: FinalStage | null

  // 最终排名
  champion: QualifiedTeam | null
  runnerUp: QualifiedTeam | null
  thirdPlace: QualifiedTeam | null
  fourthPlace: QualifiedTeam | null
}

// 参赛队伍信息
export interface QualifiedTeam {
  teamId: string | number
  teamName: string
  regionName: string
  annualPoints: number
  globalRank: number
}

// 积分奖励常量
export const SUPER_POINTS = {
  CHAMPION: 35,
  RUNNER_UP: 30,
  THIRD_PLACE: 25,
  FOURTH_PLACE: 20,
  CHAMPION_PREP_LOSER: 8,       // 第三阶段淘汰
  CHALLENGER_PROMOTION_LOSER: 5, // 晋级赛淘汰
  FIGHTER_GROUP_ELIMINATED: 2    // Fighter组预选赛淘汰
} as const
