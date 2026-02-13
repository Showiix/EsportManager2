<template>
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
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  TrendCharts,
  Trophy,
  Medal,
  Promotion,
  Sell,
  Money,
  User,
  Goods,
  Setting,
  DataLine,
  CircleCheckFilled,
  CircleCheck,
  WarningFilled,
  CircleCloseFilled,
  WarnTriangleFilled,
  Coin,
  DataAnalysis,
  InfoFilled
} from '@element-plus/icons-vue'

const activePrizeTab = ref('worlds')
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
  .guide-content {
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
  }
}
</style>
