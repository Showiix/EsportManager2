<template>
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
          <div class="factor-icon"><el-icon><Trophy /></el-icon></div>
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
</template>

<script setup lang="ts">
import {
  DataAnalysis,
  Star,
  Trophy,
  ArrowRight,
  Medal,
  StarFilled,
  User,
  CircleCheck
} from '@element-plus/icons-vue'
</script>

<style scoped lang="scss">
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

  // 公式展示
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

    .formula-note {
      font-size: 12px;
      color: #9ca3af;
      margin-top: 6px;
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
}

@media (max-width: 768px) {
  .guide-content {
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
  }
}
</style>
