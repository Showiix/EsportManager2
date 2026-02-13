<template>
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
</template>

<script setup lang="ts">
import {
  Calendar,
  Memo
} from '@element-plus/icons-vue'
</script>

<style scoped lang="scss">
.guide-content {
  padding: 8px 0;

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
}
</style>
