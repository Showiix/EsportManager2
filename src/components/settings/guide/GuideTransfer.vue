<template>
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
            <span class="personality-icon"><el-icon><Trophy /></el-icon></span>
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
            <span class="personality-icon"><el-icon><TrendCharts /></el-icon></span>
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
            <span class="personality-icon"><el-icon><Setting /></el-icon></span>
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
</template>

<script setup lang="ts">
import {
  Timer,
  List,
  Cpu,
  Search,
  DataAnalysis,
  Money,
  Check,
  Wallet,
  Calendar,
  InfoFilled,
  Avatar,
  Trophy,
  TrendCharts,
  Setting,
  CircleCheckFilled,
  CircleCheck,
  WarningFilled,
  CircleCloseFilled,
  ArrowRight
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
}

@media (max-width: 768px) {
  .guide-content {
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

    // GM人格响应式
    .gm-personality-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}
</style>
