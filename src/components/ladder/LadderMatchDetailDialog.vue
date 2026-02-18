<template>
  <el-dialog
    v-model="dialogVisible"
    :show-header="false"
    width="860px"
    :close-on-click-modal="true"
    class="ladder-match-dialog"
    @close="handleClose"
  >
    <template v-if="detail">
      <div class="scoreboard">
        <div class="scoreboard-main">
          <div class="sb-team blue" :class="{ winner: detail.winner_side === 'blue' }">
            <span class="sb-side">蓝方</span>
            <span class="sb-rating">{{ detail.blue_avg_rating }}</span>
          </div>
          <div class="sb-center">
            <span class="sb-round">第{{ detail.round_number }}轮 #{{ detail.match_number }}</span>
            <span class="sb-vs">VS</span>
            <span v-if="detail.game_duration" class="sb-duration">{{ detail.game_duration }}min</span>
          </div>
          <div class="sb-team red" :class="{ winner: detail.winner_side === 'red' }">
            <span class="sb-side">红方</span>
            <span class="sb-rating">{{ detail.red_avg_rating }}</span>
          </div>
        </div>
        <div class="sb-meta">
          <span class="winner-tag" :class="detail.winner_side === 'blue' ? 'blue-win' : 'red-win'">
            {{ detail.winner_side === 'blue' ? '蓝方' : '红方' }}胜
          </span>
          <span v-if="detail.mvp_player_name" class="mvp-tag">
            <span class="mvp-label">MVP</span>
            {{ mvpGameId }}
          </span>
        </div>
      </div>

      <div class="power-bar-section">
        <div class="power-bar-header">
          <span class="blue-text">{{ detail.blue_power.toFixed(1) }}</span>
          <span class="power-title">战力对比</span>
          <span class="red-text">{{ detail.red_power.toFixed(1) }}</span>
        </div>
        <div class="power-bar-track">
          <div class="power-bar blue-bar" :style="{ width: bluePowerPercent + '%' }"></div>
          <div class="power-bar red-bar" :style="{ width: redPowerPercent + '%' }"></div>
        </div>
        <div class="power-diff" :class="powerDiffClass">{{ powerDiffText }}</div>
      </div>

      <div class="players-section">
        <div class="team-block">
          <div class="team-header blue-header">
            <span>蓝方</span>
            <span class="team-header-labels">
              <span class="th-label">位置</span>
              <span class="th-label">选手</span>
              <span class="th-label">战队</span>
              <span class="th-label">Rating</span>
              <span class="th-label">影响力</span>
              <span class="th-label">分数</span>
            </span>
          </div>
          <div v-for="p in blueTeamSorted" :key="p.player_id" class="player-row" :class="{ 'is-mvp-row': p.player_id === detail.mvp_player_id }">
            <span class="pr-pos"><el-tag :type="posType(p.position)" size="small">{{ p.position }}</el-tag></span>
            <span class="pr-name">{{ p.game_id || p.player_name }}</span>
            <span class="pr-team">{{ p.team_name || '-' }}</span>
            <span class="pr-rating">{{ p.rating }}</span>
            <span class="pr-influence">
              <span class="influence-value" :class="getInfluenceClass(getInfluence(p.player_id, 'blue'))">{{ formatInfluence(getInfluence(p.player_id, 'blue')) }}</span>
            </span>
            <span class="pr-delta" :class="getRatingDeltaClass(getRatingDelta(p.player_id))">{{ formatRatingDelta(getRatingDelta(p.player_id)) }}</span>
            <span v-if="p.player_id === detail.mvp_player_id" class="pr-mvp">MVP</span>
          </div>
        </div>
        <div class="team-block">
          <div class="team-header red-header">
            <span>红方</span>
            <span class="team-header-labels">
              <span class="th-label">位置</span>
              <span class="th-label">选手</span>
              <span class="th-label">战队</span>
              <span class="th-label">Rating</span>
              <span class="th-label">影响力</span>
              <span class="th-label">分数</span>
            </span>
          </div>
          <div v-for="p in redTeamSorted" :key="p.player_id" class="player-row" :class="{ 'is-mvp-row': p.player_id === detail.mvp_player_id }">
            <span class="pr-pos"><el-tag :type="posType(p.position)" size="small">{{ p.position }}</el-tag></span>
            <span class="pr-name">{{ p.game_id || p.player_name }}</span>
            <span class="pr-team">{{ p.team_name || '-' }}</span>
            <span class="pr-rating">{{ p.rating }}</span>
            <span class="pr-influence">
              <span class="influence-value" :class="getInfluenceClass(getInfluence(p.player_id, 'red'))">{{ formatInfluence(getInfluence(p.player_id, 'red')) }}</span>
            </span>
            <span class="pr-delta" :class="getRatingDeltaClass(getRatingDelta(p.player_id))">{{ formatRatingDelta(getRatingDelta(p.player_id)) }}</span>
            <span v-if="p.player_id === detail.mvp_player_id" class="pr-mvp">MVP</span>
          </div>
        </div>
      </div>

      <div v-if="draftData" class="bp-section">
        <button class="bp-toggle" @click="bpOpen = !bpOpen">
          <span class="toggle-icon" :class="{ open: bpOpen }">&#9654;</span>
          Ban/Pick
        </button>
        <div v-if="bpOpen" class="bp-content">
          <div class="bp-grid">
            <div class="bp-col">
              <div class="bp-label blue-text">蓝方 Ban</div>
              <div class="bp-tags">
                <el-tag v-for="b in blueBans" :key="b.champion_id" type="danger" size="small" effect="dark">{{ getChampionName(b.champion_id) }}</el-tag>
              </div>
              <div class="bp-label blue-text" style="margin-top:10px">蓝方 Pick</div>
              <div class="bp-picks">
                <div v-for="pk in bluePicks" :key="pk.champion_id" class="bp-pick-row">
                  <span class="bp-pos">{{ pk.position }}</span>
                  <span class="bp-player-name">{{ getPickPlayerName(pk.player_id) }}</span>
                  <el-tag size="small">{{ getChampionName(pk.champion_id) }}</el-tag>
                  <span v-if="pk.mastery_tier" class="bp-tier" :class="'tier-' + pk.mastery_tier">{{ pk.mastery_tier }}</span>
                  <span v-if="getPickBpModifier(pk.player_id) !== undefined" class="bp-modifier" :class="getPickBpModifier(pk.player_id)! >= 0 ? 'positive' : 'negative'">
                    {{ getPickBpModifier(pk.player_id)! >= 0 ? '+' : '' }}{{ getPickBpModifier(pk.player_id)!.toFixed(1) }}%
                  </span>
                </div>
              </div>
              <div v-if="draftData.home_comp" class="bp-comp">体系: <el-tag type="warning" size="small">{{ compName(draftData.home_comp) }}</el-tag></div>
            </div>
            <div class="bp-col">
              <div class="bp-label red-text">红方 Ban</div>
              <div class="bp-tags">
                <el-tag v-for="b in redBans" :key="b.champion_id" type="danger" size="small" effect="dark">{{ getChampionName(b.champion_id) }}</el-tag>
              </div>
              <div class="bp-label red-text" style="margin-top:10px">红方 Pick</div>
              <div class="bp-picks">
                <div v-for="pk in redPicks" :key="pk.champion_id" class="bp-pick-row">
                  <span class="bp-pos">{{ pk.position }}</span>
                  <span class="bp-player-name">{{ getPickPlayerName(pk.player_id) }}</span>
                  <el-tag size="small">{{ getChampionName(pk.champion_id) }}</el-tag>
                  <span v-if="pk.mastery_tier" class="bp-tier" :class="'tier-' + pk.mastery_tier">{{ pk.mastery_tier }}</span>
                  <span v-if="getPickBpModifier(pk.player_id) !== undefined" class="bp-modifier" :class="getPickBpModifier(pk.player_id)! >= 0 ? 'positive' : 'negative'">
                    {{ getPickBpModifier(pk.player_id)! >= 0 ? '+' : '' }}{{ getPickBpModifier(pk.player_id)!.toFixed(1) }}%
                  </span>
                </div>
              </div>
              <div v-if="draftData.away_comp" class="bp-comp">体系: <el-tag type="warning" size="small">{{ compName(draftData.away_comp) }}</el-tag></div>
            </div>
          </div>

          <div v-if="parsedNarrative" class="bp-narrative">
            <div class="bp-narrative-title">💭 BP博弈心理</div>
            <div class="bp-narrative-row">
              <div class="bp-narrative-col">
                <div class="bp-label blue-text">蓝方</div>
                <div class="bp-narrative-list">
                  <div
                    v-for="(entry, idx) in parsedNarrative.home_entries"
                    :key="'hn-' + idx"
                    class="bp-narrative-item"
                    :class="'phase-' + entry.phase"
                  >
                    <span class="narrative-phase">{{ phaseLabel(entry.phase) }}</span>
                    <span class="narrative-msg">{{ entry.message }}</span>
                  </div>
                </div>
              </div>
              <div class="bp-narrative-col">
                <div class="bp-label red-text">红方</div>
                <div class="bp-narrative-list">
                  <div
                    v-for="(entry, idx) in parsedNarrative.away_entries"
                    :key="'an-' + idx"
                    class="bp-narrative-item"
                    :class="'phase-' + entry.phase"
                  >
                    <span class="narrative-phase">{{ phaseLabel(entry.phase) }}</span>
                    <span class="narrative-msg">{{ entry.message }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template v-else-if="loading">
      <div style="padding:32px"><el-skeleton :rows="8" animated /></div>
    </template>
    <template v-else>
      <el-empty description="暂无对局详情" />
    </template>

    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { getLadderMatchDetail, type LadderMatchDetail, type LadderPlayerInfo } from '@/api/ladder'
import { getChampionList, type ChampionInfo } from '@/api/tauri'

interface Props { visible: boolean; matchId: number | null }
const props = defineProps<Props>()
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void }>()

