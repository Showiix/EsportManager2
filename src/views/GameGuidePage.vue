<template>
  <div class="guide-page">
    <!-- 快捷导航 -->
    <nav class="guide-nav">
      <a v-for="nav in navItems" :key="nav.id" :href="'#' + nav.id" class="nav-link" :class="nav.color">
        <el-icon><component :is="nav.icon" /></el-icon>
        <span>{{ nav.label }}</span>
      </a>
    </nav>

    <!-- 顶部横幅 -->
    <section class="hero-banner">
      <div class="hero-content">
        <div class="hero-badge">
          <el-icon><Trophy /></el-icon>
          <span>EsportManager 2</span>
        </div>
        <h1>游戏指南</h1>
        <p class="hero-subtitle">从赛季推进、赛事体系、转会博弈到财务经营，全面掌握游戏核心玩法</p>
        <div class="hero-stat-row">
          <div class="hero-stat" v-for="s in heroStats" :key="s.label">
            <span class="stat-num">{{ s.value }}</span>
            <span class="stat-label">{{ s.label }}</span>
          </div>
        </div>
      </div>
      <div class="hero-goals">
        <div class="goal-card" v-for="g in goals" :key="g.title">
          <el-icon class="goal-icon"><component :is="g.icon" /></el-icon>
          <div>
            <div class="goal-title">{{ g.title }}</div>
            <div class="goal-desc">{{ g.desc }}</div>
          </div>
        </div>
      </div>
    </section>

    <!-- 1. 赛季周期 -->
    <section id="season" class="guide-section">
      <div class="section-header violet">
        <el-icon><Calendar /></el-icon>
        <h2>赛季周期</h2>
        <span class="section-badge">15 个阶段</span>
      </div>
      <p class="section-desc">游戏时间按 S1、S2、S3... 推进，每个赛季包含 15 个阶段的完整循环。</p>

      <div class="phase-timeline">
        <div class="phase-group" v-for="group in phaseGroups" :key="group.title">
          <div class="phase-group-title">{{ group.title }}</div>
          <div class="phase-group-items">
            <div v-for="(phase, i) in group.items" :key="phase.name"
              class="phase-card" :class="phase.type">
              <div class="phase-num">{{ phase.num }}</div>
              <div class="phase-name">{{ phase.name }}</div>
              <div class="phase-sub">{{ phase.sub }}</div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 2. 联赛体系 -->
    <section id="league" class="guide-section">
      <div class="section-header blue">
        <el-icon><Aim /></el-icon>
        <h2>联赛体系</h2>
        <span class="section-badge">BO3 双循环 + 双败 BO5</span>
      </div>

      <div class="two-col">
        <div class="info-card">
          <h3><el-icon><List /></el-icon> 常规赛</h3>
          <ul class="rule-list">
            <li><strong>赛制：</strong>BO3 双循环，每队与其他 13 队各交手 2 次</li>
            <li><strong>积分规则：</strong>
              <div class="inline-tags">
                <el-tag type="success" size="small">2:0 胜 +3</el-tag>
                <el-tag type="primary" size="small">2:1 胜 +2</el-tag>
                <el-tag type="warning" size="small">1:2 负 +1</el-tag>
                <el-tag type="info" size="small">0:2 负 +0</el-tag>
              </div>
            </li>
            <li><strong>排名依据：</strong>积分 > 交锋胜率 > 总净胜小局</li>
          </ul>
        </div>
        <div class="info-card">
          <h3><el-icon><Trophy /></el-icon> 季后赛（双败淘汰 BO5）</h3>
          <p>常规赛前 8 名进入季后赛</p>
          <div class="bracket-mini">
            <div class="bracket-row">
              <span class="bracket-label winners">胜者组</span>
              <div class="bracket-matches">
                <span class="seed">1</span> vs <span class="seed">4</span>
                <span class="sep">|</span>
                <span class="seed">2</span> vs <span class="seed">3</span>
              </div>
            </div>
            <div class="bracket-row">
              <span class="bracket-label losers">败者组</span>
              <div class="bracket-matches">
                <span class="seed">5</span> vs <span class="seed">8</span>
                <span class="sep">|</span>
                <span class="seed">6</span> vs <span class="seed">7</span>
              </div>
            </div>
            <div class="bracket-row">
              <span class="bracket-label final">总决赛</span>
              <div class="bracket-matches">
                <span class="seed winner">胜者组冠军</span> vs <span class="seed loser">败者组冠军</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 3. 国际赛事 -->
    <section id="tournament" class="guide-section">
      <div class="section-header purple">
        <el-icon><Trophy /></el-icon>
        <h2>国际赛事</h2>
        <span class="section-badge">7 项赛事</span>
      </div>
      <p class="section-desc">每赛季共有 7 项国际赛事，按时间顺序举行</p>

      <div class="event-timeline-bar">
        <div v-for="e in eventTimeline" :key="e.key"
          class="timeline-dot" :class="[e.type, { active: activeEvent === e.key }]"
          @click="activeEvent = e.key">
          <span class="dot-label">{{ e.short }}</span>
        </div>
      </div>

      <div class="event-detail-card" v-if="activeTournament" :key="activeTournament.key">
        <div class="event-top">
          <div>
            <div class="event-name">{{ activeTournament.fullName }}</div>
            <div class="event-timing">{{ activeTournament.timing }}</div>
          </div>
          <div class="event-badges">
            <span class="badge teams">{{ activeTournament.teams }}</span>
            <span class="badge format">{{ activeTournament.format }}</span>
            <span v-if="activeTournament.highlight" class="badge highlight">{{ activeTournament.highlight }}</span>
          </div>
        </div>

        <div class="event-grid">
          <div class="event-block">
            <div class="block-title">参赛资格</div>
            <p>{{ activeTournament.qualification }}</p>
          </div>
          <div class="event-block">
            <div class="block-title">赛制规则</div>
            <ul>
              <li v-for="r in activeTournament.rules" :key="r">{{ r }}</li>
            </ul>
          </div>
          <div class="event-block">
            <div class="block-title">积分 / 奖金</div>
            <div class="points-row" v-for="p in activeTournament.rewards" :key="p.rank">
              <span class="reward-rank" :class="p.cls">{{ p.rank }}</span>
              <span class="reward-pts">{{ p.pts }}分</span>
              <span class="reward-prize">{{ p.prize }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 4. 年度积分 -->
    <section id="points" class="guide-section">
      <div class="section-header green">
        <el-icon><DataAnalysis /></el-icon>
        <h2>年度积分体系</h2>
        <span class="section-badge">Top 16 进入 Super 赛</span>
      </div>
      <p class="section-desc">年度积分决定 Super 洲际邀请赛参赛资格（全球前 16 名）</p>

      <div class="two-col">
        <div class="info-card">
          <h3>联赛季后赛积分（春/夏各一次）</h3>
          <div class="pts-table">
            <div class="pts-row" v-for="r in leaguePointsData" :key="r.rank">
              <span class="pts-rank">{{ r.rank }}</span>
              <el-tag type="success" size="small">+{{ r.points }}</el-tag>
            </div>
          </div>
        </div>
        <div class="info-card">
          <h3>国际赛事积分</h3>
          <div class="pts-table">
            <div class="pts-row header">
              <span>名次</span><span>MSI/上海</span><span>马德里/C洲际</span><span>S世界赛</span>
            </div>
            <div class="pts-row" v-for="r in intlPointsData" :key="r.rank">
              <span class="pts-rank">{{ r.rank }}</span>
              <span>{{ r.msi }}</span><span>{{ r.madrid }}</span><span>{{ r.worlds }}</span>
            </div>
          </div>
          <p class="note">Super 洲际赛不发放积分，它是对全年积分的最终奖励。</p>
        </div>
      </div>
    </section>

    <!-- 5. 选手体系 -->
    <section id="player" class="guide-section">
      <div class="section-header pink">
        <el-icon><User /></el-icon>
        <h2>选手体系</h2>
        <span class="section-badge">成长 / 属性 / 身价</span>
      </div>

      <!-- 核心属性 -->
      <h3 class="sub-title"><el-icon><Aim /></el-icon> 核心属性</h3>
      <div class="attr-grid">
        <div class="attr-card" v-for="a in playerAttrs" :key="a.name">
          <div class="attr-icon" :class="a.cls"><el-icon><component :is="a.icon" /></el-icon></div>
          <div class="attr-name">{{ a.name }}</div>
          <div class="attr-desc">{{ a.desc }}</div>
        </div>
      </div>

      <!-- 成长标签 -->
      <h3 class="sub-title"><el-icon><PriceTag /></el-icon> 成长标签</h3>
      <div class="three-col">
        <div class="tag-card genius">
          <span class="tag-label">天才 Genius</span>
          <span class="tag-effect">+3 能力/赛季</span>
          <span class="tag-note">身价 x1.2</span>
        </div>
        <div class="tag-card normal">
          <span class="tag-label">一般 Normal</span>
          <span class="tag-effect">+2 能力/赛季</span>
          <span class="tag-note">身价 x1.0</span>
        </div>
        <div class="tag-card ordinary">
          <span class="tag-label">平庸 Ordinary</span>
          <span class="tag-effect">+1 能力/赛季</span>
          <span class="tag-note">身价 x0.9</span>
        </div>
      </div>

      <!-- 年龄曲线 -->
      <h3 class="sub-title"><el-icon><TrendCharts /></el-icon> 选手生命周期</h3>
      <div class="lifecycle-bar">
        <div class="lc-phase growth">
          <div class="lc-title">成长期</div>
          <div class="lc-age">16-24 岁</div>
          <div class="lc-detail">能力按标签增长</div>
        </div>
        <div class="lc-phase peak">
          <div class="lc-title">巅峰期</div>
          <div class="lc-age">25-29 岁</div>
          <div class="lc-detail">稳定性最高</div>
        </div>
        <div class="lc-phase decline">
          <div class="lc-title">衰退期</div>
          <div class="lc-age">30+ 岁</div>
          <div class="lc-detail">能力 -0.5~-2/赛季</div>
        </div>
        <div class="lc-phase retire">
          <div class="lc-title">退役</div>
          <div class="lc-age">35+ 岁</div>
          <div class="lc-detail">能力 &lt;60 概率退役</div>
        </div>
      </div>

      <!-- 身价公式 -->
      <h3 class="sub-title"><el-icon><Money /></el-icon> 身价计算</h3>
      <div class="formula-box">
        <div class="formula">身价 = 能力值 x 基础系数 x 年龄系数 x 潜力系数 x 标签系数 x 位置系数 x 荣誉系数 x 赛区系数</div>
      </div>
      <div class="coef-grid">
        <div class="coef-card">
          <div class="coef-title">位置系数</div>
          <div class="coef-items">
            <span>MID x1.2</span><span>ADC x1.15</span><span>JUG x1.1</span><span>TOP x1.0</span><span>SUP x0.9</span>
          </div>
        </div>
        <div class="coef-card">
          <div class="coef-title">赛区系数</div>
          <div class="coef-items">
            <span>LPL x1.3</span><span>LCK x1.2</span><span>LEC x1.0</span><span>LCS x0.9</span>
          </div>
        </div>
        <div class="coef-card">
          <div class="coef-title">年龄系数</div>
          <div class="coef-items">
            <span>17-19 x1.5</span><span>20-22 x1.3</span><span>23-25 x1.0</span><span>26-27 x0.85</span><span>30+ x0.5</span>
          </div>
        </div>
      </div>
    </section>

    <!-- 6. 选手特性 -->
    <section id="traits" class="guide-section">
      <div class="section-header orange">
        <el-icon><StarFilled /></el-icon>
        <h2>选手特性</h2>
        <span class="section-badge">14 种特性</span>
      </div>
      <p class="section-desc">特性根据比赛情境动态修正选手的能力、稳定性和状态</p>

      <div class="trait-category" v-for="cat in traitCategories" :key="cat.title">
        <div class="trait-cat-title">{{ cat.title }}</div>
        <div class="trait-list">
          <div class="trait-item" v-for="t in cat.traits" :key="t.name">
            <span class="trait-name">{{ t.name }}</span>
            <span class="trait-effect">{{ t.effect }}</span>
            <span class="trait-trigger" v-if="t.trigger">{{ t.trigger }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- 7. 比赛模拟 -->
    <section id="simulation" class="guide-section">
      <div class="section-header sky">
        <el-icon><Cpu /></el-icon>
        <h2>比赛模拟系统</h2>
        <span class="section-badge">三层正态分布</span>
      </div>

      <div class="sim-layers">
        <div class="sim-layer">
          <div class="layer-num">1</div>
          <div class="layer-body">
            <div class="layer-title">选手发挥计算</div>
            <div class="formula-box compact">
              <div class="formula">发挥值 = 能力值 + 状态值 + N(0, sigma)</div>
              <div class="formula-note">sigma = (100 - 稳定性) / 10</div>
            </div>
          </div>
        </div>
        <div class="layer-arrow"><el-icon><Bottom /></el-icon></div>
        <div class="sim-layer">
          <div class="layer-num">2</div>
          <div class="layer-body">
            <div class="layer-title">队伍战力计算</div>
            <div class="formula-box compact">
              <div class="formula">队伍战力 = sum(5人发挥值) / 5</div>
              <div class="formula-note">受 Meta 版本位置权重加成</div>
            </div>
          </div>
        </div>
        <div class="layer-arrow"><el-icon><Bottom /></el-icon></div>
        <div class="sim-layer">
          <div class="layer-num">3</div>
          <div class="layer-body">
            <div class="layer-title">胜负判定</div>
            <div class="formula-box compact">
              <div class="formula">战力差 = 主队 - 客队 + N(0, 3)</div>
              <div class="formula-note">战力差 > 0 主队胜，否则客队胜</div>
            </div>
          </div>
        </div>
      </div>

      <h3 class="sub-title">战力差胜率参考</h3>
      <div class="winrate-grid">
        <div class="wr-item" v-for="w in winRates" :key="w.diff">
          <span class="wr-diff">差值 {{ w.diff }}</span>
          <div class="wr-bar"><div class="wr-fill" :style="{ width: w.rate }"></div></div>
          <span class="wr-rate">{{ w.rate }}</span>
        </div>
      </div>
    </section>

    <!-- 8. Meta 系统 -->
    <section id="meta" class="guide-section">
      <div class="section-header teal">
        <el-icon><SetUp /></el-icon>
        <h2>Meta 版本系统</h2>
        <span class="section-badge">20 种版本</span>
      </div>
      <p class="section-desc">每赛季有不同的 Meta 版本影响五个位置的权重。S1 固定为均衡版本，S2 起随机轮换（不连续重复）。</p>

      <div class="formula-box">
        <div class="formula">加权战力 = sum(位置权重 x 选手能力) / 5 + 短板惩罚(x0.5) + 长板奖励(x0.3)</div>
        <div class="formula-note">低于平均的位置惩罚更大(0.5)，鼓励阵容均衡</div>
      </div>

      <div class="meta-examples">
        <div class="meta-card" v-for="m in metaExamples" :key="m.name">
          <div class="meta-name">{{ m.name }}</div>
          <div class="meta-weights">
            <span v-for="(w, pos) in m.weights" :key="pos" class="meta-w" :class="{ hot: w > 1.1, cold: w < 0.9 }">
              {{ pos }} {{ w }}
            </span>
          </div>
          <div class="meta-focus">{{ m.focus }}</div>
        </div>
      </div>
    </section>

    <!-- 9. 转会系统 -->
    <section id="transfer" class="guide-section">
      <div class="section-header cyan">
        <el-icon><Switch /></el-icon>
        <h2>转会系统</h2>
        <span class="section-badge">8 轮流程</span>
      </div>

      <div class="round-flow">
        <div class="round-card" v-for="r in transferRounds" :key="r.step">
          <div class="round-num">{{ r.step }}</div>
          <div class="round-name">{{ r.name }}</div>
          <div class="round-desc">{{ r.desc }}</div>
        </div>
      </div>

      <!-- GM 人格 -->
      <h3 class="sub-title"><el-icon><Avatar /></el-icon> GM 人格系统</h3>
      <div class="gm-grid">
        <div class="gm-card" v-for="gm in gmPersonalities" :key="gm.name" :class="gm.cls">
          <div class="gm-name">{{ gm.name }}</div>
          <div class="gm-desc">{{ gm.desc }}</div>
          <div class="gm-stats">
            <span>预算 {{ gm.budget }}</span>
            <span>能力阈值 {{ gm.threshold }}</span>
            <span>溢价 {{ gm.premium }}</span>
          </div>
        </div>
      </div>

      <!-- 跨赛区偏好 -->
      <h3 class="sub-title"><el-icon><Connection /></el-icon> 跨赛区转会偏好</h3>
      <div class="loyalty-bars">
        <div class="loyalty-row" v-for="l in regionLoyalty" :key="l.region">
          <span class="loyalty-label">{{ l.region }}</span>
          <div class="loyalty-track"><div class="loyalty-fill" :style="{ width: l.pct }"></div></div>
          <span class="loyalty-value">{{ l.range }}</span>
        </div>
      </div>
      <p class="note">跨赛区意愿度 = 基础意愿 x (100 - region_loyalty) / 100，LPL 选手最不愿外流</p>
    </section>

    <!-- 10. 财政系统 -->
    <section id="finance" class="guide-section">
      <div class="section-header emerald">
        <el-icon><Wallet /></el-icon>
        <h2>财政系统</h2>
      </div>

      <div class="two-col">
        <div class="info-card">
          <h3><el-icon><TrendCharts /></el-icon> 收入来源</h3>
          <div class="finance-list">
            <div class="fl-item"><span class="fl-label">联赛分成</span><span class="fl-value income">125-200 万/赛季</span></div>
            <div class="fl-item"><span class="fl-label">赛事奖金</span><span class="fl-value income">根据名次</span></div>
            <div class="fl-item"><span class="fl-label">赞助收入</span><span class="fl-value income">50-600+ 万</span></div>
            <div class="fl-item"><span class="fl-label">转会收入</span><span class="fl-value income">出售选手</span></div>
          </div>
        </div>
        <div class="info-card">
          <h3><el-icon><Money /></el-icon> 支出项目</h3>
          <div class="finance-list">
            <div class="fl-item"><span class="fl-label">选手薪资</span><span class="fl-value expense">赛季初扣除</span></div>
            <div class="fl-item"><span class="fl-label">运营成本</span><span class="fl-value expense">150万 + 薪资x15%</span></div>
            <div class="fl-item"><span class="fl-label">转会支出</span><span class="fl-value expense">购买选手</span></div>
          </div>
        </div>
      </div>

      <h3 class="sub-title"><el-icon><Coin /></el-icon> 主要奖金池</h3>
      <div class="prize-grid">
        <div class="prize-card" v-for="p in prizePools" :key="p.event">
          <div class="prize-event">{{ p.event }}</div>
          <div class="prize-total">总 {{ p.total }}</div>
          <div class="prize-champion">冠军 {{ p.champion }}</div>
        </div>
      </div>

      <h3 class="sub-title"><el-icon><DataLine /></el-icon> 财务状态</h3>
      <div class="status-bar">
        <div class="status-item" v-for="fs in financeStatus" :key="fs.name" :class="fs.cls">
          <div class="fs-name">{{ fs.name }}</div>
          <div class="fs-range">{{ fs.range }}</div>
          <div class="fs-strategy">{{ fs.strategy }}</div>
        </div>
      </div>
    </section>

    <!-- 11. 荣誉系统 -->
    <section id="honor" class="guide-section">
      <div class="section-header gold">
        <el-icon><Medal /></el-icon>
        <h2>荣誉系统</h2>
      </div>

      <div class="two-col">
        <div class="info-card">
          <h3>荣誉类型</h3>
          <div class="honor-grid">
            <div class="honor-item" v-for="h in honorTypes" :key="h">{{ h }}</div>
          </div>
        </div>
        <div class="info-card">
          <h3>荣誉身价加成（部分）</h3>
          <div class="bonus-list">
            <div class="bonus-row" v-for="b in honorBonuses" :key="b.name">
              <span>{{ b.name }}</span>
              <span class="bonus-value">{{ b.value }}</span>
              <span class="bonus-dur">{{ b.dur }}</span>
            </div>
          </div>
          <div class="formula-box compact">
            <div class="formula">荣誉系数 = 1.0 + sum(有效加成)，上限 3.0</div>
          </div>
        </div>
      </div>
    </section>

    <!-- 12. 数据中心 -->
    <section id="data" class="guide-section">
      <div class="section-header indigo">
        <el-icon><TrendCharts /></el-icon>
        <h2>数据中心</h2>
      </div>
      <p class="section-desc">追踪选手比赛表现，用于 MVP 评选、年度最佳阵容和 Top20 评分</p>

      <div class="two-col">
        <div class="info-card">
          <h3>影响力分数</h3>
          <div class="formula-box compact">
            <div class="formula">影响力 = 选手发挥 - 队伍平均</div>
          </div>
          <p>正值 = Carry 表现，负值 = 拖后腿</p>
        </div>
        <div class="info-card">
          <h3>年度 Top20 评分</h3>
          <div class="formula-box compact">
            <div class="formula">评分 = 影响力(35%) + 表现(20%) + 冠军(20%) + 稳定(15%) + 出场(10%)</div>
          </div>
          <div class="top20-tiers">
            <span class="tier t1">Top 1-5：身价 +20%</span>
            <span class="tier t2">Top 6-10：身价 +15%</span>
            <span class="tier t3">Top 11-20：身价 +10%</span>
          </div>
        </div>
      </div>
    </section>

    <!-- 13. 选秀系统 -->
    <section id="draft" class="guide-section">
      <div class="section-header lavender">
        <el-icon><UserFilled /></el-icon>
        <h2>选秀系统</h2>
        <span class="section-badge">每赛季进行</span>
      </div>
      <p class="section-desc">每个赛季的转会期结束后都会进行选秀，为各战队注入新鲜血液</p>

      <div class="two-col">
        <div class="info-card">
          <h3>选秀规则</h3>
          <ul class="rule-list">
            <li>每赛区 50 名新秀存入选秀池（共 200 名）</li>
            <li>选秀时每赛区随机抽取 14 名</li>
            <li>按 <strong>能力x0.4 + 潜力x0.6</strong> 排名</li>
            <li>选秀顺位由夏季常规赛排名加权抽签决定</li>
            <li>排名越靠后，抽到高顺位概率越大</li>
          </ul>
        </div>
        <div class="info-card">
          <h3>新秀合同</h3>
          <div class="draft-stats">
            <div class="ds-item"><span>合同年限</span><strong>3 年</strong></div>
            <div class="ds-item"><span>年薪</span><strong>8-60 万</strong></div>
            <div class="ds-item"><span>年龄范围</span><strong>16-19 岁</strong></div>
            <div class="ds-item"><span>忠诚/满意</span><strong>50 / 50</strong></div>
          </div>
          <p class="draft-salary-note">薪资按能力分档（8/15/25/40万）+ 潜力加成（+10或+20万）</p>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Calendar, Trophy, Aim, List, DataAnalysis, User, StarFilled, Cpu,
  Switch, Wallet, Medal, TrendCharts, Money, Coin, DataLine, SetUp,
  PriceTag, UserFilled, Bottom, Connection, Avatar, Star,
} from '@element-plus/icons-vue'

const activeEvent = ref('msi')

const heroStats = [
  { value: '4', label: '赛区' },
  { value: '56', label: '战队' },
  { value: '280', label: '选手' },
  { value: '15', label: '赛季阶段' },
  { value: '7', label: '国际赛事' },
  { value: '20', label: 'Meta 版本' },
]

const goals = [
  { icon: Trophy, title: 'S 世界赛冠军', desc: '带领战队问鼎最高荣誉' },
  { icon: Star, title: 'Super 洲际赛', desc: '累积年度积分争夺席位' },
  { icon: Medal, title: '荣誉殿堂王朝', desc: '培养传奇选手铸就辉煌' },
]

const navItems = [
  { id: 'season', label: '赛季', icon: Calendar, color: 'violet' },
  { id: 'league', label: '联赛', icon: Aim, color: 'blue' },
  { id: 'tournament', label: '赛事', icon: Trophy, color: 'purple' },
  { id: 'points', label: '积分', icon: DataAnalysis, color: 'green' },
  { id: 'player', label: '选手', icon: User, color: 'pink' },
  { id: 'traits', label: '特性', icon: StarFilled, color: 'orange' },
  { id: 'simulation', label: '模拟', icon: Cpu, color: 'sky' },
  { id: 'meta', label: 'Meta', icon: SetUp, color: 'teal' },
  { id: 'transfer', label: '转会', icon: Switch, color: 'cyan' },
  { id: 'finance', label: '财政', icon: Wallet, color: 'emerald' },
  { id: 'honor', label: '荣誉', icon: Medal, color: 'gold' },
  { id: 'data', label: '数据', icon: TrendCharts, color: 'indigo' },
  { id: 'draft', label: '选秀', icon: UserFilled, color: 'lavender' },
]

const phaseGroups = [
  {
    title: '春季',
    items: [
      { num: 1, name: '春季赛常规赛', sub: 'BO3 双循环', type: 'spring' },
      { num: 2, name: '春季赛季后赛', sub: '双败淘汰 BO5', type: 'spring' },
    ],
  },
  {
    title: '春季国际赛',
    items: [
      { num: 3, name: 'MSI 季中赛', sub: '12 队双败淘汰', type: 'intl' },
      { num: 4, name: '马德里大师赛', sub: '32 队小组+淘汰', type: 'intl' },
    ],
  },
  {
    title: '夏季',
    items: [
      { num: 5, name: '夏季赛常规赛', sub: 'BO3 双循环', type: 'summer' },
      { num: 6, name: '夏季赛季后赛', sub: '双败淘汰 BO5', type: 'summer' },
    ],
  },
  {
    title: '夏季国际赛',
    items: [
      { num: 7, name: 'Claude 洲际赛', sub: '32 队小组+淘汰', type: 'intl' },
      { num: 8, name: 'S 世界赛', sub: '16 队瑞士轮+淘汰', type: 'intl' },
    ],
  },
  {
    title: '赛季末国际赛',
    items: [
      { num: 9, name: '上海大师赛', sub: '12 队双败淘汰', type: 'intl' },
      { num: 10, name: 'ICP 洲际对抗赛', sub: '24 队赛区对抗', type: 'intl' },
      { num: 11, name: 'Super 洲际赛', sub: '年度 Top16', type: 'intl' },
    ],
  },
  {
    title: '休赛期',
    items: [
      { num: 12, name: '年度颁奖', sub: 'MVP/Top20/最佳阵容', type: 'offseason' },
      { num: 13, name: '转会期', sub: '8 轮转会流程', type: 'offseason' },
      { num: 14, name: '选秀', sub: '每赛季进行', type: 'offseason' },
      { num: 15, name: '赛季结束', sub: '进入下赛季', type: 'end' },
    ],
  },
]

const eventTimeline = [
  { key: 'msi', short: 'MSI', type: 'intl' },
  { key: 'madrid', short: '马德里', type: 'intl' },
  { key: 'claude', short: 'C洲际', type: 'intl' },
  { key: 'worlds', short: '世界赛', type: 'worlds' },
  { key: 'shanghai', short: '上海', type: 'intl' },
  { key: 'icp', short: 'ICP', type: 'intl' },
  { key: 'super', short: 'Super', type: 'super' },
]

const tournaments = [
  {
    key: 'msi', fullName: 'MSI 季中邀请赛', timing: '春季赛后',
    teams: '12 队', format: '双败淘汰 BO5', highlight: '',
    qualification: '各赛区春季赛季后赛冠军、亚军、季军（共 12 队）',
    rules: ['12 队分为传奇组(4冠军)、挑战者组(4亚军)、资格赛组(4季军)', '双败淘汰 BO5', '败者组冠军 vs 胜者组冠军争夺总冠军'],
    rewards: [
      { rank: '冠军', pts: 20, prize: '2000万', cls: 'gold' },
      { rank: '亚军', pts: 16, prize: '1000万', cls: 'silver' },
      { rank: '3-4名', pts: 12, prize: '500万', cls: 'bronze' },
      { rank: '5-8名', pts: 6, prize: '200万', cls: '' },
    ],
  },
  {
    key: 'madrid', fullName: '马德里大师赛', timing: 'MSI 后',
    teams: '32 队', format: '小组赛+淘汰赛', highlight: '',
    qualification: '各赛区春季常规赛前 8 名（共 32 队）',
    rules: ['小组赛：8 组 x 4 队，BO3 单循环', '同赛区蛇形分组', '小组前 2 名晋级 16 强', '淘汰赛：BO5 单淘汰'],
    rewards: [
      { rank: '冠军', pts: 30, prize: '800万', cls: 'gold' },
      { rank: '亚军', pts: 24, prize: '400万', cls: 'silver' },
      { rank: '3-4名', pts: 18, prize: '200万', cls: 'bronze' },
      { rank: '5-8名', pts: 12, prize: '100万', cls: '' },
    ],
  },
  {
    key: 'claude', fullName: 'Claude 洲际赛', timing: '夏季赛季后赛后',
    teams: '32 队', format: '小组赛+淘汰赛', highlight: '',
    qualification: '各赛区夏季常规赛前 8 名（共 32 队）',
    rules: ['赛制与马德里大师赛相同', '小组赛 8 组 x 4 队 BO3', '16 强 BO5 单淘汰'],
    rewards: [
      { rank: '冠军', pts: 30, prize: '800万', cls: 'gold' },
      { rank: '亚军', pts: 24, prize: '400万', cls: 'silver' },
      { rank: '3-4名', pts: 18, prize: '200万', cls: 'bronze' },
      { rank: '5-8名', pts: 12, prize: '100万', cls: '' },
    ],
  },
  {
    key: 'worlds', fullName: 'S 世界赛', timing: 'C 洲际赛后',
    teams: '16 队', format: '瑞士轮+淘汰赛', highlight: '年度最高荣誉',
    qualification: '各赛区夏季赛季后赛前 4 名（共 16 队）',
    rules: ['瑞士轮：3 胜晋级 / 3 败淘汰', '同战绩队伍对阵', '产生 8 支队伍进入淘汰赛', '淘汰赛：8 强/4 强/决赛均 BO5'],
    rewards: [
      { rank: '冠军', pts: 100, prize: '5000万', cls: 'gold' },
      { rank: '亚军', pts: 80, prize: '2500万', cls: 'silver' },
      { rank: '3-4名', pts: 60, prize: '1200万', cls: 'bronze' },
      { rank: '5-8名', pts: 40, prize: '600万', cls: '' },
    ],
  },
  {
    key: 'shanghai', fullName: '上海大师赛', timing: 'S 世界赛后',
    teams: '12 队', format: '双败淘汰 BO5', highlight: '',
    qualification: '各赛区夏季赛季后赛冠军、亚军、季军（共 12 队）',
    rules: ['赛制与 MSI 相同', '12 队双败淘汰 BO5'],
    rewards: [
      { rank: '冠军', pts: 20, prize: '1000万', cls: 'gold' },
      { rank: '亚军', pts: 16, prize: '500万', cls: 'silver' },
      { rank: '3-4名', pts: 12, prize: '250万', cls: 'bronze' },
      { rank: '5-8名', pts: 6, prize: '120万', cls: '' },
    ],
  },
  {
    key: 'icp', fullName: 'ICP 洲际对抗赛', timing: '上海大师赛后',
    teams: '24 队', format: '小组赛+淘汰赛', highlight: '',
    qualification: '各赛区夏季常规赛前 6 名（共 24 队）',
    rules: ['小组赛：4 组 x 6 队，BO1 单循环', '小组前 2 名晋级 8 强', '淘汰赛：BO5 单淘汰'],
    rewards: [
      { rank: '冠军', pts: 25, prize: '1200万', cls: 'gold' },
      { rank: '亚军', pts: 20, prize: '600万', cls: 'silver' },
      { rank: '3-4名', pts: 15, prize: '300万', cls: 'bronze' },
      { rank: '5-8名', pts: 10, prize: '150万', cls: '' },
    ],
  },
  {
    key: 'super', fullName: 'Super 洲际年度邀请赛', timing: '赛季末',
    teams: '16 队', format: '分档淘汰 BO5', highlight: '年度收官战',
    qualification: '全球年度积分前 16 名',
    rules: ['1-4 名：直通半决赛', '5-8 名：从八强开始', '9-16 名：从 16 强开始', '全程 BO5 单淘汰'],
    rewards: [
      { rank: '冠军', pts: 0, prize: '6000万', cls: 'gold' },
      { rank: '亚军', pts: 0, prize: '3000万', cls: 'silver' },
      { rank: '3-4名', pts: 0, prize: '1500万', cls: 'bronze' },
      { rank: '5-8名', pts: 0, prize: '750万', cls: '' },
    ],
  },
]

const activeTournament = computed(() => tournaments.find(t => t.key === activeEvent.value))

const leaguePointsData = [
  { rank: '冠军', points: 12 }, { rank: '亚军', points: 10 },
  { rank: '季军', points: 8 }, { rank: '殿军', points: 6 }, { rank: '5-8名', points: 3 },
]

const intlPointsData = [
  { rank: '冠军', msi: 20, madrid: 30, worlds: 100 },
  { rank: '亚军', msi: 16, madrid: 24, worlds: 80 },
  { rank: '3-4名', msi: 12, madrid: 18, worlds: 60 },
  { rank: '5-8名', msi: 6, madrid: 12, worlds: 40 },
]

const playerAttrs = [
  { name: '能力值', desc: '当前实力 0-100', icon: Aim, cls: 'ability' },
  { name: '潜力值', desc: '能力上限', icon: TrendCharts, cls: 'potential' },
  { name: '稳定性', desc: '发挥波动程度', icon: SetUp, cls: 'stability' },
  { name: '忠诚度', desc: '离队意愿', icon: StarFilled, cls: 'loyalty' },
  { name: '状态值', desc: '-10 ~ +10 当前状态', icon: DataLine, cls: 'condition' },
  { name: '年龄', desc: '影响成长/衰退', icon: Calendar, cls: 'age' },
]

const traitCategories = [
  {
    title: '大赛表现',
    traits: [
      { name: 'Clutch', effect: '状态 +3', trigger: '季后赛/国际赛' },
      { name: 'SlowStarter', effect: '首局 -2，3+ 局 +2', trigger: 'BO3/BO5' },
      { name: 'FastStarter', effect: '首局 +2，3+ 局 -1', trigger: 'BO3/BO5' },
    ],
  },
  {
    title: '稳定性',
    traits: [
      { name: 'Explosive', effect: '稳定 -15，能力上限 +5', trigger: '' },
      { name: 'Consistent', effect: '稳定 +10，能力上限 -3', trigger: '' },
    ],
  },
  {
    title: '心态',
    traits: [
      { name: 'ComebackKing', effect: '落后时状态 +3', trigger: '比分落后' },
      { name: 'Tilter', effect: '领先 -2 / 落后 -3', trigger: '比分变化' },
      { name: 'MentalFortress', effect: '动量影响 x0.5', trigger: '始终生效' },
      { name: 'Fragile', effect: '输局后动量 -2', trigger: '输掉一局后' },
    ],
  },
  {
    title: '耐力',
    traits: [
      { name: 'Ironman', effect: '无疲劳惩罚', trigger: '' },
      { name: 'Volatile', effect: '稳定 -10', trigger: '' },
    ],
  },
  {
    title: '特殊',
    traits: [
      { name: 'RisingStar', effect: '能力 +3', trigger: '首赛季 & 年龄<=20' },
      { name: 'Veteran', effect: '稳定 +15', trigger: '年龄 30+' },
      { name: 'TeamLeader', effect: '队友状态 +1', trigger: '能力>=65' },
    ],
  },
]

const winRates = [
  { diff: 0, rate: '50%' }, { diff: 1, rate: '63%' }, { diff: 2, rate: '75%' },
  { diff: 3, rate: '84%' }, { diff: 5, rate: '95%' },
]

const metaExamples = [
  { name: '均衡', weights: { T: 1.0, J: 1.0, M: 1.0, A: 1.0, S: 1.0 }, focus: '五位置等权' },
  { name: '中路为王', weights: { T: 0.85, J: 0.9, M: 1.4, A: 0.95, S: 0.9 }, focus: 'Mid 核心' },
  { name: '下路霸权', weights: { T: 0.8, J: 0.9, M: 0.9, A: 1.35, S: 1.05 }, focus: 'ADC+SUP' },
  { name: '上单 Carry', weights: { T: 1.35, J: 0.9, M: 0.9, A: 0.9, S: 0.95 }, focus: 'Top 核心' },
  { name: '打野节奏', weights: { T: 0.85, J: 1.4, M: 0.9, A: 0.95, S: 0.9 }, focus: 'Jungle 主导' },
  { name: '辅助时代', weights: { T: 0.9, J: 0.9, M: 0.9, A: 0.9, S: 1.4 }, focus: 'Support 核心' },
]

const transferRounds = [
  { step: 'R1', name: '赛季结算', desc: '年龄/能力更新、退役' },
  { step: 'R2', name: '双向评估', desc: 'AI 评估阵容需求' },
  { step: 'R3', name: '续约谈判', desc: '到期合同续约' },
  { step: 'R4', name: '自由球员竞标', desc: '多队争夺自由人' },
  { step: 'R5', name: '合同选手转会', desc: '支付转会费交易' },
  { step: 'R6', name: '财务调整', desc: '困难队伍瘦身' },
  { step: 'R7', name: '最终补救', desc: '补足阵容名单' },
  { step: 'R8', name: '选秀权拍卖', desc: '每赛季启用' },
]

const gmPersonalities = [
  { name: '争冠型', desc: '追求顶级选手，高价挖角', budget: '80%', threshold: '>=80', premium: '30%', cls: 'championship' },
  { name: '青训型', desc: '培养年轻人，卖老将换资金', budget: '50%', threshold: '>=65', premium: '10%', cls: 'youth' },
  { name: '稳健型', desc: '性价比优先，控制风险', budget: '60%', threshold: '>=70', premium: '15%', cls: 'balanced' },
  { name: '投机型', desc: '买低卖高，赚取差价', budget: '70%', threshold: '>=60', premium: '20%', cls: 'speculator' },
  { name: '重建型', desc: '大规模清洗，低价收人', budget: '40%', threshold: '>=55', premium: '5%', cls: 'rebuilding' },
]

const regionLoyalty = [
  { region: 'LPL 忠诚度', range: '75-90%', pct: '82%' },
  { region: 'LCK 忠诚度', range: '55-75%', pct: '65%' },
  { region: 'LEC 忠诚度', range: '45-65%', pct: '55%' },
  { region: 'LCS 忠诚度', range: '40-60%', pct: '50%' },
]

const prizePools = [
  { event: 'Super 洲际赛', total: '1.5 亿', champion: '6000 万' },
  { event: 'S 世界赛', total: '1.2 亿', champion: '5000 万' },
  { event: 'MSI', total: '4000 万', champion: '2000 万' },
  { event: 'ICP', total: '3000 万', champion: '1200 万' },
  { event: '上海大师赛', total: '2500 万', champion: '1000 万' },
  { event: '马德里/C洲际', total: '2000 万', champion: '800 万' },
]

const financeStatus = [
  { name: '富裕', range: '> 1000万', strategy: '积极买入强援', cls: 'wealthy' },
  { name: '健康', range: '500-1000万', strategy: '被动观望市场', cls: 'healthy' },
  { name: '紧张', range: '100-500万', strategy: '限制高薪', cls: 'tight' },
  { name: '赤字', range: '0-100万', strategy: '必须卖人', cls: 'deficit' },
  { name: '破产', range: '< 0', strategy: '强制清洗', cls: 'bankrupt' },
]

const honorTypes = [
  '战队冠军/亚军/季军/殿军', '选手冠军成员', '赛事 MVP / 决赛 MVP',
  '年度 MVP', '年度 Top20', '最佳阵容一/二/三阵', '年度最佳新秀',
]

const honorBonuses = [
  { name: '世界赛冠军', value: '+80%', dur: '永久' },
  { name: '世界赛亚军', value: '+40%', dur: '永久' },
  { name: 'MSI 冠军', value: '+50%', dur: '永久' },
  { name: '赛区冠军', value: '+30%', dur: '3 赛季' },
  { name: '赛区 MVP', value: '+20%', dur: '2 赛季' },
  { name: '最佳一阵', value: '+25%', dur: '2 赛季' },
  { name: '最佳新秀', value: '+20%', dur: '3 赛季' },
]
</script>

<style scoped>
/* ==================== 基础布局 ==================== */
.guide-page {
  padding: 0 0 40px;
  color: var(--el-text-color-primary);
}

/* ==================== 快捷导航 ==================== */
.guide-nav {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 10px 0;
  margin-bottom: 16px;
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 16px;
  font-size: 12px;
  text-decoration: none;
  color: var(--el-text-color-secondary);
  background: var(--el-fill-color-light);
  transition: all 0.2s;
  border: 1px solid transparent;
}
.nav-link:hover { border-color: var(--el-color-primary-light-5); color: var(--el-color-primary); }
.nav-link.violet:hover { color: #8b5cf6; border-color: #c4b5fd; }
.nav-link.blue:hover { color: #3b82f6; border-color: #93c5fd; }
.nav-link.purple:hover { color: #7c3aed; border-color: #c4b5fd; }
.nav-link.green:hover { color: #22c55e; border-color: #86efac; }
.nav-link.pink:hover { color: #ec4899; border-color: #f9a8d4; }
.nav-link.orange:hover { color: #f97316; border-color: #fdba74; }
.nav-link.sky:hover { color: #0ea5e9; border-color: #7dd3fc; }
.nav-link.teal:hover { color: #14b8a6; border-color: #5eead4; }
.nav-link.cyan:hover { color: #06b6d4; border-color: #67e8f9; }
.nav-link.emerald:hover { color: #10b981; border-color: #6ee7b7; }
.nav-link.gold:hover { color: #eab308; border-color: #fde047; }
.nav-link.indigo:hover { color: #6366f1; border-color: #a5b4fc; }
.nav-link.lavender:hover { color: #a855f7; border-color: #d8b4fe; }

/* ==================== 顶部横幅 ==================== */
.hero-banner {
  background: linear-gradient(135deg, #eff6ff, #f5f3ff, #fdf2f8);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 16px;
  padding: 28px;
  margin-bottom: 20px;
  display: grid;
  grid-template-columns: 1.6fr 1fr;
  gap: 24px;
}
.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 99px;
  background: linear-gradient(135deg, #3b82f6, #8b5cf6);
  color: white;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 12px;
}
.hero-banner h1 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 800;
  background: linear-gradient(135deg, #1e40af, #7c3aed);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.hero-subtitle {
  margin: 0 0 16px;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}
.hero-stat-row {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}
.hero-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.stat-num {
  font-size: 22px;
  font-weight: 800;
  color: var(--el-color-primary);
}
.stat-label {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}
.hero-goals {
  display: flex;
  flex-direction: column;
  gap: 10px;
  justify-content: center;
}
.goal-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  background: white;
  border: 1px solid var(--el-border-color-lighter);
  box-shadow: 0 1px 3px rgba(0,0,0,0.04);
}
.goal-icon { font-size: 24px; color: #f59e0b; }
.goal-title { font-size: 14px; font-weight: 600; }
.goal-desc { font-size: 12px; color: var(--el-text-color-secondary); }

/* ==================== 通用 Section ==================== */
.guide-section {
  margin-bottom: 24px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 16px;
  padding: 24px;
  background: var(--el-bg-color);
}
.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 20px;
}
.section-header h2 { margin: 0; font-size: 20px; font-weight: 700; color: var(--el-text-color-primary); }
.section-badge {
  margin-left: auto;
  padding: 2px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  background: var(--el-fill-color-light);
  color: var(--el-text-color-secondary);
}
.section-header.violet { color: #8b5cf6; }
.section-header.blue { color: #3b82f6; }
.section-header.purple { color: #7c3aed; }
.section-header.green { color: #22c55e; }
.section-header.pink { color: #ec4899; }
.section-header.orange { color: #f97316; }
.section-header.sky { color: #0ea5e9; }
.section-header.teal { color: #14b8a6; }
.section-header.cyan { color: #06b6d4; }
.section-header.emerald { color: #10b981; }
.section-header.gold { color: #eab308; }
.section-header.indigo { color: #6366f1; }
.section-header.lavender { color: #a855f7; }

.section-desc { color: var(--el-text-color-secondary); font-size: 14px; margin: 0 0 16px; }
.sub-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 20px 0 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color-extra-light);
}
.note { font-size: 12px; color: var(--el-text-color-secondary); margin-top: 8px; }

/* ==================== 布局工具 ==================== */
.two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.three-col { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.info-card {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 12px;
  padding: 16px;
  background: var(--el-fill-color-blank);
}
.info-card h3 {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 0 10px;
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.info-card p { margin: 6px 0; font-size: 13px; color: var(--el-text-color-regular); line-height: 1.7; }
.rule-list { margin: 0; padding-left: 16px; }
.rule-list li { font-size: 13px; color: var(--el-text-color-regular); line-height: 1.8; }
.inline-tags { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 4px; }

/* ==================== 赛季阶段 ==================== */
.phase-timeline { display: flex; flex-direction: column; gap: 16px; }
.phase-group-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--el-text-color-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 8px;
}
.phase-group-items { display: flex; flex-wrap: wrap; gap: 8px; }
.phase-card {
  min-width: 150px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
  transition: transform 0.2s, border-color 0.2s;
}
.phase-card:hover { transform: translateY(-2px); border-color: var(--el-color-primary-light-3); }
.phase-card.spring { border-left: 3px solid #4ade80; }
.phase-card.summer { border-left: 3px solid #fbbf24; }
.phase-card.intl { border-left: 3px solid #60a5fa; }
.phase-card.offseason { border-left: 3px solid #f87171; }
.phase-card.end { border-left: 3px solid #c084fc; }
.phase-num { font-size: 11px; color: var(--el-text-color-placeholder); }
.phase-name { font-size: 13px; font-weight: 600; margin-top: 2px; }
.phase-sub { font-size: 11px; color: var(--el-text-color-secondary); margin-top: 2px; }

/* ==================== 联赛 ==================== */
.bracket-mini { margin-top: 12px; display: flex; flex-direction: column; gap: 8px; }
.bracket-row { display: flex; align-items: center; gap: 10px; }
.bracket-label {
  min-width: 80px;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  text-align: center;
}
.bracket-label.winners { background: #dcfce7; color: #166534; }
.bracket-label.losers { background: #fee2e2; color: #991b1b; }
.bracket-label.final { background: #fef9c3; color: #854d0e; }
.bracket-matches { font-size: 13px; color: var(--el-text-color-regular); }
.seed { font-weight: 700; color: var(--el-color-primary); }
.seed.winner { color: #16a34a; }
.seed.loser { color: #dc2626; }
.sep { color: var(--el-border-color); margin: 0 6px; }

/* ==================== 赛事时间线 ==================== */
.event-timeline-bar {
  display: flex;
  justify-content: center;
  gap: 4px;
  margin-bottom: 16px;
}
.timeline-dot {
  padding: 6px 14px;
  border-radius: 99px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
  color: var(--el-text-color-secondary);
  transition: all 0.2s;
}
.timeline-dot:hover { border-color: var(--el-color-primary-light-3); }
.timeline-dot.active { background: var(--el-color-primary); color: white; border-color: var(--el-color-primary); }
.timeline-dot.active.worlds { background: #f59e0b; border-color: #f59e0b; }
.timeline-dot.active.super { background: #8b5cf6; border-color: #8b5cf6; }

.event-detail-card {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 12px;
  padding: 20px;
  background: var(--el-fill-color-blank);
}
.event-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}
.event-name { font-size: 18px; font-weight: 700; }
.event-timing { font-size: 12px; color: var(--el-text-color-secondary); margin-top: 2px; }
.event-badges { display: flex; gap: 6px; flex-wrap: wrap; }
.badge {
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
}
.badge.teams { background: #dbeafe; color: #1e40af; }
.badge.format { background: #e0e7ff; color: #4338ca; }
.badge.highlight { background: #fef3c7; color: #92400e; }

.event-grid { display: grid; grid-template-columns: 1fr 1.2fr 1fr; gap: 14px; }
.event-block {
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-extra-light);
  background: var(--el-fill-color-lighter);
}
.block-title { font-size: 12px; font-weight: 700; color: var(--el-text-color-secondary); margin-bottom: 8px; }
.event-block p { margin: 0; font-size: 13px; line-height: 1.7; color: var(--el-text-color-regular); }
.event-block ul { margin: 0; padding-left: 14px; }
.event-block li { font-size: 13px; line-height: 1.7; color: var(--el-text-color-regular); }

.points-row { display: flex; align-items: center; gap: 8px; padding: 3px 0; font-size: 13px; }
.reward-rank { min-width: 48px; font-weight: 700; }
.reward-rank.gold { color: #f59e0b; }
.reward-rank.silver { color: #94a3b8; }
.reward-rank.bronze { color: #d97706; }
.reward-pts { min-width: 40px; color: var(--el-color-primary); font-weight: 600; }
.reward-prize { color: var(--el-text-color-secondary); }

/* ==================== 积分表格 ==================== */
.pts-table { display: flex; flex-direction: column; gap: 6px; }
.pts-row { display: flex; align-items: center; gap: 12px; font-size: 13px; padding: 4px 0; }
.pts-row.header { font-weight: 700; color: var(--el-text-color-secondary); font-size: 12px; border-bottom: 1px solid var(--el-border-color-extra-light); padding-bottom: 8px; }
.pts-row span { min-width: 60px; }
.pts-rank { font-weight: 600; min-width: 60px; }

/* ==================== 选手属性 ==================== */
.attr-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.attr-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
}
.attr-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: white;
}
.attr-icon.ability { background: linear-gradient(135deg, #ef4444, #f97316); }
.attr-icon.potential { background: linear-gradient(135deg, #8b5cf6, #a78bfa); }
.attr-icon.stability { background: linear-gradient(135deg, #3b82f6, #60a5fa); }
.attr-icon.loyalty { background: linear-gradient(135deg, #f59e0b, #fbbf24); }
.attr-icon.condition { background: linear-gradient(135deg, #22c55e, #4ade80); }
.attr-icon.age { background: linear-gradient(135deg, #64748b, #94a3b8); }
.attr-name { font-size: 13px; font-weight: 600; }
.attr-desc { font-size: 11px; color: var(--el-text-color-secondary); }

/* ==================== 成长标签 ==================== */
.tag-card {
  padding: 14px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  display: flex;
  flex-direction: column;
  gap: 6px;
  text-align: center;
}
.tag-card.genius { border-color: #fbbf24; background: #fffbeb; }
.tag-card.normal { border-color: #60a5fa; background: #eff6ff; }
.tag-card.ordinary { border-color: #94a3b8; background: #f8fafc; }
.tag-label { font-size: 15px; font-weight: 700; }
.tag-card.genius .tag-label { color: #b45309; }
.tag-card.normal .tag-label { color: #2563eb; }
.tag-card.ordinary .tag-label { color: #64748b; }
.tag-effect { font-size: 13px; font-weight: 600; color: var(--el-text-color-regular); }
.tag-note { font-size: 11px; color: var(--el-text-color-secondary); }

/* ==================== 生命周期 ==================== */
.lifecycle-bar { display: flex; gap: 2px; border-radius: 10px; overflow: hidden; }
.lc-phase {
  flex: 1;
  padding: 12px;
  text-align: center;
  color: white;
}
.lc-phase.growth { background: linear-gradient(135deg, #22c55e, #4ade80); }
.lc-phase.peak { background: linear-gradient(135deg, #3b82f6, #60a5fa); }
.lc-phase.decline { background: linear-gradient(135deg, #f97316, #fb923c); }
.lc-phase.retire { background: linear-gradient(135deg, #ef4444, #f87171); }
.lc-title { font-size: 14px; font-weight: 700; }
.lc-age { font-size: 12px; opacity: 0.9; margin-top: 2px; }
.lc-detail { font-size: 11px; opacity: 0.8; margin-top: 2px; }

/* ==================== 公式框 ==================== */
.formula-box {
  padding: 14px 18px;
  border-radius: 10px;
  background: #1e293b;
  margin: 8px 0;
}
.formula-box.compact { padding: 10px 14px; }
.formula {
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 13px;
  color: #e2e8f0;
  line-height: 1.6;
}
.formula-note {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 4px;
}

/* ==================== 系数 ==================== */
.coef-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.coef-card {
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
}
.coef-title { font-size: 12px; font-weight: 700; color: var(--el-text-color-secondary); margin-bottom: 8px; }
.coef-items { display: flex; flex-wrap: wrap; gap: 6px; }
.coef-items span {
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 11px;
  background: var(--el-fill-color-light);
  color: var(--el-text-color-regular);
}

/* ==================== 特性系统 ==================== */
.trait-category { margin-bottom: 14px; }
.trait-cat-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;
  padding-bottom: 4px;
  border-bottom: 1px dashed var(--el-border-color-extra-light);
}
.trait-list { display: flex; flex-wrap: wrap; gap: 8px; }
.trait-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
  font-size: 13px;
}
.trait-name { font-weight: 700; color: var(--el-color-primary); }
.trait-effect { color: var(--el-text-color-regular); }
.trait-trigger { font-size: 11px; color: var(--el-text-color-placeholder); }

/* ==================== 比赛模拟 ==================== */
.sim-layers { display: flex; flex-direction: column; align-items: center; gap: 0; }
.sim-layer {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  width: 100%;
  max-width: 600px;
  padding: 14px;
  border-radius: 12px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
}
.layer-num {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--el-color-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  font-size: 14px;
  flex-shrink: 0;
}
.layer-body { flex: 1; }
.layer-title { font-size: 14px; font-weight: 700; margin-bottom: 6px; }
.layer-arrow { color: var(--el-text-color-placeholder); font-size: 18px; padding: 4px 0; }

/* 胜率 */
.winrate-grid { display: flex; flex-direction: column; gap: 8px; max-width: 500px; }
.wr-item { display: flex; align-items: center; gap: 10px; }
.wr-diff { min-width: 60px; font-size: 13px; color: var(--el-text-color-secondary); }
.wr-bar { flex: 1; height: 8px; border-radius: 99px; background: var(--el-fill-color-light); overflow: hidden; }
.wr-fill { height: 100%; border-radius: 99px; background: linear-gradient(90deg, #3b82f6, #8b5cf6); }
.wr-rate { min-width: 40px; font-size: 13px; font-weight: 700; color: var(--el-color-primary); }

/* ==================== Meta ==================== */
.meta-examples { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.meta-card {
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
}
.meta-name { font-size: 14px; font-weight: 700; margin-bottom: 8px; }
.meta-weights { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 6px; }
.meta-w {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-family: monospace;
  background: var(--el-fill-color-light);
  color: var(--el-text-color-secondary);
}
.meta-w.hot { background: #fef3c7; color: #b45309; font-weight: 700; }
.meta-w.cold { background: #eff6ff; color: #6b7280; }
.meta-focus { font-size: 11px; color: var(--el-text-color-placeholder); }

/* ==================== 转会 ==================== */
.round-flow { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.round-card {
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
}
.round-num { font-size: 12px; font-weight: 800; color: var(--el-color-primary); }
.round-name { font-size: 13px; font-weight: 700; margin-top: 2px; }
.round-desc { font-size: 11px; color: var(--el-text-color-secondary); margin-top: 4px; }

.gm-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 8px; }
.gm-card {
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
  text-align: center;
}
.gm-name { font-size: 14px; font-weight: 700; margin-bottom: 4px; }
.gm-card.championship .gm-name { color: #dc2626; }
.gm-card.youth .gm-name { color: #22c55e; }
.gm-card.balanced .gm-name { color: #3b82f6; }
.gm-card.speculator .gm-name { color: #f97316; }
.gm-card.rebuilding .gm-name { color: #8b5cf6; }
.gm-desc { font-size: 11px; color: var(--el-text-color-secondary); margin-bottom: 8px; }
.gm-stats { display: flex; flex-direction: column; gap: 2px; }
.gm-stats span { font-size: 11px; color: var(--el-text-color-regular); }

/* 忠诚度条 */
.loyalty-bars { display: flex; flex-direction: column; gap: 8px; max-width: 500px; }
.loyalty-row { display: flex; align-items: center; gap: 10px; }
.loyalty-label { min-width: 100px; font-size: 13px; color: var(--el-text-color-regular); }
.loyalty-track { flex: 1; height: 8px; border-radius: 99px; background: var(--el-fill-color-light); overflow: hidden; }
.loyalty-fill { height: 100%; border-radius: 99px; background: linear-gradient(90deg, #22c55e, #3b82f6); }
.loyalty-value { min-width: 60px; font-size: 12px; font-weight: 700; color: var(--el-text-color-primary); }

/* ==================== 财政 ==================== */
.finance-list { display: flex; flex-direction: column; gap: 8px; }
.fl-item { display: flex; justify-content: space-between; align-items: center; padding: 6px 0; border-bottom: 1px solid var(--el-border-color-extra-light); }
.fl-label { font-size: 13px; color: var(--el-text-color-regular); }
.fl-value { font-size: 13px; font-weight: 600; }
.fl-value.income { color: #22c55e; }
.fl-value.expense { color: #ef4444; }

.prize-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.prize-card {
  padding: 14px;
  border-radius: 10px;
  border: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-blank);
  text-align: center;
}
.prize-event { font-size: 14px; font-weight: 700; }
.prize-total { font-size: 12px; color: var(--el-text-color-secondary); margin-top: 4px; }
.prize-champion { font-size: 13px; font-weight: 700; color: #f59e0b; margin-top: 2px; }

.status-bar { display: flex; gap: 2px; border-radius: 10px; overflow: hidden; margin-top: 8px; }
.status-item { flex: 1; padding: 10px; text-align: center; color: white; }
.status-item.wealthy { background: #22c55e; }
.status-item.healthy { background: #3b82f6; }
.status-item.tight { background: #f59e0b; }
.status-item.deficit { background: #f97316; }
.status-item.bankrupt { background: #ef4444; }
.fs-name { font-size: 13px; font-weight: 700; }
.fs-range { font-size: 11px; opacity: 0.85; margin-top: 2px; }
.fs-strategy { font-size: 10px; opacity: 0.75; margin-top: 2px; }

/* ==================== 荣誉 ==================== */
.honor-grid { display: flex; flex-wrap: wrap; gap: 6px; }
.honor-item {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  background: var(--el-fill-color-light);
  color: var(--el-text-color-regular);
}
.bonus-list { display: flex; flex-direction: column; gap: 6px; }
.bonus-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid var(--el-border-color-extra-light);
  font-size: 13px;
}
.bonus-value { font-weight: 700; color: #f59e0b; }
.bonus-dur { font-size: 11px; color: var(--el-text-color-placeholder); }

/* ==================== 数据中心 ==================== */
.top20-tiers { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
.tier {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
}
.tier.t1 { background: #fef3c7; color: #92400e; }
.tier.t2 { background: #e0e7ff; color: #4338ca; }
.tier.t3 { background: #f0fdf4; color: #166534; }

/* ==================== 选秀 ==================== */
.draft-stats { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.ds-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 8px;
  background: var(--el-fill-color-light);
  font-size: 13px;
}
.ds-item span { color: var(--el-text-color-secondary); }
.ds-item strong { color: var(--el-text-color-primary); }
.draft-salary-note {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  padding: 6px 10px;
  background: var(--el-fill-color-lighter);
  border-radius: 6px;
  line-height: 1.5;
}

/* ==================== 响应式 ==================== */
@media (max-width: 1100px) {
  .hero-banner { grid-template-columns: 1fr; }
  .event-grid { grid-template-columns: 1fr; }
  .meta-examples { grid-template-columns: repeat(2, 1fr); }
  .gm-grid { grid-template-columns: repeat(3, 1fr); }
  .round-flow { grid-template-columns: repeat(2, 1fr); }
  .prize-grid { grid-template-columns: repeat(2, 1fr); }
  .coef-grid { grid-template-columns: 1fr; }
  .attr-grid { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 768px) {
  .two-col, .three-col { grid-template-columns: 1fr; }
  .meta-examples, .gm-grid, .round-flow, .prize-grid { grid-template-columns: 1fr; }
  .attr-grid { grid-template-columns: 1fr; }
  .lifecycle-bar { flex-direction: column; }
  .status-bar { flex-direction: column; }
  .event-timeline-bar { flex-wrap: wrap; }
  .guide-nav { gap: 4px; }
  .nav-link { padding: 3px 8px; font-size: 11px; }
}
</style>
