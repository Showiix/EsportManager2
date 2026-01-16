<template>
  <div class="llm-transfer-market-v2">
    <!-- ==================== 页面标题 ==================== -->
    <div class="page-header">
      <div class="header-left">
        <h1>
          <el-icon><TrendCharts /></el-icon>
          转会市场
        </h1>
        <p class="season-info">第{{ currentSeason }}赛季 - {{ currentPhaseName }}</p>
      </div>
      <div class="header-actions">
        <el-button @click="handleRefresh" :loading="isLoading">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button type="danger" plain @click="handleResetMarket">
          <el-icon><Delete /></el-icon>
          重置市场
        </el-button>
        <el-button @click="router.back()">
          <el-icon><Back /></el-icon>
          返回
        </el-button>
      </div>
    </div>

    <!-- ==================== 执行模式配置 ==================== -->
    <el-card v-if="isMarketInitialized && !isMarketComplete" class="mode-config-card">
      <template #header>
        <div class="mode-card-header">
          <div class="mode-header-left">
            <el-icon size="24"><Setting /></el-icon>
            <span class="mode-card-title">执行模式配置</span>
          </div>
          <el-switch
            v-model="useRuleEngine"
            size="large"
            inline-prompt
            active-text="规则引擎"
            inactive-text="LLM引擎"
            style="--el-switch-on-color: #67c23a; --el-switch-off-color: #409eff;"
          />
        </div>
      </template>

      <div v-if="useRuleEngine" class="mode-content">
        <!-- 规则引擎模式 -->
        <div class="mode-banner rule-engine">
          <div class="banner-icon">⚡</div>
          <div class="banner-content">
            <h3>规则引擎模式（NBA 2K 风格）</h3>
            <p>基于数据驱动的纯规则决策系统，高效、稳定、零成本</p>
          </div>
        </div>

        <el-row :gutter="20" style="margin-top: 20px;">
          <el-col :span="12">
            <el-card shadow="hover" class="feature-card">
              <div class="feature-header">
                <el-icon size="20" color="#409eff"><DataAnalysis /></el-icon>
                <h4>核心算法：7维度兴趣度评分</h4>
              </div>
              <div class="feature-content">
                <el-space wrap>
                  <el-tag effect="dark">位置匹配 30分</el-tag>
                  <el-tag effect="dark">能力匹配 30分</el-tag>
                  <el-tag effect="dark">年龄偏好 15分</el-tag>
                  <el-tag effect="dark">性价比 15分</el-tag>
                  <el-tag effect="dark">荣誉加成 10分</el-tag>
                  <el-tag effect="dark">表现加成 10分</el-tag>
                  <el-tag effect="dark">潜力加成 10分</el-tag>
                </el-space>
                <p class="feature-desc">
                  根据GM人格调整权重，不同球队关注不同维度。<br/>
                  争冠型重能力，青训型重潜力，稳健型重性价比。
                </p>
              </div>
            </el-card>
          </el-col>

          <el-col :span="12">
            <el-card shadow="hover" class="feature-card">
              <div class="feature-header">
                <el-icon size="20" color="#67c23a"><TrendCharts /></el-icon>
                <h4>执行流程：三层筛选</h4>
              </div>
              <div class="feature-content">
                <el-steps direction="vertical" :active="3" finish-status="success">
                  <el-step title="球探筛选" description="177选手 → 基础过滤 → 兴趣度评分 → 8个候选" />
                  <el-step title="报价决策" description="从8候选选最优 → 预算检查 → 挖角难度评估" />
                  <el-step title="选手决策" description="评估所有报价 → 综合评分 → 选择最优" />
                </el-steps>
              </div>
            </el-card>
          </el-col>
        </el-row>

        <el-row :gutter="20" style="margin-top: 20px;">
          <el-col :span="8">
            <el-card shadow="hover" class="metric-card success">
              <el-statistic title="⚡ 执行效率" value="<500" suffix="ms/轮">
                <template #prefix>
                  <el-icon color="#67c23a"><Cpu /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc">相比LLM提升 3600 倍</div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="metric-card success">
              <el-statistic title="💰 Token 成本" :value="0">
                <template #prefix>
                  <el-icon color="#67c23a"><Money /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc">纯规则计算，完全免费</div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="metric-card success">
              <el-statistic title="🎯 预期报价" value="20-40" suffix="个/轮">
                <template #prefix>
                  <el-icon color="#409eff"><Message /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc">活跃的转会市场</div>
            </el-card>
          </el-col>
        </el-row>

        <el-card shadow="hover" class="data-integration-card" style="margin-top: 20px;">
          <div class="integration-header">
            <el-icon size="20" color="#e6a23c"><Star /></el-icon>
            <h4>完整数据整合系统</h4>
          </div>
          <el-row :gutter="16">
            <el-col :span="6">
              <div class="integration-item">
                <el-icon size="32" color="#409eff"><Star /></el-icon>
                <div>
                  <strong>荣誉系统</strong>
                  <p>世界赛/MSI冠军<br/>影响兴趣度+10分</p>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="integration-item">
                <el-icon size="32" color="#67c23a"><TrendCharts /></el-icon>
                <div>
                  <strong>数据中心</strong>
                  <p>本赛季表现等级<br/>影响评分±12分</p>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="integration-item">
                <el-icon size="32" color="#e6a23c"><Money /></el-icon>
                <div>
                  <strong>身价计算</strong>
                  <p>能力×年龄×位置<br/>×赛区×荣誉</p>
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="integration-item">
                <el-icon size="32" color="#f56c6c"><User /></el-icon>
                <div>
                  <strong>满意度/忠诚度</strong>
                  <p>影响挖角难度<br/>（10-100分）</p>
                </div>
              </div>
            </el-col>
          </el-row>
        </el-card>
      </div>

      <div v-else class="mode-content">
        <!-- LLM引擎模式 -->
        <div class="mode-banner llm-engine">
          <div class="banner-icon">🤖</div>
          <div class="banner-content">
            <h3>LLM 引擎模式</h3>
            <p>AI 深度决策，完整思考过程，适合观察AI推理逻辑</p>
          </div>
        </div>

        <el-alert type="warning" :closable="false" style="margin-top: 20px;">
          <template #title>⚠️ 性能提示</template>
          <div>
            <p><strong>执行速度：</strong>每轮需要 20-30 分钟</p>
            <p><strong>Token 消耗：</strong>约 50 万 Token/轮（成本较高）</p>
            <p><strong>决策特点：</strong>可能出现"保守决策"（所有选手都不想离队，导致0个报价）</p>
            <p style="margin-top: 12px; color: #e6a23c;">
              💡 <strong>建议：</strong>切换到规则引擎模式，效率提升 3600 倍，效果更好
            </p>
          </div>
        </el-alert>

        <el-row :gutter="20" style="margin-top: 20px;">
          <el-col :span="8">
            <el-card shadow="hover" class="metric-card warning">
              <el-statistic title="🐌 执行效率" value="20-30" suffix="分钟/轮">
                <template #prefix>
                  <el-icon color="#f56c6c"><Clock /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc warning">非常慢</div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="metric-card warning">
              <el-statistic title="💸 Token 成本" value="50" suffix="万/轮">
                <template #prefix>
                  <el-icon color="#e6a23c"><Money /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc warning">成本高</div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="metric-card warning">
              <el-statistic title="😔 实际报价" value="0-5" suffix="个/轮">
                <template #prefix>
                  <el-icon color="#909399"><Message /></el-icon>
                </template>
              </el-statistic>
              <div class="metric-desc warning">过于保守</div>
            </el-card>
          </el-col>
        </el-row>
      </div>
    </el-card>

    <!-- ==================== 阶段进度 ==================== -->
    <el-card class="phase-card">
      <div class="phase-header-info">
        <span class="phase-title">市场阶段进度</span>
        <el-tag :type="phaseTagType" size="large">{{ currentPhaseName }}</el-tag>
      </div>

      <el-steps :active="currentPhaseIndex" finish-status="success" align-center>
        <el-step
          v-for="phase in phases"
          :key="phase.key"
          :title="phase.name"
        />
      </el-steps>

      <div class="phase-description">
        {{ currentPhaseDescription }}
      </div>
    </el-card>

    <!-- ==================== 实时统计 ==================== -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="4">
        <el-card class="stat-card">
          <el-statistic :value="marketState?.free_agents_count ?? 0">
            <template #title>
              <el-icon><User /></el-icon> 自由球员
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :span="4">
        <el-card class="stat-card">
          <el-statistic :value="marketState?.poachable_players_count ?? 0">
            <template #title>
              <el-icon><Star /></el-icon> 可挖角（85+）
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :span="4">
        <el-card class="stat-card">
          <el-statistic :value="marketState?.active_negotiations_count ?? 0">
            <template #title>
              <el-icon><Message /></el-icon> 进行中谈判
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :span="4">
        <el-card class="stat-card">
          <el-statistic :value="marketState?.completed_transfers_count ?? 0">
            <template #title>
              <el-icon><CircleCheck /></el-icon> 已完成转会
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :span="4">
        <el-card class="stat-card">
          <el-statistic :value="marketState?.progress_percentage ?? 0" suffix="%">
            <template #title>
              <el-icon><TrendCharts /></el-icon> 整体进度
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :span="4">
        <el-card class="stat-card">
          <template v-if="lastExecutionTime">
            <el-statistic :value="lastExecutionTime" suffix="ms">
              <template #title>
                <el-icon><Cpu /></el-icon> 上轮耗时
              </template>
            </el-statistic>
          </template>
          <template v-else>
            <el-statistic value="--">
              <template #title>
                <el-icon><Cpu /></el-icon> 上轮耗时
              </template>
            </el-statistic>
          </template>
        </el-card>
      </el-col>
    </el-row>

    <!-- ==================== 操作按钮（统一） ==================== -->
    <el-card class="action-card">
      <div class="action-container">
        <template v-if="!isMarketInitialized">
          <el-button type="primary" size="large" :loading="isLoading" @click="handleInitMarket">
            <el-icon><Plus /></el-icon>
            初始化转会市场
          </el-button>
        </template>

        <template v-else-if="!isMarketComplete">
          <!-- 主操作按钮 -->
          <el-button
            type="primary"
            size="large"
            :loading="isExecutingAction"
            @click="handlePrimaryAction"
          >
            <el-icon><CaretRight /></el-icon>
            {{ primaryActionText }}
          </el-button>

          <!-- 快进按钮（仅在自由市场和挖角阶段显示） -->
          <el-button
            v-if="canExecuteRound"
            type="warning"
            size="large"
            :loading="isExecutingAction"
            @click="handleFastForward"
          >
            <el-icon><DArrowRight /></el-icon>
            快进完成
          </el-button>

          <!-- 推进阶段按钮 -->
          <el-button
            v-if="canAdvancePhase"
            type="success"
            size="large"
            @click="handleAdvancePhase"
          >
            <el-icon><ArrowRight /></el-icon>
            推进到下一阶段
          </el-button>
        </template>

        <template v-else>
          <el-result icon="success" title="转会窗口已关闭" sub-title="所有转会事项已完成">
            <template #extra>
              <el-button type="primary" @click="router.back()">返回主页</el-button>
            </template>
          </el-result>
        </template>
      </div>
    </el-card>

    <!-- ==================== 进度条（如果有任务在执行） ==================== -->
    <el-card v-if="generationProgress && !generationProgress.is_completed" class="progress-card">
      <div class="progress-info">
        <span class="progress-title">{{ progressTitle }}</span>
        <span class="progress-text">{{ generationProgress.current }}/{{ generationProgress.total }}</span>
      </div>
      <el-progress
        :percentage="generationProgress.percentage"
        :status="generationProgress.errors.length > 0 ? 'exception' : undefined"
      >
        <span v-if="generationProgress.current_item">{{ generationProgress.current_item }}</span>
      </el-progress>
      <div v-if="generationProgress.errors.length > 0" class="progress-errors">
        <el-alert type="error" :closable="false">
          错误：{{ generationProgress.errors.join(', ') }}
        </el-alert>
      </div>
    </el-card>

    <!-- ==================== 详细信息Tabs ==================== -->
    <el-card class="content-card">
      <el-tabs v-model="activeTab" class="market-tabs">
        <!-- Tab 1: 离队候选人 -->
        <el-tab-pane name="departures">
          <template #label>
            <span class="tab-label">
              <el-icon><UserFilled /></el-icon>
              离队候选人 ({{ departureCandidates.length }})
            </span>
          </template>

          <div class="tab-content">
            <div class="tab-header">
              <el-input
                v-model="searchText"
                placeholder="搜索选手..."
                :prefix-icon="Search"
                clearable
                style="width: 300px;"
              />
              <el-select v-model="filterWantsLeave" placeholder="离队意愿" style="width: 150px;">
                <el-option label="全部" :value="null" />
                <el-option label="想离队" :value="true" />
                <el-option label="不想离队" :value="false" />
              </el-select>
            </div>

            <div v-if="departureCandidates.length === 0" class="empty-state">
              <el-empty description="暂无离队候选人数据" />
            </div>

            <div v-else class="candidates-list">
              <el-collapse v-model="activeDepartures" accordion>
                <el-collapse-item
                  v-for="candidate in filteredDepartures"
                  :key="candidate.player_id"
                  :name="candidate.player_id"
                >
                  <template #title>
                    <div class="candidate-header">
                      <div class="candidate-basic">
                        <el-tag :type="candidate.wants_to_leave ? 'danger' : 'success'">
                          {{ candidate.wants_to_leave ? '想离队' : '愿意留队' }}
                        </el-tag>
                        <span class="candidate-name">{{ candidate.player_name }}</span>
                        <el-tag size="small">{{ candidate.position }}</el-tag>
                        <el-tag size="small" type="info">能力 {{ candidate.ability }}</el-tag>
                        <el-tag size="small" type="warning">{{ candidate.age }}岁</el-tag>
                      </div>
                      <div class="candidate-meta">
                        <span>满意度: {{ candidate.satisfaction }}</span>
                        <span>忠诚度: {{ candidate.loyalty }}</span>
                        <span>置信度: {{ candidate.decision_confidence }}%</span>
                      </div>
                    </div>
                  </template>

                  <!-- 详细内容 -->
                  <div class="candidate-details">
                    <el-descriptions :column="2" border size="small">
                      <el-descriptions-item label="当前球队">{{ candidate.current_team }}</el-descriptions-item>
                      <el-descriptions-item label="当前薪资">{{ candidate.current_salary }}万/年</el-descriptions-item>
                      <el-descriptions-item label="期望薪资">{{ candidate.expected_salary }}万/年</el-descriptions-item>
                      <el-descriptions-item label="最低接受">{{ candidate.expected_min_salary }}万/年</el-descriptions-item>
                      <el-descriptions-item label="期望年限">{{ candidate.expected_years }}年</el-descriptions-item>
                      <el-descriptions-item label="要求首发">{{ candidate.requires_starter ? '是' : '否' }}</el-descriptions-item>
                    </el-descriptions>

                    <div v-if="candidate.wants_to_leave" class="departure-reasons">
                      <h4>离队原因：</h4>
                      <el-space wrap>
                        <el-tag v-for="reason in candidate.departure_reasons" :key="reason" type="danger">
                          {{ reason }}
                        </el-tag>
                      </el-space>
                      <p class="reasoning-text">{{ candidate.leave_reasoning }}</p>
                    </div>

                    <div v-if="candidate.preferred_teams && candidate.preferred_teams.length > 0" class="preferred-teams">
                      <h4>偏好球队：</h4>
                      <el-timeline>
                        <el-timeline-item
                          v-for="(team, index) in candidate.preferred_teams"
                          :key="team.team_id"
                          :type="index === 0 ? 'primary' : 'info'"
                        >
                          <strong>{{ index + 1 }}. {{ team.team_name }}</strong>
                          <div>原因：{{ team.reason }}</div>
                          <div v-if="team.attraction_score">吸引力：{{ team.attraction_score }}分</div>
                        </el-timeline-item>
                      </el-timeline>
                    </div>
                  </div>
                </el-collapse-item>
              </el-collapse>
            </div>
          </div>
        </el-tab-pane>

        <!-- Tab 2: 球队策略 -->
        <el-tab-pane name="strategies">
          <template #label>
            <span class="tab-label">
              <el-icon><DataAnalysis /></el-icon>
              球队策略 (60)
            </span>
          </template>

          <div class="tab-content">
            <div v-if="teamStrategies.length === 0" class="empty-state">
              <el-empty description="暂无球队策略数据">
                <el-button type="primary" @click="loadTeamStrategies">加载球队策略</el-button>
              </el-empty>
            </div>

            <div v-else>
              <el-table
                :data="teamStrategies"
                stripe
                style="width: 100%"
                @row-click="handleViewStrategyDetail"
                :row-style="{ cursor: 'pointer' }"
              >
                <el-table-column prop="team_name" label="球队" width="150" fixed />
                <el-table-column prop="overall_strategy" label="策略类型" width="120" />
                <el-table-column label="预算（万）" width="120">
                  <template #default="{ row }">
                    {{ row.budget_allocation.total_budget }}
                  </template>
                </el-table-column>
                <el-table-column label="引援目标" width="100">
                  <template #default="{ row }">
                    <el-tag type="primary">{{ row.targets.length }}个</el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="出售候选" width="100">
                  <template #default="{ row }">
                    <el-tag type="warning">{{ row.willing_to_sell.length }}个</el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="reasoning" label="决策理由" show-overflow-tooltip />
                <el-table-column label="生成方式" width="100">
                  <template #default="{ row }">
                    <el-tag :type="row.is_mock ? 'info' : 'success'" size="small">
                      {{ row.is_mock ? '规则AI' : 'LLM' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="100" fixed="right">
                  <template #default="{ row }">
                    <el-button text type="primary" size="small" @click.stop="handleViewStrategyDetail(row)">
                      查看详情
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </el-tab-pane>

        <!-- Tab 3: 续约结果 -->
        <el-tab-pane name="renewals">
          <template #label>
            <span class="tab-label">
              <el-icon><DocumentChecked /></el-icon>
              续约结果 ({{ renewalResult?.total_processed ?? 0 }})
            </span>
          </template>

          <div class="tab-content">
            <div v-if="renewalResult && renewalResult.total_processed > 0">
              <!-- 汇总统计 -->
              <el-row :gutter="16" style="margin-bottom: 20px;">
                <el-col :span="8">
                  <el-statistic title="总处理" :value="renewalResult.total_processed" />
                </el-col>
                <el-col :span="8">
                  <el-statistic title="续约成功" :value="renewalResult.successful_renewals" value-style="color: #67c23a" />
                </el-col>
                <el-col :span="8">
                  <el-statistic title="续约失败" :value="renewalResult.team_rejections + renewalResult.player_rejections" value-style="color: #f56c6c" />
                </el-col>
              </el-row>

              <el-divider />

              <!-- 续约结果表格 -->
              <el-table
                :data="renewalResult.decisions"
                stripe
                style="width: 100%"
                @row-click="handleViewRenewalDetail"
                :row-style="{ cursor: 'pointer' }"
              >
                <el-table-column label="结果" width="100" fixed>
                  <template #default="{ row }">
                    <el-tag :type="row.renewal_successful ? 'success' : 'danger'">
                      {{ row.renewal_successful ? '✅ 成功' : '❌ 失败' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="player_name" label="选手" width="120" />
                <el-table-column prop="team_name" label="球队" width="150" />
                <el-table-column label="球队报价" width="150">
                  <template #default="{ row }">
                    <span v-if="row.team_wants_renewal">
                      {{ row.offered_salary }}万/年，{{ row.offered_years }}年
                    </span>
                    <el-tag v-else type="danger" size="small">拒绝续约</el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="最终合同" width="150">
                  <template #default="{ row }">
                    <span v-if="row.renewal_successful">
                      {{ row.final_salary }}万/年，{{ row.final_years }}年
                    </span>
                    <span v-else>-</span>
                  </template>
                </el-table-column>
                <el-table-column prop="summary" label="总结" show-overflow-tooltip />
                <el-table-column label="操作" width="100" fixed="right">
                  <template #default="{ row }">
                    <el-button text type="primary" size="small" @click.stop="handleViewRenewalDetail(row)">
                      查看详情
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>

            <el-empty v-else description="暂无续约结果数据" />
          </div>
        </el-tab-pane>

        <!-- Tab 4: 进行中谈判 -->
        <el-tab-pane name="negotiations">
          <template #label>
            <span class="tab-label">
              <el-icon><Message /></el-icon>
              进行中谈判 ({{ negotiations.length }})
            </span>
          </template>

          <div class="tab-content">
            <el-empty v-if="negotiations.length === 0" description="暂无进行中的谈判" />

            <div v-else class="negotiations-list">
              <el-card
                v-for="neg in negotiations"
                :key="neg.id"
                class="negotiation-card"
                shadow="hover"
              >
                <div class="neg-header">
                  <div class="neg-player">
                    <strong>{{ neg.player_name }}</strong>
                    <el-tag size="small">{{ neg.player_position }}</el-tag>
                    <el-tag size="small" type="info">能力 {{ neg.player_ability }}</el-tag>
                  </div>
                  <el-tag :type="neg.status === 'OPEN' ? 'warning' : 'success'">
                    {{ neg.status_name }}
                  </el-tag>
                </div>

                <div class="neg-stats">
                  <span>当前轮次: {{ neg.current_round }}</span>
                  <span>竞争球队: {{ neg.competing_teams_count }}</span>
                  <span>报价数: {{ neg.offers_count }}</span>
                </div>

                <el-button text type="primary" @click="handleViewNegotiationDetail(neg.id)">
                  查看详情 →
                </el-button>
              </el-card>
            </div>
          </div>
        </el-tab-pane>

        <!-- Tab 5: 市场动态 -->
        <el-tab-pane name="events">
          <template #label>
            <span class="tab-label">
              <el-icon><Bell /></el-icon>
              市场动态 ({{ events.length }})
            </span>
          </template>

          <div class="tab-content">
            <div class="tab-header">
              <el-select v-model="filterEventType" placeholder="事件类型" style="width: 200px;">
                <el-option label="全部事件" value="" />
                <el-option label="报价事件" value="OFFER_MADE" />
                <el-option label="签约事件" value="SIGNING_COMPLETED" />
                <el-option label="思考过程" value="TEAM_THINKING" />
              </el-select>
              <el-input
                v-model="searchEventText"
                placeholder="搜索事件..."
                clearable
                style="width: 300px;"
              />
            </div>

            <!-- 分页显示，每页20条 -->
            <el-timeline class="events-timeline">
              <el-timeline-item
                v-for="event in paginatedEvents"
                :key="event.id"
                :type="getEventType(event.event_type)"
                :timestamp="event.created_at"
              >
                <div class="event-content">
                  <h4>{{ event.title }}</h4>
                  <p>{{ event.description }}</p>
                  <el-button v-if="event.ai_analysis" text type="primary" size="small" @click="showEventDetail(event)">
                    查看详细分析 →
                  </el-button>
                </div>
              </el-timeline-item>
            </el-timeline>

            <!-- 分页器 -->
            <el-pagination
              v-if="filteredEvents.length > 20"
              v-model:current-page="eventPage"
              v-model:page-size="eventPageSize"
              :page-sizes="[20, 50, 100, 200]"
              :total="filteredEvents.length"
              layout="total, sizes, prev, pager, next, jumper"
              style="margin-top: 20px; justify-content: center;"
            />
          </div>
        </el-tab-pane>

        <!-- Tab 6: 球队状态 -->
        <el-tab-pane name="teams">
          <template #label>
            <span class="tab-label">
              <el-icon><OfficeBuilding /></el-icon>
              球队状态 ({{ teamStates.length }})
            </span>
          </template>

          <div class="tab-content">
            <el-table :data="teamStates" stripe style="width: 100%">
              <el-table-column prop="team_name" label="球队" width="150" />
              <el-table-column prop="remaining_budget" label="剩余预算（万）" width="120" />
              <el-table-column prop="spent_amount" label="已花费（万）" width="120" />
              <el-table-column prop="roster_count" label="阵容人数" width="100" />
              <el-table-column prop="pending_negotiations" label="谈判中" width="100" />
              <el-table-column prop="completed_signings" label="已签约" width="100" />
              <el-table-column label="状态" width="120">
                <template #default="{ row }">
                  <el-tag v-if="row.needs_emergency_signing" type="danger">需要补人</el-tag>
                  <el-tag v-else-if="row.roster_count >= row.min_roster_size" type="success">正常</el-tag>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- ==================== 球队策略详情对话框 ==================== -->
    <el-dialog
      v-model="showStrategyDialog"
      :title="`${currentStrategy?.team_name} - 球队策略详情`"
      width="800px"
    >
      <div v-if="currentStrategy">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="策略类型">{{ currentStrategy.overall_strategy }}</el-descriptions-item>
          <el-descriptions-item label="生成方式">
            <el-tag :type="currentStrategy.is_mock ? 'info' : 'success'">
              {{ currentStrategy.is_mock ? '规则AI' : 'LLM' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="总预算">{{ currentStrategy.budget_allocation.total_budget }}万</el-descriptions-item>
          <el-descriptions-item label="转会费预算">{{ currentStrategy.budget_allocation.transfer_spend }}万</el-descriptions-item>
          <el-descriptions-item label="薪资预算">{{ currentStrategy.budget_allocation.salary_spend }}万</el-descriptions-item>
          <el-descriptions-item label="预留资金">{{ currentStrategy.budget_allocation.reserve }}万</el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <h4>📥 引援目标 ({{ currentStrategy.targets?.length ?? 0 }})</h4>
        <el-table v-if="currentStrategy.targets?.length > 0" :data="currentStrategy.targets" stripe max-height="300">
          <el-table-column type="index" label="#" width="50" />
          <el-table-column prop="player_name" label="选手" width="120" />
          <el-table-column prop="position" label="位置" width="80" />
          <el-table-column prop="ability" label="能力" width="80" />
          <el-table-column prop="age" label="年龄" width="80" />
          <el-table-column prop="priority" label="优先级" width="80">
            <template #default="{ row }">{{ row.priority }}/10</template>
          </el-table-column>
          <el-table-column prop="max_offer" label="最高出价（万）" width="120" />
          <el-table-column prop="reasoning" label="理由" show-overflow-tooltip />
        </el-table>
        <el-empty v-else description="无引援目标" />

        <el-divider />

        <h4>📤 出售候选 ({{ currentStrategy.willing_to_sell?.length ?? 0 }})</h4>
        <el-table v-if="currentStrategy.willing_to_sell?.length > 0" :data="currentStrategy.willing_to_sell" stripe max-height="300">
          <el-table-column type="index" label="#" width="50" />
          <el-table-column prop="player_name" label="选手" width="120" />
          <el-table-column prop="position" label="位置" width="80" />
          <el-table-column prop="ability" label="能力" width="80" />
          <el-table-column prop="age" label="年龄" width="80" />
          <el-table-column prop="urgency" label="紧迫度" width="80">
            <template #default="{ row }">{{ row.urgency }}/10</template>
          </el-table-column>
          <el-table-column prop="min_price" label="最低价（万）" width="120" />
          <el-table-column prop="reasoning" label="理由" show-overflow-tooltip />
        </el-table>
        <el-empty v-else description="无出售计划" />

        <el-divider />

        <h4>💡 决策理由</h4>
        <div class="strategy-reasoning">{{ currentStrategy.reasoning }}</div>
      </div>

      <template #footer>
        <el-button @click="showStrategyDialog = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- ==================== 续约详情对话框 ==================== -->
    <el-dialog
      v-model="showRenewalDialog"
      :title="`${currentRenewal?.player_name} - 续约详情`"
      width="700px"
    >
      <div v-if="currentRenewal">
        <el-result
          :icon="currentRenewal.renewal_successful ? 'success' : 'error'"
          :title="currentRenewal.renewal_successful ? '续约成功' : '续约失败'"
        >
          <template #sub-title>{{ currentRenewal.summary }}</template>
        </el-result>

        <el-divider />

        <el-descriptions :column="2" border>
          <el-descriptions-item label="球队">{{ currentRenewal.team_name }}</el-descriptions-item>
          <el-descriptions-item label="选手">{{ currentRenewal.player_name }}</el-descriptions-item>

          <el-descriptions-item label="球队态度">
            <el-tag :type="currentRenewal.team_wants_renewal ? 'success' : 'danger'">
              {{ currentRenewal.team_wants_renewal ? '愿意续约' : '拒绝续约' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="选手态度">
            <el-tag :type="currentRenewal.player_accepts ? 'success' : 'danger'">
              {{ currentRenewal.player_accepts ? '接受' : '拒绝' }}
            </el-tag>
          </el-descriptions-item>

          <el-descriptions-item v-if="currentRenewal.team_wants_renewal" label="球队报价">
            {{ currentRenewal.offered_salary }}万/年，{{ currentRenewal.offered_years }}年
          </el-descriptions-item>
          <el-descriptions-item v-if="currentRenewal.renewal_successful" label="最终合同">
            {{ currentRenewal.final_salary }}万/年，{{ currentRenewal.final_years }}年
          </el-descriptions-item>

          <el-descriptions-item v-if="!currentRenewal.team_wants_renewal" label="球队拒绝原因" :span="2">
            {{ currentRenewal.team_rejection_reason }}
          </el-descriptions-item>
          <el-descriptions-item v-if="currentRenewal.team_wants_renewal && !currentRenewal.player_accepts" label="选手拒绝原因" :span="2">
            {{ currentRenewal.player_rejection_reason }}
          </el-descriptions-item>
        </el-descriptions>
      </div>

      <template #footer>
        <el-button @click="showRenewalDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  User, Message, CircleCheck, TrendCharts, Plus, UserFilled,
  CaretRight, DArrowRight, Refresh, Back, Bell, Check, Close, Clock,
  QuestionFilled, InfoFilled, ArrowDown, ArrowRight, Warning, Loading, DataAnalysis,
  Star, Money, Minus, ChatDotRound, Delete, DocumentChecked, RemoveFilled, CloseBold, OfficeBuilding,
  Cpu, Position, Setting, Search, Trophy,
} from '@element-plus/icons-vue'
import { useLLMMarketStore, PHASE_NAMES } from '@/stores/useLLMMarketStore'
import { useGameStore } from '@/stores/useGameStore'
import { llmMarketApi } from '@/api/tauri'

const router = useRouter()
const llmMarketStore = useLLMMarketStore()
const gameStore = useGameStore()

// ==================== Store 状态 ====================
const {
  marketState,
  teamStates,
  negotiations,
  departureCandidates,
  events,
  isLoading,
  isGeneratingIntentions,
  isGeneratingStrategies,
  isProcessingRenewals,
  isExecutingRound,
  useRuleEngine,
  generationProgress,
  renewalResult,
  isMarketInitialized,
  currentPhase,
  currentPhaseName,
  currentPhaseDescription,
  canGenerateIntentions,
  canGenerateStrategies,
  canProcessRenewals,
  canExecuteRound,
  canAdvancePhase,
  isMarketComplete,
} = storeToRefs(llmMarketStore)

const { currentSeason } = storeToRefs(gameStore)

// ==================== 本地状态 ====================
const activeTab = ref('departures')
const searchText = ref('')
const searchEventText = ref('')
const filterWantsLeave = ref<boolean | null>(null)
const filterEventType = ref('')
const activeDepartures = ref<number[]>([])
const lastExecutionTime = ref<number | null>(null)
const teamStrategies = ref<any[]>([])
const eventPage = ref(1)
const eventPageSize = ref(20)
const showStrategyDialog = ref(false)
const currentStrategy = ref<any>(null)
const showRenewalDialog = ref(false)
const currentRenewal = ref<any>(null)

// ==================== Computed ====================

// 6个阶段
const phases = [
  { key: 'INTENTION_GENERATION', name: '选手意愿' },
  { key: 'STRATEGY_GENERATION', name: '战队策略' },
  { key: 'RENEWAL_PROCESSING', name: '续约处理' },
  { key: 'FREE_MARKET', name: '自由市场' },
  { key: 'TRANSFER_ROUNDS', name: '挖角转会' },
  { key: 'COMPLETED', name: '完成' },
]

const currentPhaseIndex = computed(() => {
  const index = phases.findIndex(p => p.key === currentPhase.value)
  return index >= 0 ? index : 0
})

const phaseTagType = computed(() => {
  if (isMarketComplete.value) return 'success'
  if (currentPhaseIndex.value >= 4) return 'danger'
  if (currentPhaseIndex.value >= 3) return 'warning'
  return 'primary'
})

// 主操作按钮文案
const primaryActionText = computed(() => {
  if (!isMarketInitialized.value) return '初始化转会市场'

  switch (currentPhase.value) {
    case 'INTENTION_GENERATION':
      return `生成选手意愿（${useRuleEngine.value ? '规则AI，<1秒' : 'LLM，10-20分钟'}）`
    case 'STRATEGY_GENERATION':
      return `生成球队策略（${useRuleEngine.value ? '规则AI，<1秒' : 'LLM，5-10分钟'}）`
    case 'RENEWAL_PROCESSING':
      return `处理续约（${useRuleEngine.value ? '规则AI，<1秒' : 'LLM，5-10分钟'}）`
    case 'FREE_MARKET':
      return `执行自由市场第${marketState.value?.current_round ?? 0}轮（${useRuleEngine.value ? '<500ms' : '20-30分钟'}）`
    case 'TRANSFER_ROUNDS':
      return `执行挖角第${marketState.value?.transfer_round ?? 0}轮（${useRuleEngine.value ? '<500ms' : '20-30分钟'}）`
    default:
      return '推进到下一阶段'
  }
})

const isExecutingAction = computed(() =>
  isLoading.value || isExecutingRound.value || isGeneratingIntentions.value ||
  isGeneratingStrategies.value || isProcessingRenewals.value
)

const progressTitle = computed(() => {
  if (!generationProgress.value) return ''
  switch (generationProgress.value.task_type) {
    case 'player_intentions': return '生成选手意愿'
    case 'team_strategies': return '生成球队策略'
    case 'renewals': return '处理续约'
    default: return '执行中'
  }
})

// 过滤后的离队候选人
const filteredDepartures = computed(() => {
  let result = departureCandidates.value

  if (filterWantsLeave.value !== null) {
    result = result.filter(c => c.wants_to_leave === filterWantsLeave.value)
  }

  if (searchText.value) {
    const search = searchText.value.toLowerCase()
    result = result.filter(c =>
      c.player_name.toLowerCase().includes(search) ||
      c.position.toLowerCase().includes(search)
    )
  }

  return result
})

// 过滤后的事件
const filteredEvents = computed(() => {
  let result = events.value

  if (filterEventType.value) {
    result = result.filter(e => e.event_type === filterEventType.value)
  }

  if (searchEventText.value) {
    const search = searchEventText.value.toLowerCase()
    result = result.filter(e =>
      e.title.toLowerCase().includes(search) ||
      e.description.toLowerCase().includes(search)
    )
  }

  return result
})

// 分页后的事件（性能优化）
const paginatedEvents = computed(() => {
  const start = (eventPage.value - 1) * eventPageSize.value
  const end = start + eventPageSize.value
  return filteredEvents.value.slice(start, end)
})

// ==================== 方法 ====================

// 统一的主操作处理
const handlePrimaryAction = async () => {
  try {
    if (!isMarketInitialized.value) {
      await llmMarketStore.initMarket()
      ElMessage.success('市场初始化成功')
      return
    }

    const startTime = performance.now()

    switch (currentPhase.value) {
      case 'INTENTION_GENERATION':
        await llmMarketStore.generateIntentions()
        ElMessage.success('选手意愿生成完成')
        break
      case 'STRATEGY_GENERATION':
        await llmMarketStore.generateStrategies()
        await llmMarketStore.loadDepartureCandidates()
        await loadTeamStrategies()
        ElMessage.success('球队策略生成完成')
        break
      case 'RENEWAL_PROCESSING':
        await llmMarketStore.processRenewals()
        ElMessage.success('续约处理完成')
        break
      case 'FREE_MARKET':
      case 'TRANSFER_ROUNDS':
        await llmMarketStore.executeRound()
        ElMessage.success('执行完成')
        break
      default:
        ElMessage.info('当前阶段无可执行操作')
    }

    const elapsed = performance.now() - startTime
    lastExecutionTime.value = Math.round(elapsed)
  } catch (error) {
    console.error('执行失败:', error)
    ElMessage.error(`执行失败: ${error}`)
  }
}

const handleInitMarket = async () => {
  await handlePrimaryAction()
}

const handleGenerateIntentions = async () => {
  await handlePrimaryAction()
}

const handleGenerateStrategies = async () => {
  await handlePrimaryAction()
}

const handleProcessRenewals = async () => {
  await handlePrimaryAction()
}

const handleExecuteRound = async () => {
  await handlePrimaryAction()
}

const handleAdvancePhase = async () => {
  try {
    await llmMarketStore.advancePhase()
    ElMessage.success('阶段推进成功')
  } catch (error) {
    ElMessage.error('阶段推进失败')
  }
}

const handleFastForward = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动执行所有剩余轮次直到阶段结束，是否继续？',
      '快进确认',
      { type: 'warning' }
    )

    let maxRounds = 10
    while (canExecuteRound.value && maxRounds > 0) {
      await llmMarketStore.executeRound()
      maxRounds--
    }

    ElMessage.success('快进完成')
  } catch {
    // 用户取消
  }
}

const handleRefresh = async () => {
  await llmMarketStore.loadMarketState()
  await llmMarketStore.loadTeamStates()
  await llmMarketStore.loadDepartureCandidates()
  await loadTeamStrategies()
  ElMessage.success('状态已刷新')
}

const handleResetMarket = async () => {
  try {
    await ElMessageBox.confirm(
      '将清空所有转会市场数据，是否继续？',
      '重置确认',
      { type: 'warning', confirmButtonText: '确认重置', cancelButtonText: '取消' }
    )

    await llmMarketStore.resetMarket()
    ElMessage.success('市场已重置')
  } catch {
    // 用户取消
  }
}

const loadTeamStrategies = async () => {
  try {
    // 加载所有球队策略
    const allStrategies = []
    for (const team of teamStates.value) {
      const strategy = await llmMarketApi.getTeamStrategy(team.team_id)
      if (strategy) {
        allStrategies.push(strategy)
      }
    }
    teamStrategies.value = allStrategies
  } catch (error) {
    console.error('加载球队策略失败:', error)
  }
}

const handleViewStrategyDetail = (strategy: any) => {
  currentStrategy.value = strategy
  showStrategyDialog.value = true
}

const handleViewRenewalDetail = (renewal: any) => {
  currentRenewal.value = renewal
  showRenewalDialog.value = true
}

const handleViewNegotiationDetail = async (negId: number) => {
  try {
    const detail = await llmMarketApi.getNegotiationDetail(negId)
    // TODO: 显示详情对话框
    console.log('谈判详情:', detail)
  } catch (error) {
    ElMessage.error('加载谈判详情失败')
  }
}

const showEventDetail = (event: any) => {
  ElMessageBox.alert(event.ai_analysis || event.description, event.title, {
    confirmButtonText: '关闭',
    dangerouslyUseHTMLString: false,
  })
}

const getEventType = (eventType: string) => {
  if (eventType.includes('SIGNING') || eventType.includes('ACCEPTED')) return 'success'
  if (eventType.includes('REJECTED') || eventType.includes('FAILED')) return 'danger'
  if (eventType.includes('THINKING')) return 'primary'
  return 'info'
}

// ==================== 生命周期 ====================
onMounted(async () => {
  await llmMarketStore.loadMarketState()
  await llmMarketStore.loadTeamStates()
  await llmMarketStore.loadDepartureCandidates()
  await loadTeamStrategies()
})
</script>

<style scoped>
.llm-transfer-market-v2 {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left h1 {
  margin: 0;
  font-size: 28px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.season-info {
  margin: 8px 0 0 0;
  color: #909399;
  font-size: 14px;
}

.header-actions {
  display: flex;
  gap: 12px;
}

/* 模式配置卡片 */
.mode-config-card {
  margin-bottom: 20px;
  border: 2px solid #409eff;
}

.mode-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.mode-header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mode-card-title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.mode-content {
  padding: 0;
}

/* 模式横幅 */
.mode-banner {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 24px;
  border-radius: 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.mode-banner.rule-engine {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.mode-banner.llm-engine {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.banner-icon {
  font-size: 48px;
  line-height: 1;
}

.banner-content h3 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
}

.banner-content p {
  margin: 0;
  font-size: 14px;
  opacity: 0.9;
}

/* 功能卡片 */
.feature-card {
  height: 100%;
  border: 1px solid #e4e7ed;
  transition: all 0.3s;
}

.feature-card:hover {
  border-color: #409eff;
  transform: translateY(-4px);
}

.feature-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.feature-header h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.feature-content {
  font-size: 14px;
}

.feature-desc {
  margin-top: 12px;
  color: #606266;
  line-height: 1.6;
  font-size: 13px;
}

/* 指标卡片 */
.metric-card {
  text-align: center;
  transition: all 0.3s;
}

.metric-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0,0,0,0.1);
}

.metric-card.success {
  border: 2px solid #67c23a;
}

.metric-card.warning {
  border: 2px solid #e6a23c;
}

.metric-desc {
  margin-top: 8px;
  color: #909399;
  font-size: 13px;
}

.metric-desc.warning {
  color: #e6a23c;
  font-weight: 500;
}

/* 数据整合卡片 */
.data-integration-card {
  background: linear-gradient(to bottom, #f5f7fa, #ffffff);
  border: 1px solid #e4e7ed;
}

.integration-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 20px;
}

.integration-header h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.integration-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 16px;
  background: white;
  border-radius: 8px;
  transition: all 0.3s;
  border: 1px solid #e4e7ed;
}

.integration-item:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  border-color: #409eff;
}

.integration-item div {
  margin-top: 12px;
}

.integration-item strong {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: #303133;
}

.integration-item p {
  margin: 0;
  font-size: 12px;
  color: #606266;
  line-height: 1.6;
}

/* 阶段进度卡片 */
.phase-card {
  margin-bottom: 20px;
}

.phase-header-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.phase-title {
  font-size: 18px;
  font-weight: 600;
}

.phase-description {
  margin-top: 16px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
  color: #606266;
  text-align: center;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  text-align: center;
}

/* 操作按钮 */
.action-card {
  margin-bottom: 20px;
}

.action-container {
  display: flex;
  gap: 12px;
  justify-content: center;
  align-items: center;
}

/* 进度卡片 */
.progress-card {
  margin-bottom: 20px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
}

.progress-title {
  font-weight: 600;
}

.progress-errors {
  margin-top: 12px;
}

/* 内容卡片 */
.content-card {
  min-height: 500px;
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-content {
  padding: 20px 0;
}

.tab-header {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

/* 离队候选人 */
.candidates-list {
  max-height: 800px;
  overflow-y: auto;
}

.candidate-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding-right: 20px;
}

.candidate-basic {
  display: flex;
  align-items: center;
  gap: 12px;
}

.candidate-name {
  font-size: 16px;
  font-weight: 600;
}

.candidate-meta {
  display: flex;
  gap: 16px;
  color: #909399;
  font-size: 13px;
}

.candidate-details {
  padding: 16px;
}

.departure-reasons,
.preferred-teams {
  margin-top: 16px;
}

.departure-reasons h4,
.preferred-teams h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #606266;
}

.reasoning-text {
  margin-top: 8px;
  color: #606266;
  line-height: 1.6;
}

/* 球队策略 */

.strategy-card {
  height: 100%;
}

.strategy-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.strategy-content {
  font-size: 13px;
}

.strategy-section {
  margin-bottom: 16px;
}

.strategy-section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #303133;
}

.targets-list,
.sell-list {
  font-size: 13px;
}

.target-item,
.sell-item {
  padding: 8px;
  border-left: 3px solid #409eff;
  background: #f5f7fa;
  margin-bottom: 8px;
  border-radius: 4px;
}

.target-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.target-priority {
  background: #e6a23c;
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.target-details {
  color: #909399;
  font-size: 12px;
}

/* 续约结果 */



.renewal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.renewal-summary {
  margin-top: 12px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
  color: #606266;
}

/* 谈判列表 */
.negotiations-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}

.negotiation-card {
  cursor: pointer;
  transition: all 0.3s;
}

.negotiation-card:hover {
  transform: translateY(-2px);
}

.neg-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.neg-player {
  display: flex;
  align-items: center;
  gap: 8px;
}

.neg-stats {
  display: flex;
  gap: 16px;
  color: #909399;
  font-size: 13px;
  margin-top: 8px;
}

/* 事件时间线（优化性能） */
.events-timeline {
  max-height: 600px;
  overflow-y: auto;
}

.event-content h4 {
  margin: 0 0 8px 0;
  font-size: 15px;
  font-weight: 600;
}

.event-content p {
  margin: 0 0 8px 0;
  color: #606266;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

/* 球队策略对话框 */
.strategy-reasoning {
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
  color: #606266;
  line-height: 1.6;
  white-space: pre-wrap;
}

/* 空状态 */
.empty-state {
  padding: 60px 0;
  text-align: center;
}

/* 响应式 */
@media (max-width: 1200px) {
  .negotiations-list {
    grid-template-columns: 1fr;
  }
}
</style>

