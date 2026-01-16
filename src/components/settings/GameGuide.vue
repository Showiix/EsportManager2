<template>
  <div class="game-guide">
    <el-collapse v-model="activeNames" accordion>
      <!-- 游戏概述 -->
      <el-collapse-item name="overview">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #6366f1;"><Monitor /></el-icon>
            <span>游戏概述</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="overview-intro">
            <p class="guide-desc">
              <strong>电竞经理模拟器 2</strong> 是一款基于 <el-tag size="small">Tauri</el-tag> + <el-tag size="small">Vue3</el-tag> + <el-tag size="small">Rust</el-tag> 的桌面端电竞赛事模拟游戏。
            </p>
            <p class="guide-desc">
              玩家管理 <strong>4 个赛区</strong>（LPL/LCK/LEC/LCS）共 <strong>56 支战队</strong>，体验从春季赛到年度 Super 洲际邀请赛的完整赛事周期。
            </p>
          </div>

          <div class="section">
            <h4><el-icon><Collection /></el-icon> 核心系统</h4>
            <div class="system-grid">
              <div class="system-item">
                <div class="system-icon time"><el-icon><Clock /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">时间推进系统</div>
                  <div class="system-desc">控制赛季流转，14个阶段循环</div>
                </div>
              </div>
              <div class="system-item">
                <div class="system-icon match"><el-icon><VideoPlay /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">比赛模拟引擎</div>
                  <div class="system-desc">三层正态分布模型计算胜负</div>
                </div>
              </div>
              <div class="system-item">
                <div class="system-icon honor"><el-icon><Medal /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">荣誉系统</div>
                  <div class="system-desc">赛事结束自动颁发荣誉</div>
                </div>
              </div>
              <div class="system-item">
                <div class="system-icon points"><el-icon><DataAnalysis /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">年度积分系统</div>
                  <div class="system-desc">决定Super赛参赛资格</div>
                </div>
              </div>
              <div class="system-item">
                <div class="system-icon data"><el-icon><TrendCharts /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">数据中心系统</div>
                  <div class="system-desc">选手表现数据与MVP评选</div>
                </div>
              </div>
              <div class="system-item">
                <div class="system-icon transfer"><el-icon><Switch /></el-icon></div>
                <div class="system-info">
                  <div class="system-name">转会系统</div>
                  <div class="system-desc">选手合同与AI转会决策</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Aim /></el-icon> 游戏目标</h4>
            <div class="goals-list">
              <div class="goal-item">
                <el-icon class="goal-icon"><Trophy /></el-icon>
                <span>带领你的战队赢得 <strong>S 世界赛冠军</strong></span>
              </div>
              <div class="goal-item">
                <el-icon class="goal-icon"><StarFilled /></el-icon>
                <span>累积年度积分，参加 <strong>Super 洲际邀请赛</strong></span>
              </div>
              <div class="goal-item">
                <el-icon class="goal-icon"><Medal /></el-icon>
                <span>培养明星选手，建立 <strong>荣誉殿堂</strong></span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 赛季周期 -->
      <el-collapse-item name="season">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #f59e0b;"><Calendar /></el-icon>
            <span>赛季周期</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">游戏时间按 S1、S2、S3... 推进，每个赛季包含以下阶段：</p>

          <!-- 赛季流程图 -->
          <div class="season-flow">
            <div class="flow-container">
              <div class="flow-item spring">
                <div class="flow-icon">
                  <el-icon><Sunny /></el-icon>
                </div>
                <div class="flow-label">春季联赛</div>
                <div class="flow-sub">常规赛+季后赛</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item msi">
                <div class="flow-icon">
                  <el-icon><Trophy /></el-icon>
                </div>
                <div class="flow-label">MSI季中赛</div>
                <div class="flow-sub">12队双败淘汰</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item madrid">
                <div class="flow-icon">
                  <el-icon><Flag /></el-icon>
                </div>
                <div class="flow-label">马德里大师赛</div>
                <div class="flow-sub">32队小组+淘汰</div>
              </div>
            </div>

            <div class="flow-container">
              <div class="flow-item summer">
                <div class="flow-icon">
                  <el-icon><Sunny /></el-icon>
                </div>
                <div class="flow-label">夏季联赛</div>
                <div class="flow-sub">常规赛+季后赛</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item clauch">
                <div class="flow-icon">
                  <el-icon><Promotion /></el-icon>
                </div>
                <div class="flow-label">C洲际赛</div>
                <div class="flow-sub">32队小组+淘汰</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item worlds">
                <div class="flow-icon">
                  <el-icon><Trophy /></el-icon>
                </div>
                <div class="flow-label">S世界赛</div>
                <div class="flow-sub">瑞士轮+淘汰赛</div>
              </div>
            </div>

            <div class="flow-container">
              <div class="flow-item shanghai">
                <div class="flow-icon">
                  <el-icon><Star /></el-icon>
                </div>
                <div class="flow-label">上海大师赛</div>
                <div class="flow-sub">12队双败淘汰</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item icp">
                <div class="flow-icon">
                  <el-icon><Link /></el-icon>
                </div>
                <div class="flow-label">洲际对抗赛</div>
                <div class="flow-sub">4赛区对抗</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item super">
                <div class="flow-icon">
                  <el-icon><StarFilled /></el-icon>
                </div>
                <div class="flow-label">Super洲际杯</div>
                <div class="flow-sub">年度前16队</div>
              </div>
            </div>

            <div class="flow-container last">
              <div class="flow-item transfer">
                <div class="flow-icon">
                  <el-icon><Switch /></el-icon>
                </div>
                <div class="flow-label">转会期</div>
                <div class="flow-sub">买卖选手</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item draft">
                <div class="flow-icon">
                  <el-icon><User /></el-icon>
                </div>
                <div class="flow-label">选秀</div>
                <div class="flow-sub">每4赛季一次</div>
              </div>
              <div class="flow-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="flow-item end">
                <div class="flow-icon">
                  <el-icon><CircleCheck /></el-icon>
                </div>
                <div class="flow-label">赛季结束</div>
                <div class="flow-sub">进入下赛季</div>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 联赛体系 -->
      <el-collapse-item name="league">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #3b82f6;"><Aim /></el-icon>
            <span>联赛体系</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="section">
            <h4><el-icon><List /></el-icon> 常规赛</h4>
            <ul class="rule-list">
              <li><strong>赛制：</strong>BO3双循环，每队与其他13队各交手2次</li>
              <li><strong>积分规则：</strong>
                <div class="inline-tags">
                  <el-tag type="success">2:0胜 +3分</el-tag>
                  <el-tag type="primary">2:1胜 +2分</el-tag>
                  <el-tag type="warning">1:2负 +1分</el-tag>
                  <el-tag type="danger">0:2负 +0分</el-tag>
                </div>
              </li>
              <li><strong>排名依据：</strong>积分 → 历史交锋胜率 → 总净胜小局</li>
            </ul>
          </div>

          <div class="section">
            <h4><el-icon><Trophy /></el-icon> 季后赛（双败淘汰BO5）</h4>
            <p>常规赛前8名进入季后赛，1-4名进胜者组，5-8名进败者组</p>

            <div class="bracket-visual">
              <div class="bracket-stage">
                <div class="stage-title">胜者组第一轮</div>
                <div class="match-pair">
                  <span class="seed">1</span> vs <span class="seed">4</span>
                </div>
                <div class="match-pair">
                  <span class="seed">2</span> vs <span class="seed">3</span>
                </div>
              </div>
              <div class="bracket-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="bracket-stage">
                <div class="stage-title">败者组第一轮</div>
                <div class="match-pair">
                  <span class="seed">5</span> vs <span class="seed">8</span>
                </div>
                <div class="match-pair">
                  <span class="seed">6</span> vs <span class="seed">7</span>
                </div>
              </div>
              <div class="bracket-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
              <div class="bracket-stage final">
                <div class="stage-title">总决赛</div>
                <div class="match-pair">
                  <span class="seed winner">胜者组冠军</span>
                  <span>vs</span>
                  <span class="seed loser">败者组冠军</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 国际赛事 -->
      <el-collapse-item name="international">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #8b5cf6;"><Trophy /></el-icon>
            <span>国际赛事</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">每赛季共有7项国际赛事，按时间顺序举行</p>

          <el-tabs v-model="activeEventTab" type="card">
            <!-- MSI季中邀请赛 -->
            <el-tab-pane label="MSI" name="msi">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">MSI 季中邀请赛</span>
                    <span class="event-timing">春季赛后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">12队</span>
                    <span class="event-format">双败淘汰BO5</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>春季赛季后赛</strong>冠军、亚军、季军（共12队）</p>
                  <div class="qualification-grid">
                    <div class="qual-item">
                      <span class="qual-region">LPL</span>
                      <span class="qual-slots">冠/亚/季军</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LCK</span>
                      <span class="qual-slots">冠/亚/季军</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LEC</span>
                      <span class="qual-slots">冠/亚/季军</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LCS</span>
                      <span class="qual-slots">冠/亚/季军</span>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <div class="bracket-explanation">
                    <div class="bracket-part winners">
                      <div class="bracket-label">胜者组</div>
                      <div class="bracket-desc">
                        <p>12队分为3组抽签对阵</p>
                        <p>传奇组（4冠军）、挑战者组（4亚军）、资格赛组（4季军）</p>
                        <p>胜者组失败后进入败者组</p>
                      </div>
                    </div>
                    <div class="bracket-part losers">
                      <div class="bracket-label">败者组</div>
                      <div class="bracket-desc">
                        <p>败者组失败即淘汰</p>
                        <p>败者组冠军与胜者组冠军争夺总冠军</p>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+20</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+16</span></div>
                    <div class="point-item bronze"><span class="rank">季军</span><span class="pts">+12</span></div>
                    <div class="point-item fourth"><span class="rank">殿军</span><span class="pts">+8</span></div>
                    <div class="point-item other"><span class="rank">5-6名</span><span class="pts">+6</span></div>
                    <div class="point-item other"><span class="rank">7-8名</span><span class="pts">+4</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- 马德里大师赛 -->
            <el-tab-pane label="马德里" name="madrid">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">马德里大师赛</span>
                    <span class="event-timing">MSI后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">32队</span>
                    <span class="event-format">小组赛+淘汰赛</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>春季赛常规赛</strong>前8名（共32队）</p>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <div class="stage-detail-box">
                    <div class="stage-item">
                      <div class="stage-num">1</div>
                      <div class="stage-info">
                        <div class="stage-name">小组赛</div>
                        <ul>
                          <li>32队分为<strong>8组</strong>，每组4队</li>
                          <li>同赛区队伍分入不同组（蛇形分组）</li>
                          <li><strong>BO3单循环</strong>，每队打3场</li>
                          <li>小组前2名（共16队）晋级淘汰赛</li>
                        </ul>
                      </div>
                    </div>
                    <div class="stage-item">
                      <div class="stage-num">2</div>
                      <div class="stage-info">
                        <div class="stage-name">淘汰赛</div>
                        <ul>
                          <li>16队分为<strong>东西两个半区</strong></li>
                          <li>小组第1 vs 另一组第2（交叉对阵）</li>
                          <li><strong>BO5单淘汰</strong></li>
                          <li>两个半区冠军会师总决赛</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+30</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+24</span></div>
                    <div class="point-item bronze"><span class="rank">3-4名</span><span class="pts">+18</span></div>
                    <div class="point-item fourth"><span class="rank">5-8名</span><span class="pts">+12</span></div>
                    <div class="point-item other"><span class="rank">9-16名</span><span class="pts">+8</span></div>
                    <div class="point-item other"><span class="rank">17-32名</span><span class="pts">+4</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- Claude洲际赛 -->
            <el-tab-pane label="C洲际" name="claude">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">Claude 洲际赛</span>
                    <span class="event-timing">夏季赛季后赛后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">32队</span>
                    <span class="event-format">小组赛+淘汰赛</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>夏季赛常规赛</strong>前8名（共32队）</p>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <p>赛制与马德里大师赛相同：小组赛（8组×4队BO3）→ 淘汰赛（16强BO5）</p>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+30</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+24</span></div>
                    <div class="point-item bronze"><span class="rank">3-4名</span><span class="pts">+18</span></div>
                    <div class="point-item fourth"><span class="rank">5-8名</span><span class="pts">+12</span></div>
                    <div class="point-item other"><span class="rank">9-16名</span><span class="pts">+8</span></div>
                    <div class="point-item other"><span class="rank">17-32名</span><span class="pts">+4</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- S世界赛 -->
            <el-tab-pane label="S世界赛" name="worlds">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">S 世界赛</span>
                    <span class="event-timing">C洲际赛后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">16队</span>
                    <span class="event-format">瑞士轮+淘汰赛</span>
                    <span class="event-highlight">年度最高荣誉</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>夏季赛季后赛</strong>前4名（共16队）</p>
                  <div class="qualification-grid">
                    <div class="qual-item">
                      <span class="qual-region">LPL</span>
                      <span class="qual-slots">1-4名</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LCK</span>
                      <span class="qual-slots">1-4名</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LEC</span>
                      <span class="qual-slots">1-4名</span>
                    </div>
                    <div class="qual-item">
                      <span class="qual-region">LCS</span>
                      <span class="qual-slots">1-4名</span>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则 - 瑞士轮</div>
                  <div class="swiss-explanation">
                    <div class="swiss-rule">
                      <div class="swiss-icon win">3胜</div>
                      <div class="swiss-desc">累计<strong>3胜</strong>晋级淘汰赛（共8队）</div>
                    </div>
                    <div class="swiss-rule">
                      <div class="swiss-icon lose">3败</div>
                      <div class="swiss-desc">累计<strong>3败</strong>淘汰出局（共8队）</div>
                    </div>
                  </div>
                  <div class="swiss-rounds">
                    <div class="round-item">
                      <span class="round-num">R1</span>
                      <span class="round-desc">16队 → 8胜8败</span>
                    </div>
                    <div class="round-item">
                      <span class="round-num">R2</span>
                      <span class="round-desc">同战绩对阵</span>
                    </div>
                    <div class="round-item">
                      <span class="round-num">R3</span>
                      <span class="round-desc">2-0 vs 2-0 (晋级战)</span>
                    </div>
                    <div class="round-item">
                      <span class="round-num">R4</span>
                      <span class="round-desc">0-2 vs 0-2 (淘汰战)</span>
                    </div>
                    <div class="round-item">
                      <span class="round-num">R5</span>
                      <span class="round-desc">最后晋级/淘汰战</span>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则 - 淘汰赛</div>
                  <div class="knockout-bracket">
                    <div class="ko-stage">
                      <div class="ko-name">八强</div>
                      <div class="ko-format">BO5</div>
                    </div>
                    <el-icon><ArrowRight /></el-icon>
                    <div class="ko-stage">
                      <div class="ko-name">半决赛</div>
                      <div class="ko-format">BO5</div>
                    </div>
                    <el-icon><ArrowRight /></el-icon>
                    <div class="ko-stage final">
                      <div class="ko-name">总决赛</div>
                      <div class="ko-format">BO5</div>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution worlds">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+100</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+80</span></div>
                    <div class="point-item bronze"><span class="rank">3-4名</span><span class="pts">+60</span></div>
                    <div class="point-item fourth"><span class="rank">5-8名</span><span class="pts">+40</span></div>
                    <div class="point-item other"><span class="rank">9-16名</span><span class="pts">+20</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- 上海大师赛 -->
            <el-tab-pane label="上海" name="shanghai">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">上海大师赛</span>
                    <span class="event-timing">S世界赛后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">12队</span>
                    <span class="event-format">双败淘汰BO5</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>夏季赛季后赛</strong>冠军、亚军、季军（共12队）</p>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <p>赛制与MSI相同：12队双败淘汰BO5</p>
                  <div class="bracket-explanation">
                    <div class="bracket-part winners">
                      <div class="bracket-label">胜者组</div>
                      <div class="bracket-desc">
                        <p>传奇组（4冠军）、挑战者组（4亚军）、资格赛组（4季军）</p>
                      </div>
                    </div>
                    <div class="bracket-part losers">
                      <div class="bracket-label">败者组</div>
                      <div class="bracket-desc">
                        <p>胜者组落败队伍进入败者组继续比赛</p>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+20</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+16</span></div>
                    <div class="point-item bronze"><span class="rank">季军</span><span class="pts">+12</span></div>
                    <div class="point-item fourth"><span class="rank">殿军</span><span class="pts">+8</span></div>
                    <div class="point-item other"><span class="rank">5-6名</span><span class="pts">+6</span></div>
                    <div class="point-item other"><span class="rank">7-8名</span><span class="pts">+4</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- ICP洲际对抗赛 -->
            <el-tab-pane label="ICP" name="icp">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">ICP 洲际对抗赛</span>
                    <span class="event-timing">上海大师赛后</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">24队</span>
                    <span class="event-format">赛区对抗</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>各赛区<strong>夏季赛常规赛</strong>前6名（共24队）</p>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <div class="icp-format">
                    <div class="icp-stage">
                      <div class="icp-stage-name">小组赛阶段</div>
                      <ul>
                        <li>24队分为<strong>4组</strong>，每组6队（每赛区各1-2队）</li>
                        <li><strong>BO1单循环</strong>，每队打5场</li>
                        <li>小组前2名（共8队）晋级淘汰赛</li>
                      </ul>
                    </div>
                    <div class="icp-stage">
                      <div class="icp-stage-name">淘汰赛阶段</div>
                      <ul>
                        <li>8强<strong>BO5单淘汰</strong></li>
                      </ul>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+25</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+20</span></div>
                    <div class="point-item bronze"><span class="rank">3-4名</span><span class="pts">+15</span></div>
                    <div class="point-item fourth"><span class="rank">5-8名</span><span class="pts">+10</span></div>
                    <div class="point-item other"><span class="rank">9-16名</span><span class="pts">+6</span></div>
                    <div class="point-item other"><span class="rank">17-24名</span><span class="pts">+3</span></div>
                  </div>
                </div>
              </div>
            </el-tab-pane>

            <!-- Super洲际邀请赛 -->
            <el-tab-pane label="Super" name="super">
              <div class="event-card detailed">
                <div class="event-header">
                  <div class="event-title-row">
                    <span class="event-name">Super 洲际年度邀请赛</span>
                    <span class="event-timing">赛季末</span>
                  </div>
                  <div class="event-tags">
                    <span class="event-teams">16队</span>
                    <span class="event-format">分组+淘汰赛</span>
                    <span class="event-highlight">年度收官战</span>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">参赛资格</div>
                  <p>全球<strong>年度积分排名前16名</strong>战队</p>
                  <div class="super-qualification">
                    <div class="super-qual-item">
                      <span class="super-qual-rank">1-4名</span>
                      <span class="super-qual-group">传奇组</span>
                      <span class="super-qual-desc">直接进入半决赛</span>
                    </div>
                    <div class="super-qual-item">
                      <span class="super-qual-rank">5-8名</span>
                      <span class="super-qual-group">挑战者组</span>
                      <span class="super-qual-desc">从八强开始</span>
                    </div>
                    <div class="super-qual-item">
                      <span class="super-qual-rank">9-16名</span>
                      <span class="super-qual-group">竞争者组</span>
                      <span class="super-qual-desc">从16强开始</span>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">赛制规则</div>
                  <div class="super-bracket">
                    <div class="super-stage">
                      <div class="super-stage-name">16强</div>
                      <div class="super-stage-detail">
                        <p>9-16名 vs 9-16名</p>
                        <p>BO5，胜者晋级</p>
                      </div>
                    </div>
                    <el-icon><ArrowRight /></el-icon>
                    <div class="super-stage">
                      <div class="super-stage-name">八强</div>
                      <div class="super-stage-detail">
                        <p>5-8名 vs 16强胜者</p>
                        <p>BO5，胜者晋级</p>
                      </div>
                    </div>
                    <el-icon><ArrowRight /></el-icon>
                    <div class="super-stage">
                      <div class="super-stage-name">半决赛</div>
                      <div class="super-stage-detail">
                        <p>1-4名 vs 八强胜者</p>
                        <p>BO5</p>
                      </div>
                    </div>
                    <el-icon><ArrowRight /></el-icon>
                    <div class="super-stage final">
                      <div class="super-stage-name">总决赛</div>
                      <div class="super-stage-detail">
                        <p>BO5</p>
                        <p>年度总冠军</p>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="event-section">
                  <div class="section-title">积分说明</div>
                  <div class="no-points-notice">
                    <el-icon class="notice-icon"><InfoFilled /></el-icon>
                    <div class="notice-content">
                      <div class="notice-title">Super 赛不颁发年度积分</div>
                      <div class="notice-desc">
                        Super 洲际邀请赛是全年表现的<strong>最终奖励</strong>，参加 Super 赛本身就是对年度积分的认可。
                        赛季结束后，年度积分将清零，开始新赛季的积分累积。
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </el-tab-pane>
          </el-tabs>

          <!-- 赛事时间线 -->
          <div class="event-timeline">
            <div class="timeline-title">赛事时间线</div>
            <div class="timeline-flow">
              <div class="timeline-item spring">
                <span class="timeline-event">春季赛</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item msi">
                <span class="timeline-event">MSI</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item madrid">
                <span class="timeline-event">马德里</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item summer">
                <span class="timeline-event">夏季赛</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item claude">
                <span class="timeline-event">C洲际</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item worlds">
                <span class="timeline-event">S世界赛</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item shanghai">
                <span class="timeline-event">上海</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item icp">
                <span class="timeline-event">ICP</span>
              </div>
              <el-icon><ArrowRight /></el-icon>
              <div class="timeline-item super">
                <span class="timeline-event">Super</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 积分体系 -->
      <el-collapse-item name="points">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #22c55e;"><DataAnalysis /></el-icon>
            <span>积分体系</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">年度积分决定Super洲际杯参赛资格（全球前16名）</p>

          <div class="points-tables">
            <div class="points-table">
              <h4>联赛季后赛积分</h4>
              <el-table :data="leaguePoints" size="small" stripe>
                <el-table-column prop="rank" label="名次" width="80" />
                <el-table-column prop="points" label="积分" width="80">
                  <template #default="{ row }">
                    <el-tag type="success">+{{ row.points }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>

            <div class="points-table">
              <h4>国际赛事积分</h4>
              <el-table :data="internationalPoints" size="small" stripe>
                <el-table-column prop="rank" label="名次" width="100" />
                <el-table-column prop="points" label="积分" width="80">
                  <template #default="{ row }">
                    <el-tag type="warning">+{{ row.points }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 荣誉系统 -->
      <el-collapse-item name="honor">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #eab308;"><Medal /></el-icon>
            <span>荣誉系统</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">荣誉系统在赛事结束时自动推断结果并颁发荣誉，记录战队和选手的辉煌成就。</p>

          <div class="section">
            <h4><el-icon><Trophy /></el-icon> 战队荣誉</h4>
            <div class="honor-grid">
              <div class="honor-item gold">
                <div class="honor-badge"><el-icon><Trophy /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">冠军</div>
                  <div class="honor-desc">赛事第一名</div>
                </div>
              </div>
              <div class="honor-item silver">
                <div class="honor-badge"><el-icon><Medal /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">亚军</div>
                  <div class="honor-desc">赛事第二名</div>
                </div>
              </div>
              <div class="honor-item bronze">
                <div class="honor-badge"><el-icon><Medal /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">季军</div>
                  <div class="honor-desc">赛事第三名</div>
                </div>
              </div>
              <div class="honor-item fourth">
                <div class="honor-badge"><el-icon><Medal /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">殿军</div>
                  <div class="honor-desc">赛事第四名</div>
                </div>
              </div>
              <div class="honor-item regular">
                <div class="honor-badge"><el-icon><Star /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">常规赛第一</div>
                  <div class="honor-desc">联赛常规赛冠军</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><User /></el-icon> 选手荣誉</h4>
            <div class="honor-grid">
              <div class="honor-item player-champion">
                <div class="honor-badge"><el-icon><Trophy /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">冠军选手</div>
                  <div class="honor-desc">冠军队伍首发选手</div>
                </div>
              </div>
              <div class="honor-item mvp">
                <div class="honor-badge"><el-icon><StarFilled /></el-icon></div>
                <div class="honor-info">
                  <div class="honor-name">赛事 MVP</div>
                  <div class="honor-desc">赛事最有价值选手</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Clock /></el-icon> 颁发时机</h4>
            <div class="timing-flow">
              <div class="timing-step">
                <div class="timing-icon"><el-icon><CircleCheck /></el-icon></div>
                <div class="timing-content">
                  <div class="timing-title">阶段完成</div>
                  <div class="timing-desc">所有比赛结束后点击"完成并推进"</div>
                </div>
              </div>
              <el-icon class="timing-arrow"><ArrowRight /></el-icon>
              <div class="timing-step">
                <div class="timing-icon"><el-icon><DataAnalysis /></el-icon></div>
                <div class="timing-content">
                  <div class="timing-title">结果推断</div>
                  <div class="timing-desc">系统自动计算冠亚季殿军</div>
                </div>
              </div>
              <el-icon class="timing-arrow"><ArrowRight /></el-icon>
              <div class="timing-step">
                <div class="timing-icon"><el-icon><Medal /></el-icon></div>
                <div class="timing-content">
                  <div class="timing-title">荣誉颁发</div>
                  <div class="timing-desc">创建战队和选手荣誉记录</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 荣誉对身价的影响 -->
          <div class="section">
            <h4><el-icon><Money /></el-icon> 荣誉对身价的影响</h4>
            <p class="guide-desc">荣誉会为选手身价带来加成，不同荣誉有不同的加成值和有效期。</p>

            <div class="honor-bonus-table">
              <div class="bonus-category">
                <div class="category-title">永久荣誉（国际赛事）</div>
                <div class="bonus-grid">
                  <div class="bonus-item worlds">
                    <span class="bonus-name">世界赛冠军</span>
                    <span class="bonus-value">+80%</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">世界赛亚军</span>
                    <span class="bonus-value">+40%</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">世界赛四强</span>
                    <span class="bonus-value">+20%</span>
                  </div>
                  <div class="bonus-item msi">
                    <span class="bonus-name">MSI冠军</span>
                    <span class="bonus-value">+50%</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">MSI亚军</span>
                    <span class="bonus-value">+25%</span>
                  </div>
                  <div class="bonus-item mvp">
                    <span class="bonus-name">世界赛MVP</span>
                    <span class="bonus-value">+40%</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">MSI MVP</span>
                    <span class="bonus-value">+30%</span>
                  </div>
                </div>
              </div>

              <div class="bonus-category">
                <div class="category-title">有时效荣誉（赛区赛事）</div>
                <div class="bonus-grid">
                  <div class="bonus-item regional">
                    <span class="bonus-name">赛区冠军</span>
                    <span class="bonus-value">+30%</span>
                    <span class="bonus-duration">3赛季</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">赛区亚军</span>
                    <span class="bonus-value">+15%</span>
                    <span class="bonus-duration">2赛季</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">赛区MVP</span>
                    <span class="bonus-value">+20%</span>
                    <span class="bonus-duration">2赛季</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">最佳阵容一阵</span>
                    <span class="bonus-value">+25%</span>
                    <span class="bonus-duration">2赛季</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">最佳阵容二阵</span>
                    <span class="bonus-value">+15%</span>
                    <span class="bonus-duration">2赛季</span>
                  </div>
                  <div class="bonus-item">
                    <span class="bonus-name">最佳阵容三阵</span>
                    <span class="bonus-value">+10%</span>
                    <span class="bonus-duration">2赛季</span>
                  </div>
                  <div class="bonus-item rookie">
                    <span class="bonus-name">年度最佳新秀</span>
                    <span class="bonus-value">+20%</span>
                    <span class="bonus-duration">3赛季</span>
                  </div>
                </div>
              </div>

              <div class="honor-formula">
                <div class="formula-title">荣誉系数计算</div>
                <div class="formula-content">
                  <div class="formula-box dark">
                    <div class="formula">荣誉系数 = 1.0 + Σ(有效荣誉加成)</div>
                    <div class="formula-note">上限 3.0（多冠王封顶）</div>
                  </div>
                  <div class="formula-example">
                    <span>例：世界赛冠军(+80%) + MSI冠军(+50%) + 赛区冠军(+30%)</span>
                    <span class="calc-result">= 1 + 0.8 + 0.5 + 0.3 = <strong>2.6倍身价</strong></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Reading /></el-icon> 荣誉殿堂</h4>
            <p>所有获得的荣誉都会永久保存，可以在"荣誉殿堂"页面查看：</p>
            <ul class="rule-list">
              <li><strong>战队荣誉排名</strong>：按获得荣誉数量排名</li>
              <li><strong>选手荣誉排名</strong>：按获得荣誉数量排名</li>
              <li><strong>赛事冠军榜</strong>：各赛事历届冠军</li>
            </ul>
          </div>
        </div>
      </el-collapse-item>

      <!-- 选手体系 -->
      <el-collapse-item name="player">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #ec4899;"><User /></el-icon>
            <span>选手体系</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="section">
            <h4><el-icon><DataLine /></el-icon> 选手属性</h4>
            <div class="attribute-grid">
              <div class="attr-item">
                <div class="attr-icon ability">
                  <el-icon><Aim /></el-icon>
                </div>
                <div class="attr-info">
                  <div class="attr-name">能力值</div>
                  <div class="attr-desc">选手当前实力（0-100）</div>
                </div>
              </div>
              <div class="attr-item">
                <div class="attr-icon potential">
                  <el-icon><TrendCharts /></el-icon>
                </div>
                <div class="attr-info">
                  <div class="attr-name">潜力值</div>
                  <div class="attr-desc">能力值上限</div>
                </div>
              </div>
              <div class="attr-item">
                <div class="attr-icon stability">
                  <el-icon><SetUp /></el-icon>
                </div>
                <div class="attr-info">
                  <div class="attr-name">稳定性</div>
                  <div class="attr-desc">发挥波动程度</div>
                </div>
              </div>
              <div class="attr-item">
                <div class="attr-icon age">
                  <el-icon><Clock /></el-icon>
                </div>
                <div class="attr-info">
                  <div class="attr-name">年龄</div>
                  <div class="attr-desc">16-30岁，影响稳定性</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><PriceTag /></el-icon> 选手标签</h4>
            <div class="tag-showcase">
              <div class="tag-item genius">
                <span class="tag-label">天才</span>
                <span class="tag-effect">每赛季 +3 能力</span>
              </div>
              <div class="tag-item normal">
                <span class="tag-label">一般</span>
                <span class="tag-effect">每赛季 +2 能力</span>
              </div>
              <div class="tag-item ordinary">
                <span class="tag-label">平庸</span>
                <span class="tag-effect">每赛季 +1 能力</span>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Histogram /></el-icon> 年龄与稳定性</h4>
            <div class="age-table">
              <div class="age-row header">
                <div>年龄段</div>
                <div>稳定性</div>
                <div>特点</div>
              </div>
              <div class="age-row young">
                <div>18-24岁</div>
                <div>60-75</div>
                <div>神经刀：上限高但波动大</div>
              </div>
              <div class="age-row peak">
                <div>25-29岁</div>
                <div>75-85</div>
                <div>巅峰期：稳定可靠</div>
              </div>
              <div class="age-row veteran">
                <div>30+岁</div>
                <div>85-95</div>
                <div>老将：极稳但难爆发</div>
              </div>
            </div>
          </div>

          <!-- 忠诚度系统 -->
          <div class="section">
            <h4><el-icon><StarFilled /></el-icon> 忠诚度系统</h4>
            <p class="guide-desc">忠诚度影响选手的离队意愿、转会费要求和续约谈判。</p>

            <div class="loyalty-table">
              <div class="loyalty-row header">
                <div>忠诚度</div>
                <div>类型</div>
                <div>离队阈值</div>
                <div>特点</div>
              </div>
              <div class="loyalty-row devoted">
                <div>90-100</div>
                <div>忠心耿耿</div>
                <div>满意度&lt;20</div>
                <div>几乎不会主动离队，可降薪30%</div>
              </div>
              <div class="loyalty-row loyal">
                <div>70-89</div>
                <div>忠诚</div>
                <div>满意度&lt;35</div>
                <div>倾向留队，40%拒绝挖角</div>
              </div>
              <div class="loyalty-row neutral">
                <div>50-69</div>
                <div>中立</div>
                <div>满意度&lt;50</div>
                <div>视情况决定，10%拒绝挖角</div>
              </div>
              <div class="loyalty-row opportunist">
                <div>30-49</div>
                <div>机会主义</div>
                <div>满意度&lt;60</div>
                <div>容易被挖角，要求涨薪20%</div>
              </div>
              <div class="loyalty-row mercenary">
                <div>0-29</div>
                <div>雇佣兵</div>
                <div>满意度&lt;70</div>
                <div>哪里给钱去哪里，要求涨薪30%</div>
              </div>
            </div>

            <div class="loyalty-effects">
              <div class="effect-item">
                <el-tag type="warning">转会费溢价</el-tag>
                <span>忠诚度80+要求130%身价，60-79要求115%</span>
              </div>
              <div class="effect-item">
                <el-tag type="success">老东家偏好</el-tag>
                <span>忠诚度80+自由球员优先考虑老东家续约(+30%吸引力)</span>
              </div>
            </div>
          </div>

          <!-- 满意度系统 -->
          <div class="section">
            <h4><el-icon><Star /></el-icon> 满意度系统</h4>
            <p class="guide-desc">满意度反映选手对当前状况的满意程度，低于离队阈值时想离开球队。</p>

            <div class="satisfaction-factors">
              <div class="factor-title">满意度影响因素：</div>
              <div class="factor-grid">
                <div class="factor-item positive">
                  <el-icon><Top /></el-icon>
                  <span>首发出场、球队夺冠、加薪</span>
                </div>
                <div class="factor-item negative">
                  <el-icon><Bottom /></el-icon>
                  <span>长期替补、球队战绩差、降薪</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 身价计算系统 -->
          <div class="section">
            <h4><el-icon><Money /></el-icon> 身价计算系统</h4>
            <p class="guide-desc">选手身价由基础身价和各种系数计算得出。</p>

            <div class="formula-box dark">
              <div class="formula">身价 = 能力值 × 基础系数 × 年龄系数 × 潜力系数 × 标签系数 × 位置系数 × 荣誉系数 × 赛区系数</div>
            </div>

            <!-- 基础系数 -->
            <div class="market-value-section">
              <div class="mv-title">基础系数（按能力值分档）</div>
              <div class="mv-table compact">
                <div class="mv-row header">
                  <div>能力值</div>
                  <div>系数</div>
                  <div>典型身价</div>
                </div>
                <div class="mv-row">
                  <div>95-100</div>
                  <div>×50</div>
                  <div>4750-5000万</div>
                </div>
                <div class="mv-row">
                  <div>90-94</div>
                  <div>×35</div>
                  <div>3150-3290万</div>
                </div>
                <div class="mv-row">
                  <div>85-89</div>
                  <div>×20</div>
                  <div>1700-1780万</div>
                </div>
                <div class="mv-row">
                  <div>80-84</div>
                  <div>×12</div>
                  <div>960-1008万</div>
                </div>
                <div class="mv-row">
                  <div>75-79</div>
                  <div>×7</div>
                  <div>525-553万</div>
                </div>
                <div class="mv-row">
                  <div>70-74</div>
                  <div>×4</div>
                  <div>280-296万</div>
                </div>
                <div class="mv-row">
                  <div>&lt;70</div>
                  <div>×1-2</div>
                  <div>&lt;140万</div>
                </div>
              </div>
            </div>

            <!-- 年龄系数 -->
            <div class="market-value-section">
              <div class="mv-title">年龄系数</div>
              <div class="coefficient-pills">
                <div class="coef-pill high">17-19岁 ×1.5 超新星</div>
                <div class="coef-pill medium">20-22岁 ×1.3 潜力股</div>
                <div class="coef-pill normal">23-25岁 ×1.0 黄金期</div>
                <div class="coef-pill low">26-27岁 ×0.85 巅峰末</div>
                <div class="coef-pill lower">28-29岁 ×0.7 下滑期</div>
                <div class="coef-pill lowest">30+岁 ×0.5 老将</div>
              </div>
            </div>

            <!-- 其他系数 -->
            <div class="market-value-section">
              <div class="mv-title">其他系数</div>
              <div class="other-coefficients">
                <div class="coef-group">
                  <div class="coef-label">潜力系数</div>
                  <div class="coef-values">
                    <span>潜力-能力&gt;10: ×1.25</span>
                    <span>潜力-能力≥5: ×1.1</span>
                    <span>其他: ×1.0</span>
                  </div>
                </div>
                <div class="coef-group">
                  <div class="coef-label">标签系数</div>
                  <div class="coef-values">
                    <span>天才: ×1.2</span>
                    <span>普通: ×1.0</span>
                    <span>平庸: ×0.9</span>
                  </div>
                </div>
                <div class="coef-group">
                  <div class="coef-label">位置系数</div>
                  <div class="coef-values">
                    <span>MID: ×1.2</span>
                    <span>ADC: ×1.15</span>
                    <span>JUG: ×1.1</span>
                    <span>TOP: ×1.0</span>
                    <span>SUP: ×0.9</span>
                  </div>
                </div>
                <div class="coef-group">
                  <div class="coef-label">赛区系数</div>
                  <div class="coef-values">
                    <span>LPL: ×1.3</span>
                    <span>LCK: ×1.2</span>
                    <span>LEC: ×1.0</span>
                    <span>LCS: ×0.9</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 身价更新时机 -->
            <div class="market-value-section">
              <div class="mv-title">身价更新时机</div>
              <div class="update-timing">
                <div class="timing-item">
                  <el-tag type="primary">年度颁奖后</el-tag>
                  <span>全部选手重算（含荣誉系数）</span>
                </div>
                <div class="timing-item">
                  <el-tag type="success">年度Top20</el-tag>
                  <span>Top1-5涨20%，Top6-10涨15%，Top11-20涨10%</span>
                </div>
                <div class="timing-item">
                  <el-tag type="warning">最佳阵容</el-tag>
                  <span>身价涨20%</span>
                </div>
                <div class="timing-item">
                  <el-tag>最佳新秀</el-tag>
                  <span>身价涨30%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 比赛模拟 -->
      <el-collapse-item name="simulation">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #f97316;"><Cpu /></el-icon>
            <span>比赛模拟</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="section">
            <h4><el-icon><DataAnalysis /></el-icon> 三层正态分布算法</h4>
            <p>比赛结果基于<strong>三层正态分布</strong>模型，战力高的队伍获胜概率更大，但存在爆冷可能。</p>

            <div class="three-layer-model">
              <div class="layer-item">
                <div class="layer-header">
                  <div class="layer-num">1</div>
                  <div class="layer-title">选手发挥计算</div>
                </div>
                <div class="layer-content">
                  <div class="formula-box">
                    <div class="formula">发挥值 = 能力值 + 状态值 + N(0, σ)</div>
                    <div class="formula-note">σ = (100 - 稳定性) / 10</div>
                  </div>
                  <div class="layer-desc">
                    <p>每位选手独立计算发挥值，稳定性越高波动越小</p>
                  </div>
                </div>
              </div>

              <div class="layer-arrow">
                <el-icon><Bottom /></el-icon>
              </div>

              <div class="layer-item">
                <div class="layer-header">
                  <div class="layer-num">2</div>
                  <div class="layer-title">队伍战力计算</div>
                </div>
                <div class="layer-content">
                  <div class="formula-box">
                    <div class="formula">队伍战力 = Σ(选手发挥值) / 5</div>
                  </div>
                  <div class="layer-desc">
                    <p>5名首发选手发挥值的平均值</p>
                  </div>
                </div>
              </div>

              <div class="layer-arrow">
                <el-icon><Bottom /></el-icon>
              </div>

              <div class="layer-item">
                <div class="layer-header">
                  <div class="layer-num">3</div>
                  <div class="layer-title">胜负判定</div>
                </div>
                <div class="layer-content">
                  <div class="formula-box">
                    <div class="formula">战力差 = 主队战力 - 客队战力 + N(0, 3)</div>
                    <div class="formula-note">战力差 > 0 → 主队胜，否则客队胜</div>
                  </div>
                  <div class="layer-desc">
                    <p>额外的随机因素模拟比赛中的不确定性</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Warning /></el-icon> 爆冷机制</h4>
            <div class="upset-example">
              <div class="team-vs">
                <div class="team team-a">
                  <div class="team-power">80</div>
                  <div class="team-label">强队A</div>
                  <div class="team-perf lose">发挥: 76</div>
                </div>
                <div class="vs">VS</div>
                <div class="team team-b">
                  <div class="team-power">70</div>
                  <div class="team-label">弱队B</div>
                  <div class="team-perf win">发挥: 78</div>
                </div>
              </div>
              <div class="result-badge">
                <el-icon><SuccessFilled /></el-icon>
                弱队B爆冷获胜！
              </div>
            </div>
            <div class="upset-tips">
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>稳定性低的队伍更容易爆发或失常</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>BO5 系列赛强队胜率更稳定（5局取3胜）</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 数据中心 -->
      <el-collapse-item name="datacenter">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #8b5cf6;"><TrendCharts /></el-icon>
            <span>数据中心</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">数据中心系统追踪选手的比赛表现，用于MVP评选、年度最佳阵容和选手评分。</p>

          <div class="section">
            <h4><el-icon><DataAnalysis /></el-icon> 选手赛季数据</h4>
            <p>每场比赛后自动记录选手的表现数据：</p>

            <div class="data-stats-grid">
              <div class="stat-category">
                <div class="category-header">基础数据</div>
                <div class="stat-items">
                  <div class="stat-item"><span class="stat-name">击杀</span><span class="stat-desc">比赛中击杀敌方次数</span></div>
                  <div class="stat-item"><span class="stat-name">死亡</span><span class="stat-desc">比赛中死亡次数</span></div>
                  <div class="stat-item"><span class="stat-name">助攻</span><span class="stat-desc">比赛中助攻次数</span></div>
                  <div class="stat-item"><span class="stat-name">KDA</span><span class="stat-desc">(击杀+助攻)/死亡</span></div>
                </div>
              </div>
              <div class="stat-category">
                <div class="category-header">胜负数据</div>
                <div class="stat-items">
                  <div class="stat-item"><span class="stat-name">出场数</span><span class="stat-desc">参与比赛总场次</span></div>
                  <div class="stat-item"><span class="stat-name">胜场</span><span class="stat-desc">获胜场次</span></div>
                  <div class="stat-item"><span class="stat-name">负场</span><span class="stat-desc">失败场次</span></div>
                  <div class="stat-item"><span class="stat-name">胜率</span><span class="stat-desc">胜场/出场数</span></div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Star /></el-icon> 影响力分数</h4>
            <p class="guide-desc">影响力分数衡量选手在比赛中的实际贡献，用于MVP评选。</p>

            <div class="formula-box dark">
              <div class="formula">影响力 = 击杀×3 + 助攻×1.5 - 死亡×2 + 胜场加成</div>
              <div class="formula-note">胜场加成 = 胜场数 × 2</div>
            </div>

            <div class="impact-factors">
              <div class="factor-card kill">
                <div class="factor-icon">⚔️</div>
                <div class="factor-info">
                  <div class="factor-name">击杀</div>
                  <div class="factor-weight">×3.0</div>
                </div>
              </div>
              <div class="factor-card assist">
                <div class="factor-icon">🤝</div>
                <div class="factor-info">
                  <div class="factor-name">助攻</div>
                  <div class="factor-weight">×1.5</div>
                </div>
              </div>
              <div class="factor-card death">
                <div class="factor-icon">💀</div>
                <div class="factor-info">
                  <div class="factor-name">死亡</div>
                  <div class="factor-weight">-2.0</div>
                </div>
              </div>
              <div class="factor-card win">
                <div class="factor-icon">🏆</div>
                <div class="factor-info">
                  <div class="factor-name">胜场</div>
                  <div class="factor-weight">+2.0</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Trophy /></el-icon> MVP 评选</h4>
            <p class="guide-desc">每项赛事结束后，系统会根据影响力分数评选MVP。</p>

            <div class="mvp-process">
              <div class="process-step">
                <div class="step-num">1</div>
                <div class="step-content">
                  <div class="step-title">收集数据</div>
                  <div class="step-desc">统计所有参赛选手的比赛数据</div>
                </div>
              </div>
              <el-icon class="process-arrow"><ArrowRight /></el-icon>
              <div class="process-step">
                <div class="step-num">2</div>
                <div class="step-content">
                  <div class="step-title">计算影响力</div>
                  <div class="step-desc">按公式计算每位选手的影响力分数</div>
                </div>
              </div>
              <el-icon class="process-arrow"><ArrowRight /></el-icon>
              <div class="process-step">
                <div class="step-num">3</div>
                <div class="step-content">
                  <div class="step-title">冠军加成</div>
                  <div class="step-desc">冠军队伍选手影响力×1.2</div>
                </div>
              </div>
              <el-icon class="process-arrow"><ArrowRight /></el-icon>
              <div class="process-step highlight">
                <div class="step-num">MVP</div>
                <div class="step-content">
                  <div class="step-title">评选MVP</div>
                  <div class="step-desc">影响力最高者获得MVP</div>
                </div>
              </div>
            </div>

            <div class="mvp-bonus-info">
              <el-tag type="warning">冠军加成</el-tag>
              <span>冠军队伍选手在MVP评选中影响力×1.2，增加夺冠队伍产生MVP的概率</span>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Medal /></el-icon> 年度最佳阵容</h4>
            <p class="guide-desc">赛季结束时，根据全年表现评选最佳阵容。</p>

            <div class="all-pro-teams">
              <div class="all-pro-item first">
                <div class="all-pro-badge">一阵</div>
                <div class="all-pro-info">
                  <div class="all-pro-title">年度最佳一阵</div>
                  <div class="all-pro-desc">每位置年度影响力最高的选手</div>
                  <div class="all-pro-bonus">身价加成 +25%（2赛季）</div>
                </div>
              </div>
              <div class="all-pro-item second">
                <div class="all-pro-badge">二阵</div>
                <div class="all-pro-info">
                  <div class="all-pro-title">年度最佳二阵</div>
                  <div class="all-pro-desc">每位置年度影响力第2名的选手</div>
                  <div class="all-pro-bonus">身价加成 +15%（2赛季）</div>
                </div>
              </div>
              <div class="all-pro-item third">
                <div class="all-pro-badge">三阵</div>
                <div class="all-pro-info">
                  <div class="all-pro-title">年度最佳三阵</div>
                  <div class="all-pro-desc">每位置年度影响力第3名的选手</div>
                  <div class="all-pro-bonus">身价加成 +10%（2赛季）</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><StarFilled /></el-icon> 年度 Top20 评分</h4>
            <p class="guide-desc">赛季结束时，评选全球年度Top20选手。</p>

            <div class="formula-box dark">
              <div class="formula">年度评分 = 能力值×0.4 + 年度影响力×0.3 + 荣誉加分×0.3</div>
            </div>

            <div class="top20-tiers">
              <div class="tier-item tier-top">
                <div class="tier-rank">Top 1-5</div>
                <div class="tier-bonus">身价涨幅 +20%</div>
              </div>
              <div class="tier-item tier-mid">
                <div class="tier-rank">Top 6-10</div>
                <div class="tier-bonus">身价涨幅 +15%</div>
              </div>
              <div class="tier-item tier-low">
                <div class="tier-rank">Top 11-20</div>
                <div class="tier-bonus">身价涨幅 +10%</div>
              </div>
            </div>

            <div class="top20-scoring">
              <div class="scoring-item">
                <el-tag type="primary">能力值权重</el-tag>
                <span>40% - 选手当前实力</span>
              </div>
              <div class="scoring-item">
                <el-tag type="success">影响力权重</el-tag>
                <span>30% - 全年比赛表现</span>
              </div>
              <div class="scoring-item">
                <el-tag type="warning">荣誉加分权重</el-tag>
                <span>30% - 获得的冠军、MVP等荣誉</span>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><User /></el-icon> 最佳新秀</h4>
            <p class="guide-desc">每赛季评选各赛区的最佳新秀（首个职业赛季的选手）。</p>

            <div class="rookie-criteria">
              <div class="criteria-item">
                <el-icon><CircleCheck /></el-icon>
                <span>参赛条件：首个职业赛季</span>
              </div>
              <div class="criteria-item">
                <el-icon><CircleCheck /></el-icon>
                <span>评选标准：赛季影响力分数最高</span>
              </div>
              <div class="criteria-item">
                <el-icon><CircleCheck /></el-icon>
                <span>荣誉奖励：身价加成 +20%（3赛季有效）</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 转会系统 -->
      <el-collapse-item name="transfer">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #06b6d4;"><Switch /></el-icon>
            <span>转会系统</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="section">
            <h4><el-icon><Timer /></el-icon> 转会窗口</h4>
            <p>转会期在每赛季<strong>世界赛结束后</strong>开放，选秀之前结束。</p>
          </div>

          <div class="section">
            <h4><el-icon><List /></el-icon> 转会类型</h4>
            <div class="transfer-types">
              <div class="transfer-type">
                <el-tag>合同到期</el-tag>
                <span>选手变为自由球员，无需转会费</span>
              </div>
              <div class="transfer-type">
                <el-tag type="success">主动求购</el-tag>
                <span>AI球队向其他球队报价</span>
              </div>
              <div class="transfer-type">
                <el-tag type="warning">被动出售</el-tag>
                <span>财政困难或阵容冗余时挂牌</span>
              </div>
              <div class="transfer-type">
                <el-tag type="danger">退役</el-tag>
                <span>年龄30+且能力60以下</span>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Cpu /></el-icon> AI转会决策逻辑</h4>
            <p class="guide-desc">AI战队会根据以下逻辑自动进行转会操作：</p>

            <!-- AI补强需求评估 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><Search /></el-icon>
                <span>1. 补强需求评估</span>
              </div>
              <div class="logic-content">
                <p>AI会评估每个位置是否需要补强：</p>
                <ul class="logic-list">
                  <li><el-tag type="danger" size="small">需补强</el-tag> 位置选手能力 &lt; 70</li>
                  <li><el-tag type="warning" size="small">考虑补强</el-tag> 位置选手年龄 &gt; 28岁</li>
                  <li><el-tag type="info" size="small">优先级</el-tag> 能力越低 / 年龄越大 → 补强优先级越高</li>
                </ul>
              </div>
            </div>

            <!-- 选手价值评估 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><DataAnalysis /></el-icon>
                <span>2. 选手价值评估</span>
              </div>
              <div class="logic-content">
                <div class="formula-box dark">
                  <div class="formula">身价 = 能力值 × 10 × 年龄系数 × 潜力系数</div>
                </div>
                <div class="coefficient-grid">
                  <div class="coef-item">
                    <div class="coef-title">年龄系数</div>
                    <div class="coef-values">
                      <span class="coef-tag high">&lt;23岁: ×1.5</span>
                      <span class="coef-tag mid">23-26岁: ×1.0</span>
                      <span class="coef-tag low">&gt;26岁: ×0.6</span>
                    </div>
                  </div>
                  <div class="coef-item">
                    <div class="coef-title">潜力系数</div>
                    <div class="coef-values">
                      <span class="coef-tag">潜力值 / 80</span>
                    </div>
                  </div>
                </div>
                <div class="example-calc">
                  <div class="example-title">示例计算</div>
                  <div class="example-content">
                    <span>能力80 × 22岁 × 潜力90</span>
                    <span class="calc-arrow">→</span>
                    <span>80 × 10 × 1.5 × 1.125 = <strong>1350万</strong></span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 报价决策 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><Money /></el-icon>
                <span>3. 报价决策</span>
              </div>
              <div class="logic-content">
                <div class="decision-flow">
                  <div class="flow-step">
                    <div class="step-num">1</div>
                    <div class="step-content">
                      <div class="step-title">检查预算</div>
                      <div class="step-desc">预算 &gt; 选手身价 才会报价</div>
                    </div>
                  </div>
                  <div class="flow-connector">
                    <el-icon><ArrowRight /></el-icon>
                  </div>
                  <div class="flow-step">
                    <div class="step-num">2</div>
                    <div class="step-content">
                      <div class="step-title">评估提升</div>
                      <div class="step-desc">目标 &gt; 现有选手能力+5</div>
                    </div>
                  </div>
                  <div class="flow-connector">
                    <el-icon><ArrowRight /></el-icon>
                  </div>
                  <div class="flow-step">
                    <div class="step-num">3</div>
                    <div class="step-content">
                      <div class="step-title">生成报价</div>
                      <div class="step-desc">身价 × (0.9~1.1)</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 接受报价逻辑 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><Check /></el-icon>
                <span>4. 接受/拒绝报价</span>
              </div>
              <div class="logic-content">
                <div class="accept-conditions">
                  <div class="condition accept">
                    <div class="condition-icon">
                      <el-icon><CircleCheckFilled /></el-icon>
                    </div>
                    <div class="condition-info">
                      <div class="condition-title">接受报价</div>
                      <ul>
                        <li>报价 ≥ 身价 × 0.9</li>
                        <li>或 财政紧张且报价 ≥ 身价 × 0.7</li>
                        <li>或 选手为替补且报价 ≥ 身价 × 0.8</li>
                      </ul>
                    </div>
                  </div>
                  <div class="condition reject">
                    <div class="condition-icon">
                      <el-icon><CircleCloseFilled /></el-icon>
                    </div>
                    <div class="condition-info">
                      <div class="condition-title">拒绝报价</div>
                      <ul>
                        <li>报价 &lt; 身价 × 0.7</li>
                        <li>或 选手是核心首发且报价 &lt; 身价 × 1.2</li>
                        <li>或 该位置无替补</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 工资计算 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><Wallet /></el-icon>
                <span>5. 合同工资计算</span>
              </div>
              <div class="logic-content">
                <div class="formula-box dark">
                  <div class="formula">年薪 = 能力值 × 2 × 位置系数（万元/年）</div>
                </div>
                <div class="position-salary">
                  <div class="pos-item mid">
                    <span class="pos-name">MID</span>
                    <span class="pos-coef">×1.2</span>
                  </div>
                  <div class="pos-item adc">
                    <span class="pos-name">ADC</span>
                    <span class="pos-coef">×1.1</span>
                  </div>
                  <div class="pos-item jug">
                    <span class="pos-name">JUG</span>
                    <span class="pos-coef">×1.0</span>
                  </div>
                  <div class="pos-item top">
                    <span class="pos-name">TOP</span>
                    <span class="pos-coef">×1.0</span>
                  </div>
                  <div class="pos-item sup">
                    <span class="pos-name">SUP</span>
                    <span class="pos-coef">×0.9</span>
                  </div>
                </div>
                <div class="salary-example">
                  <span>能力85的MID选手年薪：85 × 2 × 1.2 = <strong>204万/年</strong></span>
                </div>
              </div>
            </div>

            <!-- 合同年限 -->
            <div class="ai-logic-box">
              <div class="logic-title">
                <el-icon><Calendar /></el-icon>
                <span>6. 合同年限决策</span>
              </div>
              <div class="logic-content">
                <div class="contract-years">
                  <div class="year-item">
                    <div class="year-badge long">3年</div>
                    <div class="year-condition">年龄 ≤ 24 且 潜力 ≥ 85</div>
                  </div>
                  <div class="year-item">
                    <div class="year-badge medium">2年</div>
                    <div class="year-condition">年龄 ≤ 27 或 能力 ≥ 80</div>
                  </div>
                  <div class="year-item">
                    <div class="year-badge short">1年</div>
                    <div class="year-condition">年龄 &gt; 27 且 能力 &lt; 80</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Money /></el-icon> 财政状态</h4>
            <div class="finance-states">
              <div class="state rich">
                <div class="state-icon">
                  <el-icon><CircleCheckFilled /></el-icon>
                </div>
                <div class="state-info">
                  <div class="state-name">富裕</div>
                  <div class="state-range">1000万+</div>
                </div>
              </div>
              <div class="state healthy">
                <div class="state-icon">
                  <el-icon><CircleCheck /></el-icon>
                </div>
                <div class="state-info">
                  <div class="state-name">健康</div>
                  <div class="state-range">500-1000万</div>
                </div>
              </div>
              <div class="state tight">
                <div class="state-icon">
                  <el-icon><WarningFilled /></el-icon>
                </div>
                <div class="state-info">
                  <div class="state-name">紧张</div>
                  <div class="state-range">100-500万</div>
                </div>
              </div>
              <div class="state deficit">
                <div class="state-icon">
                  <el-icon><CircleCloseFilled /></el-icon>
                </div>
                <div class="state-info">
                  <div class="state-name">赤字</div>
                  <div class="state-range">0-100万</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><InfoFilled /></el-icon> 转会小贴士</h4>
            <div class="tips-list">
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>豪门战队（预算充足）倾向于直接购买成名选手</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>中小战队更注重性价比，偏好年轻高潜选手</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>财政紧张的战队会主动挂牌高薪选手</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>同赛区转会可能触发竞争加价</span>
              </div>
            </div>
          </div>

          <!-- GM 人格系统 -->
          <div class="section">
            <h4><el-icon><Avatar /></el-icon> GM 人格系统</h4>
            <p class="guide-desc">每支战队有一个 GM 人格，决定其转会策略风格。可在"GM配置"页面查看和调整。</p>

            <div class="gm-personality-grid">
              <div class="personality-card championship">
                <div class="personality-header">
                  <span class="personality-icon">🏆</span>
                  <span class="personality-name">争冠型</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">80%</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">≥80</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">30%</span>
                  </div>
                </div>
                <div class="personality-desc">追求顶级选手，愿意高价挖角</div>
              </div>

              <div class="personality-card youth">
                <div class="personality-header">
                  <span class="personality-icon">🌱</span>
                  <span class="personality-name">青训型</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">50%</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">≥65</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">10%</span>
                  </div>
                </div>
                <div class="personality-desc">培养年轻人，卖掉老将换资金</div>
              </div>

              <div class="personality-card balanced">
                <div class="personality-header">
                  <span class="personality-icon">⚖️</span>
                  <span class="personality-name">稳健型</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">60%</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">≥70</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">15%</span>
                  </div>
                </div>
                <div class="personality-desc">性价比优先，控制风险</div>
              </div>

              <div class="personality-card speculator">
                <div class="personality-header">
                  <span class="personality-icon">📈</span>
                  <span class="personality-name">投机型</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">70%</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">≥60</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">20%</span>
                  </div>
                </div>
                <div class="personality-desc">买低卖高，赚取差价</div>
              </div>

              <div class="personality-card rebuilding">
                <div class="personality-header">
                  <span class="personality-icon">🔄</span>
                  <span class="personality-name">重建型</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">40%</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">≥55</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">5%</span>
                  </div>
                </div>
                <div class="personality-desc">大规模清洗，低价收人重建</div>
              </div>

              <div class="personality-card custom">
                <div class="personality-header">
                  <span class="personality-icon">⚙️</span>
                  <span class="personality-name">自定义</span>
                </div>
                <div class="personality-stats">
                  <div class="stat-row">
                    <span class="stat-label">预算比例</span>
                    <span class="stat-value">可调</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">能力阈值</span>
                    <span class="stat-value">可调</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">溢价容忍</span>
                    <span class="stat-value">可调</span>
                  </div>
                </div>
                <div class="personality-desc">完全自定义所有参数</div>
              </div>
            </div>

            <div class="gm-features">
              <div class="feature-item">
                <el-tag type="primary">LLM AI 策略</el-tag>
                <span>配置 API 后可使用大语言模型生成更智能的转会策略</span>
              </div>
              <div class="feature-item">
                <el-tag type="success">选秀偏好</el-tag>
                <span>能力权重(0.3-0.7)和年轻偏好(-0.3-0.5)影响选秀决策</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 财政系统 -->
      <el-collapse-item name="finance">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #10b981;"><Wallet /></el-icon>
            <span>财政系统</span>
          </div>
        </template>
        <div class="guide-content">
          <p class="guide-desc">财政系统追踪战队所有收支，影响转会能力和阵容建设。合理的财务管理是成功的关键！</p>

          <div class="section">
            <h4><el-icon><TrendCharts /></el-icon> 收入来源</h4>
            <div class="income-grid">
              <div class="income-item league">
                <div class="income-icon"><el-icon><Trophy /></el-icon></div>
                <div class="income-info">
                  <div class="income-name">联赛分成</div>
                  <div class="income-amount">+500万/赛季</div>
                  <div class="income-desc">所有参赛队伍平均分配联赛收入</div>
                </div>
              </div>
              <div class="income-item prize">
                <div class="income-icon"><el-icon><Medal /></el-icon></div>
                <div class="income-info">
                  <div class="income-name">赛事奖金</div>
                  <div class="income-amount">根据名次</div>
                  <div class="income-desc">在各项赛事中取得好成绩获得奖金</div>
                </div>
              </div>
              <div class="income-item sponsor">
                <div class="income-icon"><el-icon><Promotion /></el-icon></div>
                <div class="income-info">
                  <div class="income-name">赞助收入</div>
                  <div class="income-amount">200-800万</div>
                  <div class="income-desc">根据战力、胜率和荣誉计算</div>
                </div>
              </div>
              <div class="income-item transfer">
                <div class="income-icon"><el-icon><Sell /></el-icon></div>
                <div class="income-info">
                  <div class="income-name">转会收入</div>
                  <div class="income-amount">不定</div>
                  <div class="income-desc">出售选手获得的转会费</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Money /></el-icon> 支出项目</h4>
            <div class="expense-grid">
              <div class="expense-item salary">
                <div class="expense-icon"><el-icon><User /></el-icon></div>
                <div class="expense-info">
                  <div class="expense-name">选手薪资</div>
                  <div class="expense-desc">每赛季支付所有签约选手薪资</div>
                </div>
              </div>
              <div class="expense-item transfer-out">
                <div class="expense-icon"><el-icon><Goods /></el-icon></div>
                <div class="expense-info">
                  <div class="expense-name">转会支出</div>
                  <div class="expense-desc">购买选手支付的转会费</div>
                </div>
              </div>
              <div class="expense-item other">
                <div class="expense-icon"><el-icon><Setting /></el-icon></div>
                <div class="expense-info">
                  <div class="expense-name">其他支出</div>
                  <div class="expense-desc">运营成本、罚款等</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><DataLine /></el-icon> 财务状态</h4>
            <p>战队的财务状态决定了转会策略：</p>
            <div class="finance-status-grid">
              <div class="status-item wealthy">
                <div class="status-badge">
                  <el-icon><CircleCheckFilled /></el-icon>
                  <span>富裕</span>
                </div>
                <div class="status-range">&gt; 1000万</div>
                <div class="status-strategy">积极买入强援</div>
              </div>
              <div class="status-item healthy">
                <div class="status-badge">
                  <el-icon><CircleCheck /></el-icon>
                  <span>健康</span>
                </div>
                <div class="status-range">500-1000万</div>
                <div class="status-strategy">被动观望市场</div>
              </div>
              <div class="status-item tight">
                <div class="status-badge">
                  <el-icon><WarningFilled /></el-icon>
                  <span>紧张</span>
                </div>
                <div class="status-range">100-500万</div>
                <div class="status-strategy">必须卖出选手</div>
              </div>
              <div class="status-item deficit">
                <div class="status-badge">
                  <el-icon><CircleCloseFilled /></el-icon>
                  <span>赤字</span>
                </div>
                <div class="status-range">0-100万</div>
                <div class="status-strategy">强制清洗阵容</div>
              </div>
              <div class="status-item bankrupt">
                <div class="status-badge">
                  <el-icon><WarnTriangleFilled /></el-icon>
                  <span>破产</span>
                </div>
                <div class="status-range">&lt; 0万</div>
                <div class="status-strategy">强制清洗阵容</div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><Coin /></el-icon> 赛事奖金池</h4>
            <el-tabs v-model="activePrizeTab" type="card" class="prize-tabs">
              <el-tab-pane label="S世界赛" name="worlds">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>12000万</strong></span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">5000万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">2500万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">1200万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">600万</span></div>
                    <div class="prize-row"><span class="prize-rank">16强</span><span class="prize-amount">300万</span></div>
                    <div class="prize-row"><span class="prize-rank">入围赛</span><span class="prize-amount">100万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="Super" name="super">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>15000万</strong></span>
                    <span class="prize-note highlight">（年度最高奖金）</span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">6000万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">3000万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">1500万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">750万</span></div>
                    <div class="prize-row"><span class="prize-rank">16强</span><span class="prize-amount">375万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="MSI" name="msi">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>4000万</strong></span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">2000万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">1000万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">500万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">200万</span></div>
                    <div class="prize-row"><span class="prize-rank">小组赛</span><span class="prize-amount">100万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="ICP" name="icp">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>3000万</strong></span>
                    <span class="prize-note">（洲际对抗赛）</span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">1200万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">600万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">300万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">150万</span></div>
                    <div class="prize-row"><span class="prize-rank">小组赛</span><span class="prize-amount">75万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="上海大师赛" name="shanghai">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>2500万</strong></span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">1000万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">500万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">250万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">120万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="马德里大师赛" name="madrid">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>2000万</strong></span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">800万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">400万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">200万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">100万</span></div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="C洲际赛" name="claude">
                <div class="prize-table">
                  <div class="prize-header">
                    <span class="prize-total">总奖金池: <strong>2000万</strong></span>
                  </div>
                  <div class="prize-list">
                    <div class="prize-row gold"><span class="prize-rank">冠军</span><span class="prize-amount">800万</span></div>
                    <div class="prize-row silver"><span class="prize-rank">亚军</span><span class="prize-amount">400万</span></div>
                    <div class="prize-row bronze"><span class="prize-rank">四强</span><span class="prize-amount">200万</span></div>
                    <div class="prize-row"><span class="prize-rank">八强</span><span class="prize-amount">100万</span></div>
                    <div class="prize-row"><span class="prize-rank">小组赛</span><span class="prize-amount">50万</span></div>
                  </div>
                </div>
              </el-tab-pane>
            </el-tabs>
          </div>

          <div class="section">
            <h4><el-icon><DataAnalysis /></el-icon> 赞助收入计算</h4>
            <div class="sponsor-formula">
              <div class="formula-box dark">
                <div class="formula">赞助收入 = 基础(200万) + 战力加成 + 胜率加成 + 荣誉加成</div>
              </div>
              <div class="sponsor-factors">
                <div class="factor-item">
                  <div class="factor-name">战力加成</div>
                  <div class="factor-formula">战力值 / 100 × 300万</div>
                  <div class="factor-example">战力80 → +240万</div>
                </div>
                <div class="factor-item">
                  <div class="factor-name">胜率加成</div>
                  <div class="factor-formula">胜率 × 200万</div>
                  <div class="factor-example">胜率60% → +120万</div>
                </div>
                <div class="factor-item">
                  <div class="factor-name">荣誉加成</div>
                  <div class="factor-formula">每个冠军 × 50万</div>
                  <div class="factor-example">2个冠军 → +100万</div>
                </div>
              </div>
            </div>
          </div>

          <div class="section">
            <h4><el-icon><InfoFilled /></el-icon> 财务小贴士</h4>
            <div class="tips-list">
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>保持余额在 500万以上，确保转会窗口有充足资金</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>高薪选手表现不佳时，考虑出售减轻财务压力</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>赛事成绩越好，赞助收入和奖金越高，形成良性循环</span>
              </div>
              <div class="tip-item">
                <el-icon class="tip-icon"><InfoFilled /></el-icon>
                <span>财务紧张时 AI 会主动挂牌选手，可能有捡漏机会</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <!-- 选秀系统 -->
      <el-collapse-item name="draft">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #a855f7;"><UserFilled /></el-icon>
            <span>选秀系统</span>
          </div>
        </template>
        <div class="guide-content">
          <div class="section">
            <h4><el-icon><Calendar /></el-icon> 选秀时间</h4>
            <p>每<strong>4个赛季</strong>举行一次选秀（S2、S6、S10...）</p>
          </div>

          <div class="section">
            <h4><el-icon><Memo /></el-icon> 选秀规则</h4>
            <ul class="rule-list">
              <li>选秀顺位根据<strong>夏季赛常规赛排名</strong>确定</li>
              <li>排名越靠后，获得高顺位的概率越高</li>
              <li>状元、榜眼、探花等新秀已预先确定</li>
              <li>选中的新秀直接加入队伍</li>
            </ul>
          </div>

          <div class="draft-visual">
            <div class="draft-order">
              <div class="draft-pick top">
                <span class="pick-num">1</span>
                <span class="pick-label">状元</span>
              </div>
              <div class="draft-pick">
                <span class="pick-num">2</span>
                <span class="pick-label">榜眼</span>
              </div>
              <div class="draft-pick">
                <span class="pick-num">3</span>
                <span class="pick-label">探花</span>
              </div>
              <div class="draft-pick">
                <span class="pick-num">4-14</span>
                <span class="pick-label">其他顺位</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  Calendar,
  Sunny,
  Trophy,
  Flag,
  Star,
  Switch,
  User,
  CircleCheck,
  ArrowRight,
  List,
  DataAnalysis,
  DataLine,
  TrendCharts,
  SetUp,
  Clock,
  PriceTag,
  Histogram,
  Cpu,
  Warning,
  SuccessFilled,
  Timer,
  Money,
  CircleCheckFilled,
  WarningFilled,
  CircleCloseFilled,
  UserFilled,
  Memo,
  Aim,
  Link,
  StarFilled,
  Promotion,
  Search,
  Check,
  Wallet,
  InfoFilled,
  Monitor,
  Collection,
  Medal,
  VideoPlay,
  Bottom,
  Reading,
  Sell,
  Goods,
  Setting,
  Coin,
  WarnTriangleFilled,
  Avatar,
  Top
} from '@element-plus/icons-vue'

const activeNames = ref('overview')
const activeEventTab = ref('msi')
const activePrizeTab = ref('worlds')

// 联赛积分数据
const leaguePoints = [
  { rank: '冠军', points: 12 },
  { rank: '亚军', points: 10 },
  { rank: '季军', points: 8 },
  { rank: '殿军', points: 6 },
  { rank: '5-8名', points: 3 },
]

// 国际赛事积分
const internationalPoints = [
  { rank: '冠军', points: 20 },
  { rank: '亚军', points: 16 },
  { rank: '季军', points: 12 },
  { rank: '殿军', points: 8 },
  { rank: '其他', points: '2-6' },
]
</script>

<style scoped lang="scss">
.game-guide {
  .collapse-title {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
    font-weight: 600;

    .title-icon {
      font-size: 20px;
    }
  }

  .guide-content {
    padding: 8px 0;

    .guide-desc {
      color: #6b7280;
      margin-bottom: 16px;
      font-size: 14px;
    }

    .section {
      margin-bottom: 24px;

      h4 {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 15px;
        font-weight: 600;
        color: #374151;
        margin: 0 0 12px 0;
        padding-bottom: 8px;
        border-bottom: 1px solid #e5e7eb;
      }
    }
  }

  // 赛季流程图
  .season-flow {
    .flow-container {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 12px;
      margin-bottom: 16px;
      flex-wrap: wrap;

      &.last {
        margin-bottom: 0;
      }
    }

    .flow-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 12px 16px;
      border-radius: 12px;
      min-width: 100px;

      .flow-icon {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 20px;
        color: white;
        margin-bottom: 8px;
      }

      .flow-label {
        font-size: 13px;
        font-weight: 600;
        color: #374151;
      }

      .flow-sub {
        font-size: 11px;
        color: #9ca3af;
        margin-top: 2px;
      }

      &.spring, &.summer {
        background: #fef3c7;
        .flow-icon { background: #f59e0b; }
      }
      &.msi, &.worlds {
        background: #dbeafe;
        .flow-icon { background: #3b82f6; }
      }
      &.madrid, &.clauch {
        background: #f3e8ff;
        .flow-icon { background: #8b5cf6; }
      }
      &.shanghai {
        background: #fce7f3;
        .flow-icon { background: #ec4899; }
      }
      &.icp {
        background: #cffafe;
        .flow-icon { background: #06b6d4; }
      }
      &.super {
        background: #fef9c3;
        .flow-icon { background: #eab308; }
      }
      &.transfer {
        background: #d1fae5;
        .flow-icon { background: #10b981; }
      }
      &.draft {
        background: #e0e7ff;
        .flow-icon { background: #6366f1; }
      }
      &.end {
        background: #f3f4f6;
        .flow-icon { background: #6b7280; }
      }
    }

    .flow-arrow {
      color: #d1d5db;
      font-size: 20px;
    }
  }

  // 规则列表
  .rule-list {
    list-style: none;
    padding: 0;
    margin: 0;

    li {
      padding: 8px 0;
      border-bottom: 1px dashed #e5e7eb;
      font-size: 14px;
      color: #4b5563;

      &:last-child {
        border-bottom: none;
      }
    }
  }

  .inline-tags {
    display: flex;
    gap: 8px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  // 对阵图
  .bracket-visual {
    display: flex;
    align-items: center;
    gap: 16px;
    overflow-x: auto;
    padding: 16px 0;

    .bracket-stage {
      background: #f9fafb;
      padding: 12px 16px;
      border-radius: 8px;
      text-align: center;
      min-width: 120px;

      .stage-title {
        font-size: 12px;
        color: #6b7280;
        margin-bottom: 8px;
      }

      .match-pair {
        font-size: 13px;
        margin: 4px 0;

        .seed {
          display: inline-block;
          padding: 2px 8px;
          background: #e5e7eb;
          border-radius: 4px;
          font-weight: 600;

          &.winner { background: #bbf7d0; color: #166534; }
          &.loser { background: #fecaca; color: #991b1b; }
        }
      }

      &.final {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border: 2px solid #f59e0b;
      }
    }

    .bracket-arrow {
      color: #d1d5db;
    }
  }

  // 赛事卡片
  .event-card {
    padding: 16px;

    .event-header {
      display: flex;
      gap: 12px;
      margin-bottom: 12px;

      .event-teams {
        background: #3b82f6;
        color: white;
        padding: 4px 12px;
        border-radius: 16px;
        font-size: 13px;
        font-weight: 600;
      }

      .event-format {
        background: #f3f4f6;
        color: #6b7280;
        padding: 4px 12px;
        border-radius: 16px;
        font-size: 13px;
      }
    }

    // 详细赛事卡片
    &.detailed {
      .event-header {
        flex-direction: column;
        gap: 8px;
        padding-bottom: 12px;
        border-bottom: 1px solid #e5e7eb;
        margin-bottom: 16px;

        .event-title-row {
          display: flex;
          justify-content: space-between;
          align-items: center;

          .event-name {
            font-size: 18px;
            font-weight: 700;
            color: #1f2937;
          }

          .event-timing {
            font-size: 12px;
            color: #9ca3af;
            background: #f3f4f6;
            padding: 4px 10px;
            border-radius: 12px;
          }
        }

        .event-tags {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;

          .event-teams {
            background: #3b82f6;
          }

          .event-format {
            background: #e0e7ff;
            color: #4338ca;
          }

          .event-highlight {
            background: linear-gradient(135deg, #fef3c7, #fde68a);
            color: #92400e;
            padding: 4px 12px;
            border-radius: 16px;
            font-size: 13px;
            font-weight: 600;
          }
        }
      }

      .event-section {
        margin-bottom: 16px;

        .section-title {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
          margin-bottom: 10px;
          padding-left: 10px;
          border-left: 3px solid #3b82f6;
        }

        p {
          font-size: 13px;
          color: #6b7280;
          margin-bottom: 10px;
        }
      }
    }
  }

  // 参赛资格网格
  .qualification-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin-top: 10px;

    .qual-item {
      background: #f9fafb;
      padding: 10px;
      border-radius: 8px;
      text-align: center;
      border: 1px solid #e5e7eb;

      .qual-region {
        display: block;
        font-weight: 700;
        color: #1f2937;
        margin-bottom: 4px;
      }

      .qual-slots {
        font-size: 12px;
        color: #6b7280;
      }
    }
  }

  // 对阵说明
  .bracket-explanation {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;

    .bracket-part {
      padding: 12px;
      border-radius: 8px;

      &.winners {
        background: #d1fae5;
        border: 1px solid #a7f3d0;
      }

      &.losers {
        background: #fee2e2;
        border: 1px solid #fecaca;
      }

      .bracket-label {
        font-weight: 600;
        font-size: 14px;
        margin-bottom: 8px;
      }

      &.winners .bracket-label { color: #065f46; }
      &.losers .bracket-label { color: #991b1b; }

      .bracket-desc {
        p {
          font-size: 12px;
          margin: 4px 0;
          opacity: 0.9;
        }
      }

      &.winners .bracket-desc p { color: #065f46; }
      &.losers .bracket-desc p { color: #991b1b; }
    }
  }

  // 积分分配
  .points-distribution {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;

    &.worlds .point-item {
      min-width: 70px;
    }

    .point-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 10px 14px;
      border-radius: 8px;
      min-width: 60px;

      .rank {
        font-size: 12px;
        margin-bottom: 4px;
      }

      .pts {
        font-size: 16px;
        font-weight: 700;
      }

      &.gold {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        color: #92400e;
      }

      &.silver {
        background: linear-gradient(135deg, #e5e7eb, #d1d5db);
        color: #374151;
      }

      &.bronze {
        background: linear-gradient(135deg, #fed7aa, #fdba74);
        color: #9a3412;
      }

      &.fourth {
        background: #dbeafe;
        color: #1e40af;
      }

      &.other {
        background: #f3f4f6;
        color: #6b7280;
      }
    }
  }

  // 阶段详情框
  .stage-detail-box {
    display: flex;
    flex-direction: column;
    gap: 12px;

    .stage-item {
      display: flex;
      gap: 12px;
      padding: 12px;
      background: #f9fafb;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .stage-num {
        width: 32px;
        height: 32px;
        background: #3b82f6;
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        flex-shrink: 0;
      }

      .stage-info {
        .stage-name {
          font-weight: 600;
          color: #1f2937;
          margin-bottom: 6px;
        }

        ul {
          list-style: none;
          padding: 0;
          margin: 0;

          li {
            font-size: 12px;
            color: #6b7280;
            padding: 3px 0;
            padding-left: 12px;
            position: relative;

            &::before {
              content: '•';
              position: absolute;
              left: 0;
              color: #9ca3af;
            }
          }
        }
      }
    }
  }

  // 特殊规则提示
  .special-rule {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: #fef3c7;
    border-radius: 8px;
    margin-top: 10px;
    font-size: 13px;
    color: #92400e;
  }

  // 赛区积分
  .region-points {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;

    .region-point-item {
      padding: 12px;
      border-radius: 8px;
      text-align: center;

      .region-rank {
        display: block;
        font-size: 13px;
        font-weight: 600;
        margin-bottom: 4px;
      }

      .region-pts {
        font-size: 12px;
        opacity: 0.9;
      }

      &.first {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        color: #92400e;
      }

      &.second {
        background: linear-gradient(135deg, #e5e7eb, #d1d5db);
        color: #374151;
      }

      &.third {
        background: linear-gradient(135deg, #fed7aa, #fdba74);
        color: #9a3412;
      }

      &.fourth {
        background: #dbeafe;
        color: #1e40af;
      }
    }
  }

  // 瑞士轮说明
  .swiss-explanation {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;

    .swiss-rule {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 12px;
      border-radius: 8px;

      .swiss-icon {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 14px;
        color: white;

        &.win { background: #10b981; }
        &.lose { background: #ef4444; }
      }

      .swiss-desc {
        font-size: 13px;
        color: #4b5563;
      }
    }
  }

  .swiss-rounds {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;

    .round-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 10px 14px;
      background: #f9fafb;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .round-num {
        font-weight: 700;
        color: #3b82f6;
        margin-bottom: 4px;
      }

      .round-desc {
        font-size: 11px;
        color: #6b7280;
        text-align: center;
      }
    }
  }

  // 淘汰赛对阵
  .knockout-bracket {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;

    .ko-stage {
      padding: 16px 20px;
      background: #f9fafb;
      border-radius: 8px;
      text-align: center;
      border: 1px solid #e5e7eb;

      .ko-name {
        font-weight: 600;
        color: #1f2937;
        margin-bottom: 4px;
      }

      .ko-format {
        font-size: 12px;
        color: #6b7280;
      }

      &.final {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border-color: #f59e0b;

        .ko-name { color: #92400e; }
        .ko-format { color: #b45309; }
      }
    }
  }

  // ICP赛制
  .icp-format {
    display: flex;
    flex-direction: column;
    gap: 12px;

    .icp-stage {
      padding: 12px;
      background: #f9fafb;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .icp-stage-name {
        font-weight: 600;
        color: #1f2937;
        margin-bottom: 8px;
      }

      ul {
        list-style: none;
        padding: 0;
        margin: 0;

        li {
          font-size: 12px;
          color: #6b7280;
          padding: 4px 0;
          padding-left: 12px;
          position: relative;

          &::before {
            content: '•';
            position: absolute;
            left: 0;
            color: #9ca3af;
          }
        }
      }
    }
  }

  // Super资格
  .super-qualification {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 10px;

    .super-qual-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 12px;
      background: #f9fafb;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .super-qual-rank {
        background: #3b82f6;
        color: white;
        padding: 4px 10px;
        border-radius: 6px;
        font-size: 12px;
        font-weight: 600;
      }

      .super-qual-group {
        font-weight: 600;
        color: #1f2937;
      }

      .super-qual-desc {
        font-size: 12px;
        color: #6b7280;
        margin-left: auto;
      }
    }
  }

  // Super对阵
  .super-bracket {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    flex-wrap: wrap;

    .super-stage {
      padding: 12px 16px;
      background: #f9fafb;
      border-radius: 8px;
      text-align: center;
      border: 1px solid #e5e7eb;
      min-width: 100px;

      .super-stage-name {
        font-weight: 600;
        color: #1f2937;
        margin-bottom: 6px;
      }

      .super-stage-detail {
        p {
          font-size: 11px;
          color: #6b7280;
          margin: 2px 0;
        }
      }

      &.final {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border-color: #f59e0b;

        .super-stage-name { color: #92400e; }
        .super-stage-detail p { color: #b45309; }
      }
    }
  }

  // 赛事时间线
  .event-timeline {
    margin-top: 24px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 12px;
    border: 1px solid #e5e7eb;

    .timeline-title {
      font-size: 14px;
      font-weight: 600;
      color: #374151;
      margin-bottom: 12px;
      text-align: center;
    }

    .timeline-flow {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      flex-wrap: wrap;

      .el-icon {
        color: #d1d5db;
        font-size: 14px;
      }

      .timeline-item {
        padding: 6px 12px;
        border-radius: 6px;
        font-size: 12px;
        font-weight: 600;

        &.spring, &.summer { background: #fef3c7; color: #92400e; }
        &.msi { background: #dbeafe; color: #1e40af; }
        &.madrid { background: #f3e8ff; color: #6b21a8; }
        &.claude { background: #cffafe; color: #0e7490; }
        &.worlds { background: #fee2e2; color: #991b1b; }
        &.shanghai { background: #fce7f3; color: #9d174d; }
        &.icp { background: #d1fae5; color: #065f46; }
        &.super { background: linear-gradient(135deg, #fef3c7, #fde68a); color: #92400e; }
      }
    }
  }

  .group-visual {
    display: flex;
    gap: 12px;
    margin-top: 16px;

    .group-item {
      flex: 1;
      padding: 16px;
      border-radius: 8px;
      text-align: center;

      .group-name {
        font-weight: 600;
        margin-bottom: 4px;
      }

      .group-desc {
        font-size: 12px;
        opacity: 0.8;
      }

      &.legendary {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        color: #92400e;
      }
      &.challenger {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        color: #1e40af;
      }
      &.qualifier {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        color: #065f46;
      }
    }
  }

  .stage-flow {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 16px;

    .stage-box {
      padding: 16px 20px;
      background: #f9fafb;
      border-radius: 8px;
      text-align: center;

      .stage-name {
        font-weight: 600;
        color: #374151;
        margin-bottom: 4px;
      }

      .stage-detail {
        font-size: 12px;
        color: #6b7280;
        line-height: 1.4;
      }

      &.highlight {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border: 2px solid #f59e0b;
      }
    }
  }

  // 积分表格
  .points-tables {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;

    .points-table {
      h4 {
        font-size: 14px;
        font-weight: 600;
        margin: 0 0 12px 0;
        color: #374151;
      }
    }
  }

  // 选手属性
  .attribute-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;

    .attr-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px;
      background: #f9fafb;
      border-radius: 8px;

      .attr-icon {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 20px;
        color: white;

        &.ability { background: #3b82f6; }
        &.potential { background: #8b5cf6; }
        &.stability { background: #10b981; }
        &.age { background: #f59e0b; }
      }

      .attr-info {
        .attr-name {
          font-weight: 600;
          color: #374151;
          font-size: 14px;
        }
        .attr-desc {
          font-size: 12px;
          color: #9ca3af;
        }
      }
    }
  }

  // 标签展示
  .tag-showcase {
    display: flex;
    gap: 16px;

    .tag-item {
      flex: 1;
      padding: 16px;
      border-radius: 8px;
      text-align: center;

      .tag-label {
        display: block;
        font-size: 16px;
        font-weight: 700;
        margin-bottom: 4px;
      }

      .tag-effect {
        font-size: 12px;
        opacity: 0.8;
      }

      &.genius {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        color: #92400e;
      }
      &.normal {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        color: #1e40af;
      }
      &.ordinary {
        background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
        color: #6b7280;
      }
    }
  }

  // 年龄表格
  .age-table {
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #e5e7eb;

    .age-row {
      display: grid;
      grid-template-columns: 100px 80px 1fr;
      padding: 12px 16px;
      font-size: 13px;

      &.header {
        background: #f9fafb;
        font-weight: 600;
        color: #6b7280;
      }

      &.young {
        background: #fef3c7;
        color: #92400e;
      }
      &.peak {
        background: #d1fae5;
        color: #065f46;
      }
      &.veteran {
        background: #e0e7ff;
        color: #3730a3;
      }
    }
  }

  // 公式展示
  .formula-box {
    background: #1f2937;
    padding: 16px 20px;
    border-radius: 8px;
    margin: 16px 0;

    .formula {
      font-family: 'Courier New', monospace;
      color: #34d399;
      font-size: 14px;
      margin-bottom: 8px;
    }

    .formula-desc {
      font-size: 12px;
      color: #9ca3af;
    }
  }

  // 概率可视化
  .probability-visual {
    background: #f9fafb;
    padding: 20px;
    border-radius: 12px;
    text-align: center;

    .prob-title {
      font-size: 13px;
      color: #6b7280;
      margin-bottom: 16px;
    }

    .prob-chart {
      display: flex;
      align-items: flex-end;
      justify-content: center;
      height: 100px;
      gap: 4px;

      .prob-bar {
        width: 40px;
        background: #dbeafe;
        border-radius: 4px 4px 0 0;
        display: flex;
        align-items: flex-end;
        justify-content: center;
        padding-bottom: 4px;

        span {
          font-size: 10px;
          color: #6b7280;
        }

        &.highlight {
          background: #3b82f6;
          span { color: white; font-weight: 600; }
        }
      }
    }

    .prob-labels {
      display: flex;
      justify-content: center;
      gap: 16px;
      margin-top: 8px;
      font-size: 11px;
      color: #9ca3af;
    }
  }

  // 爆冷示例
  .upset-example {
    background: #f9fafb;
    padding: 20px;
    border-radius: 12px;

    .team-vs {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 24px;
      margin-bottom: 16px;

      .team {
        text-align: center;
        padding: 16px 24px;
        border-radius: 8px;

        &.team-a { background: #fee2e2; }
        &.team-b { background: #d1fae5; }

        .team-power {
          font-size: 32px;
          font-weight: 700;
          color: #374151;
        }

        .team-label {
          font-size: 13px;
          color: #6b7280;
          margin: 4px 0;
        }

        .team-perf {
          font-size: 12px;
          padding: 4px 8px;
          border-radius: 4px;

          &.win { background: #10b981; color: white; }
          &.lose { background: #ef4444; color: white; }
        }
      }

      .vs {
        font-size: 24px;
        font-weight: 700;
        color: #9ca3af;
      }
    }

    .result-badge {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      padding: 12px;
      background: #d1fae5;
      border-radius: 8px;
      color: #065f46;
      font-weight: 600;
    }
  }

  // 转会类型
  .transfer-types {
    display: flex;
    flex-direction: column;
    gap: 8px;

    .transfer-type {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 8px 0;

      span {
        font-size: 13px;
        color: #6b7280;
      }
    }
  }

  // 财政状态
  .finance-states {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;

    .state {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 16px;
      border-radius: 8px;
      text-align: center;

      .state-icon {
        font-size: 24px;
        margin-bottom: 8px;
      }

      .state-name {
        font-weight: 600;
        font-size: 14px;
      }

      .state-range {
        font-size: 12px;
        opacity: 0.8;
        margin-top: 2px;
      }

      &.rich {
        background: #d1fae5;
        color: #065f46;
      }
      &.healthy {
        background: #dbeafe;
        color: #1e40af;
      }
      &.tight {
        background: #fef3c7;
        color: #92400e;
      }
      &.deficit {
        background: #fee2e2;
        color: #991b1b;
      }
    }
  }

  // 选秀可视化
  .draft-visual {
    margin-top: 16px;

    .draft-order {
      display: flex;
      justify-content: center;
      gap: 16px;

      .draft-pick {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 16px 24px;
        background: #f3f4f6;
        border-radius: 12px;

        .pick-num {
          font-size: 24px;
          font-weight: 700;
          color: #374151;
        }

        .pick-label {
          font-size: 12px;
          color: #6b7280;
          margin-top: 4px;
        }

        &.top {
          background: linear-gradient(135deg, #fef3c7, #fde68a);

          .pick-num { color: #92400e; }
        }
      }
    }
  }

  // AI转会逻辑样式
  .ai-logic-box {
    background: #f9fafb;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 16px;
    border: 1px solid #e5e7eb;

    .logic-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 14px;
      font-weight: 600;
      color: #374151;
      margin-bottom: 12px;
      padding-bottom: 8px;
      border-bottom: 1px dashed #e5e7eb;
    }

    .logic-content {
      p {
        font-size: 13px;
        color: #6b7280;
        margin-bottom: 12px;
      }

      .logic-list {
        list-style: none;
        padding: 0;
        margin: 0;

        li {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 0;
          font-size: 13px;
          color: #4b5563;
        }
      }
    }
  }

  .formula-box.dark {
    background: #1f2937;
    padding: 12px 16px;
    border-radius: 8px;
    margin-bottom: 12px;

    .formula {
      font-family: 'Courier New', monospace;
      color: #34d399;
      font-size: 13px;
    }
  }

  .coefficient-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 12px;

    .coef-item {
      background: white;
      padding: 12px;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .coef-title {
        font-size: 12px;
        color: #6b7280;
        margin-bottom: 8px;
      }

      .coef-values {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;

        .coef-tag {
          font-size: 11px;
          padding: 4px 8px;
          border-radius: 4px;
          background: #f3f4f6;
          color: #374151;

          &.high { background: #d1fae5; color: #065f46; }
          &.mid { background: #dbeafe; color: #1e40af; }
          &.low { background: #fee2e2; color: #991b1b; }
        }
      }
    }
  }

  .example-calc {
    background: white;
    padding: 12px;
    border-radius: 8px;
    border: 1px solid #e5e7eb;

    .example-title {
      font-size: 11px;
      color: #9ca3af;
      margin-bottom: 6px;
    }

    .example-content {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 13px;
      color: #4b5563;

      .calc-arrow {
        color: #9ca3af;
      }

      strong {
        color: #059669;
      }
    }
  }

  .decision-flow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;

    .flow-step {
      display: flex;
      align-items: center;
      gap: 10px;
      background: white;
      padding: 12px 16px;
      border-radius: 8px;
      border: 1px solid #e5e7eb;
      flex: 1;
      min-width: 140px;

      .step-num {
        width: 28px;
        height: 28px;
        background: #3b82f6;
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: 14px;
      }

      .step-content {
        .step-title {
          font-size: 13px;
          font-weight: 600;
          color: #374151;
        }

        .step-desc {
          font-size: 11px;
          color: #9ca3af;
        }
      }
    }

    .flow-connector {
      color: #d1d5db;
      font-size: 16px;
    }
  }

  .accept-conditions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;

    .condition {
      padding: 16px;
      border-radius: 8px;

      &.accept {
        background: #d1fae5;
        border: 1px solid #a7f3d0;
      }

      &.reject {
        background: #fee2e2;
        border: 1px solid #fecaca;
      }

      .condition-icon {
        font-size: 24px;
        margin-bottom: 8px;
      }

      &.accept .condition-icon { color: #059669; }
      &.reject .condition-icon { color: #dc2626; }

      .condition-title {
        font-size: 14px;
        font-weight: 600;
        margin-bottom: 8px;
      }

      &.accept .condition-title { color: #065f46; }
      &.reject .condition-title { color: #991b1b; }

      ul {
        list-style: none;
        padding: 0;
        margin: 0;

        li {
          font-size: 12px;
          padding: 4px 0;
          opacity: 0.9;
        }
      }

      &.accept ul li { color: #065f46; }
      &.reject ul li { color: #991b1b; }
    }
  }

  .position-salary {
    display: flex;
    justify-content: space-around;
    gap: 8px;
    margin: 12px 0;

    .pos-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 12px 16px;
      border-radius: 8px;
      min-width: 60px;

      .pos-name {
        font-size: 14px;
        font-weight: 700;
      }

      .pos-coef {
        font-size: 12px;
        opacity: 0.8;
      }

      &.mid { background: #fef3c7; color: #92400e; }
      &.adc { background: #fee2e2; color: #991b1b; }
      &.jug { background: #d1fae5; color: #065f46; }
      &.top { background: #dbeafe; color: #1e40af; }
      &.sup { background: #e0e7ff; color: #3730a3; }
    }
  }

  .salary-example {
    text-align: center;
    font-size: 13px;
    color: #6b7280;
    padding: 8px;
    background: white;
    border-radius: 6px;
    border: 1px dashed #e5e7eb;

    strong {
      color: #059669;
    }
  }

  .contract-years {
    display: flex;
    gap: 12px;

    .year-item {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px 16px;
      background: white;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .year-badge {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 14px;
        color: white;

        &.long { background: #059669; }
        &.medium { background: #3b82f6; }
        &.short { background: #f59e0b; }
      }

      .year-condition {
        font-size: 12px;
        color: #6b7280;
      }
    }
  }

  .tips-list {
    display: flex;
    flex-direction: column;
    gap: 8px;

    .tip-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 10px 12px;
      background: #eff6ff;
      border-radius: 8px;
      font-size: 13px;
      color: #1e40af;

      .tip-icon {
        color: #3b82f6;
        font-size: 16px;
      }
    }
  }

  // 游戏概述 - 系统网格
  .system-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .system-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px;
      background: #f9fafb;
      border-radius: 10px;
      border: 1px solid #e5e7eb;
      transition: all 0.2s ease;

      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
      }

      .system-icon {
        width: 42px;
        height: 42px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 20px;
        color: white;

        &.time { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.match { background: linear-gradient(135deg, #3b82f6, #2563eb); }
        &.honor { background: linear-gradient(135deg, #eab308, #ca8a04); }
        &.points { background: linear-gradient(135deg, #22c55e, #16a34a); }
        &.data { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
        &.transfer { background: linear-gradient(135deg, #06b6d4, #0891b2); }
      }

      .system-info {
        flex: 1;

        .system-name {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }

        .system-desc {
          font-size: 12px;
          color: #9ca3af;
          margin-top: 2px;
        }
      }
    }
  }

  // 游戏目标
  .goals-list {
    display: flex;
    flex-direction: column;
    gap: 10px;

    .goal-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px 16px;
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      border-radius: 10px;
      font-size: 14px;
      color: #92400e;

      .goal-icon {
        font-size: 20px;
        color: #d97706;
      }
    }
  }

  // 荣誉系统 - 荣誉网格
  .honor-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .honor-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px;
      border-radius: 10px;
      border: 1px solid transparent;

      .honor-badge {
        width: 44px;
        height: 44px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 22px;
        color: white;
      }

      .honor-info {
        .honor-name {
          font-size: 14px;
          font-weight: 600;
        }
        .honor-desc {
          font-size: 12px;
          opacity: 0.8;
          margin-top: 2px;
        }
      }

      &.gold {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        .honor-badge { background: linear-gradient(135deg, #f59e0b, #d97706); }
        .honor-name { color: #92400e; }
        .honor-desc { color: #b45309; }
      }

      &.silver {
        background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
        .honor-badge { background: linear-gradient(135deg, #9ca3af, #6b7280); }
        .honor-name { color: #374151; }
        .honor-desc { color: #6b7280; }
      }

      &.bronze {
        background: linear-gradient(135deg, #fef3c7, #fed7aa);
        .honor-badge { background: linear-gradient(135deg, #f97316, #ea580c); }
        .honor-name { color: #9a3412; }
        .honor-desc { color: #c2410c; }
      }

      &.fourth {
        background: linear-gradient(135deg, #e0e7ff, #c7d2fe);
        .honor-badge { background: linear-gradient(135deg, #6366f1, #4f46e5); }
        .honor-name { color: #3730a3; }
        .honor-desc { color: #4f46e5; }
      }

      &.regular {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        .honor-badge { background: linear-gradient(135deg, #3b82f6, #2563eb); }
        .honor-name { color: #1e40af; }
        .honor-desc { color: #2563eb; }
      }

      &.player-champion {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        .honor-badge { background: linear-gradient(135deg, #10b981, #059669); }
        .honor-name { color: #065f46; }
        .honor-desc { color: #059669; }
      }

      &.mvp {
        background: linear-gradient(135deg, #fce7f3, #fbcfe8);
        .honor-badge { background: linear-gradient(135deg, #ec4899, #db2777); }
        .honor-name { color: #9d174d; }
        .honor-desc { color: #db2777; }
      }
    }
  }

  // 荣誉颁发时机流程
  .timing-flow {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-top: 12px;

    .timing-step {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 16px 20px;
      background: #f9fafb;
      border-radius: 10px;
      border: 1px solid #e5e7eb;

      .timing-icon {
        width: 40px;
        height: 40px;
        border-radius: 10px;
        background: linear-gradient(135deg, #3b82f6, #2563eb);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 20px;
      }

      .timing-content {
        .timing-title {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }
        .timing-desc {
          font-size: 12px;
          color: #9ca3af;
          margin-top: 2px;
        }
      }
    }

    .timing-arrow {
      color: #d1d5db;
      font-size: 18px;
    }
  }

  // 三层模型
  .three-layer-model {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 16px;

    .layer-item {
      background: #f9fafb;
      border-radius: 12px;
      border: 1px solid #e5e7eb;
      overflow: hidden;

      .layer-header {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 12px 16px;
        background: #f3f4f6;
        border-bottom: 1px solid #e5e7eb;

        .layer-num {
          width: 28px;
          height: 28px;
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          color: white;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 700;
          font-size: 14px;
        }

        .layer-title {
          font-size: 15px;
          font-weight: 600;
          color: #374151;
        }
      }

      .layer-content {
        padding: 16px;

        .formula-box {
          background: #1f2937;
          padding: 12px 16px;
          border-radius: 8px;
          margin-bottom: 12px;

          .formula {
            font-family: 'Courier New', monospace;
            color: #34d399;
            font-size: 14px;
            margin: 0;
          }

          .formula-note {
            font-size: 12px;
            color: #9ca3af;
            margin-top: 6px;
          }
        }

        .layer-desc {
          p {
            font-size: 13px;
            color: #6b7280;
            margin: 0;
          }
        }
      }
    }

    .layer-arrow {
      display: flex;
      justify-content: center;
      color: #d1d5db;
      font-size: 20px;
    }
  }

  // 爆冷提示
  .upset-tips {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;

    .tip-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 10px 14px;
      background: #eff6ff;
      border-radius: 8px;
      font-size: 13px;
      color: #1e40af;

      .tip-icon {
        color: #3b82f6;
        font-size: 16px;
      }
    }
  }

  // Super 赛不颁发积分提示
  .no-points-notice {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 16px;
    background: linear-gradient(135deg, #fef3c7, #fde68a);
    border-radius: 10px;
    border: 1px solid #f59e0b;

    .notice-icon {
      font-size: 24px;
      color: #d97706;
      flex-shrink: 0;
    }

    .notice-content {
      .notice-title {
        font-size: 15px;
        font-weight: 600;
        color: #92400e;
        margin-bottom: 6px;
      }

      .notice-desc {
        font-size: 13px;
        color: #b45309;
        line-height: 1.5;
      }
    }
  }

  // 财政系统 - 收入网格
  .income-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;

    .income-item {
      display: flex;
      align-items: flex-start;
      gap: 14px;
      padding: 16px;
      border-radius: 12px;
      border: 1px solid transparent;

      .income-icon {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 24px;
        color: white;
        flex-shrink: 0;
      }

      .income-info {
        flex: 1;

        .income-name {
          font-size: 15px;
          font-weight: 600;
          margin-bottom: 4px;
        }

        .income-amount {
          font-size: 14px;
          font-weight: 700;
          margin-bottom: 6px;
        }

        .income-desc {
          font-size: 12px;
          opacity: 0.8;
        }
      }

      &.league {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        .income-icon { background: linear-gradient(135deg, #3b82f6, #2563eb); }
        .income-name { color: #1e40af; }
        .income-amount { color: #1d4ed8; }
        .income-desc { color: #3b82f6; }
      }

      &.prize {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        .income-icon { background: linear-gradient(135deg, #f59e0b, #d97706); }
        .income-name { color: #92400e; }
        .income-amount { color: #b45309; }
        .income-desc { color: #d97706; }
      }

      &.sponsor {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        .income-icon { background: linear-gradient(135deg, #10b981, #059669); }
        .income-name { color: #065f46; }
        .income-amount { color: #047857; }
        .income-desc { color: #059669; }
      }

      &.transfer {
        background: linear-gradient(135deg, #fce7f3, #fbcfe8);
        .income-icon { background: linear-gradient(135deg, #ec4899, #db2777); }
        .income-name { color: #9d174d; }
        .income-amount { color: #be185d; }
        .income-desc { color: #db2777; }
      }
    }
  }

  // 财政系统 - 支出网格
  .expense-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .expense-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px;
      background: #fef2f2;
      border-radius: 10px;
      border: 1px solid #fecaca;

      .expense-icon {
        width: 40px;
        height: 40px;
        border-radius: 10px;
        background: linear-gradient(135deg, #ef4444, #dc2626);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 18px;
        color: white;
        flex-shrink: 0;
      }

      .expense-info {
        .expense-name {
          font-size: 14px;
          font-weight: 600;
          color: #991b1b;
        }

        .expense-desc {
          font-size: 12px;
          color: #b91c1c;
          margin-top: 2px;
        }
      }
    }
  }

  // 财政系统 - 财务状态网格
  .finance-status-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 10px;
    margin-top: 12px;

    .status-item {
      padding: 16px 12px;
      border-radius: 10px;
      text-align: center;

      .status-badge {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        font-size: 14px;
        font-weight: 600;
        margin-bottom: 8px;
      }

      .status-range {
        font-size: 13px;
        font-weight: 500;
        margin-bottom: 6px;
      }

      .status-strategy {
        font-size: 11px;
        opacity: 0.8;
      }

      &.wealthy {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        .status-badge { color: #065f46; }
        .status-range { color: #047857; }
        .status-strategy { color: #059669; }
      }

      &.healthy {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        .status-badge { color: #1e40af; }
        .status-range { color: #1d4ed8; }
        .status-strategy { color: #3b82f6; }
      }

      &.tight {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        .status-badge { color: #92400e; }
        .status-range { color: #b45309; }
        .status-strategy { color: #d97706; }
      }

      &.deficit {
        background: linear-gradient(135deg, #fee2e2, #fecaca);
        .status-badge { color: #991b1b; }
        .status-range { color: #b91c1c; }
        .status-strategy { color: #dc2626; }
      }

      &.bankrupt {
        background: linear-gradient(135deg, #1f2937, #374151);
        .status-badge { color: #f87171; }
        .status-range { color: #fca5a5; }
        .status-strategy { color: #fecaca; }
      }
    }
  }

  // 财政系统 - 奖金池
  .prize-tabs {
    margin-top: 12px;
  }

  .prize-table {
    .prize-header {
      padding: 12px 16px;
      background: #f9fafb;
      border-radius: 8px 8px 0 0;
      border: 1px solid #e5e7eb;
      border-bottom: none;
      display: flex;
      align-items: center;
      gap: 12px;

      .prize-total {
        font-size: 14px;
        color: #6b7280;

        strong {
          color: #059669;
          font-size: 16px;
        }
      }

      .prize-note {
        font-size: 12px;
        color: #9ca3af;
        padding: 2px 8px;
        background: #f3f4f6;
        border-radius: 4px;

        &.highlight {
          background: linear-gradient(135deg, #fef3c7, #fde68a);
          color: #b45309;
          font-weight: 600;
        }
      }
    }

    .prize-list {
      border: 1px solid #e5e7eb;
      border-radius: 0 0 8px 8px;
      overflow: hidden;

      .prize-row {
        display: flex;
        justify-content: space-between;
        padding: 12px 16px;
        border-bottom: 1px solid #f3f4f6;

        &:last-child {
          border-bottom: none;
        }

        .prize-rank {
          font-size: 14px;
          font-weight: 500;
          color: #374151;
        }

        .prize-amount {
          font-size: 14px;
          font-weight: 600;
          color: #059669;
        }

        &.gold {
          background: linear-gradient(135deg, #fef3c7, #fde68a);
          .prize-rank { color: #92400e; font-weight: 600; }
          .prize-amount { color: #b45309; }
        }

        &.silver {
          background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
          .prize-rank { color: #374151; font-weight: 600; }
          .prize-amount { color: #4b5563; }
        }

        &.bronze {
          background: linear-gradient(135deg, #fed7aa, #fdba74);
          .prize-rank { color: #9a3412; font-weight: 600; }
          .prize-amount { color: #c2410c; }
        }
      }
    }
  }

  // 财政系统 - 赞助公式
  .sponsor-formula {
    .sponsor-factors {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 12px;
      margin-top: 12px;

      .factor-item {
        padding: 14px;
        background: #f9fafb;
        border-radius: 10px;
        border: 1px solid #e5e7eb;
        text-align: center;

        .factor-name {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
          margin-bottom: 8px;
        }

        .factor-formula {
          font-size: 12px;
          color: #6b7280;
          margin-bottom: 8px;
          font-family: 'Courier New', monospace;
        }

        .factor-example {
          font-size: 12px;
          padding: 6px 10px;
          background: #d1fae5;
          color: #065f46;
          border-radius: 6px;
          display: inline-block;
        }
      }
    }
  }

  // 数据中心 - 数据统计网格
  .data-stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;

    .stat-category {
      background: #f9fafb;
      border-radius: 12px;
      border: 1px solid #e5e7eb;
      overflow: hidden;

      .category-header {
        padding: 12px 16px;
        background: linear-gradient(135deg, #3b82f6, #2563eb);
        color: white;
        font-weight: 600;
        font-size: 14px;
      }

      .stat-items {
        padding: 12px;

        .stat-item {
          display: flex;
          justify-content: space-between;
          padding: 8px 12px;
          border-bottom: 1px dashed #e5e7eb;

          &:last-child {
            border-bottom: none;
          }

          .stat-name {
            font-weight: 600;
            color: #374151;
          }

          .stat-desc {
            font-size: 12px;
            color: #9ca3af;
          }
        }
      }
    }
  }

  // 数据中心 - 影响力因素卡片
  .impact-factors {
    display: flex;
    justify-content: space-around;
    gap: 12px;
    margin-top: 16px;

    .factor-card {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 16px 20px;
      border-radius: 12px;
      min-width: 100px;

      .factor-icon {
        font-size: 28px;
      }

      .factor-info {
        .factor-name {
          font-size: 14px;
          font-weight: 600;
        }

        .factor-weight {
          font-size: 18px;
          font-weight: 700;
        }
      }

      &.kill {
        background: linear-gradient(135deg, #fee2e2, #fecaca);
        .factor-info { color: #991b1b; }
      }

      &.assist {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        .factor-info { color: #1e40af; }
      }

      &.death {
        background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
        .factor-info { color: #374151; }
      }

      &.win {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        .factor-info { color: #065f46; }
      }
    }
  }

  // 数据中心 - MVP流程
  .mvp-process {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-top: 16px;

    .process-step {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px 18px;
      background: #f9fafb;
      border-radius: 10px;
      border: 1px solid #e5e7eb;

      .step-num {
        width: 32px;
        height: 32px;
        background: linear-gradient(135deg, #3b82f6, #2563eb);
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 14px;
      }

      .step-content {
        .step-title {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }

        .step-desc {
          font-size: 12px;
          color: #9ca3af;
        }
      }

      &.highlight {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border-color: #f59e0b;

        .step-num {
          background: linear-gradient(135deg, #f59e0b, #d97706);
        }

        .step-title { color: #92400e; }
        .step-desc { color: #b45309; }
      }
    }

    .process-arrow {
      color: #d1d5db;
      font-size: 18px;
    }
  }

  .mvp-bonus-info {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 16px;
    padding: 12px 16px;
    background: #fef3c7;
    border-radius: 8px;
    font-size: 13px;
    color: #92400e;
  }

  // 数据中心 - 最佳阵容
  .all-pro-teams {
    display: flex;
    gap: 16px;
    margin-top: 12px;

    .all-pro-item {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 14px;
      padding: 16px;
      border-radius: 12px;
      border: 2px solid transparent;

      .all-pro-badge {
        width: 50px;
        height: 50px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 18px;
        color: white;
      }

      .all-pro-info {
        flex: 1;

        .all-pro-title {
          font-size: 15px;
          font-weight: 600;
        }

        .all-pro-desc {
          font-size: 12px;
          margin: 4px 0;
        }

        .all-pro-bonus {
          font-size: 12px;
          font-weight: 600;
        }
      }

      &.first {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border-color: #f59e0b;

        .all-pro-badge { background: linear-gradient(135deg, #f59e0b, #d97706); }
        .all-pro-title { color: #92400e; }
        .all-pro-desc { color: #b45309; }
        .all-pro-bonus { color: #d97706; }
      }

      &.second {
        background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
        border-color: #9ca3af;

        .all-pro-badge { background: linear-gradient(135deg, #9ca3af, #6b7280); }
        .all-pro-title { color: #374151; }
        .all-pro-desc { color: #6b7280; }
        .all-pro-bonus { color: #4b5563; }
      }

      &.third {
        background: linear-gradient(135deg, #fed7aa, #fdba74);
        border-color: #f97316;

        .all-pro-badge { background: linear-gradient(135deg, #f97316, #ea580c); }
        .all-pro-title { color: #9a3412; }
        .all-pro-desc { color: #c2410c; }
        .all-pro-bonus { color: #ea580c; }
      }
    }
  }

  // 数据中心 - Top20层级
  .top20-tiers {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin: 16px 0;

    .tier-item {
      padding: 16px 24px;
      border-radius: 10px;
      text-align: center;
      min-width: 120px;

      .tier-rank {
        font-size: 16px;
        font-weight: 700;
        margin-bottom: 6px;
      }

      .tier-bonus {
        font-size: 13px;
        font-weight: 600;
      }

      &.tier-top {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        .tier-rank { color: #92400e; }
        .tier-bonus { color: #d97706; }
      }

      &.tier-mid {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        .tier-rank { color: #1e40af; }
        .tier-bonus { color: #2563eb; }
      }

      &.tier-low {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        .tier-rank { color: #065f46; }
        .tier-bonus { color: #059669; }
      }
    }
  }

  .top20-scoring {
    display: flex;
    flex-direction: column;
    gap: 8px;

    .scoring-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 14px;
      background: #f9fafb;
      border-radius: 8px;
      font-size: 13px;
      color: #4b5563;
    }
  }

  // 数据中心 - 新秀条件
  .rookie-criteria {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;

    .criteria-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 12px 16px;
      background: linear-gradient(135deg, #d1fae5, #a7f3d0);
      border-radius: 8px;
      font-size: 14px;
      color: #065f46;

      .el-icon {
        color: #10b981;
        font-size: 18px;
      }
    }
  }

  // 忠诚度表格
  .loyalty-table {
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #e5e7eb;

    .loyalty-row {
      display: grid;
      grid-template-columns: 80px 100px 100px 1fr;
      padding: 12px 16px;
      font-size: 13px;

      &.header {
        background: #f9fafb;
        font-weight: 600;
        color: #6b7280;
      }

      &.devoted {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        color: #065f46;
      }

      &.loyal {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        color: #1e40af;
      }

      &.neutral {
        background: #f9fafb;
        color: #374151;
      }

      &.opportunist {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        color: #92400e;
      }

      &.mercenary {
        background: linear-gradient(135deg, #fee2e2, #fecaca);
        color: #991b1b;
      }
    }
  }

  .loyalty-effects {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 12px;

    .effect-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 10px 14px;
      background: #f9fafb;
      border-radius: 8px;
      font-size: 13px;
      color: #4b5563;
    }
  }

  .satisfaction-factors {
    margin-top: 12px;

    .factor-title {
      font-size: 13px;
      font-weight: 600;
      color: #374151;
      margin-bottom: 8px;
    }

    .factor-grid {
      display: flex;
      gap: 12px;

      .factor-item {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 12px 16px;
        border-radius: 8px;
        font-size: 13px;

        &.positive {
          background: #d1fae5;
          color: #065f46;
        }

        &.negative {
          background: #fee2e2;
          color: #991b1b;
        }
      }
    }
  }

  // 荣誉加成表格
  .honor-bonus-table {
    .bonus-category {
      margin-bottom: 16px;

      .category-title {
        font-size: 14px;
        font-weight: 600;
        color: #374151;
        margin-bottom: 10px;
        padding-left: 10px;
        border-left: 3px solid #3b82f6;
      }

      .bonus-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;

        .bonus-item {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 12px 16px;
          background: #f9fafb;
          border-radius: 8px;
          border: 1px solid #e5e7eb;
          min-width: 100px;

          .bonus-name {
            font-size: 12px;
            color: #6b7280;
            margin-bottom: 4px;
          }

          .bonus-value {
            font-size: 18px;
            font-weight: 700;
            color: #059669;
          }

          .bonus-duration {
            font-size: 11px;
            color: #9ca3af;
            margin-top: 4px;
          }

          &.worlds {
            background: linear-gradient(135deg, #fef3c7, #fde68a);
            border-color: #f59e0b;
            .bonus-value { color: #d97706; }
          }

          &.msi {
            background: linear-gradient(135deg, #dbeafe, #bfdbfe);
            border-color: #3b82f6;
            .bonus-value { color: #2563eb; }
          }

          &.mvp {
            background: linear-gradient(135deg, #fce7f3, #fbcfe8);
            border-color: #ec4899;
            .bonus-value { color: #db2777; }
          }

          &.regional {
            background: linear-gradient(135deg, #d1fae5, #a7f3d0);
            border-color: #10b981;
            .bonus-value { color: #059669; }
          }

          &.rookie {
            background: linear-gradient(135deg, #e0e7ff, #c7d2fe);
            border-color: #6366f1;
            .bonus-value { color: #4f46e5; }
          }
        }
      }
    }

    .honor-formula {
      margin-top: 16px;

      .formula-title {
        font-size: 14px;
        font-weight: 600;
        color: #374151;
        margin-bottom: 10px;
      }

      .formula-content {
        .formula-example {
          display: flex;
          flex-direction: column;
          gap: 4px;
          padding: 12px 16px;
          background: #f9fafb;
          border-radius: 8px;
          font-size: 13px;
          color: #6b7280;
          margin-top: 10px;

          .calc-result {
            color: #374151;
            strong { color: #059669; }
          }
        }
      }
    }
  }

  // 身价系数药丸
  .coefficient-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 12px 0;

    .coef-pill {
      padding: 8px 14px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: 500;

      &.high { background: #d1fae5; color: #065f46; }
      &.medium { background: #dbeafe; color: #1e40af; }
      &.normal { background: #f3f4f6; color: #374151; }
      &.low { background: #fef3c7; color: #92400e; }
      &.lower { background: #fed7aa; color: #9a3412; }
      &.lowest { background: #fee2e2; color: #991b1b; }
    }
  }

  // 身价计算相关
  .market-value-section {
    margin-top: 16px;

    .mv-title {
      font-size: 14px;
      font-weight: 600;
      color: #374151;
      margin-bottom: 10px;
    }

    .mv-table {
      border-radius: 8px;
      overflow: hidden;
      border: 1px solid #e5e7eb;

      &.compact .mv-row {
        padding: 8px 16px;
      }

      .mv-row {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        padding: 10px 16px;
        font-size: 13px;
        border-bottom: 1px solid #f3f4f6;

        &:last-child {
          border-bottom: none;
        }

        &.header {
          background: #f9fafb;
          font-weight: 600;
          color: #6b7280;
        }
      }
    }

    .other-coefficients {
      display: flex;
      flex-direction: column;
      gap: 12px;

      .coef-group {
        padding: 12px 16px;
        background: #f9fafb;
        border-radius: 8px;

        .coef-label {
          font-size: 13px;
          font-weight: 600;
          color: #374151;
          margin-bottom: 8px;
        }

        .coef-values {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;

          span {
            padding: 4px 10px;
            background: white;
            border-radius: 4px;
            font-size: 12px;
            color: #6b7280;
            border: 1px solid #e5e7eb;
          }
        }
      }
    }

    .update-timing {
      display: flex;
      flex-direction: column;
      gap: 8px;

      .timing-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 14px;
        background: #f9fafb;
        border-radius: 8px;
        font-size: 13px;
        color: #4b5563;
      }
    }
  }

  // GM人格系统
  .gm-personality-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .personality-card {
      padding: 16px;
      border-radius: 12px;
      border: 2px solid transparent;

      .personality-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 12px;

        .personality-icon {
          font-size: 24px;
        }

        .personality-name {
          font-size: 15px;
          font-weight: 700;
        }
      }

      .personality-stats {
        margin-bottom: 12px;

        .stat-row {
          display: flex;
          justify-content: space-between;
          padding: 6px 0;
          font-size: 12px;
          border-bottom: 1px dashed rgba(0,0,0,0.1);

          &:last-child {
            border-bottom: none;
          }

          .stat-label {
            color: inherit;
            opacity: 0.8;
          }

          .stat-value {
            font-weight: 600;
          }
        }
      }

      .personality-desc {
        font-size: 12px;
        opacity: 0.8;
      }

      &.championship {
        background: linear-gradient(135deg, #fef3c7, #fde68a);
        border-color: #f59e0b;
        color: #92400e;
      }

      &.youth {
        background: linear-gradient(135deg, #d1fae5, #a7f3d0);
        border-color: #10b981;
        color: #065f46;
      }

      &.balanced {
        background: linear-gradient(135deg, #dbeafe, #bfdbfe);
        border-color: #3b82f6;
        color: #1e40af;
      }

      &.speculator {
        background: linear-gradient(135deg, #fce7f3, #fbcfe8);
        border-color: #ec4899;
        color: #9d174d;
      }

      &.rebuilding {
        background: linear-gradient(135deg, #e0e7ff, #c7d2fe);
        border-color: #6366f1;
        color: #3730a3;
      }

      &.custom {
        background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
        border-color: #9ca3af;
        color: #374151;
      }
    }
  }

  .gm-features {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 16px;

    .feature-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 10px 14px;
      background: #f9fafb;
      border-radius: 8px;
      font-size: 13px;
      color: #4b5563;
    }
  }

  // Avatar图标
  .el-icon.avatar-icon {
    background: linear-gradient(135deg, #8b5cf6, #7c3aed);
    color: white;
    border-radius: 8px;
    padding: 8px;
  }
}

@media (max-width: 768px) {
  .game-guide {
    .points-tables {
      grid-template-columns: 1fr;
    }

    .attribute-grid {
      grid-template-columns: 1fr;
    }

    .finance-states {
      grid-template-columns: repeat(2, 1fr);
    }

    .coefficient-grid {
      grid-template-columns: 1fr;
    }

    .accept-conditions {
      grid-template-columns: 1fr;
    }

    .contract-years {
      flex-direction: column;
    }

    .decision-flow {
      flex-direction: column;

      .flow-connector {
        transform: rotate(90deg);
      }
    }

    .position-salary {
      flex-wrap: wrap;
      justify-content: center;
    }

    .system-grid {
      grid-template-columns: 1fr;
    }

    .honor-grid {
      grid-template-columns: 1fr;
    }

    .timing-flow {
      flex-direction: column;

      .timing-arrow {
        transform: rotate(90deg);
      }
    }

    .three-layer-model {
      .layer-arrow {
        .el-icon {
          transform: none;
        }
      }
    }

    .income-grid {
      grid-template-columns: 1fr;
    }

    .expense-grid {
      grid-template-columns: 1fr;
    }

    .finance-status-grid {
      grid-template-columns: repeat(2, 1fr);

      .status-item:last-child {
        grid-column: span 2;
      }
    }

    .sponsor-factors {
      grid-template-columns: 1fr !important;
    }

    // 数据中心响应式
    .data-stats-grid {
      grid-template-columns: 1fr;
    }

    .impact-factors {
      flex-wrap: wrap;
      justify-content: center;
    }

    .mvp-process {
      flex-direction: column;

      .process-arrow {
        transform: rotate(90deg);
      }
    }

    .all-pro-teams {
      flex-direction: column;
    }

    .top20-tiers {
      flex-direction: column;
      align-items: center;
    }

    // GM人格响应式
    .gm-personality-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    // 忠诚度表格响应式
    .loyalty-table .loyalty-row {
      grid-template-columns: 60px 80px 80px 1fr;
      font-size: 11px;
      padding: 8px 12px;
    }

    .satisfaction-factors .factor-grid {
      flex-direction: column;
    }

    // 身价系数响应式
    .coefficient-pills {
      justify-content: center;
    }

    .market-value-section .mv-table .mv-row {
      grid-template-columns: 1fr 1fr 1fr;
      font-size: 11px;
    }
  }
}
</style>
