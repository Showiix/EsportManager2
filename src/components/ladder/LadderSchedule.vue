<template>
  <div class="ladder-schedule">
    <div v-for="round in totalRounds" :key="round" class="round-section">
      <div class="round-bar" @click="toggleRound(round)">
        <span class="round-arrow" :class="{ open: openRounds.has(round) }">&#9654;</span>
        <span class="round-label">第 {{ round }} 轮</span>
        <el-tag v-if="getRoundMatches(round).length > 0" size="small" type="info">{{ getRoundMatches(round).length }} 场</el-tag>
        <span v-else class="round-pending">未开始</span>
      </div>

      <template v-if="openRounds.has(round)">
        <div v-if="getRoundMatches(round).length === 0" class="round-empty">
          <span>本轮暂无比赛</span>
        </div>

        <div v-else class="match-grid">
        <div
          v-for="match in getRoundMatches(round)"
          :key="match.id"
          class="match-chip"
          :class="{ expanded: expandedId === match.id }"
          @click="toggleExpand(match.id)"
        >
          <div class="chip-main">
            <div class="chip-side blue-side">
              <span class="chip-rating">{{ match.blue_avg_rating }}</span>
            </div>
            <div class="chip-center">
              <span v-if="match.winner_side" class="chip-result" :class="match.winner_side === 'blue' ? 'blue-win' : 'red-win'">
                {{ match.winner_side === 'blue' ? '蓝胜' : '红胜' }}
              </span>
              <span v-else class="chip-vs">VS</span>
              <span v-if="match.mvp_player_name" class="chip-mvp">MVP {{ match.mvp_player_name }}</span>
            </div>
            <div class="chip-side red-side">
              <span class="chip-rating">{{ match.red_avg_rating }}</span>
            </div>
          </div>

          <div v-if="expandedId === match.id" class="chip-detail" @click.stop>
            <div class="detail-teams">
              <div class="detail-col">
                <div class="detail-header blue-bg">蓝方</div>
                <div v-for="p in match.blue_team" :key="p.player_id" class="detail-player">
                  <el-tag :type="posType(p.position)" size="small">{{ p.position }}</el-tag>
                  <span class="dp-name">{{ p.game_id || p.player_name }}</span>
                  <span class="dp-rating">{{ p.rating }}</span>
                </div>
              </div>
              <div class="detail-col">
                <div class="detail-header red-bg">红方</div>
                <div v-for="p in match.red_team" :key="p.player_id" class="detail-player">
                  <el-tag :type="posType(p.position)" size="small">{{ p.position }}</el-tag>
                  <span class="dp-name">{{ p.game_id || p.player_name }}</span>
                  <span class="dp-rating">{{ p.rating }}</span>
                </div>
              </div>
            </div>
            <button class="detail-btn" @click.stop="showMatchDetail(match)">查看详情</button>
          </div>
        </div>
      </div>
      </template>
    </div>

    <div v-if="!loading && matches.length === 0" class="empty-state">
      <el-empty description="暂无赛程数据" :image-size="60" />
    </div>

    <LadderMatchDetailDialog
      :visible="detailVisible"
      :match-id="selectedMatchId"
      @update:visible="detailVisible = $event"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { LadderMatchInfo } from '@/api/ladder'
import LadderMatchDetailDialog from './LadderMatchDetailDialog.vue'

const props = defineProps<{
  matches: LadderMatchInfo[]
  totalRounds: number
  loading: boolean
}>()

const expandedId = ref<number | null>(null)
const openRounds = ref<Set<number>>(new Set())
const detailVisible = ref(false)
const selectedMatchId = ref<number | null>(null)

const getRoundMatches = (round: number) => props.matches.filter(m => m.round_number === round)

const toggleRound = (round: number) => {
  const s = new Set(openRounds.value)
  if (s.has(round)) { s.delete(round) } else { s.add(round) }
  openRounds.value = s
}

const toggleExpand = (id: number) => {
  expandedId.value = expandedId.value === id ? null : id
}

const posType = (p: string) => ({ Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info' }[p] || '') as '' | 'success' | 'warning' | 'info' | 'danger'

const showMatchDetail = (match: LadderMatchInfo) => {
  selectedMatchId.value = match.id
  detailVisible.value = true
}
</script>

<style scoped>
.ladder-schedule { padding: 16px 0; }

.round-section { margin-bottom: 12px; }
.round-bar {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; background: #f8fafc; border-radius: 6px;
  border: 1px solid #e2e8f0;
  margin-bottom: 8px; cursor: pointer; user-select: none;
  transition: background 0.15s;
}
.round-bar:hover { background: #f1f5f9; }
.round-arrow {
  font-size: 10px; color: #94a3b8; transition: transform 0.2s; display: inline-block;
}
.round-arrow.open { transform: rotate(90deg); }
.round-label { font-size: 14px; font-weight: 700; color: #0f172a; }
.round-pending { font-size: 12px; color: #cbd5e1; }
.round-empty { padding: 12px; text-align: center; font-size: 12px; color: #cbd5e1; }

.match-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 8px;
}

.match-chip {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.15s;
  background: #fff;
}
.match-chip:hover { border-color: #6366f1; }
.match-chip.expanded { border-color: #6366f1; background: #fefefe; }

.chip-main {
  display: flex; align-items: center; padding: 10px 12px; gap: 8px;
}
.chip-side { display: flex; flex-direction: column; align-items: center; min-width: 44px; }
.chip-rating { font-size: 16px; font-weight: 800; font-variant-numeric: tabular-nums; }
.blue-side .chip-rating { color: #3b82f6; }
.red-side .chip-rating { color: #ef4444; }

.chip-center { flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px; }
.chip-vs { font-size: 11px; font-weight: 800; color: #cbd5e1; letter-spacing: 1px; }
.chip-result { font-size: 11px; font-weight: 800; padding: 1px 8px; border-radius: 10px; }
.chip-result.blue-win { color: #3b82f6; background: rgba(59,130,246,0.08); }
.chip-result.red-win { color: #ef4444; background: rgba(239,68,68,0.08); }
.chip-mvp { font-size: 10px; color: #f59e0b; font-weight: 600; }

.chip-detail {
  border-top: 1px solid #f1f5f9;
  padding: 10px 12px;
  background: #f8fafc;
}
.detail-teams { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.detail-header {
  font-size: 11px; font-weight: 700; color: #fff;
  padding: 3px 8px; border-radius: 6px; margin-bottom: 4px; text-align: center;
}
.blue-bg { background: #3b82f6; }
.red-bg { background: #ef4444; }
.detail-player {
  display: flex; align-items: center; gap: 4px;
  padding: 2px 0; font-size: 11px;
}
.dp-name { flex: 1; font-weight: 600; color: #0f172a; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dp-rating { font-size: 10px; color: #94a3b8; font-variant-numeric: tabular-nums; }

.detail-btn {
  margin-top: 8px; width: 100%;
  padding: 5px 14px; border: 1px solid #e2e8f0; border-radius: 6px;
  background: #ffffff; color: #475569; font-size: 12px;
  font-weight: 500; cursor: pointer; transition: all 0.15s;
}
.detail-btn:hover { border-color: #6366f1; color: #6366f1; background: #f5f3ff; }

.empty-state { padding: 40px 0; text-align: center; }
</style>
