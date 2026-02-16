<template>
  <div class="game-detail-view">
    <!-- 局数标题 + 战力/发挥对比（紧凑并排） -->
    <div class="game-header-row">
      <div class="game-title">
        <el-tag :type="game.isUpset ? 'warning' : 'info'" size="small">
          第{{ game.gameNumber }}局
        </el-tag>
        <el-tag v-if="game.isUpset" type="danger" size="small" effect="dark">
          爆冷
        </el-tag>
        <span class="winner-info">
          <span class="winner-label">获胜:</span>
          <span class="winner-name">{{ game.winnerName }}</span>
        </span>
      </div>
    </div>

    <!-- 最终战力 MR (可展开) -->
    <div class="mr-comparison-block" :class="{ 'is-open': mrBreakdownOpen }" @click="toggleMrBreakdown">
      <div class="comparison-label">
        <span class="team-label">{{ game.teamAName }}</span>
        <span class="vs-label mr-label">
          最终战力 MR
          <span class="expand-icon" :class="{ open: mrBreakdownOpen }">▼</span>
        </span>
        <span class="team-label">{{ game.teamBName }}</span>
      </div>
      
      <!-- Main MR Bar -->
      <div class="bar-row">
        <span class="bar-value team-a">{{ formatPower(finalPowerA) }}</span>
        <div class="progress-container">
          <div class="progress-bar team-a" :style="{ width: finalPowerAPercent + '%' }" :class="{ winner: finalWinnerA }"></div>
          <div class="progress-bar team-b" :style="{ width: finalPowerBPercent + '%' }" :class="{ winner: finalWinnerB }"></div>
        </div>
        <span class="bar-value team-b">{{ formatPower(finalPowerB) }}</span>
      </div>
      
      <div class="bar-diff" :class="finalDiffClass">
        {{ formatDiff(finalPowerDiff) }}
        <span v-if="!mrBreakdownOpen" class="click-hint"> (点击查看详情)</span>
      </div>

      <!-- Breakdown Table (Expanded) -->
      <div v-if="mrBreakdownOpen" class="mr-breakdown-table" @click.stop>
        <div class="breakdown-divider"></div>
        
        <!-- Base Power -->
        <div class="mr-row">
          <span class="mr-val team-a">{{ formatPower(game.teamABasePower) }}</span>
          <span class="mr-label-item">基础战力</span>
          <span class="mr-val team-b">{{ formatPower(game.teamBBasePower) }}</span>
        </div>

        <!-- BP Bonus -->
        <div class="mr-row">
          <span class="mr-val" :class="getBonusClass(game.teamABpBonus)">{{ formatBonusVal(game.teamABpBonus) }}</span>
          <span class="mr-label-item">BP加成</span>
          <span class="mr-val" :class="getBonusClass(game.teamBBpBonus)">{{ formatBonusVal(game.teamBBpBonus) }}</span>
        </div>

        <!-- Version Bonus -->
        <div class="mr-row">
          <span class="mr-val" :class="getBonusClass(game.teamAVersionBonus)">{{ formatBonusVal(game.teamAVersionBonus) }}</span>
          <span class="mr-label-item">版本适配</span>
          <span class="mr-val" :class="getBonusClass(game.teamBVersionBonus)">{{ formatBonusVal(game.teamBVersionBonus) }}</span>
        </div>

        <!-- Synergy Bonus -->
        <div class="mr-row">
          <span class="mr-val" :class="getBonusClass(game.teamASynergyBonus)">{{ formatBonusVal(game.teamASynergyBonus) }}</span>
          <span class="mr-label-item">协同加成</span>
          <span class="mr-val" :class="getBonusClass(game.teamBSynergyBonus)">{{ formatBonusVal(game.teamBSynergyBonus) }}</span>
        </div>

        <!-- Meta Multiplier (Implicit) -->
        <div class="mr-row meta-mult-row">
          <span class="mr-val meta-text">{{ formatMetaMult(game.teamABasePower, game.teamABpBonus, game.teamAVersionBonus, game.teamASynergyBonus, finalPowerA) }}</span>
          <span class="mr-label-item">META加权</span>
          <span class="mr-val meta-text">{{ formatMetaMult(game.teamBBasePower, game.teamBBpBonus, game.teamBVersionBonus, game.teamBSynergyBonus, finalPowerB) }}</span>
        </div>

        <div class="breakdown-divider"></div>

        <!-- Final MR (Repeated for clarity in table) -->
        <div class="mr-row final-row">
          <span class="mr-val team-a">{{ formatPower(finalPowerA) }}</span>
          <span class="mr-label-item">最终 MR</span>
          <span class="mr-val team-b">{{ formatPower(finalPowerB) }}</span>
        </div>
      </div>
    </div>

    <!-- 发挥对比 (单独一行) -->
    <div class="comparison-row performance-only">
      <div class="comparison-block">
        <div class="comparison-label">
          <span class="team-label">{{ game.teamAName }}</span>
          <span class="vs-label">发挥</span>
          <span class="team-label">{{ game.teamBName }}</span>
        </div>
        <div class="bar-row">
          <span class="bar-value team-a">{{ formatPower(game.teamAPerformance) }}</span>
          <div class="progress-container">
            <div class="progress-bar team-a" :style="{ width: teamAPerfPercent + '%' }" :class="{ winner: game.winnerId === game.teamAId }"></div>
            <div class="progress-bar team-b" :style="{ width: teamBPerfPercent + '%' }" :class="{ winner: game.winnerId === game.teamBId }"></div>
          </div>
          <span class="bar-value team-b">{{ formatPower(game.teamBPerformance) }}</span>
        </div>
        <div class="bar-diff" :class="perfDiffClass">{{ formatDiff(game.performanceDifference) }}</div>
      </div>
    </div>

    <!-- 单局最佳 -->
    <div v-if="gameMvp" class="mvp-card">
      <span class="mvp-trophy">本局最佳</span>
      <span class="mvp-player-name">{{ gameMvp.playerName }}</span>
      <span class="mvp-team">{{ gameMvp.teamName }}</span>
      <span class="mvp-position">{{ getPositionName(gameMvp.position) }}</span>
      <span class="mvp-stat">
        <span class="mvp-stat-label">发挥</span>
        <span class="mvp-stat-value">{{ gameMvp.actualAbility }}</span>
      </span>
      <span class="mvp-stat">
        <span class="mvp-stat-label">影响力</span>
        <span class="mvp-stat-value" :class="getImpactClass(gameMvp.impactScore)">{{ formatImpact(gameMvp.impactScore) }}</span>
      </span>
    </div>

    <!-- 选手表现表格 -->
    <div class="players-table">
      <div class="table-header">
        <span class="col-position">位置</span>
        <span class="col-name">选手</span>
        <span class="col-traits">特性</span>
        <span class="col-base">基础</span>
        <span class="col-condition">状态</span>
        <span class="col-noise">波动</span>
        <span class="col-actual">发挥</span>
        <span class="col-impact">影响力</span>
      </div>

      <!-- A队选手 -->
      <div class="team-section">
        <div class="team-section-header team-a-header">
          <span>{{ game.teamAName }}</span>
          <span class="team-power">战力: {{ formatPower(game.teamAPower) }}</span>
        </div>
        <div
          v-for="player in sortedTeamAPlayers"
          :key="player.playerId"
          class="player-row"
          :class="{ 'high-impact': player.impactScore > 3, 'low-impact': player.impactScore < -3 }"
        >
          <span class="col-position">{{ getPositionName(player.position) }}</span>
          <span class="col-name">{{ player.playerName }}</span>
          <span class="col-traits">
            <template v-if="player.activatedTraits && player.activatedTraits.length > 0">
              <el-tooltip
                v-for="trait in player.activatedTraits"
                :key="trait.type"
                :content="`${trait.name}: ${trait.effect}`"
                placement="top"
              >
                <el-tag
                  :type="trait.isPositive ? 'success' : 'danger'"
                  size="small"
                  class="trait-tag"
                >
                  {{ trait.name }}
                </el-tag>
              </el-tooltip>
            </template>
            <template v-else-if="player.traits && player.traits.length > 0">
              <el-tooltip
                v-for="trait in player.traits"
                :key="trait"
                :content="`${getTraitName(trait)}: 未激活 - ${getTraitDescription(trait)}`"
                placement="top"
              >
                <el-tag type="info" size="small" class="trait-tag inactive">
                  {{ getTraitName(trait) }}
                </el-tag>
              </el-tooltip>
            </template>
            <span v-else class="no-trait">-</span>
          </span>
          <span class="col-base">
            <span class="base-value">{{ player.baseAbility }}</span>
            <div class="mini-bar"><div class="mini-bar-fill base-fill" :style="{ width: player.baseAbility + '%' }"></div></div>
          </span>
          <span class="col-condition" :class="getConditionClass(player.conditionBonus)">
            {{ formatBonus(player.conditionBonus) }}
          </span>
          <span class="col-noise" :class="getNoiseClass(player.stabilityNoise)">
            {{ formatBonus(player.stabilityNoise) }}
          </span>
          <span class="col-actual" :class="getActualClass(player.actualAbility, player.baseAbility)">
            {{ player.actualAbility }}
          </span>
          <span class="col-impact">
            <div class="impact-bar-wrapper">
              <div
                v-if="player.impactScore >= 0"
                class="impact-bar positive-bar"
                :style="{ width: Math.min(Math.abs(player.impactScore) * 8, 100) + '%' }"
              ></div>
              <div
                v-else
                class="impact-bar negative-bar"
                :style="{ width: Math.min(Math.abs(player.impactScore) * 8, 100) + '%' }"
              ></div>
            </div>
            <span class="impact-value" :class="getImpactClass(player.impactScore)">{{ formatImpact(player.impactScore) }}</span>
          </span>
        </div>
      </div>

      <!-- B队选手 -->
      <div class="team-section">
        <div class="team-section-header team-b-header">
          <span>{{ game.teamBName }}</span>
          <span class="team-power">战力: {{ formatPower(game.teamBPower) }}</span>
        </div>
        <div
          v-for="player in sortedTeamBPlayers"
          :key="player.playerId"
          class="player-row"
          :class="{ 'high-impact': player.impactScore > 3, 'low-impact': player.impactScore < -3 }"
        >
          <span class="col-position">{{ getPositionName(player.position) }}</span>
          <span class="col-name">{{ player.playerName }}</span>
          <span class="col-traits">
            <template v-if="player.activatedTraits && player.activatedTraits.length > 0">
              <el-tooltip
                v-for="trait in player.activatedTraits"
                :key="trait.type"
                :content="`${trait.name}: ${trait.effect}`"
                placement="top"
              >
                <el-tag
                  :type="trait.isPositive ? 'success' : 'danger'"
                  size="small"
                  class="trait-tag"
                >
                  {{ trait.name }}
                </el-tag>
              </el-tooltip>
            </template>
            <template v-else-if="player.traits && player.traits.length > 0">
              <el-tooltip
                v-for="trait in player.traits"
                :key="trait"
                :content="`${getTraitName(trait)}: 未激活 - ${getTraitDescription(trait)}`"
                placement="top"
              >
                <el-tag type="info" size="small" class="trait-tag inactive">
                  {{ getTraitName(trait) }}
                </el-tag>
              </el-tooltip>
            </template>
            <span v-else class="no-trait">-</span>
          </span>
          <span class="col-base">
            <span class="base-value">{{ player.baseAbility }}</span>
            <div class="mini-bar"><div class="mini-bar-fill base-fill" :style="{ width: player.baseAbility + '%' }"></div></div>
          </span>
          <span class="col-condition" :class="getConditionClass(player.conditionBonus)">
            {{ formatBonus(player.conditionBonus) }}
          </span>
          <span class="col-noise" :class="getNoiseClass(player.stabilityNoise)">
            {{ formatBonus(player.stabilityNoise) }}
          </span>
          <span class="col-actual" :class="getActualClass(player.actualAbility, player.baseAbility)">
            {{ player.actualAbility }}
          </span>
          <span class="col-impact">
            <div class="impact-bar-wrapper">
              <div
                v-if="player.impactScore >= 0"
                class="impact-bar positive-bar"
                :style="{ width: Math.min(Math.abs(player.impactScore) * 8, 100) + '%' }"
              ></div>
              <div
                v-else
                class="impact-bar negative-bar"
                :style="{ width: Math.min(Math.abs(player.impactScore) * 8, 100) + '%' }"
              ></div>
            </div>
            <span class="impact-value" :class="getImpactClass(player.impactScore)">{{ formatImpact(player.impactScore) }}</span>
          </span>
        </div>
      </div>
    </div>

    <!-- 发挥分解折叠面板 -->
    <div class="breakdown-panel">
      <button class="breakdown-toggle" @click="breakdownOpen = !breakdownOpen">
        <span class="toggle-arrow" :class="{ open: breakdownOpen }">&#9654;</span>
        <span>发挥分解</span>
      </button>
      <div v-if="breakdownOpen" class="breakdown-content">
        <div class="breakdown-grid">
          <!-- Team A -->
          <div class="breakdown-team">
            <div class="breakdown-team-name team-a-accent">{{ game.teamAName }}</div>
            <div class="breakdown-header-row">
              <span class="bd-col-name">选手</span>
              <span class="bd-col-base">基础</span>
              <span class="bd-col-cond">状态</span>
              <span class="bd-col-noise">波动</span>
              <span class="bd-col-arrow"></span>
              <span class="bd-col-result">发挥</span>
            </div>
            <div v-for="mp in matchedPlayers" :key="'a-' + mp.positionKey" class="breakdown-row">
              <span class="bd-col-name">{{ mp.playerA?.playerName || '-' }}</span>
              <span class="bd-col-base">
                <div class="bd-bar"><div class="bd-bar-fill" :style="{ width: (mp.playerA?.baseAbility || 0) + '%' }"></div></div>
                <span>{{ mp.playerA?.baseAbility || '-' }}</span>
              </span>
              <span class="bd-col-cond" :class="getConditionClass(mp.playerA?.conditionBonus || 0)">
                {{ formatBonus(mp.playerA?.conditionBonus) }}
              </span>
              <span class="bd-col-noise bd-noise-val">
                {{ formatBonus(mp.playerA?.stabilityNoise) }}
              </span>
              <span class="bd-col-arrow">→</span>
              <span class="bd-col-result bd-result-val">{{ mp.playerA?.actualAbility || '-' }}</span>
            </div>
          </div>
          <!-- Team B -->
          <div class="breakdown-team">
            <div class="breakdown-team-name team-b-accent">{{ game.teamBName }}</div>
            <div class="breakdown-header-row">
              <span class="bd-col-name">选手</span>
              <span class="bd-col-base">基础</span>
              <span class="bd-col-cond">状态</span>
              <span class="bd-col-noise">波动</span>
              <span class="bd-col-arrow"></span>
              <span class="bd-col-result">发挥</span>
            </div>
            <div v-for="mp in matchedPlayers" :key="'b-' + mp.positionKey" class="breakdown-row">
              <span class="bd-col-name">{{ mp.playerB?.playerName || '-' }}</span>
              <span class="bd-col-base">
                <div class="bd-bar"><div class="bd-bar-fill" :style="{ width: (mp.playerB?.baseAbility || 0) + '%' }"></div></div>
                <span>{{ mp.playerB?.baseAbility || '-' }}</span>
              </span>
              <span class="bd-col-cond" :class="getConditionClass(mp.playerB?.conditionBonus || 0)">
                {{ formatBonus(mp.playerB?.conditionBonus) }}
              </span>
              <span class="bd-col-noise bd-noise-val">
                {{ formatBonus(mp.playerB?.stabilityNoise) }}
              </span>
              <span class="bd-col-arrow">→</span>
              <span class="bd-col-result bd-result-val">{{ mp.playerB?.actualAbility || '-' }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 对位差折叠面板 -->
    <div class="breakdown-panel matchup-panel">
      <button class="breakdown-toggle" @click="matchupOpen = !matchupOpen">
        <span class="toggle-arrow" :class="{ open: matchupOpen }">&#9654;</span>
        <span>对位差</span>
      </button>
      <div v-if="matchupOpen" class="breakdown-content">
        <div class="matchup-header">
          <span class="matchup-team-name team-a-accent">{{ game.teamAName }}</span>
          <span class="matchup-vs">VS</span>
          <span class="matchup-team-name team-b-accent">{{ game.teamBName }}</span>
        </div>
        <div v-for="mp in matchedPlayers" :key="'mu-' + mp.positionKey" class="matchup-row" :class="{ 'mu-key-row': mp.positionKey === keyMatchupPos }">
          <span class="mu-position">
            {{ getPositionName(mp.positionKey as PlayerPosition) }}
            <span v-if="mp.positionKey === keyMatchupPos" class="mu-key-badge">KEY</span>
          </span>
          <div class="mu-player-a">
            <span class="mu-name">{{ mp.playerA?.playerName || '-' }}</span>
            <span class="mu-value" :class="{ 'mu-winner': getMuDiff(mp) > 0 }">{{ mp.playerA?.actualAbility || '-' }}</span>
          </div>
          <div class="mu-bar-container">
            <div class="mu-bar-side mu-bar-a" :style="{ width: getMuBarWidth(mp, 'a') + '%' }">
              <div class="mu-bar-fill mu-fill-a" :class="{ 'mu-bar-winner': getMuDiff(mp) > 0 }"></div>
            </div>
            <span class="mu-diff-label" :class="getMuDiffClass(mp)">{{ formatMuDiff(mp) }}</span>
            <div class="mu-bar-side mu-bar-b" :style="{ width: getMuBarWidth(mp, 'b') + '%' }">
              <div class="mu-bar-fill mu-fill-b" :class="{ 'mu-bar-winner': getMuDiff(mp) < 0 }"></div>
            </div>
          </div>
          <div class="mu-player-b">
            <span class="mu-value" :class="{ 'mu-winner': getMuDiff(mp) < 0 }">{{ mp.playerB?.actualAbility || '-' }}</span>
            <span class="mu-name">{{ mp.playerB?.playerName || '-' }}</span>
          </div>
        </div>
        <div class="matchup-summary">
          <span class="mu-summary-item">
            <span class="mu-summary-label">A队优势位:</span>
            <span class="mu-summary-value team-a-accent">{{ matchupSummary.aWins }}</span>
          </span>
          <span class="mu-summary-item">
            <span class="mu-summary-label">B队优势位:</span>
            <span class="mu-summary-value team-b-accent">{{ matchupSummary.bWins }}</span>
          </span>
          <span class="mu-summary-item">
            <span class="mu-summary-label">总对位差:</span>
            <span class="mu-summary-value" :class="matchupSummary.totalDiff > 0 ? 'team-a-accent' : matchupSummary.totalDiff < 0 ? 'team-b-accent' : ''">
              {{ matchupSummary.totalDiff > 0 ? '+' : '' }}{{ matchupSummary.totalDiff.toFixed(1) }}
            </span>
          </span>
        </div>
      </div>
    </div>

    <!-- 图例说明 -->
    <div class="legend">
      <div class="legend-item">
        <span class="legend-color positive"></span>
        <span>正向影响</span>
      </div>
      <div class="legend-item">
        <span class="legend-color negative"></span>
        <span>负向影响</span>
      </div>
      <div class="legend-item">
        <span class="legend-color high-impact"></span>
        <span>关键发挥 (|影响| > 3)</span>
      </div>
      <div class="legend-item">
        <el-tag type="success" size="small">特性</el-tag>
        <span>激活特性</span>
      </div>
      <div class="legend-item">
        <el-tag type="info" size="small" class="inactive">特性</el-tag>
        <span>未激活特性</span>
      </div>
    </div>

    <!-- BP 面板 -->
    <div v-if="draftData" class="bp-panel">
      <button class="breakdown-toggle" @click="bpOpen = !bpOpen">
        <span class="toggle-arrow" :class="{ open: bpOpen }">&#9654;</span>
        <span>Ban/Pick</span>
      </button>
      <div v-if="bpOpen" class="breakdown-content">
        <!-- Ban 分两队显示 -->
        <div class="bp-ban-row">
          <div class="bp-ban-col">
            <div class="bp-team-label team-a-accent">{{ game.teamAName }} Ban</div>
            <div class="bp-list">
              <el-tag
                v-for="ban in homeBans"
                :key="'ban-a-' + ban.champion_id"
                type="danger"
                size="small"
                effect="dark"
                class="bp-tag"
              >
                {{ getChampionName(ban.champion_id) }}
              </el-tag>
            </div>
          </div>
          <div class="bp-ban-col">
            <div class="bp-team-label team-b-accent">{{ game.teamBName }} Ban</div>
            <div class="bp-list">
              <el-tag
                v-for="ban in awayBans"
                :key="'ban-b-' + ban.champion_id"
                type="danger"
                size="small"
                effect="dark"
                class="bp-tag"
              >
                {{ getChampionName(ban.champion_id) }}
              </el-tag>
            </div>
          </div>
        </div>
        <!-- Pick 分两队显示，带选手名和熟练度 -->
        <div class="bp-teams-row">
          <div class="bp-team-col">
            <div class="bp-team-label team-a-accent">{{ game.teamAName }} Pick</div>
            <div class="bp-pick-list">
              <div v-for="pick in parsedHomePicks" :key="'hp-' + pick.champion_id" class="bp-pick-item">
                <span class="bp-pick-pos">{{ pick.position }}</span>
                <span class="bp-player-name">{{ getPlayerName(pick.player_id) }}</span>
                <el-tag size="small" effect="plain">{{ getChampionName(pick.champion_id) }}</el-tag>
                <span v-if="pick.mastery_tier" class="bp-mastery" :class="'mastery-' + pick.mastery_tier">{{ pick.mastery_tier }}</span>
                <span v-if="getPlayerBpModifier(pick.player_id) !== undefined" class="bp-modifier" :class="getPlayerBpModifier(pick.player_id)! >= 0 ? 'positive' : 'negative'">
                  {{ getPlayerBpModifier(pick.player_id)! >= 0 ? '+' : '' }}{{ getPlayerBpModifier(pick.player_id)!.toFixed(1) }}%
                </span>
              </div>
            </div>
            <div v-if="draftData.home_comp" class="bp-comp">
              体系: <el-tag type="warning" size="small">{{ compDisplayName(draftData.home_comp) }}</el-tag>
            </div>
          </div>
          <div class="bp-team-col">
            <div class="bp-team-label team-b-accent">{{ game.teamBName }} Pick</div>
            <div class="bp-pick-list">
              <div v-for="pick in parsedAwayPicks" :key="'ap-' + pick.champion_id" class="bp-pick-item">
                <span class="bp-pick-pos">{{ pick.position }}</span>
                <span class="bp-player-name">{{ getPlayerName(pick.player_id) }}</span>
                <el-tag size="small" effect="plain">{{ getChampionName(pick.champion_id) }}</el-tag>
                <span v-if="pick.mastery_tier" class="bp-mastery" :class="'mastery-' + pick.mastery_tier">{{ pick.mastery_tier }}</span>
                <span v-if="getPlayerBpModifier(pick.player_id) !== undefined" class="bp-modifier" :class="getPlayerBpModifier(pick.player_id)! >= 0 ? 'positive' : 'negative'">
                  {{ getPlayerBpModifier(pick.player_id)! >= 0 ? '+' : '' }}{{ getPlayerBpModifier(pick.player_id)!.toFixed(1) }}%
                </span>
              </div>
            </div>
            <div v-if="draftData.away_comp" class="bp-comp">
              体系: <el-tag type="warning" size="small">{{ compDisplayName(draftData.away_comp) }}</el-tag>
            </div>
          </div>
        </div>

        <div v-if="parsedNarrative" class="bp-narrative">
          <div class="bp-narrative-title">💭 BP博弈心理</div>
          <div class="bp-narrative-row">
            <div class="bp-narrative-col">
              <div class="bp-team-label team-a-accent">{{ game.teamAName }}</div>
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
              <div class="bp-team-label team-b-accent">{{ game.teamBName }}</div>
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
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import type { GameDetail } from '@/types/matchDetail'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES, getTraitDescription, getTraitName } from '@/types/player'
import { getDraftResult, getChampionList } from '@/api/tauri'
import type { DraftResultInfo, ChampionInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'

interface Props {
  game: GameDetail
  matchId?: number | string
}

const props = defineProps<Props>()

const mrBreakdownOpen = ref(false)
const toggleMrBreakdown = () => { mrBreakdownOpen.value = !mrBreakdownOpen.value }

const breakdownOpen = ref(false)
const matchupOpen = ref(false)
const bpOpen = ref(false)

const gameStore = useGameStore()
const draftData = ref<DraftResultInfo | null>(null)
const championMap = ref<Map<number, ChampionInfo>>(new Map())

interface StoredBanEntry { champion_id: number; team_side: string; ban_phase: number }
interface StoredPickEntry { champion_id: number; player_id: number; position: string; mastery_tier: string }
interface NarrativeEntry { phase: string; message: string }
interface DraftNarrativeData {
  home_entries: NarrativeEntry[]
  away_entries: NarrativeEntry[]
}

const parsedBans = computed<StoredBanEntry[]>(() => {
  if (!draftData.value) return []
  try { return JSON.parse(draftData.value.bans_json) } catch { return [] }
})

const homeBans = computed<StoredBanEntry[]>(() => {
  return parsedBans.value.filter(b => b.team_side === 'Home')
})

const awayBans = computed<StoredBanEntry[]>(() => {
  return parsedBans.value.filter(b => b.team_side === 'Away')
})

const BP_POS_ORDER: Record<string, number> = { TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4, Top: 0, Jug: 1, Mid: 2, Adc: 3, Sup: 4 }

const parsedHomePicks = computed<StoredPickEntry[]>(() => {
  if (!draftData.value) return []
  try {
    const picks: StoredPickEntry[] = JSON.parse(draftData.value.home_picks_json)
    return picks.sort((a, b) => (BP_POS_ORDER[a.position] ?? 99) - (BP_POS_ORDER[b.position] ?? 99))
  } catch { return [] }
})

const parsedAwayPicks = computed<StoredPickEntry[]>(() => {
  if (!draftData.value) return []
  try {
    const picks: StoredPickEntry[] = JSON.parse(draftData.value.away_picks_json)
    return picks.sort((a, b) => (BP_POS_ORDER[a.position] ?? 99) - (BP_POS_ORDER[b.position] ?? 99))
  } catch { return [] }
})

const parsedNarrative = computed<DraftNarrativeData | null>(() => {
  if (!draftData.value?.draft_narrative_json) return null
  try {
    return JSON.parse(draftData.value.draft_narrative_json)
  } catch {
    return null
  }
})

const COMP_NAMES: Record<string, string> = {
  Rush: '速推', PickOff: '抓单', AllIn: '莽夫', MidJungle: '中野联动', TopJungle: '上野联动',
  Protect: '保C', Fortress: '铁桶阵', UtilityComp: '功能流', Stall: '龟缩', BotLane: '下路统治',
  Teamfight: '团战', Dive: '开团', Skirmish: '小规模团战', DualCarry: '双C', Flex: '全能',
  Splitpush: '分推', SideLane: '4-1分带', Control: '运营', TripleThreat: '三线施压', LateGame: '后期发育',
}

const compDisplayName = (type: string) => COMP_NAMES[type] || type

const phaseLabel = (phase: string) => {
  const labels: Record<string, string> = {
    plan: '🎯 规划',
    ban_phase1: '🚫 Ban①',
    ban_phase2: '🚫 Ban②',
    pick: '✅ Pick',
    switch: '🔄 切换',
  }
  return labels[phase] || phase
}

const getChampionName = (id: number): string => {
  const champ = championMap.value.get(id)
  return champ ? champ.name_cn : `#${id}`
}

const getPlayerBpModifier = (playerId: number): number | undefined => {
  const allPlayers = [...(props.game.teamAPlayers || []), ...(props.game.teamBPlayers || [])]
  const player = allPlayers.find(p => String(p.playerId) === String(playerId))
  return player?.bpModifier
}

const getPlayerName = (playerId: number): string => {
  const allPlayers = [...(props.game.teamAPlayers || []), ...(props.game.teamBPlayers || [])]
  const player = allPlayers.find(p => String(p.playerId) === String(playerId))
  return player?.playerName || ''
}

const loadBpData = async () => {
  const saveId = gameStore.currentSave?.id
  const mid = props.matchId
  if (!saveId || !mid) return

  const numericMatchId = typeof mid === 'number'
    ? mid
    : parseInt(String(mid).replace(/\D/g, ''))
  if (isNaN(numericMatchId) || numericMatchId <= 0) return

  try {
    if (championMap.value.size === 0) {
      const list = await getChampionList()
      const map = new Map<number, ChampionInfo>()
      for (const c of list) map.set(c.id, c)
      championMap.value = map
    }

    const result = await getDraftResult(saveId, numericMatchId, props.game.gameNumber)
    draftData.value = result
  } catch {
    draftData.value = null
  }
}

watch(() => [props.matchId, props.game.gameNumber], () => { loadBpData() }, { immediate: true })
onMounted(() => { loadBpData() })

// 监听弹窗可见性变化，重新加载数据
const reloadKey = ref(0)
defineExpose({
  reload: () => {
    reloadKey.value++
    loadBpData()
  }
})

// 位置排序顺序
const POSITION_ORDER: Record<string, number> = { TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4 }
const POSITION_KEYS = ['TOP', 'JUG', 'MID', 'ADC', 'SUP'] as const

const sortedTeamAPlayers = computed(() =>
  [...(props.game.teamAPlayers || [])].sort(
    (a, b) => (POSITION_ORDER[a.position] ?? 99) - (POSITION_ORDER[b.position] ?? 99)
  )
)

const sortedTeamBPlayers = computed(() =>
  [...(props.game.teamBPlayers || [])].sort(
    (a, b) => (POSITION_ORDER[a.position] ?? 99) - (POSITION_ORDER[b.position] ?? 99)
  )
)

// 按位置配对两队选手（用于折叠面板对比）
const matchedPlayers = computed(() => {
  return POSITION_KEYS.map(pos => ({
    positionKey: pos,
    playerA: sortedTeamAPlayers.value.find(p => p.position === pos) || null,
    playerB: sortedTeamBPlayers.value.find(p => p.position === pos) || null,
  }))
})

// 计算战力百分比
// 使用 metaPower 作为最终战力，如果没有则使用 teamPower
const finalPowerA = computed(() => props.game.teamAMetaPower ?? props.game.teamAPower)
const finalPowerB = computed(() => props.game.teamBMetaPower ?? props.game.teamBPower)
const finalPowerDiff = computed(() => finalPowerA.value - finalPowerB.value)
const finalWinnerA = computed(() => props.game.winnerId === props.game.teamAId)
const finalWinnerB = computed(() => props.game.winnerId === props.game.teamBId)

const totalPower = computed(() => finalPowerA.value + finalPowerB.value)
const finalPowerAPercent = computed(() =>
  totalPower.value > 0 ? (finalPowerA.value / totalPower.value) * 100 : 50
)
const finalPowerBPercent = computed(() =>
  totalPower.value > 0 ? (finalPowerB.value / totalPower.value) * 100 : 50
)

const finalDiffClass = computed(() => {
  if (finalPowerDiff.value > 0) return 'positive'
  if (finalPowerDiff.value < 0) return 'negative'
  return ''
})

// 辅助函数
const formatMetaMult = (base: number | undefined, bp: number | undefined, ver: number | undefined, syn: number | undefined, final: number) => {
  const sum = (base || 0) + (bp || 0) + (ver || 0) + (syn || 0)
  if (sum === 0) return '-'
  
  // 1. 如果 base 为 undefined，说明是旧数据，无法计算倍率，直接返回 "-"
  if (base === undefined) return '-'

  // 2. 正常计算 sum，如果 final ≈ sum，说明没有 meta multiplier
  if (Math.abs(final - sum) < 0.05) return '×1.00'
  
  // 3. 计算倍率
  const mult = final / sum
  return `×${mult.toFixed(2)}`
}


const formatBonusVal = (val: number | undefined) => {
  if (val === undefined || val === null) return '-'
  if (val > 0) return `+${val.toFixed(2)}`
  return val.toFixed(2)
}

const getBonusClass = (val: number | undefined) => {
  if (val === undefined || val === null || val === 0) return 'neutral'
  return val > 0 ? 'positive' : 'negative'
}

// 发挥对比百分比
const totalPerf = computed(() => props.game.teamAPerformance + props.game.teamBPerformance)
const teamAPerfPercent = computed(() =>
  totalPerf.value > 0 ? (props.game.teamAPerformance / totalPerf.value) * 100 : 50
)
const teamBPerfPercent = computed(() =>
  totalPerf.value > 0 ? (props.game.teamBPerformance / totalPerf.value) * 100 : 50
)

const perfDiffClass = computed(() => {
  const diff = props.game.performanceDifference
  if (diff > 0) return 'positive'
  if (diff < 0) return 'negative'
  return ''
})

// 获取位置名称
const getPositionName = (position: PlayerPosition): string => {
  return POSITION_NAMES[position] || position
}

// 格式化加成数值
const formatBonus = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(1)}`
  return value.toFixed(1)
}

// 格式化影响力分数
const formatImpact = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(2)}`
  return value.toFixed(2)
}

