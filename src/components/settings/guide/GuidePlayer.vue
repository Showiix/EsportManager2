<template>
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
</template>

<script setup lang="ts">
import {
  DataLine,
  Aim,
  TrendCharts,
  SetUp,
  Clock,
  PriceTag,
  Histogram,
  StarFilled,
  Star,
  Top,
  Bottom,
  Money
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
}

@media (max-width: 768px) {
  .guide-content {
    .attribute-grid {
      grid-template-columns: 1fr;
    }

    .loyalty-table .loyalty-row {
      grid-template-columns: 60px 80px 80px 1fr;
      font-size: 11px;
      padding: 8px 12px;
    }

    .satisfaction-factors .factor-grid {
      flex-direction: column;
    }

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
