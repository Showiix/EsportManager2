<template>
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
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  ArrowRight,
  InfoFilled
} from '@element-plus/icons-vue'

const activeEventTab = ref('msi')
</script>

<style scoped lang="scss">
.guide-content {
  padding: 8px 0;

  .guide-desc {
    color: #6b7280;
    margin-bottom: 16px;
    font-size: 14px;
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
}
</style>