// 格式化战力/发挥值
const formatPower = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  return value.toFixed(2)
}

// 格式化差值
const formatDiff = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(2)} (A队优势)`
  if (value < 0) return `${value.toFixed(2)} (B队优势)`
  return '0 (势均力敌)'
}

// 状态加成样式
const getConditionClass = (value: number): string => {
  if (value > 3) return 'very-positive'
  if (value > 0) return 'positive'
  if (value < -3) return 'very-negative'
  if (value < 0) return 'negative'
  return ''
}

// 波动样式
const getNoiseClass = (value: number): string => {
  if (value > 2) return 'positive'
  if (value < -2) return 'negative'
  return ''
}

// 影响力样式
const getImpactClass = (value: number): string => {
  if (value > 5) return 'very-positive'
  if (value > 0) return 'positive'
  if (value < -5) return 'very-negative'
  if (value < 0) return 'negative'
  return ''
}

// 发挥值与基础值对比样式
const getActualClass = (actual: number, base: number): string => {
  if (actual > base) return 'actual-above'
  if (actual < base) return 'actual-below'
  return ''
}

// --- 单局 MVP ---

const gameMvp = computed(() => {
  const allPlayers = [
    ...(props.game.teamAPlayers || []).map(p => ({ ...p, teamName: props.game.teamAName, teamSide: 'a' as const })),
    ...(props.game.teamBPlayers || []).map(p => ({ ...p, teamName: props.game.teamBName, teamSide: 'b' as const })),
  ]
  if (allPlayers.length === 0) return null
  return allPlayers.reduce((best, p) => (p.impactScore > best.impactScore ? p : best), allPlayers[0])
})

// --- 对位差相关 ---

type MatchedPlayer = { positionKey: string; playerA: any; playerB: any }

const getMuDiff = (mp: MatchedPlayer): number => {
  const a = mp.playerA?.actualAbility || 0
  const b = mp.playerB?.actualAbility || 0
  return a - b
}

const getMuBarWidth = (mp: MatchedPlayer, side: 'a' | 'b'): number => {
  const diff = Math.abs(getMuDiff(mp))
  const maxDiff = 30
  const pct = Math.min(diff / maxDiff, 1) * 100
  if (side === 'a') return getMuDiff(mp) > 0 ? pct : 0
  return getMuDiff(mp) < 0 ? pct : 0
}

const formatMuDiff = (mp: MatchedPlayer): string => {
  const diff = getMuDiff(mp)
  if (diff === 0) return '0'
  return diff > 0 ? `+${diff.toFixed(1)}` : diff.toFixed(1)
}

const getMuDiffClass = (mp: MatchedPlayer): string => {
  const diff = getMuDiff(mp)
  if (diff > 0) return 'mu-diff-a'
  if (diff < 0) return 'mu-diff-b'
  return ''
}

const matchupSummary = computed(() => {
  let aWins = 0
  let bWins = 0
  let totalDiff = 0
  for (const mp of matchedPlayers.value) {
    const diff = getMuDiff(mp)
    totalDiff += diff
    if (diff > 0) aWins++
    else if (diff < 0) bWins++
  }
  return { aWins, bWins, totalDiff }
})

const keyMatchupPos = computed(() => {
  let maxAbsDiff = 0
  let keyPos = ''
  for (const mp of matchedPlayers.value) {
    const absDiff = Math.abs(getMuDiff(mp))
    if (absDiff > maxAbsDiff) {
      maxAbsDiff = absDiff
      keyPos = mp.positionKey
    }
  }
  return keyPos
})
</script>

<style scoped>
.game-detail-view {
  padding: 24px;
}

/* 局数头部 */
.game-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.game-title {
  display: flex;
  gap: 8px;
  align-items: center;
}

.winner-info {
  margin-left: 8px;
  font-size: 13px;
}

.winner-label {
  color: #86909c;
}

.winner-name {
  color: #10b981;
  font-weight: 700;
  margin-left: 4px;
}

/* 战力/发挥对比并排 */
.comparison-row {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  margin-bottom: 12px;
}

.performance-only {
  /* 只有发挥条时，也可以用 grid */
  display: block;
}

.mr-comparison-block {
  padding: 14px 16px;
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.05) 0%, rgba(59, 130, 246, 0.05) 100%);
  border-radius: 12px;
  border: 1px solid rgba(139, 92, 246, 0.15);
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: 12px;
  position: relative;
}

.mr-comparison-block:hover {
  border-color: rgba(139, 92, 246, 0.3);
  box-shadow: 0 4px 12px rgba(139, 92, 246, 0.05);
}

.mr-comparison-block.is-open {
  border-color: rgba(139, 92, 246, 0.3);
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.08) 0%, rgba(59, 130, 246, 0.08) 100%);
}

.mr-label {
  color: #8b5cf6;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 6px;
}

.expand-icon {
  font-size: 10px;
  transition: transform 0.2s;
  opacity: 0.6;
}

.expand-icon.open {
  transform: rotate(180deg);
}

.click-hint {
  font-size: 10px;
  color: #a0aec0;
  font-weight: 400;
  margin-left: 4px;
}

/* MR Breakdown Table */
.mr-breakdown-table {
  margin-top: 12px;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.breakdown-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.06);
  margin: 8px 0;
}

.mr-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 28px;
  font-size: 12px;
}

.mr-label-item {
  color: #64748b;
  font-weight: 500;
  font-size: 11px;
}

.mr-val {
  width: 60px;
  text-align: center;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.mr-val.positive { color: #10b981; }
.mr-val.negative { color: #ef4444; }
.mr-val.text-gray { color: #94a3b8; }

.mr-val.team-a { text-align: left; color: #3b82f6; font-weight: 700; }
.mr-val.team-b { text-align: right; color: #f59e0b; font-weight: 700; }

.meta-mult-row {
  background: rgba(139, 92, 246, 0.05);
  border-radius: 4px;
  margin: 2px 0;
}

.meta-text {
  color: #8b5cf6;
  font-weight: 700;
}

.final-row {
  font-weight: 800;
  font-size: 13px;
  margin-top: 4px;
}

.final-row .mr-label-item {
  color: #1d2129;
  font-weight: 800;
}

.comparison-block {
  padding: 14px 16px;
  background: #f7f8fa;
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.03);
}

.comparison-label {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
  font-size: 12px;
}

.team-label {
  font-weight: 700;
  color: #1d2129;
}

.vs-label {
  color: #86909c;
  font-weight: 500;
  text-transform: uppercase;
  font-size: 10px;
  letter-spacing: 0.5px;
}

.bar-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bar-value {
  font-size: 14px;
  font-weight: 800;
  min-width: 48px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.bar-value.team-a { color: #3b82f6; }
.bar-value.team-b { color: #f59e0b; }

.progress-container {
  flex: 1;
  display: flex;
  height: 16px;
  background: #e5e7eb;
  border-radius: 10px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  transition: width 0.4s ease;
}

.progress-bar.team-a {
  background: linear-gradient(to right, #60a5fa, #3b82f6);
}

.progress-bar.team-b {
  background: linear-gradient(to left, #fbbf24, #f59e0b);
}

.progress-bar.winner {
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.3);
}

.bar-diff {
  text-align: center;
  margin-top: 8px;
  font-size: 11px;
  color: #86909c;
  font-weight: 500;
}

.bar-diff.positive { color: #3b82f6; }
.bar-diff.negative { color: #f59e0b; }

/* Meta加权战力区块 */
.meta-row {
  grid-template-columns: 1fr;
}

.meta-block {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.05) 0%, rgba(59, 130, 246, 0.05) 100%);
  border: 1px solid rgba(139, 92, 246, 0.15);
}

.meta-label {
  color: #8b5cf6;
  font-weight: 700;
}

.meta-hint {
  text-align: center;
  margin-top: 6px;
  font-size: 11px;
  color: #a0aec0;
}

/* 选手数据表格 */
.players-table {
  margin-top: 4px;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #e5e7eb;
}

.table-header {
  display: grid;
  grid-template-columns: 52px 1fr 80px 80px 52px 52px 64px 90px;
  gap: 6px;
  padding: 10px 16px;
  background: #1d2129;
  color: rgba(255, 255, 255, 0.9);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.team-section-header {
  display: flex;
  justify-content: space-between;
  padding: 8px 16px;
  font-weight: 700;
  font-size: 13px;
  color: #1d2129;
  border-bottom: 1px solid #e5e7eb;
  border-top: 1px solid #e5e7eb;
}

.team-a-header {
  background: rgba(59, 130, 246, 0.04);
}

.team-b-header {
  background: rgba(245, 158, 11, 0.04);
}

.team-power {
  color: #86909c;
  font-weight: 500;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.player-row {
  display: grid;
  grid-template-columns: 52px 1fr 80px 80px 52px 52px 64px 90px;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  border-bottom: 1px solid #f0f1f3;
  transition: background 0.15s ease;
  align-items: center;
}

.player-row:hover {
  background: #f7f8fa;
}

.player-row.high-impact {
  background: linear-gradient(to right, rgba(16, 185, 129, 0.06), transparent);
}

.player-row.low-impact {
  background: linear-gradient(to right, rgba(239, 68, 68, 0.05), transparent);
}

.team-section:last-child .player-row:last-child {
  border-bottom: none;
}

.col-position {
  color: #86909c;
  font-weight: 500;
  font-size: 12px;
}

.col-name {
  font-weight: 600;
  color: #1d2129;
}

/* 基础能力列（含迷你条形图） */
.col-base {
  display: flex;
  align-items: center;
  gap: 4px;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.base-value {
  min-width: 24px;
  text-align: right;
}

.mini-bar {
  flex: 1;
  height: 4px;
  background: #e5e7eb;
  border-radius: 2px;
  overflow: hidden;
}

.mini-bar-fill.base-fill {
  height: 100%;
  background: #93c5fd;
  border-radius: 2px;
  transition: width 0.3s;
}

/* 发挥列 */
.col-actual {
  text-align: center;
  font-variant-numeric: tabular-nums;
  font-weight: 800;
  font-size: 14px;
}

.col-actual.actual-above {
  color: #10b981;
}

.col-actual.actual-below {
  color: #ef4444;
}

.col-condition,
.col-noise {
  text-align: center;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

/* 影响力列 */
.col-impact {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.impact-bar-wrapper {
  width: 100%;
  height: 4px;
  background: #f0f1f3;
  border-radius: 2px;
  overflow: hidden;
}

.impact-bar {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s;
}

.positive-bar {
  background: #10b981;
}

.negative-bar {
  background: #ef4444;
}

.impact-value {
  font-size: 12px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.positive { color: #10b981; }
.negative { color: #ef4444; }
.very-positive { color: #059669; font-weight: 800; }
.very-negative { color: #dc2626; font-weight: 800; }

/* 发挥分解折叠面板 */
.breakdown-panel {
  margin-top: 16px;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  overflow: hidden;
}

.breakdown-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 12px 16px;
  background: #f7f8fa;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 700;
  color: #1d2129;
  text-align: left;
  transition: background 0.15s;
}

.breakdown-toggle:hover {
  background: #f0f1f3;
}

.toggle-arrow {
  font-size: 10px;
  color: #86909c;
  transition: transform 0.2s;
  display: inline-block;
}

.toggle-arrow.open {
  transform: rotate(90deg);
}

.breakdown-content {
  padding: 16px;
  border-top: 1px solid #e5e7eb;
}

.breakdown-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.breakdown-team-name {
  font-weight: 800;
  font-size: 13px;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 2px solid #e5e7eb;
}

.team-a-accent { color: #3b82f6; border-color: #3b82f6; }
.team-b-accent { color: #f59e0b; border-color: #f59e0b; }

.breakdown-header-row {
  display: grid;
  grid-template-columns: 1fr 100px 48px 48px 20px 48px;
  gap: 4px;
  font-size: 10px;
  color: #86909c;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  padding: 4px 0;
  margin-bottom: 2px;
}

.breakdown-row {
  display: grid;
  grid-template-columns: 1fr 100px 48px 48px 20px 48px;
  gap: 4px;
  padding: 4px 0;
  font-size: 12px;
  border-bottom: 1px solid #f7f8fa;
  align-items: center;
}

.bd-col-name {
  font-weight: 600;
  color: #1d2129;
}

.bd-col-base {
  display: flex;
  align-items: center;
  gap: 4px;
  font-variant-numeric: tabular-nums;
}

.bd-bar {
  flex: 1;
  height: 6px;
  background: #e5e7eb;
  border-radius: 3px;
  overflow: hidden;
}

.bd-bar-fill {
  height: 100%;
  background: #60a5fa;
  border-radius: 3px;
}

.bd-col-cond {
  text-align: center;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.bd-noise-val {
  text-align: center;
  color: #86909c;
  font-variant-numeric: tabular-nums;
}

.bd-col-arrow {
  text-align: center;
  color: #c0c4cc;
  font-size: 11px;
}

.bd-result-val {
  text-align: center;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  color: #1d2129;
}

/* MVP 卡片（通用样式，GameDetailView 和 MatchDetailDialog 共用） */
.mvp-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  margin-bottom: 12px;
  border-radius: 10px;
  border: 1px solid rgba(251, 191, 36, 0.25);
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.08) 0%, rgba(245, 158, 11, 0.04) 100%);
}

.mvp-trophy {
  font-size: 10px;
  font-weight: 800;
  color: #92400e;
  background: linear-gradient(135deg, #fde68a, #fbbf24);
  padding: 2px 10px;
  border-radius: 20px;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.mvp-player-name {
  font-size: 14px;
  font-weight: 800;
  color: #1d2129;
}

.mvp-team {
  font-size: 11px;
  color: #86909c;
}

.mvp-position {
  font-size: 11px;
  color: #86909c;
  padding: 1px 6px;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 4px;
}

.mvp-stat {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
}

.mvp-stat:last-child {
  margin-left: 0;
}

.mvp-stat-label {
  font-size: 10px;
  color: #a0aec0;
  text-transform: uppercase;
}

.mvp-stat-value {
  font-size: 14px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  color: #1d2129;
}

/* 对位差折叠面板 */
.matchup-panel {
  margin-top: 8px;
}

.matchup-header {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f0f1f3;
}

.matchup-team-name {
  font-weight: 800;
  font-size: 13px;
}

.matchup-vs {
  color: #c0c4cc;
  font-size: 11px;
  font-weight: 600;
}

.matchup-row {
  display: grid;
  grid-template-columns: 40px 1fr 1fr 1fr;
  gap: 8px;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid #f7f8fa;
}

.matchup-row:last-of-type {
  border-bottom: none;
}

.matchup-row.mu-key-row {
  background: linear-gradient(90deg, rgba(239, 68, 68, 0.04) 0%, rgba(239, 68, 68, 0.08) 50%, rgba(239, 68, 68, 0.04) 100%);
  border-radius: 6px;
  padding: 6px 4px;
}

.mu-position {
  font-size: 11px;
  font-weight: 600;
  color: #86909c;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.mu-key-badge {
  font-size: 8px;
  font-weight: 800;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
  padding: 0 4px;
  border-radius: 3px;
  letter-spacing: 0.5px;
}

.mu-player-a {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-end;
}

.mu-player-b {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-start;
}

.mu-name {
  font-size: 12px;
  font-weight: 600;
  color: #4e5969;
}

.mu-value {
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: #86909c;
  min-width: 28px;
  text-align: center;
}

.mu-value.mu-winner {
  color: #1d2129;
  font-weight: 800;
}

.mu-bar-container {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 20px;
}

.mu-bar-side {
  flex: 1;
  height: 8px;
  position: relative;
}

.mu-bar-a {
  display: flex;
  justify-content: flex-end;
}

.mu-bar-b {
  display: flex;
  justify-content: flex-start;
}

.mu-bar-fill {
  height: 100%;
  width: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.mu-fill-a {
  background: rgba(59, 130, 246, 0.25);
  border-radius: 4px 0 0 4px;
}

.mu-fill-a.mu-bar-winner {
  background: linear-gradient(to left, #3b82f6, #60a5fa);
}

.mu-fill-b {
  background: rgba(245, 158, 11, 0.25);
  border-radius: 0 4px 4px 0;
}

.mu-fill-b.mu-bar-winner {
  background: linear-gradient(to right, #f59e0b, #fbbf24);
}

.mu-diff-label {
  font-size: 11px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  min-width: 36px;
  text-align: center;
  color: #86909c;
  flex-shrink: 0;
}

.mu-diff-label.mu-diff-a {
  color: #3b82f6;
}

.mu-diff-label.mu-diff-b {
  color: #f59e0b;
}

.matchup-summary {
  display: flex;
  justify-content: center;
  gap: 24px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid #f0f1f3;
}

.mu-summary-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.mu-summary-label {
  color: #86909c;
}

.mu-summary-value {
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

/* 图例 */
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid #f0f1f3;
  font-size: 12px;
  color: #86909c;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-color {
  width: 10px;
  height: 10px;
  border-radius: 3px;
}

.legend-color.positive { background: #10b981; }
.legend-color.negative { background: #ef4444; }
.legend-color.high-impact {
  background: linear-gradient(to right, #10b981, #ef4444);
}

/* 特性标签 */
.col-traits {
  display: flex;
  gap: 3px;
  align-items: center;
  flex-wrap: wrap;
}

.trait-tag {
  font-size: 10px;
  padding: 1px 5px;
  height: auto;
  line-height: 1.2;
  border-radius: 4px;
}

.trait-tag.inactive {
  opacity: 0.5;
  border-style: dashed;
}

.no-trait {
  color: #c0c4cc;
  font-size: 11px;
}

.bp-panel {
  margin-top: 8px;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  overflow: hidden;
}

.bp-section {
  margin-bottom: 12px;
}

.bp-label {
  font-size: 12px;
  font-weight: 700;
  color: #86909c;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.bp-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.bp-tag {
  font-size: 11px;
}

.bp-empty {
  color: #c0c4cc;
  font-size: 12px;
}

.bp-teams-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.bp-team-col {
  padding: 8px 12px;
  background: #f7f8fa;
  border-radius: 8px;
}

.bp-team-label {
  font-weight: 700;
  font-size: 13px;
  margin-bottom: 8px;
  padding-bottom: 4px;
  border-bottom: 2px solid #e5e7eb;
}

.bp-pick-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bp-pick-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.bp-mastery {
  font-size: 10px;
  font-weight: 800;
  padding: 0 4px;
  border-radius: 3px;
}

.bp-mastery.mastery-SS {
  color: #b91c1c;
  background: rgba(239, 68, 68, 0.1);
}

.bp-mastery.mastery-S {
  color: #d97706;
  background: rgba(245, 158, 11, 0.1);
}

.bp-mastery.mastery-A {
  color: #059669;
  background: rgba(16, 185, 129, 0.1);
}

.bp-mastery.mastery-B {
  color: #86909c;
  background: rgba(0, 0, 0, 0.04);
}

.bp-comp {
  margin-top: 8px;
  font-size: 12px;
  color: #4e5969;
}

.bp-ban-row {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.bp-ban-col {
  flex: 1;
}

.bp-ban-col .bp-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.bp-pick-pos {
  font-size: 10px;
  font-weight: 800;
  color: #667eea;
  text-transform: uppercase;
  min-width: 28px;
}

.bp-player-name {
  font-size: 12px;
  font-weight: 600;
  color: #1d2129;
  min-width: 60px;
}

.bp-modifier {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 4px;
  border-radius: 3px;
}

.bp-modifier.positive {
  color: #059669;
  background: rgba(16, 185, 129, 0.1);
}

.bp-modifier.negative {
  color: #dc2626;
  background: rgba(220, 38, 38, 0.1);
}

.bp-narrative {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.bp-narrative-title {
  font-size: 13px;
  font-weight: 600;
  color: rgba(200, 200, 220, 0.9);
  margin-bottom: 10px;
}

.bp-narrative-row {
  display: flex;
  gap: 16px;
}

.bp-narrative-col {
  flex: 1;
}

.bp-narrative-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bp-narrative-item {
  font-size: 12px;
  color: rgba(200, 200, 220, 0.85);
  padding: 5px 10px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.03);
  display: flex;
  align-items: baseline;
  gap: 6px;
  line-height: 1.5;
}

.bp-narrative-item.phase-plan {
  background: rgba(64, 158, 255, 0.1);
  color: rgba(130, 190, 255, 0.95);
  border-left: 2px solid rgba(64, 158, 255, 0.5);
}

.bp-narrative-item.phase-ban_phase1,
.bp-narrative-item.phase-ban_phase2 {
  background: rgba(245, 108, 108, 0.08);
  color: rgba(245, 160, 160, 0.9);
  border-left: 2px solid rgba(245, 108, 108, 0.4);
}

.bp-narrative-item.phase-pick {
  background: rgba(103, 194, 58, 0.08);
  color: rgba(160, 220, 140, 0.9);
  border-left: 2px solid rgba(103, 194, 58, 0.4);
}

.bp-narrative-item.phase-switch {
  background: rgba(255, 165, 0, 0.12);
  color: rgba(255, 200, 100, 0.95);
  border-left: 2px solid rgba(255, 165, 0, 0.5);
}

.narrative-phase {
  flex-shrink: 0;
  font-weight: 600;
  font-size: 11px;
}

.narrative-msg {
  flex: 1;
}

@media (max-width: 768px) {
  .bp-narrative-row {
    flex-direction: column;
  }
}
</style>
