<template>
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
</template>

<script setup lang="ts">
import {
  DataAnalysis,
  Bottom,
  Warning,
  SuccessFilled,
  InfoFilled
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
}

@media (max-width: 768px) {
  .guide-content {
    .three-layer-model {
      .layer-arrow {
        .el-icon {
          transform: none;
        }
      }
    }
  }
}
</style>
