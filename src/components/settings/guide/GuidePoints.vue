<template>
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
</template>

<script setup lang="ts">
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
.guide-content {
  padding: 8px 0;

  .guide-desc {
    color: #6b7280;
    margin-bottom: 16px;
    font-size: 14px;
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
}

@media (max-width: 768px) {
  .guide-content {
    .points-tables {
      grid-template-columns: 1fr;
    }
  }
}
</style>
