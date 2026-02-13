import { ref, computed, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, teamApi, financeApi } from '@/api/tauri'
import type { BracketInfo, GroupStandingInfo, DetailedGameResult, PlayerGameStats, DetailedMatchResult } from '@/api/tauri'
import type { PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import type { ICPTournament, ICPSeedGroup, ICPMatch, ICPRegionStats, ICPRegionMatch, ICPGroupStanding } from '@/types/icp'
import type { StandingItem } from '@/types/tournament'
import { createLogger } from '@/utils/logger'
import { useBatchSimulation } from '@/composables/useBatchSimulation'

const logger = createLogger('useICPTournament')

export function useICPTournament() {
  const router = useRouter()
  const route = useRoute()
  const gameStore = useGameStore()
  const timeStore = useTimeStore()
  const matchDetailStore = useMatchDetailStore()
  const playerStore = usePlayerStore()

  // --- State ---
  const tournamentId = ref<number | null>(null)
  const bracketData = ref<BracketInfo | null>(null)
  const groupStandings = ref<GroupStandingInfo[]>([])
  const loading = ref(false)
  const teamMap = ref<Map<number, { name: string; regionCode: string }>>(new Map())
  
  const generatingRegionBattle = ref(false)
  const activeSeedGroup = ref('A')
  
  // Match Detail Dialog State
  const showMatchDetailDialog = ref(false)
  const currentMatchDetail = ref<MatchDetail | null>(null)

  // Batch Simulation
  const { simulationProgress: groupSimProgress, isSimulating: simulatingGroupStage } = useBatchSimulation()
  const { simulationProgress: battleSimProgress, isSimulating: simulatingRegionBattle } = useBatchSimulation()

  // Main Tournament Data
  const icpTournament = reactive<ICPTournament>({
    id: '',
    seasonYear: 2024,
    status: 'not_started',
    seedGroups: [],
    regionStats: [],
    semifinal: undefined,
    final: undefined,
    champion: undefined,
    runnerUp: undefined,
    thirdPlace: undefined,
    fourthPlace: undefined
  })

  // --- Computed ---
  
  const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

  const ICP_PHASE = 'IcpIntercontinental'
  const phaseNotReached = computed(() => {
    const currentPhase = timeStore.currentPhase
    const phaseOrder = [
      'SpringRegular', 'SpringPlayoffs', 'Msi', 'MadridMasters',
      'SummerRegular', 'SummerPlayoffs', 'ClaudeIntercontinental',
      'WorldChampionship', 'ShanghaiMasters', 'IcpIntercontinental',
      'SuperIntercontinental', 'TransferWindow', 'Draft', 'SeasonEnd'
    ]
    const currentIndex = phaseOrder.indexOf(currentPhase)
    const targetIndex = phaseOrder.indexOf(ICP_PHASE)
    return currentIndex < targetIndex
  })

  const currentPhaseDisplay = computed(() => timeStore.phaseDisplayName)

  const isGroupStageComplete = computed(() => {
    return icpTournament.seedGroups.every(group => {
      return group.matches.every(match => match.status === 'completed')
    })
  })

  const sortedRegionStats = computed(() => {
    return [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
  })

  const icpStandings = computed<StandingItem[]>(() => [
    { rank: 1, label: '最强赛区', name: icpTournament.champion?.regionName || '', regionFlag: getRegionFlag(icpTournament.champion?.region || ''), points: '', pointsDetail: ['参赛队伍: +12分', '未参赛队伍: +6分'] },
    { rank: 2, label: '第二名', name: icpTournament.runnerUp?.regionName || '', regionFlag: getRegionFlag(icpTournament.runnerUp?.region || ''), points: '', pointsDetail: ['参赛队伍: +8分', '未参赛队伍: +4分'] },
    { rank: 3, label: '第三名', name: icpTournament.thirdPlace?.regionName || '', regionFlag: getRegionFlag(icpTournament.thirdPlace?.region || ''), points: '', pointsDetail: ['参赛队伍: +6分', '未参赛队伍: +3分'] },
    { rank: 4, label: '第四名', name: icpTournament.fourthPlace?.regionName || '', regionFlag: getRegionFlag(icpTournament.fourthPlace?.region || ''), points: '', pointsDetail: ['参赛队伍: +4分', '未参赛队伍: +2分'] },
  ])

  const seedTeamsGrouped = computed(() => {
    const grouped: Record<number, Array<{ teamId: string, teamName: string, region: string }>> = {
      1: [], 2: [], 3: [], 4: []
    }

    icpTournament.seedGroups.forEach(group => {
      const seedNumber = group.seedNumber || (group.groupName.charCodeAt(0) - 'A'.charCodeAt(0) + 1)
      if (group.standings && group.standings.length > 0) {
        group.standings.forEach(team => {
          if (!grouped[seedNumber]) grouped[seedNumber] = []
          if (!grouped[seedNumber].some(t => t.teamId === String(team.teamId))) {
            grouped[seedNumber].push({
              teamId: String(team.teamId),
              teamName: team.teamName,
              region: team.region || ''
            })
          }
        })
      }
    })
    return grouped
  })

  // --- Helper Methods ---

  const getStatusType = (status: string) => {
    const typeMap: Record<string, any> = {
      'not_started': 'info',
      'group_stage': 'warning',
      'region_battle': 'warning',
      'completed': 'success'
    }
    return typeMap[status] || 'info'
  }

  const getStatusText = (status: string) => {
    const textMap: Record<string, string> = {
      'not_started': '未开始',
      'group_stage': '种子组赛进行中',
      'region_battle': '赛区对决进行中',
      'completed': '已完成'
    }
    return textMap[status] || status
  }

  const getSeedGroupLabel = (groupName: string) => {
    const labelMap: Record<string, string> = {
      'A': '一号种子',
      'B': '二号种子',
      'C': '三号种子',
      'D': '四号种子'
    }
    return labelMap[groupName] || groupName
  }

  const getRegionFlag = (region: string) => {
    const flagMap: Record<string, string> = {
      'LPL': '🇨🇳', 'LCK': '🇰🇷', 'LEC': '🇪🇺', 'LCS': '🇺🇸'
    }
    return flagMap[region] || '🏳️'
  }

  const getRegionDisplayName = (regionCode: string): string => {
    const nameMap: Record<string, string> = {
      'LPL': 'LPL (中国)',
      'LCK': 'LCK (韩国)',
      'LEC': 'LEC (欧洲)',
      'LCS': 'LCS (北美)'
    }
    return nameMap[regionCode] || regionCode
  }

  // --- Core Logic Methods ---

  const goBack = () => {
    router.push('/tournaments')
  }

  const viewMatchDetails = async (match: ICPMatch) => {
    if (match.status === 'completed') {
      const matchIdForLookup = match.backendMatchId || match.id
      let detail = matchDetailStore.getMatchDetail(matchIdForLookup) || matchDetailStore.getMatchDetail(match.id)
      if (detail) {
        currentMatchDetail.value = detail
        showMatchDetailDialog.value = true
        return
      }
      if (match.backendMatchId) {
        detail = await matchDetailStore.loadMatchDetailFromDb(match.backendMatchId)
        if (detail) {
          currentMatchDetail.value = detail
          showMatchDetailDialog.value = true
          return
        }
      }
    }
    ElMessage.info('该比赛暂无详细数据')
  }

  const handleCloseMatchDetail = () => {
    showMatchDetailDialog.value = false
    currentMatchDetail.value = null
  }

  const convertToMatchDetail = (result: DetailedMatchResult, matchId: string): MatchDetail => {
    const homeTeamId = result.home_team_id.toString()
    const awayTeamId = result.away_team_id.toString()
    const homeTeamName = teamMap.value.get(result.home_team_id)?.name || `Team ${result.home_team_id}`
    const awayTeamName = teamMap.value.get(result.away_team_id)?.name || `Team ${result.away_team_id}`

    const games = result.games.map((game: DetailedGameResult, idx: number) => {
      const teamAPower = game.home_players.length > 0
        ? game.home_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.home_players.length
        : 0
      const teamBPower = game.away_players.length > 0
        ? game.away_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.away_players.length
        : 0
      const powerDifference = teamAPower - teamBPower
      const winnerId = game.winner_id.toString()
      const winnerName = winnerId === homeTeamId ? homeTeamName : awayTeamName
      const isUpset = (powerDifference > 0 && winnerId !== homeTeamId) || (powerDifference < 0 && winnerId !== awayTeamId)

      return {
        gameNumber: idx + 1,
        winnerId, winnerName,
        duration: game.duration_minutes,
        teamAId: homeTeamId, teamAName: homeTeamName,
        teamBId: awayTeamId, teamBName: awayTeamName,
        teamAPower, teamBPower, powerDifference, isUpset,
        teamAPlayers: game.home_players.map((p: PlayerGameStats) => ({
          playerId: p.player_id.toString(), playerName: p.player_name, teamId: homeTeamId,
          position: p.position as PlayerPosition, baseAbility: p.base_ability, actualAbility: p.actual_ability,
          impactScore: p.impact_score, conditionBonus: p.condition_bonus, stability: 0, stabilityNoise: p.stability_noise,
          kills: p.kills, deaths: p.deaths, assists: p.assists, gold: p.gold, damageDealt: p.damage_dealt,
          cs: p.cs, visionScore: p.vision_score, traits: p.traits as any[],
          activatedTraits: p.activated_traits?.map(t => ({
            type: t.trait_type as any, name: t.name, effect: t.effect, value: t.value, isPositive: t.is_positive
          }))
        })),
        teamBPlayers: game.away_players.map((p: PlayerGameStats) => ({
          playerId: p.player_id.toString(), playerName: p.player_name, teamId: awayTeamId,
          position: p.position as PlayerPosition, baseAbility: p.base_ability, actualAbility: p.actual_ability,
          impactScore: p.impact_score, conditionBonus: p.condition_bonus, stability: 0, stabilityNoise: p.stability_noise,
          kills: p.kills, deaths: p.deaths, assists: p.assists, gold: p.gold, damageDealt: p.damage_dealt,
          cs: p.cs, visionScore: p.vision_score, traits: p.traits as any[],
          activatedTraits: p.activated_traits?.map(t => ({
            type: t.trait_type as any, name: t.name, effect: t.effect, value: t.value, isPositive: t.is_positive
          }))
        })),
        teamAPerformance: game.home_performance, teamAMetaPower: game.home_performance,
        teamBPerformance: game.away_performance, teamBMetaPower: game.away_performance,
        performanceDifference: game.home_performance - game.away_performance,
        metaPowerDifference: game.home_performance - game.away_performance,
        gameNoise: 0,
        mvpPlayerId: game.game_mvp?.player_id?.toString(), mvpPlayerName: game.game_mvp?.player_name, mvpTeamId: game.game_mvp?.team_id?.toString()
      }
    })

    return {
      matchId, teamAId: homeTeamId, teamAName: homeTeamName, teamBId: awayTeamId, teamBName: awayTeamName,
      bestOf: 3, finalScoreA: result.home_score, finalScoreB: result.away_score, winnerId: result.winner_id.toString(),
      games, tournamentType: 'icp', seasonId: String(icpTournament.seasonYear)
    }
  }

  const findBackendMatchId = (match: ICPMatch, stagePrefix?: string): number | null => {
    if (!bracketData.value) return null
    let allMatches = bracketData.value.matches || []
    if (stagePrefix) {
      allMatches = allMatches.filter(m => m.stage?.startsWith(stagePrefix))
    }

    // Method 1: Match by name
    for (const m of allMatches) {
      const homeTeamName = m.home_team?.short_name || m.home_team?.name || ''
      const awayTeamName = m.away_team?.short_name || m.away_team?.name || ''
      if ((homeTeamName === match.teamAName && awayTeamName === match.teamBName) ||
          (homeTeamName === match.teamBName && awayTeamName === match.teamAName)) {
        return m.match_id
      }
    }

    // Method 2: Match by ID
    if (match.teamAId && match.teamBId) {
      for (const m of allMatches) {
        const homeTeamId = m.home_team?.id
        const awayTeamId = m.away_team?.id
        if ((homeTeamId === Number(match.teamAId) && awayTeamId === Number(match.teamBId)) ||
            (homeTeamId === Number(match.teamBId) && awayTeamId === Number(match.teamAId))) {
          return m.match_id
        }
      }
    }
    return null
  }

  const updateGroupStandings = (match: ICPMatch) => {
    const group = icpTournament.seedGroups.find(g => g.groupName === match.groupName)
    if (!group) return
    const teamA = group.standings.find(s => s.teamId === match.teamAId)
    const teamB = group.standings.find(s => s.teamId === match.teamBId)

    if (teamA && teamB && match.scoreA !== undefined && match.scoreB !== undefined) {
      teamA.matchesPlayed++; teamB.matchesPlayed++
      teamA.roundsWon += match.scoreA; teamA.roundsLost += match.scoreB
      teamB.roundsWon += match.scoreB; teamB.roundsLost += match.scoreA
      teamA.roundDifferential = teamA.roundsWon - teamA.roundsLost
      teamB.roundDifferential = teamB.roundsWon - teamB.roundsLost

      if (match.scoreA > match.scoreB) {
        teamA.wins++; teamB.losses++
        teamA.points += match.scoreA === 2 && match.scoreB === 0 ? 3 : 2
        teamB.points += match.scoreB === 1 ? 1 : 0
      } else {
        teamB.wins++; teamA.losses++
        teamB.points += match.scoreB === 2 && match.scoreA === 0 ? 3 : 2
        teamA.points += match.scoreA === 1 ? 1 : 0
      }

      group.standings.sort((a, b) => {
        if (b.points !== a.points) return b.points - a.points
        if (b.roundDifferential !== a.roundDifferential) return b.roundDifferential - a.roundDifferential
        if (b.wins !== a.wins) return b.wins - a.wins
        return parseInt(String(a.teamId)) - parseInt(String(b.teamId))
      })

      group.standings.forEach((s, i) => {
        s.position = i + 1
        s.hasBadge = i < 2
      })
    }
  }

  const checkGroupCompletion = () => {
    icpTournament.seedGroups.forEach(group => {
      const isComplete = group.matches.every(m => m.status === 'completed')
      group.isComplete = isComplete
      if (isComplete) {
        group.standings.forEach(standing => {
          if (standing.hasBadge) {
            const region = icpTournament.regionStats.find(r => r.region === standing.region)
            if (region) {
              const team = region.teams.find(t => t.id === standing.teamId)
              if (team && team.badges === 0) {
                team.badges = 1
                region.totalBadges++
              }
            }
          }
        })
      }
    })
  }

  const handleSimulateMatch = async (match: ICPMatch) => {
    if (tournamentId.value) {
      try {
        const backendMatchId = match.backendMatchId || findBackendMatchId(match)
        if (backendMatchId) {
          const result = await matchApi.simulateMatchDetailed(backendMatchId)
          if (result) {
            const matchDetail = convertToMatchDetail(result, String(backendMatchId))
            match.backendMatchId = backendMatchId
            match.scoreA = result.home_score
            match.scoreB = result.away_score
            match.winnerId = result.winner_id.toString()
            match.status = 'completed'
            match.completedAt = new Date()

            matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
            // Record performances...
            matchDetail.games.forEach(game => {
              game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
              game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
            })

            await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
            updateGroupStandings(match)
            ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)
            checkGroupCompletion()
            return
          }
        }
      } catch (error) {
        logger.warn('后端 API 模拟失败，使用本地引擎:', error)
      }
    }
    ElMessage.error('无法找到后端比赛ID，请确保赛事数据已正确初始化')
  }

  const cancelUnusedMatches = async (stagePrefix: string) => {
    if (!bracketData.value) return
    const allMatches = bracketData.value.matches || []
    const matchesToCancel = allMatches.filter(m => m.stage?.startsWith(stagePrefix) && m.status?.toUpperCase() !== 'COMPLETED')
    
    for (const match of matchesToCancel) {
      try {
        await matchApi.cancelMatch(match.match_id)
      } catch (error) {
        logger.warn(`[ICP] 取消比赛失败: ${match.stage}`, error)
      }
    }
  }

  const fillKnockoutMatchTeams = async (stagePrefix: string, regionBattle: ICPRegionMatch) => {
    if (!bracketData.value) return
    const allMatches = bracketData.value.matches || []
    
    for (let i = 0; i < regionBattle.matches.length; i++) {
      const match = regionBattle.matches[i]
      const seed = i + 1
      const targetStage = `${stagePrefix}_${seed}`
      const backendMatch = allMatches.find(m => m.stage === targetStage)
      
      if (backendMatch && match.teamAId && match.teamBId) {
        match.backendMatchId = backendMatch.match_id
        try {
          await matchApi.updateMatchTeams(backendMatch.match_id, Number(match.teamAId), Number(match.teamBId))
        } catch (error) {
          logger.error(`[fillKnockoutMatchTeams] Failed to update ${targetStage}:`, error)
        }
      }
    }
    if (tournamentId.value) {
      const bracket = await internationalApi.getTournamentBracket(tournamentId.value)
      if (bracket) bracketData.value = bracket
    }
  }

  const createRegionBattle = (regionA: ICPRegionStats, regionB: ICPRegionStats, stage: 'semifinal' | 'final'): ICPRegionMatch => {
    const matches: ICPMatch[] = []
    for (let seed = 1; seed <= 4; seed++) {
      const teamA = regionA.teams.find(t => t.seed === seed)
      const teamB = regionB.teams.find(t => t.seed === seed)
      if (teamA && teamB) {
        matches.push({
          id: `${stage}-seed${seed}`, teamAId: teamA.id, teamAName: teamA.name, teamARegion: regionA.region,
          teamBId: teamB.id, teamBName: teamB.name, teamBRegion: regionB.region,
          scoreA: 0, scoreB: 0, winnerId: null, status: 'scheduled', bestOf: 5, stage: stage
        })
      }
    }
    return {
      id: `${stage}-${regionA.region}-vs-${regionB.region}`, regionA: regionA.region, regionB: regionB.region,
      regionAName: regionA.regionName, regionBName: regionB.regionName, matches,
      regionAWins: 0, regionBWins: 0, winnerId: null, status: 'scheduled', stage
    }
  }

  const handleGenerateRegionBattle = async () => {
    generatingRegionBattle.value = true
    try {
      await new Promise(resolve => setTimeout(resolve, 500))
      const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
      sortedRegions.forEach((region, index) => region.ranking = index + 1)
      const canSkipSemifinal = sortedRegions[1].totalBadges > sortedRegions[2].totalBadges

      if (canSkipSemifinal) {
        icpTournament.final = createRegionBattle(sortedRegions[0], sortedRegions[1], 'final')
        await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
        await cancelUnusedMatches('ICP_SEMI')
      } else {
        icpTournament.semifinal = createRegionBattle(sortedRegions[1], sortedRegions[2], 'semifinal')
        await fillKnockoutMatchTeams('ICP_SEMI', icpTournament.semifinal)
      }
      icpTournament.status = 'region_battle'
      ElMessage.success('赛区对决生成成功！')
    } finally {
      generatingRegionBattle.value = false
    }
  }

  const cancelUnusedMatches_battle = async (battle: ICPRegionMatch) => {
    if (!bracketData.value) return
    const stagePrefix = battle.stage === 'semifinal' ? 'ICP_SEMI' : 'ICP_FINAL'
    for (const match of battle.matches) {
      if (match.status !== 'completed') {
        const backendMatchId = match.backendMatchId || findBackendMatchId(match, stagePrefix)
        if (backendMatchId) {
          try {
            await matchApi.cancelMatch(backendMatchId)
            match.status = 'cancelled'
          } catch (error) {
            logger.warn(`[ICP] 取消种子对决失败: ${match.id}`, error)
          }
        }
      }
    }
  }

  const cancelUnusedTiebreaker = async (battle: ICPRegionMatch) => {
    if (!bracketData.value) return
    const tiebreakerStage = battle.stage === 'semifinal' ? 'ICP_SEMI_TIEBREAKER' : 'ICP_FINAL_TIEBREAKER'
    const tiebreakerMatch = bracketData.value.matches?.find(m => m.stage === tiebreakerStage)
    if (tiebreakerMatch && tiebreakerMatch.status !== 'Completed' && tiebreakerMatch.status !== 'COMPLETED') {
      try {
        await matchApi.cancelMatch(tiebreakerMatch.match_id)
      } catch (error) {
        logger.error('[ICP] 取消加赛失败:', error)
      }
    }
  }

  const setupTiebreakerMatch = async (battle: ICPRegionMatch) => {
    const seed1Match = battle.matches[0]
    if (!seed1Match) return
    const tiebreakerStage = battle.stage === 'semifinal' ? 'ICP_SEMI_TIEBREAKER' : 'ICP_FINAL_TIEBREAKER'

    battle.tiebreakerMatch = {
      id: `${battle.stage}-tiebreaker`,
      teamAId: seed1Match.teamAId, teamAName: seed1Match.teamAName, teamARegion: seed1Match.teamARegion,
      teamBId: seed1Match.teamBId, teamBName: seed1Match.teamBName, teamBRegion: seed1Match.teamBRegion,
      scoreA: 0, scoreB: 0, winnerId: null, status: 'scheduled', bestOf: 5, stage: battle.stage
    }

    if (bracketData.value) {
      const backendMatch = bracketData.value.matches?.find(m => m.stage === tiebreakerStage)
      if (backendMatch && seed1Match.teamAId && seed1Match.teamBId) {
        battle.tiebreakerMatch.backendMatchId = backendMatch.match_id
        try {
          await matchApi.updateMatchTeams(backendMatch.match_id, Number(seed1Match.teamAId), Number(seed1Match.teamBId))
        } catch (error) {
          logger.error('[setupTiebreakerMatch] 设置加赛队伍失败:', error)
        }
      }
    }
    battle.status = 'tiebreaker'
    ElMessage.warning('比分 2:2 平局！一号种子需要进行加赛决出胜者。')
  }

  const showChampionCelebration = async (championName: string) => {
    if (tournamentId.value) {
      try {
        await financeApi.distributeTournamentPrizes(tournamentId.value)
      } catch (e) {
        logger.error('发放奖金失败:', e)
      }
    }
    ElMessageBox.alert(
      `恭喜 ${championName} 成为ICP洲际对抗赛最强赛区！\n\n` +
      `✅ 奖金已发放到各战队账户\n` +
      `💡 请在时间控制面板完成阶段推进，系统将自动颁发荣誉和年度积分`,
      '🏆 最强赛区诞生! 🏆',
      { confirmButtonText: '太棒了!', customClass: 'champion-celebration-box', showClose: false, center: true }
    )
  }

  const checkTournamentCompletion = async () => {
    if (icpTournament.semifinal?.status === 'completed' && !icpTournament.final) {
      const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
      const semifinalWinner = icpTournament.regionStats.find(r => r.region === icpTournament.semifinal?.winnerId)
      if (semifinalWinner) {
        icpTournament.final = createRegionBattle(sortedRegions[0], semifinalWinner, 'final')
        await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
        ElMessage.success('半决赛完成！决赛已生成')
      }
      return
    }

    if (icpTournament.final?.status === 'completed') {
      const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
      const finalWinner = icpTournament.final.winnerId
      const finalLoser = finalWinner === icpTournament.final.regionA ? icpTournament.final.regionB : icpTournament.final.regionA
      icpTournament.champion = icpTournament.regionStats.find(r => r.region === finalWinner)
      icpTournament.runnerUp = icpTournament.regionStats.find(r => r.region === finalLoser)
      const remainingRegions = sortedRegions.filter(r => r.region !== finalWinner && r.region !== finalLoser)
      icpTournament.thirdPlace = remainingRegions[0]
      icpTournament.fourthPlace = remainingRegions[1]
      icpTournament.status = 'completed'
      showChampionCelebration(icpTournament.champion?.regionName || '')
    }
  }

  const checkRegionBattleCompletion = async (battle: ICPRegionMatch) => {
    if (battle.regionAWins >= 3) {
      battle.winnerId = battle.regionA; battle.status = 'completed'
      await cancelUnusedMatches_battle(battle); await cancelUnusedTiebreaker(battle); await checkTournamentCompletion(); return
    }
    if (battle.regionBWins >= 3) {
      battle.winnerId = battle.regionB; battle.status = 'completed'
      await cancelUnusedMatches_battle(battle); await cancelUnusedTiebreaker(battle); await checkTournamentCompletion(); return
    }
    const allComplete = battle.matches.every(m => m.status === 'completed')
    if (allComplete) {
      if (battle.regionAWins > battle.regionBWins) {
        battle.winnerId = battle.regionA; battle.status = 'completed'
        await cancelUnusedTiebreaker(battle); await checkTournamentCompletion()
      } else if (battle.regionBWins > battle.regionAWins) {
        battle.winnerId = battle.regionB; battle.status = 'completed'
        await cancelUnusedTiebreaker(battle); await checkTournamentCompletion()
      } else {
        await setupTiebreakerMatch(battle)
      }
    }
  }

  const handleSimulateTiebreaker = async (battle: ICPRegionMatch) => {
    if (!battle.tiebreakerMatch) return
    const match = battle.tiebreakerMatch
    const backendMatchId = match.backendMatchId
    if (!backendMatchId) { ElMessage.error('无法找到加赛后端比赛ID'); return }
    try {
      const result = await matchApi.simulateMatchDetailed(backendMatchId)
      if (result) {
        const matchDetail = convertToMatchDetail(result, String(backendMatchId))
        match.backendMatchId = backendMatchId
        match.scoreA = result.home_score; match.scoreB = result.away_score; match.winnerId = result.winner_id.toString()
        match.status = 'completed'; match.completedAt = new Date()
        matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
        matchDetail.games.forEach(game => {
          game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
          game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
        })
        if (tournamentId.value) await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
        if (match.teamARegion === battle.regionA) {
          battle.winnerId = result.home_score > result.away_score ? battle.regionA : battle.regionB
        } else {
          battle.winnerId = result.home_score > result.away_score ? battle.regionB : battle.regionA
        }
        battle.status = 'completed'
        ElMessage.success(`加赛完成！${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)
        await checkTournamentCompletion()
      }
    } catch (error) {
      logger.error('模拟加赛失败:', error); ElMessage.error('模拟加赛失败')
    }
  }

  const handleSimulateRegionMatch = async (battle: ICPRegionMatch, match: ICPMatch) => {
    if (tournamentId.value) {
      try {
        const stagePrefix = battle.stage === 'semifinal' ? 'ICP_SEMI' : 'ICP_FINAL'
        const backendMatchId = match.backendMatchId || findBackendMatchId(match, stagePrefix)
        if (backendMatchId) {
          const result = await matchApi.simulateMatchDetailed(backendMatchId)
          if (result) {
            const matchDetail = convertToMatchDetail(result, String(backendMatchId))
            match.backendMatchId = backendMatchId
            match.scoreA = result.home_score; match.scoreB = result.away_score; match.winnerId = result.winner_id.toString()
            match.status = 'completed'; match.completedAt = new Date()
            matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
            matchDetail.games.forEach(game => {
              game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
              game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
            })
            await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
            if (match.teamARegion === battle.regionA) {
              if (result.home_score > result.away_score) battle.regionAWins++
              else battle.regionBWins++
            } else {
              if (result.home_score > result.away_score) battle.regionBWins++
              else battle.regionAWins++
            }
            ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)
            checkRegionBattleCompletion(battle)
            return
          }
        }
      } catch (error) {
        logger.warn('后端 API 模拟失败:', error)
      }
    }
    ElMessage.error('无法找到后端比赛ID，请确保赛事数据已正确初始化')
  }

  const simulateMatchInternal = async (match: ICPMatch) => {
    const backendMatchId = match.backendMatchId || findBackendMatchId(match)
    if (!backendMatchId) return
    try {
      const result = await matchApi.simulateMatchDetailed(backendMatchId)
      if (result) {
        const matchDetail = convertToMatchDetail(result, String(backendMatchId))
        match.backendMatchId = backendMatchId
        match.scoreA = result.home_score; match.scoreB = result.away_score; match.winnerId = result.winner_id.toString()
        match.status = 'completed'; match.completedAt = new Date()
        matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
        matchDetail.games.forEach(game => {
          game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
          game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
        })
        if (tournamentId.value) await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
        updateGroupStandings(match)
        checkGroupCompletion()
      }
    } catch (error) {
      logger.error('模拟比赛失败:', error)
    }
  }

  const batchSimulateGroupStage = async () => {
    try {
      await ElMessageBox.confirm('将自动模拟所有未完成的种子组比赛。是否继续?', '模拟种子组赛确认', { confirmButtonText: '开始模拟', cancelButtonText: '取消', type: 'info' })
      simulatingGroupStage.value = true; groupSimProgress.value = 0
      const allMatches = icpTournament.seedGroups.flatMap(g => g.matches)
      const uncompletedMatches = allMatches.filter(m => m.status !== 'completed')
      for (let i = 0; i < uncompletedMatches.length; i++) {
        await simulateMatchInternal(uncompletedMatches[i])
        groupSimProgress.value = Math.floor(((i + 1) / uncompletedMatches.length) * 100)
        await new Promise(resolve => setTimeout(resolve, 80))
      }
      ElMessage.success('种子组赛模拟完成！现在可以进入赛区对决。')
    } catch (error: any) {
      if (error !== 'cancel') ElMessage.error(error.message || '种子组赛模拟失败')
    } finally {
      simulatingGroupStage.value = false; groupSimProgress.value = 0
    }
  }

  const simulateTiebreakerInternal = async (battle: ICPRegionMatch) => {
    if (!battle.tiebreakerMatch) return
    const match = battle.tiebreakerMatch
    const backendMatchId = match.backendMatchId
    if (!backendMatchId) return
    try {
      const result = await matchApi.simulateMatchDetailed(backendMatchId)
      if (result) {
        const matchDetail = convertToMatchDetail(result, String(backendMatchId))
        match.scoreA = result.home_score; match.scoreB = result.away_score; match.winnerId = result.winner_id.toString()
        match.status = 'completed'; match.completedAt = new Date()
        matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
        matchDetail.games.forEach(game => {
          game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
          game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
        })
        if (tournamentId.value) await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
        if (match.teamARegion === battle.regionA) {
          battle.winnerId = result.home_score > result.away_score ? battle.regionA : battle.regionB
        } else {
          battle.winnerId = result.home_score > result.away_score ? battle.regionB : battle.regionA
        }
        battle.status = 'completed'
        await checkTournamentCompletion()
      }
    } catch (error) {
      logger.error('模拟加赛失败:', error)
    }
  }

  const simulateRegionBattleInternal = async (battle: ICPRegionMatch) => {
    const stagePrefix = battle.stage === 'semifinal' ? 'ICP_SEMI' : 'ICP_FINAL'
    for (const match of battle.matches) {
      if (battle.regionAWins >= 3 || battle.regionBWins >= 3) break
      if (match.status !== 'completed') {
        const backendMatchId = match.backendMatchId || findBackendMatchId(match, stagePrefix)
        if (!backendMatchId) continue
        try {
          const result = await matchApi.simulateMatchDetailed(backendMatchId)
          if (result) {
            const matchDetail = convertToMatchDetail(result, String(backendMatchId))
            match.backendMatchId = backendMatchId
            match.scoreA = result.home_score; match.scoreB = result.away_score; match.winnerId = result.winner_id.toString()
            match.status = 'completed'; match.completedAt = new Date()
            matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)
            matchDetail.games.forEach(game => {
              game.teamAPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamAId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
              game.teamBPlayers.forEach(perf => playerStore.recordPerformance(perf.playerId, perf.playerName, String(match.teamBId), perf.position, perf.impactScore, perf.actualAbility, String(icpTournament.seasonYear), 'INTL'))
            })
            if (tournamentId.value) await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
            if (match.teamARegion === battle.regionA) {
              if (result.home_score > result.away_score) battle.regionAWins++
              else battle.regionBWins++
            } else {
              if (result.home_score > result.away_score) battle.regionBWins++
              else battle.regionAWins++
            }
          }
        } catch (error) {
          logger.error('模拟比赛失败:', error)
        }
        await new Promise(resolve => setTimeout(resolve, 100))
      }
    }
    if (battle.regionAWins >= 3 || battle.regionAWins > battle.regionBWins) {
      battle.winnerId = battle.regionA; battle.status = 'completed'
      await cancelUnusedMatches_battle(battle); await cancelUnusedTiebreaker(battle); await checkTournamentCompletion()
    } else if (battle.regionBWins >= 3 || battle.regionBWins > battle.regionAWins) {
      battle.winnerId = battle.regionB; battle.status = 'completed'
      await cancelUnusedMatches_battle(battle); await cancelUnusedTiebreaker(battle); await checkTournamentCompletion()
    } else {
      await setupTiebreakerMatch(battle)
      if (battle.tiebreakerMatch) await simulateTiebreakerInternal(battle)
    }
  }

  const batchSimulateRegionBattle = async () => {
    try {
      await ElMessageBox.confirm('将自动模拟所有未完成的赛区对决比赛，直到决出最强赛区。是否继续?', '模拟赛区对决确认', { confirmButtonText: '开始模拟', cancelButtonText: '取消', type: 'warning' })
      simulatingRegionBattle.value = true; battleSimProgress.value = 0
      if (icpTournament.semifinal) {
        if (icpTournament.semifinal.status === 'tiebreaker' && icpTournament.semifinal.tiebreakerMatch) await simulateTiebreakerInternal(icpTournament.semifinal)
        else if (icpTournament.semifinal.status !== 'completed') await simulateRegionBattleInternal(icpTournament.semifinal)
      }
      if (icpTournament.final) {
        if (icpTournament.final.status === 'tiebreaker' && icpTournament.final.tiebreakerMatch) await simulateTiebreakerInternal(icpTournament.final)
        else if (icpTournament.final.status !== 'completed') await simulateRegionBattleInternal(icpTournament.final)
      }
      ElMessage.success('赛区对决模拟完成！')
    } catch (error: any) {
      if (error !== 'cancel') ElMessage.error(error.message || '赛区对决模拟失败')
    } finally {
      simulatingRegionBattle.value = false; battleSimProgress.value = 0
    }
  }

  const restoreRegionBattleFromBackend = (bracket: BracketInfo) => {
    if (!bracket.matches) return
    const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
    sortedRegions.forEach((region, index) => region.ranking = index + 1)
    const semiMatches = bracket.matches.filter(m => m.stage?.startsWith('ICP_SEMI_') && !m.stage?.includes('TIEBREAKER'))
    const semiTiebreaker = bracket.matches.find(m => m.stage === 'ICP_SEMI_TIEBREAKER')
    const finalMatches = bracket.matches.filter(m => m.stage?.startsWith('ICP_FINAL_') && !m.stage?.includes('TIEBREAKER'))
    const finalTiebreaker = bracket.matches.find(m => m.stage === 'ICP_FINAL_TIEBREAKER')
    const hasAssignedSemiMatches = semiMatches.some(m => m.home_team?.id && m.away_team?.id)
    const hasAssignedFinalMatches = finalMatches.some(m => m.home_team?.id && m.away_team?.id)

    if (hasAssignedSemiMatches && semiMatches.length >= 4) {
      const firstMatch = semiMatches.find(m => m.home_team?.id && m.away_team?.id)
      if (firstMatch && firstMatch.home_team && firstMatch.away_team) {
        const regionA = firstMatch.home_team.region_code || ''
        const regionB = firstMatch.away_team.region_code || ''
        const regionStatsA = icpTournament.regionStats.find(r => r.region === regionA)
        const regionStatsB = icpTournament.regionStats.find(r => r.region === regionB)
        if (regionStatsA && regionStatsB) {
          const semifinalMatches: ICPMatch[] = []
          let regionAWins = 0; let regionBWins = 0
          const sortedSemiMatches = [...semiMatches].sort((a, b) => (parseInt(a.stage?.replace('ICP_SEMI_', '') || '0') - parseInt(b.stage?.replace('ICP_SEMI_', '') || '0')))
          
          sortedSemiMatches.forEach((m, idx) => {
            if (!m.home_team || !m.away_team) return
            const matchStatus = (m.status || '').toUpperCase()
            const isCompleted = matchStatus === 'COMPLETED'
            semifinalMatches.push({
              id: `semifinal-seed${idx + 1}`, backendMatchId: m.match_id,
              teamAId: String(m.home_team.id), teamAName: m.home_team.short_name || m.home_team.name || '', teamARegion: m.home_team.region_code || '',
              teamBId: String(m.away_team.id), teamBName: m.away_team.short_name || m.away_team.name || '', teamBRegion: m.away_team.region_code || '',
              scoreA: m.home_score || 0, scoreB: m.away_score || 0, winnerId: m.winner_id ? String(m.winner_id) : null,
              status: isCompleted ? 'completed' : matchStatus === 'CANCELLED' ? 'cancelled' : 'scheduled', bestOf: 5, stage: 'semifinal'
            })
            if (isCompleted && m.winner_id) {
              const winnerRegion = m.winner_id === m.home_team.id ? m.home_team.region_code : m.away_team.region_code
              if (winnerRegion === regionA) regionAWins++; else if (winnerRegion === regionB) regionBWins++
            }
          })

          let semifinalStatus: 'scheduled' | 'in_progress' | 'completed' | 'tiebreaker' = 'scheduled'
          let semifinalWinner: string | null = null
          if (regionAWins >= 3) { semifinalStatus = 'completed'; semifinalWinner = regionA }
          else if (regionBWins >= 3) { semifinalStatus = 'completed'; semifinalWinner = regionB }
          else if (semifinalMatches.some(m => m.status === 'completed')) {
            if (semifinalMatches.every(m => m.status === 'completed')) {
              if (regionAWins > regionBWins) { semifinalStatus = 'completed'; semifinalWinner = regionA }
              else if (regionBWins > regionAWins) { semifinalStatus = 'completed'; semifinalWinner = regionB }
              else semifinalStatus = 'tiebreaker'
            } else semifinalStatus = 'in_progress'
          }

          icpTournament.semifinal = {
            id: `semifinal-${regionA}-vs-${regionB}`, regionA, regionB, regionAName: regionStatsA.regionName, regionBName: regionStatsB.regionName,
            matches: semifinalMatches, regionAWins, regionBWins, winnerId: semifinalWinner, status: semifinalStatus, stage: 'semifinal'
          }

          if (semiTiebreaker && semiTiebreaker.home_team && semiTiebreaker.away_team) {
            const tbStatus = (semiTiebreaker.status || '').toUpperCase(); const tbCompleted = tbStatus === 'COMPLETED'
            icpTournament.semifinal.tiebreakerMatch = {
              id: 'semifinal-tiebreaker', backendMatchId: semiTiebreaker.match_id,
              teamAId: String(semiTiebreaker.home_team.id), teamAName: semiTiebreaker.home_team.short_name || semiTiebreaker.home_team.name || '', teamARegion: semiTiebreaker.home_team.region_code || '',
              teamBId: String(semiTiebreaker.away_team.id), teamBName: semiTiebreaker.away_team.short_name || semiTiebreaker.away_team.name || '', teamBRegion: semiTiebreaker.away_team.region_code || '',
              scoreA: semiTiebreaker.home_score || 0, scoreB: semiTiebreaker.away_score || 0, winnerId: semiTiebreaker.winner_id ? String(semiTiebreaker.winner_id) : null,
              status: tbCompleted ? 'completed' : 'scheduled', bestOf: 5, stage: 'semifinal'
            }
            if (tbCompleted && semiTiebreaker.winner_id) {
              const tbWinnerRegion = semiTiebreaker.winner_id === semiTiebreaker.home_team.id ? semiTiebreaker.home_team.region_code : semiTiebreaker.away_team.region_code
              icpTournament.semifinal.winnerId = tbWinnerRegion || null; icpTournament.semifinal.status = 'completed'
            }
          }
        }
      }
    }

    if (hasAssignedFinalMatches && finalMatches.length >= 4) {
      const firstMatch = finalMatches.find(m => m.home_team?.id && m.away_team?.id)
      if (firstMatch && firstMatch.home_team && firstMatch.away_team) {
        const regionA = firstMatch.home_team.region_code || ''
        const regionB = firstMatch.away_team.region_code || ''
        const regionStatsA = icpTournament.regionStats.find(r => r.region === regionA)
        const regionStatsB = icpTournament.regionStats.find(r => r.region === regionB)
        if (regionStatsA && regionStatsB) {
          const finalBattleMatches: ICPMatch[] = []
          let regionAWins = 0; let regionBWins = 0
          const sortedFinalMatches = [...finalMatches].sort((a, b) => (parseInt(a.stage?.replace('ICP_FINAL_', '') || '0') - parseInt(b.stage?.replace('ICP_FINAL_', '') || '0')))
          
          sortedFinalMatches.forEach((m, idx) => {
            if (!m.home_team || !m.away_team) return
            const matchStatus = (m.status || '').toUpperCase()
            const isCompleted = matchStatus === 'COMPLETED'
            finalBattleMatches.push({
              id: `final-seed${idx + 1}`, backendMatchId: m.match_id,
              teamAId: String(m.home_team.id), teamAName: m.home_team.short_name || m.home_team.name || '', teamARegion: m.home_team.region_code || '',
              teamBId: String(m.away_team.id), teamBName: m.away_team.short_name || m.away_team.name || '', teamBRegion: m.away_team.region_code || '',
              scoreA: m.home_score || 0, scoreB: m.away_score || 0, winnerId: m.winner_id ? String(m.winner_id) : null,
              status: isCompleted ? 'completed' : matchStatus === 'CANCELLED' ? 'cancelled' : 'scheduled', bestOf: 5, stage: 'final'
            })
            if (isCompleted && m.winner_id) {
              const winnerRegion = m.winner_id === m.home_team.id ? m.home_team.region_code : m.away_team.region_code
              if (winnerRegion === regionA) regionAWins++; else if (winnerRegion === regionB) regionBWins++
            }
          })

          let finalStatus: 'scheduled' | 'in_progress' | 'completed' | 'tiebreaker' = 'scheduled'
          let finalWinner: string | null = null
          if (regionAWins >= 3) { finalStatus = 'completed'; finalWinner = regionA }
          else if (regionBWins >= 3) { finalStatus = 'completed'; finalWinner = regionB }
          else if (finalBattleMatches.some(m => m.status === 'completed')) {
            if (finalBattleMatches.every(m => m.status === 'completed')) {
              if (regionAWins > regionBWins) { finalStatus = 'completed'; finalWinner = regionA }
              else if (regionBWins > regionAWins) { finalStatus = 'completed'; finalWinner = regionB }
              else finalStatus = 'tiebreaker'
            } else finalStatus = 'in_progress'
          }

          icpTournament.final = {
            id: `final-${regionA}-vs-${regionB}`, regionA, regionB, regionAName: regionStatsA.regionName, regionBName: regionStatsB.regionName,
            matches: finalBattleMatches, regionAWins, regionBWins, winnerId: finalWinner, status: finalStatus, stage: 'final'
          }

          if (finalTiebreaker && finalTiebreaker.home_team && finalTiebreaker.away_team) {
            const tbStatus = (finalTiebreaker.status || '').toUpperCase(); const tbCompleted = tbStatus === 'COMPLETED'
            icpTournament.final.tiebreakerMatch = {
              id: 'final-tiebreaker', backendMatchId: finalTiebreaker.match_id,
              teamAId: String(finalTiebreaker.home_team.id), teamAName: finalTiebreaker.home_team.short_name || finalTiebreaker.home_team.name || '', teamARegion: finalTiebreaker.home_team.region_code || '',
              teamBId: String(finalTiebreaker.away_team.id), teamBName: finalTiebreaker.away_team.short_name || finalTiebreaker.away_team.name || '', teamBRegion: finalTiebreaker.away_team.region_code || '',
              scoreA: finalTiebreaker.home_score || 0, scoreB: finalTiebreaker.away_score || 0, winnerId: finalTiebreaker.winner_id ? String(finalTiebreaker.winner_id) : null,
              status: tbCompleted ? 'completed' : 'scheduled', bestOf: 5, stage: 'final'
            }
            if (tbCompleted && finalTiebreaker.winner_id) {
              const tbWinnerRegion = finalTiebreaker.winner_id === finalTiebreaker.home_team.id ? finalTiebreaker.home_team.region_code : finalTiebreaker.away_team.region_code
              icpTournament.final.winnerId = tbWinnerRegion || null; icpTournament.final.status = 'completed'
            }
          }
        }
      }
    }

    if (icpTournament.semifinal || icpTournament.final) {
      icpTournament.status = 'region_battle'
      if (icpTournament.semifinal?.status === 'completed' && !icpTournament.final) {
        const semifinalWinner = icpTournament.regionStats.find(r => r.region === icpTournament.semifinal?.winnerId)
        if (semifinalWinner) {
          icpTournament.final = createRegionBattle(sortedRegions[0], semifinalWinner, 'final')
          // Using a temporary property to signal that we need to fill teams later
          ;(icpTournament.final as any)._needsFillTeams = true
        }
      }
      if (icpTournament.final?.status === 'completed') {
        const finalWinner = icpTournament.final.winnerId
        const finalLoser = finalWinner === icpTournament.final.regionA ? icpTournament.final.regionB : icpTournament.final.regionA
        icpTournament.champion = icpTournament.regionStats.find(r => r.region === finalWinner)
        icpTournament.runnerUp = icpTournament.regionStats.find(r => r.region === finalLoser)
        const remainingRegions = sortedRegions.filter(r => r.region !== finalWinner && r.region !== finalLoser)
        icpTournament.thirdPlace = remainingRegions[0]; icpTournament.fourthPlace = remainingRegions[1]
        icpTournament.status = 'completed'
      }
    }
  }

  const initializeSeedGroupsFromBackend = (bracket: BracketInfo, standings: GroupStandingInfo[]) => {
    icpTournament.seedGroups = []
    icpTournament.regionStats = []
    const teamRegionMap = new Map<number, string>()
    if (bracket.matches) {
      bracket.matches.forEach(match => {
        if (match.home_team) teamRegionMap.set(match.home_team.id, match.home_team.region_code || '')
        if (match.away_team) teamRegionMap.set(match.away_team.id, match.away_team.region_code || '')
      })
    }

    const groupMap = new Map<string, { teams: any[], matches: any[] }>()
    standings.forEach(groupStanding => {
      let groupName = groupStanding.group_name || 'A'
      groupName = groupName.replace('ICP_GROUP_', '').replace('ICP_', '').replace('GROUP_', '')
      if (!groupMap.has(groupName)) groupMap.set(groupName, { teams: [], matches: [] })
      if (groupStanding.teams) {
        groupStanding.teams.forEach(teamStats => {
          const regionCode = teamRegionMap.get(teamStats.team_id) || ''
          groupMap.get(groupName)!.teams.push({
            teamId: String(teamStats.team_id), teamName: teamStats.team_name, region: regionCode,
            wins: teamStats.wins || 0, losses: teamStats.losses || 0, points: teamStats.points || 0,
            gamesWon: teamStats.games_won || 0, gamesLost: teamStats.games_lost || 0, position: 0, hasBadge: false
          })
        })
      }
    })

    if (bracket.matches) {
      bracket.matches.forEach(match => {
        const stage = match.stage || ''
        if (!stage.startsWith('ICP_GROUP_')) return
        const groupName = stage.replace('ICP_GROUP_', '')
        if (!['A', 'B', 'C', 'D'].includes(groupName)) return
        if (!groupMap.has(groupName)) groupMap.set(groupName, { teams: [], matches: [] })
        const homeTeam = match.home_team; const awayTeam = match.away_team
        if (homeTeam && awayTeam) {
          const matchStatus = (match.status || '').toUpperCase(); const isCompleted = matchStatus === 'COMPLETED'
          groupMap.get(groupName)!.matches.push({
            id: String(match.match_id), backendMatchId: match.match_id, groupName: groupName,
            teamAId: String(homeTeam.id), teamAName: homeTeam.short_name || homeTeam.name, teamARegion: homeTeam.region_code || '',
            teamBId: String(awayTeam.id), teamBName: awayTeam.short_name || awayTeam.name, teamBRegion: awayTeam.region_code || '',
            scoreA: match.home_score || 0, scoreB: match.away_score || 0, winnerId: match.winner_id ? String(match.winner_id) : null,
            status: isCompleted ? 'completed' : 'scheduled', bestOf: 3, stage: 'group'
          })
        }
      })
    }

    const seedGroups: ICPSeedGroup[] = []
    const regionStatsMap = new Map<string, ICPRegionStats>()
    const sortedGroupNames = Array.from(groupMap.keys()).sort()
    sortedGroupNames.forEach(groupName => {
      const groupData = groupMap.get(groupName)!
      const teams = groupData.teams; const matches = groupData.matches
      teams.sort((a: any, b: any) => {
        if (b.points !== a.points) return b.points - a.points
        const aDiff = a.gamesWon - a.gamesLost; const bDiff = b.gamesWon - b.gamesLost
        if (bDiff !== aDiff) return bDiff - aDiff
        if (b.wins !== a.wins) return b.wins - a.wins
        return parseInt(String(a.teamId)) - parseInt(String(b.teamId))
      })
      teams.forEach((team: any, index: number) => { team.position = index + 1; team.hasBadge = index < 2 })
      const isComplete = matches.length > 0 && matches.every((m: any) => m.status === 'completed')
      const seedNumber = groupName.charCodeAt(0) - 'A'.charCodeAt(0) + 1
      const standings: ICPGroupStanding[] = teams.map((team: any) => ({
        teamId: team.teamId, teamName: team.teamName, region: team.region, seed: seedNumber,
        matchesPlayed: team.wins + team.losses, wins: team.wins, losses: team.losses, points: team.points,
        roundsWon: team.gamesWon, roundsLost: team.gamesLost, roundDifferential: team.gamesWon - team.gamesLost,
        position: team.position, hasBadge: team.hasBadge
      }))
      seedGroups.push({ groupName: groupName as 'A' | 'B' | 'C' | 'D', seedNumber, teams: teams.map((t: any) => ({ id: t.teamId, name: t.teamName, region: t.region })), matches, standings, isComplete })
      
      teams.forEach((team: any) => {
        const region = team.region; if (!region) return
        if (!regionStatsMap.has(region)) regionStatsMap.set(region, { region, regionName: getRegionDisplayName(region), teams: [], totalBadges: 0, ranking: 0 })
        const regionStats = regionStatsMap.get(region)!; const seed = seedGroups.length
        if (!regionStats.teams.find(t => t.id === team.teamId)) {
          regionStats.teams.push({ id: team.teamId, name: team.teamName, region: region, seed, badges: isComplete && team.hasBadge ? 1 : 0 })
          if (isComplete && team.hasBadge) regionStats.totalBadges++
        }
      })
    })

    icpTournament.seedGroups = seedGroups
    icpTournament.regionStats = Array.from(regionStatsMap.values())
    if (seedGroups.length > 0) {
      const allComplete = seedGroups.every(g => g.isComplete)
      const anyStarted = seedGroups.some(g => g.matches.some(m => m.status === 'completed'))
      if (allComplete) icpTournament.status = 'group_stage'
      else if (anyStarted) icpTournament.status = 'group_stage'
      else icpTournament.status = 'group_stage'
    }
    if (seedGroups.length > 0) activeSeedGroup.value = seedGroups[0].groupName
  }

  const loadICPData = async () => {
    loading.value = true
    try {
      await timeStore.fetchTimeState()
      const currentSave = gameStore.currentSave
      if (!currentSave) return
      const seasonId = viewingSeason.value
      const tournaments = await internationalApi.getTournamentsByType('Icp', seasonId)
      if (tournaments && tournaments.length > 0) tournamentId.value = tournaments[0].id
      if (!tournamentId.value) return
      const teams = await teamApi.getAllTeams()
      if (teams) {
        teamMap.value.clear()
        teams.forEach((team: any) => teamMap.value.set(team.id, { name: team.name, regionCode: team.region_code || team.regionCode || '' }))
      }
      const bracket = await internationalApi.getTournamentBracket(tournamentId.value)
      if (bracket) bracketData.value = bracket
      const standings = await internationalApi.getGroupStandings(tournamentId.value)
      if (standings) groupStandings.value = standings
      if (bracket && standings && standings.length > 0) {
        initializeSeedGroupsFromBackend(bracket, standings)
        restoreRegionBattleFromBackend(bracket)
        if (icpTournament.final && (icpTournament.final as any)._needsFillTeams) {
          delete (icpTournament.final as any)._needsFillTeams
          await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
        }
      }
    } catch (error) {
      logger.error('加载ICP数据失败:', error)
    } finally {
      loading.value = false
    }
  }

  onMounted(() => {
    loadICPData()
  })

  return {
    // State
    icpTournament,
    tournamentId,
    bracketData,
    groupStandings,
    loading,
    teamMap,
    generatingRegionBattle,
    activeSeedGroup,
    showMatchDetailDialog,
    currentMatchDetail,
    groupSimProgress,
    simulatingGroupStage,
    battleSimProgress,
    simulatingRegionBattle,

    // Computed
    viewingSeason,
    phaseNotReached,
    currentPhaseDisplay,
    isGroupStageComplete,
    sortedRegionStats,
    icpStandings,
    seedTeamsGrouped,

    // Methods
    loadICPData,
    goBack,
    getStatusType,
    getStatusText,
    getSeedGroupLabel,
    getRegionFlag,
    viewMatchDetails,
    handleCloseMatchDetail,
    handleSimulateMatch,
    handleSimulateRegionMatch,
    handleSimulateTiebreaker,
    batchSimulateGroupStage,
    batchSimulateRegionBattle,
    handleGenerateRegionBattle,
    getRegionDisplayName
  }
}