const dialogVisible = computed({ get: () => props.visible, set: (v) => emit('update:visible', v) })
const detail = ref<LadderMatchDetail | null>(null)
const loading = ref(false)
const bpOpen = ref(false)
const championMap = ref<Map<number, ChampionInfo>>(new Map())

interface DraftData {
  bans: Array<{ champion_id: number; team_side: string; ban_phase: number }>
  home_picks: Array<{ champion_id: number; player_id: number; position: string; mastery_tier: string }>
  away_picks: Array<{ champion_id: number; player_id: number; position: string; mastery_tier: string }>
  home_comp: string | null
  away_comp: string | null
  home_bp_modifiers?: Record<string, number>
  away_bp_modifiers?: Record<string, number>
  narrative?: string | NarrativeData
}

interface NarrativeEntry { phase: string; message: string }
interface NarrativeData { home_entries: NarrativeEntry[]; away_entries: NarrativeEntry[] }

const draftData = computed<DraftData | null>(() => {
  if (!detail.value?.draft_result_json) return null
  try { return JSON.parse(detail.value.draft_result_json) } catch { return null }
})

const POS_ORDER: Record<string, number> = { Top: 0, Jug: 1, Mid: 2, Adc: 3, Sup: 4 }
const sortByPos = (arr: LadderPlayerInfo[]) => [...arr].sort((a, b) => (POS_ORDER[a.position] ?? 99) - (POS_ORDER[b.position] ?? 99))

