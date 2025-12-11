<template>
  <div class="game-guide">
    <el-collapse v-model="activeNames" accordion>
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
                  <div class="section-title">积分分配</div>
                  <div class="points-distribution">
                    <div class="point-item gold"><span class="rank">冠军</span><span class="pts">+50</span></div>
                    <div class="point-item silver"><span class="rank">亚军</span><span class="pts">+40</span></div>
                    <div class="point-item bronze"><span class="rank">3-4名</span><span class="pts">+30</span></div>
                    <div class="point-item fourth"><span class="rank">5-8名</span><span class="pts">+20</span></div>
                    <div class="point-item other"><span class="rank">9-16名</span><span class="pts">+10</span></div>
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
            <h4><el-icon><DataAnalysis /></el-icon> 正态分布算法</h4>
            <p>比赛结果基于<strong>正态分布（高斯分布）</strong>算法，战力高的队伍获胜概率更大，但存在爆冷可能。</p>

            <div class="formula-box">
              <div class="formula">发挥值 = 正态分布(战力值, 标准差σ=6)</div>
              <div class="formula-desc">每局独立计算，发挥值高者获胜</div>
            </div>

            <div class="probability-visual">
              <div class="prob-title">发挥值分布示意</div>
              <div class="prob-chart">
                <div class="prob-bar" style="height: 30%;">
                  <span>1%</span>
                </div>
                <div class="prob-bar" style="height: 50%;">
                  <span>14%</span>
                </div>
                <div class="prob-bar" style="height: 90%;">
                  <span>34%</span>
                </div>
                <div class="prob-bar highlight" style="height: 100%;">
                  <span>战力值</span>
                </div>
                <div class="prob-bar" style="height: 90%;">
                  <span>34%</span>
                </div>
                <div class="prob-bar" style="height: 50%;">
                  <span>14%</span>
                </div>
                <div class="prob-bar" style="height: 30%;">
                  <span>1%</span>
                </div>
              </div>
              <div class="prob-labels">
                <span>-18</span>
                <span>-12</span>
                <span>-6</span>
                <span>0</span>
                <span>+6</span>
                <span>+12</span>
                <span>+18</span>
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
  InfoFilled
} from '@element-plus/icons-vue'

const activeNames = ref('season')
const activeEventTab = ref('msi')

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
  }
}
</style>
