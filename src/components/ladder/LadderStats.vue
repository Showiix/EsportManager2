<template>
  <div class="ladder-stats">
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ totalPlayers }}</span>
        <span class="stat-label">参赛选手</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ totalMatches }}</span>
        <span class="stat-label">总场次</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value highlight">{{ avgRating }}</span>
        <span class="stat-label">平均分</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value income">{{ maxRating }}</span>
        <span class="stat-label">最高分</span>
      </div>
    </div>

    <div class="boards-row">
      <div class="board">
        <div class="board-header">
          <span class="board-title">MVP 榜</span>
          <el-tag type="warning" size="small">Top 10</el-tag>
        </div>
        <div v-if="mvpList.length === 0" class="board-empty">暂无数据</div>
        <div v-else class="board-list">
          <div v-for="(item, idx) in mvpList" :key="item.player_id" class="board-item">
            <span class="bi-rank" :class="rankClass(idx)">{{ idx + 1 }}</span>
            <span class="bi-name">{{ item.game_id }}</span>
            <el-tag :type="posType(item.position)" size="small" effect="light">{{ item.position }}</el-tag>
            <span class="bi-team">{{ item.team_name || '-' }}</span>
            <span class="bi-val mvp-val">{{ item.mvp_count }}</span>
          </div>
        </div>
      </div>

      <div class="board">
        <div class="board-header">
          <span class="board-title">影响力榜</span>
          <el-tag type="success" size="small">Top 10</el-tag>
        </div>
        <div v-if="influenceList.length === 0" class="board-empty">暂无数据</div>
        <div v-else class="board-list">
          <div v-for="(item, idx) in influenceList" :key="item.player_id" class="board-item">
            <span class="bi-rank" :class="rankClass(idx)">{{ idx + 1 }}</span>
            <span class="bi-name">{{ item.game_id }}</span>
            <el-tag :type="posType(item.position)" size="small" effect="light">{{ item.position }}</el-tag>
            <span class="bi-team">{{ item.team_name || '-' }}</span>
            <span class="bi-val inf-val">{{ item.avg_influence > 0 ? '+' : '' }}{{ item.avg_influence.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="boards-row">
      <div class="board">
        <div class="board-header">
          <span class="board-title">胜率榜</span>
          <el-tag type="primary" size="small">Top 10</el-tag>
        </div>
        <div v-if="winRateList.length === 0" class="board-empty">暂无数据</div>
        <div v-else class="board-list">
          <div v-for="(item, idx) in winRateList" :key="item.player_id" class="board-item">
            <span class="bi-rank" :class="rankClass(idx)">{{ idx + 1 }}</span>
            <span class="bi-name">{{ item.game_id }}</span>
            <el-tag :type="posType(item.position)" size="small" effect="light">{{ item.position }}</el-tag>
            <span class="bi-team">{{ item.team_name || '-' }}</span>
            <span class="bi-val wr-val">{{ item.win_rate.toFixed(1) }}%</span>
          </div>
        </div>
      </div>

      <div class="board">
        <div class="board-header">
          <span class="board-title">天梯分榜</span>
          <el-tag type="danger" size="small">Top 10</el-tag>
        </div>
        <div v-if="ratingList.length === 0" class="board-empty">暂无数据</div>
        <div v-else class="board-list">
          <div v-for="(item, idx) in ratingList" :key="item.player_id" class="board-item">
            <span class="bi-rank" :class="rankClass(idx)">{{ idx + 1 }}</span>
            <span class="bi-name">{{ item.game_id }}</span>
            <el-tag :type="posType(item.position)" size="small" effect="light">{{ item.position }}</el-tag>
            <span class="bi-team">{{ item.team_name || '-' }}</span>
            <span class="bi-val rating-val">{{ item.rating }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="boards-row">
      <div class="board full">
        <div class="board-header">
          <span class="board-title">位置分布</span>
        </div>
        <div class="pos-dist">
          <div v-for="pos in posStats" :key="pos.name" class="pos-item">
            <div class="pos-bar-wrap">
              <div class="pos-bar" :style="{ height: pos.pct + '%' }" :class="'pos-' + pos.name.toLowerCase()"></div>
            </div>
            <span class="pos-name">{{ pos.name }}</span>
            <span class="pos-count">{{ pos.count }}</span>
            <span class="pos-avg">avg {{ pos.avgRating }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { LadderRankingEntry } from '@/api/ladder'

const props = defineProps<{ rankings: LadderRankingEntry[]; loading: boolean }>()

const totalPlayers = computed(() => props.rankings.length)
const totalMatches = computed(() => {
  const total = props.rankings.reduce((s, r) => s + r.games_played, 0)
  return Math.floor(total / 10)
})
const avgRating = computed(() => {
  if (props.rankings.length === 0) return 0
  return Math.round(props.rankings.reduce((s, r) => s + r.rating, 0) / props.rankings.length)
})
const maxRating = computed(() => {
  if (props.rankings.length === 0) return 0
  return Math.max(...props.rankings.map(r => r.rating))
})

const mvpList = computed(() =>
  [...props.rankings].filter(r => r.mvp_count > 0).sort((a, b) => b.mvp_count - a.mvp_count).slice(0, 10)
)
const influenceList = computed(() =>
  [...props.rankings].filter(r => r.games_played > 0).sort((a, b) => b.avg_influence - a.avg_influence).slice(0, 10)
)
const winRateList = computed(() =>
  [...props.rankings].filter(r => r.games_played >= 3).sort((a, b) => b.win_rate - a.win_rate).slice(0, 10)
)
const ratingList = computed(() =>
  [...props.rankings].sort((a, b) => b.rating - a.rating).slice(0, 10)
)

const posStats = computed(() => {
  const positions = ['Top', 'Jug', 'Mid', 'Adc', 'Sup']
  const maxCount = Math.max(...positions.map(p => props.rankings.filter(r => r.position === p).length), 1)
  return positions.map(p => {
    const players = props.rankings.filter(r => r.position === p)
    const count = players.length
    const avgR = count > 0 ? Math.round(players.reduce((s, r) => s + r.rating, 0) / count) : 0
    return { name: p, count, avgRating: avgR, pct: (count / maxCount) * 100 }
  })
})

const posType = (p: string) => ({ Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info' }[p] || '') as '' | 'success' | 'warning' | 'info' | 'danger'
const rankClass = (idx: number) => idx === 0 ? 'rank-gold' : idx === 1 ? 'rank-silver' : idx === 2 ? 'rank-bronze' : ''
</script>

<style scoped>
.ladder-stats { padding: 16px 0; }

.stats-bar {
  display: flex; align-items: center;
  padding: 14px 24px; background: #ffffff;
  border: 1px solid #e2e8f0; border-radius: 10px;
  margin-bottom: 16px;
}
.stat-item {
  display: flex; align-items: baseline; gap: 6px;
  flex: 1; justify-content: center;
}
.stat-value {
  font-size: 20px; font-weight: 700; color: #0f172a;
  font-variant-numeric: tabular-nums;
}
.stat-value.highlight { color: #6366f1; }
.stat-value.income { color: #10b981; }
.stat-label { font-size: 12px; color: #94a3b8; font-weight: 500; }
.stat-divider { width: 1px; height: 24px; background: #e2e8f0; flex-shrink: 0; }

.boards-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px; }
.board {
  background: #fff; border: 1px solid #e2e8f0; border-radius: 10px; overflow: hidden;
}
.board.full { grid-column: 1 / -1; }
.board-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 14px; background: #f8fafc; border-bottom: 1px solid #f1f5f9;
}
.board-title { font-size: 13px; font-weight: 700; color: #0f172a; }
.board-empty { padding: 24px; text-align: center; font-size: 12px; color: #cbd5e1; }

.board-list { padding: 4px 0; }
.board-item {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 14px; font-size: 12px;
  border-bottom: 1px solid #f8fafc;
  transition: background 0.15s;
}
.board-item:last-child { border-bottom: none; }
.board-item:hover { background: #f8fafc; }

.bi-rank {
  width: 22px; height: 22px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 11px; font-weight: 700; flex-shrink: 0;
  background: #f1f5f9; color: #64748b;
}
.rank-gold { background: linear-gradient(135deg, #ffd700, #ffb347); color: #1a1a2e; }
.rank-silver { background: linear-gradient(135deg, #c0c0c0, #a8a8a8); color: #1a1a2e; }
.rank-bronze { background: linear-gradient(135deg, #cd7f32, #b87333); color: #fff; }

.bi-name { flex: 1; font-weight: 600; color: #0f172a; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.bi-team { font-size: 11px; color: #94a3b8; min-width: 60px; text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.bi-val { font-weight: 700; min-width: 40px; text-align: right; font-variant-numeric: tabular-nums; }
.mvp-val { color: #f59e0b; }
.inf-val { color: #10b981; }
.wr-val { color: #3b82f6; }
.rating-val { color: #ef4444; }

.pos-dist {
  display: flex; justify-content: space-around; align-items: flex-end;
  padding: 20px 24px 12px; height: 180px;
}
.pos-item { display: flex; flex-direction: column; align-items: center; gap: 4px; flex: 1; }
.pos-bar-wrap { width: 36px; height: 100px; display: flex; align-items: flex-end; justify-content: center; }
.pos-bar { width: 100%; border-radius: 4px 4px 0 0; transition: height 0.4s; min-height: 4px; }
.pos-top { background: #ef4444; }
.pos-jug { background: #f59e0b; }
.pos-mid { background: #6366f1; }
.pos-adc { background: #10b981; }
.pos-sup { background: #3b82f6; }
.pos-name { font-size: 12px; font-weight: 700; color: #0f172a; }
.pos-count { font-size: 11px; color: #475569; font-weight: 600; }
.pos-avg { font-size: 10px; color: #94a3b8; }
</style>
