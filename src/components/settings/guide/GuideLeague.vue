<template>
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
      <p class="guide-desc">常规赛前8名进入季后赛，1-4名进胜者组，5-8名进败者组</p>

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
</template>

<script setup lang="ts">
import {
  List,
  Trophy,
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
}
</style>
