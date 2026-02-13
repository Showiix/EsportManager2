<template>
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
</template>

<script setup lang="ts">
import {
  Trophy,
  Medal,
  Star,
  User,
  StarFilled,
  Clock,
  CircleCheck,
  ArrowRight,
  DataAnalysis,
  Money,
  Reading
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
}

@media (max-width: 768px) {
  .guide-content {
    .honor-grid {
      grid-template-columns: 1fr;
    }

    .timing-flow {
      flex-direction: column;

      .timing-arrow {
        transform: rotate(90deg);
      }
    }
  }
}
</style>