const blueTeamSorted = computed(() => detail.value ? sortByPos(detail.value.blue_team) : [])
const redTeamSorted = computed(() => detail.value ? sortByPos(detail.value.red_team) : [])

const mvpGameId = computed(() => {
  if (!detail.value) return ''
  const all = [...detail.value.blue_team, ...detail.value.red_team]
  const p = all.find(x => x.player_id === detail.value!.mvp_player_id)
  return p?.game_id || p?.player_name || detail.value.mvp_player_name || ''
})

const blueBans = computed(() => draftData.value?.bans.filter(b => b.team_side === 'Home') || [])
const redBans = computed(() => draftData.value?.bans.filter(b => b.team_side === 'Away') || [])
const bluePicks = computed(() => [...(draftData.value?.home_picks || [])].sort((a, b) => (POS_ORDER[a.position] ?? 99) - (POS_ORDER[b.position] ?? 99)))
const redPicks = computed(() => [...(draftData.value?.away_picks || [])].sort((a, b) => (POS_ORDER[a.position] ?? 99) - (POS_ORDER[b.position] ?? 99)))

const totalPower = computed(() => (detail.value?.blue_power || 0) + (detail.value?.red_power || 0))
const bluePowerPercent = computed(() => totalPower.value > 0 ? ((detail.value?.blue_power || 0) / totalPower.value) * 100 : 50)
const redPowerPercent = computed(() => 100 - bluePowerPercent.value)
const powerDiff = computed(() => (detail.value?.blue_power || 0) - (detail.value?.red_power || 0))
const powerDiffClass = computed(() => powerDiff.value > 0 ? 'blue-adv' : powerDiff.value < 0 ? 'red-adv' : '')
const powerDiffText = computed(() => {
  const d = powerDiff.value
  if (d > 0) return `+${d.toFixed(2)} 蓝方优势`
  if (d < 0) return `${d.toFixed(2)} 红方优势`
  return '势均力敌'
})

const posType = (p: string) => ({ Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info' }[p] || '') as '' | 'success' | 'warning' | 'info' | 'danger'
const getChampionName = (id: number) => championMap.value.get(id)?.name_cn || `#${id}`

const COMP_NAMES: Record<string, string> = {
  Rush: '速推', PickOff: '抓单', AllIn: '莽夫', MidJungle: '中野联动', TopJungle: '上野联动',
  Protect: '保C', Fortress: '铁桶阵', UtilityComp: '功能流', Stall: '龟缩', BotLane: '下路统治',
  Teamfight: '团战', Dive: '开团', Skirmish: '小规模团战', DualCarry: '双C', Flex: '全能',
  Splitpush: '分推', SideLane: '4-1分带', Control: '运营', TripleThreat: '三线施压', LateGame: '后期发育',
}
const compName = (t: string) => COMP_NAMES[t] || t

// 影响力计算
const teamAvgPerf = (side: 'blue' | 'red'): number => {
  if (!detail.value?.performances) return 0
  const team = side === 'blue' ? detail.value.blue_team : detail.value.red_team
  const perfs = team.map(p => detail.value!.performances![String(p.player_id)] ?? 0)
  if (perfs.length === 0) return 0
  return perfs.reduce((a, b) => a + b, 0) / perfs.length
}

const getInfluence = (playerId: number, side: 'blue' | 'red'): number | null => {
  if (!detail.value?.performances) return null
  const perf = detail.value.performances[String(playerId)]
  if (perf === undefined) return null
  return perf - teamAvgPerf(side)
}

const formatInfluence = (val: number | null): string => {
  if (val === null) return '-'
  if (val > 0) return `+${val.toFixed(2)}`
  return val.toFixed(2)
}

const getInfluenceClass = (val: number | null): string => {
  if (val === null) return ''
  if (val > 3) return 'inf-very-positive'
  if (val > 0) return 'inf-positive'
  if (val < -3) return 'inf-very-negative'
  if (val < 0) return 'inf-negative'
  return ''
}

// BP 选手名和修正值
const getPickPlayerName = (playerId: number): string => {
  if (!detail.value) return ''
  const all = [...detail.value.blue_team, ...detail.value.red_team]
  const p = all.find(x => x.player_id === playerId)
  return p?.game_id || p?.player_name || ''
}

const getPickBpModifier = (playerId: number): number | undefined => {
  if (!draftData.value) return undefined
  const homeVal = draftData.value.home_bp_modifiers?.[String(playerId)]
  if (homeVal !== undefined) return homeVal
  const awayVal = draftData.value.away_bp_modifiers?.[String(playerId)]
  if (awayVal !== undefined) return awayVal
  return undefined
}

// BP 博弈心理叙事
const parsedNarrative = computed<NarrativeData | null>(() => {
  if (!draftData.value?.narrative) return null
  try {
    const raw = draftData.value.narrative
    if (typeof raw === 'string') return JSON.parse(raw)
    return raw
  } catch {
    return null
  }
})

const phaseLabel = (phase: string): string => {
  const labels: Record<string, string> = {
    plan: '🎯 规划',
    ban_phase1: '🚫 Ban①',
    ban_phase2: '🚫 Ban②',
    pick: '✅ Pick',
    counter: '🔄 反制',
    flex: '🃏 摇摆',
    switch: '🔄 切换',
  }
  return labels[phase] || phase
}

const getRatingDelta = (playerId: number): number | null => {
  if (!detail.value?.rating_changes) return null
  return detail.value.rating_changes[String(playerId)] ?? null
}

const formatRatingDelta = (val: number | null): string => {
  if (val === null) return '-'
  if (val > 0) return `+${val}`
  return String(val)
}

const getRatingDeltaClass = (val: number | null): string => {
  if (val === null) return ''
  if (val > 0) return 'delta-positive'
  if (val < 0) return 'delta-negative'
  return ''
}

const loadDetail = async () => {
  if (!props.matchId) return
  loading.value = true
  try {
    if (championMap.value.size === 0) {
      const list = await getChampionList()
      const m = new Map<number, ChampionInfo>()
      for (const c of list) m.set(c.id, c)
      championMap.value = m
    }
    detail.value = await getLadderMatchDetail(props.matchId)
  } catch (e) {
    console.error('加载天梯对局详情失败:', e)
    detail.value = null
  } finally {
    loading.value = false
  }
}

watch(() => [props.visible, props.matchId], ([vis]) => {
  if (vis && props.matchId) { bpOpen.value = false; loadDetail() }
})

const handleClose = () => { dialogVisible.value = false }
</script>

<style scoped>
.ladder-match-dialog :deep(.el-dialog) { border-radius: 14px; overflow: hidden; }
.ladder-match-dialog :deep(.el-dialog__body) { padding: 0; max-height: 80vh; overflow-y: auto; }
.ladder-match-dialog :deep(.el-dialog__footer) { border-top: 1px solid #f0f1f3; padding: 10px 20px; }

.scoreboard {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px 28px 14px; color: #fff;
}
.scoreboard-main { display: flex; justify-content: center; align-items: center; gap: 32px; }
.sb-team { display: flex; flex-direction: column; align-items: center; gap: 2px; min-width: 90px; opacity: 0.75; }
.sb-team.winner { opacity: 1; }
.sb-side { font-size: 12px; font-weight: 700; }
.sb-rating { font-size: 28px; font-weight: 800; font-variant-numeric: tabular-nums; }
.sb-team.winner .sb-rating { color: #fbbf24; text-shadow: 0 0 12px rgba(251,191,36,0.5); }
.sb-center { display: flex; flex-direction: column; align-items: center; gap: 2px; }
.sb-round { font-size: 10px; opacity: 0.5; font-weight: 600; }
.sb-vs { font-size: 11px; font-weight: 800; opacity: 0.35; letter-spacing: 2px; }
.sb-duration { font-size: 10px; opacity: 0.4; }

.sb-meta {
  display: flex; justify-content: center; gap: 10px;
  margin-top: 12px; padding-top: 12px; border-top: 1px solid rgba(255,255,255,0.12);
}
.winner-tag { padding: 3px 14px; border-radius: 16px; font-size: 11px; font-weight: 800; }
.blue-win { background: rgba(96,165,250,0.3); border: 1px solid rgba(96,165,250,0.5); }
.red-win { background: rgba(248,113,113,0.3); border: 1px solid rgba(248,113,113,0.5); }
.mvp-tag {
  display: flex; align-items: center; gap: 6px;
  padding: 3px 14px; border-radius: 16px; font-size: 12px; font-weight: 700;
  border: 1px solid rgba(251,191,36,0.35);
  background: linear-gradient(135deg, rgba(251,191,36,0.15), rgba(245,158,11,0.08));
}
.mvp-label {
  font-size: 9px; font-weight: 800; color: #92400e;
  background: linear-gradient(135deg, #fde68a, #fbbf24);
  padding: 1px 6px; border-radius: 10px;
}

.power-bar-section { padding: 12px 24px; background: #f7f8fa; border-bottom: 1px solid #f0f1f3; }
.power-bar-header { display: flex; justify-content: space-between; font-size: 13px; font-weight: 700; margin-bottom: 6px; }
.power-title { font-size: 10px; color: #86909c; font-weight: 500; text-transform: uppercase; letter-spacing: 0.5px; }
.blue-text { color: #3b82f6; }
.red-text { color: #ef4444; }
.power-bar-track { display: flex; height: 12px; background: #e5e7eb; border-radius: 6px; overflow: hidden; }
.power-bar { height: 100%; transition: width 0.4s; }
.blue-bar { background: linear-gradient(to right, #60a5fa, #3b82f6); }
.red-bar { background: linear-gradient(to left, #f87171, #ef4444); }
.power-diff { text-align: center; margin-top: 4px; font-size: 11px; color: #86909c; font-weight: 500; }
.power-diff.blue-adv { color: #3b82f6; }
.power-diff.red-adv { color: #ef4444; }

.players-section { display: grid; grid-template-columns: 1fr 1fr; gap: 0; }
.team-block { border-bottom: 1px solid #f0f1f3; }
.team-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 8px 16px; font-size: 12px; font-weight: 700; color: rgba(255,255,255,0.9);
}
.blue-header { background: #3b82f6; }
.red-header { background: #ef4444; }
.team-power { font-size: 11px; opacity: 0.8; }
.player-row {
  display: grid; grid-template-columns: 52px 1fr 80px 50px 64px 50px 36px;
  align-items: center; padding: 6px 16px; font-size: 12px;
  border-bottom: 1px solid #f5f6f8;
}
.player-row:last-child { border-bottom: none; }
.player-row:hover { background: #f7f8fa; }
.is-mvp-row { background: #fffbeb; }
.is-mvp-row:hover { background: #fef3c7; }
.pr-pos { text-align: center; }
.pr-name { font-weight: 600; color: #1d2129; }
.pr-team { color: #86909c; font-size: 11px; }
.pr-rating { font-weight: 700; color: #4e5969; font-variant-numeric: tabular-nums; text-align: center; }
.pr-mvp {
  font-size: 9px; font-weight: 800; color: #92400e;
  background: linear-gradient(135deg, #fde68a, #fbbf24);
  padding: 1px 5px; border-radius: 8px; text-align: center;
}

.bp-section { border-top: 1px solid #f0f1f3; }
.bp-toggle {
  display: flex; align-items: center; gap: 6px; width: 100%;
  padding: 10px 16px; background: #f7f8fa; border: none; cursor: pointer;
  font-size: 13px; font-weight: 700; color: #1d2129; text-align: left;
}
.bp-toggle:hover { background: #f0f1f3; }
.toggle-icon { font-size: 10px; color: #86909c; transition: transform 0.2s; display: inline-block; }
.toggle-icon.open { transform: rotate(90deg); }
.bp-content { padding: 14px 16px; border-top: 1px solid #f0f1f3; }
.bp-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.bp-label { font-weight: 700; font-size: 11px; margin-bottom: 4px; }
.bp-tags { display: flex; flex-wrap: wrap; gap: 4px; }
.bp-picks { display: flex; flex-direction: column; gap: 3px; }
.bp-pick-row { display: flex; align-items: center; gap: 6px; font-size: 12px; }
.bp-pos { font-size: 10px; font-weight: 700; color: #86909c; min-width: 26px; text-transform: uppercase; }
.bp-tier { font-size: 10px; font-weight: 700; padding: 1px 4px; border-radius: 3px; }
.tier-S { color: #d97706; background: #fef3c7; }
.tier-A { color: #059669; background: #d1fae5; }
.tier-B { color: #3b82f6; background: #dbeafe; }
.tier-C { color: #6b7280; background: #f3f4f6; }
.bp-comp { margin-top: 6px; font-size: 11px; color: #4e5969; }

/* 影响力列 */
.pr-influence { text-align: center; }
.influence-value { font-size: 12px; font-weight: 700; font-variant-numeric: tabular-nums; }
.inf-positive { color: #10b981; }
.inf-negative { color: #ef4444; }
.inf-very-positive { color: #059669; font-weight: 800; }
.inf-very-negative { color: #dc2626; font-weight: 800; }

/* Team header labels */
.team-header-labels { display: none; }

/* Rating delta */
.pr-delta { font-size: 12px; font-weight: 700; font-variant-numeric: tabular-nums; text-align: center; }
.delta-positive { color: #10b981; }
.delta-negative { color: #ef4444; }

/* tier-SS */
.tier-SS { color: #b91c1c; background: #fee2e2; }

/* BP player name & modifier */
.bp-player-name { font-size: 12px; font-weight: 600; color: #1d2129; min-width: 60px; }
.bp-modifier { font-size: 10px; font-weight: 600; padding: 1px 4px; border-radius: 3px; }
.bp-modifier.positive { color: #059669; background: rgba(16, 185, 129, 0.1); }
.bp-modifier.negative { color: #dc2626; background: rgba(220, 38, 38, 0.1); }

/* BP Narrative */
.bp-narrative { margin-top: 12px; padding-top: 12px; border-top: 1px solid #f0f1f3; }
.bp-narrative-title { font-size: 13px; font-weight: 600; color: #4e5969; margin-bottom: 10px; }
.bp-narrative-row { display: flex; gap: 16px; }
.bp-narrative-col { flex: 1; }
.bp-narrative-list { display: flex; flex-direction: column; gap: 4px; }
.bp-narrative-item {
  font-size: 12px; color: #4e5969; padding: 5px 10px; border-radius: 4px;
  background: #f7f8fa; display: flex; align-items: baseline; gap: 6px; line-height: 1.5;
}
.bp-narrative-item.phase-plan { background: rgba(64, 158, 255, 0.08); color: #3b82f6; border-left: 2px solid rgba(64, 158, 255, 0.5); }
.bp-narrative-item.phase-ban_phase1,
.bp-narrative-item.phase-ban_phase2 { background: rgba(245, 108, 108, 0.06); color: #ef4444; border-left: 2px solid rgba(245, 108, 108, 0.4); }
.bp-narrative-item.phase-pick { background: rgba(103, 194, 58, 0.06); color: #059669; border-left: 2px solid rgba(103, 194, 58, 0.4); }
.bp-narrative-item.phase-counter,
.bp-narrative-item.phase-flex,
.bp-narrative-item.phase-switch { background: rgba(255, 165, 0, 0.06); color: #d97706; border-left: 2px solid rgba(255, 165, 0, 0.4); }
.narrative-phase { flex-shrink: 0; font-weight: 600; font-size: 11px; }
.narrative-msg { flex: 1; }
</style>
