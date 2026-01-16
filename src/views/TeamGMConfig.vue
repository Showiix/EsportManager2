<template>
  <div class="gm-config-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>AI GM 配置</h1>
        <p>配置每支球队的 AI 总经理人格，影响转会窗口的决策行为</p>
      </div>
      <div class="header-actions">
        <el-button @click="generateStrategies" :loading="aiStrategyStore.isGenerating" type="primary">
          <el-icon><MagicStick /></el-icon>
          {{ aiStrategyStore.generationProgress ? `生成中 ${aiStrategyStore.generationProgress.current}/${aiStrategyStore.generationProgress.total}` : '生成 AI 策略' }}
        </el-button>
      </div>
    </div>

    <!-- 生成进度条 -->
    <el-card v-if="aiStrategyStore.generationProgress" class="progress-card">
      <div class="progress-info">
        <span class="progress-title">正在为 {{ aiStrategyStore.generationProgress.team_name }} 生成策略...</span>
        <span class="progress-count">{{ aiStrategyStore.generationProgress.current }}/{{ aiStrategyStore.generationProgress.total }}</span>
      </div>
      <el-progress
        :percentage="Math.round((aiStrategyStore.generationProgress.current / aiStrategyStore.generationProgress.total) * 100)"
        :status="aiStrategyStore.generationProgress.status === 'failed' ? 'exception' : undefined"
        :stroke-width="10"
      />
    </el-card>

    <!-- LLM 配置卡片 -->
    <el-card class="llm-config-card">
      <template #header>
        <div class="llm-header">
          <span class="llm-title">
            <el-icon><Connection /></el-icon>
            LLM AI 配置
          </span>
          <el-tag :type="llmConfig.is_configured ? 'success' : 'info'" size="small">
            {{ llmConfig.is_configured ? '已配置' : '未配置' }}
          </el-tag>
        </div>
      </template>
      <div class="llm-content">
        <div class="llm-info" v-if="llmConfig.is_configured">
          <span>提供商: <strong>{{ llmConfig.provider.toUpperCase() }}</strong></span>
          <span>模型: <strong>{{ llmConfig.model }}</strong></span>
          <el-button size="small" type="danger" @click="clearLLMConfig" plain>
            清除配置
          </el-button>
        </div>
        <div class="llm-form" v-else>
          <el-form :inline="true" size="small">
            <el-form-item label="提供商">
              <el-select v-model="llmForm.provider" style="width: 140px">
                <el-option label="OpenAI" value="openai" />
                <el-option label="Claude" value="claude" />
                <el-option label="DeepSeek" value="deepseek" />
                <el-option label="通义千问" value="qwen" />
                <el-option label="Moonshot/Kimi" value="moonshot" />
                <el-option label="智谱GLM" value="zhipu" />
              </el-select>
            </el-form-item>
            <el-form-item label="API Key">
              <el-input
                v-model="llmForm.apiKey"
                type="password"
                show-password
                placeholder="输入 API Key"
                style="width: 280px"
              />
            </el-form-item>
            <el-form-item label="模型">
              <el-select v-model="llmForm.model" style="width: 200px">
                <template v-if="llmForm.provider === 'openai'">
                  <el-option label="GPT-4o Mini (推荐)" value="gpt-4o-mini" />
                  <el-option label="GPT-4o" value="gpt-4o" />
                  <el-option label="GPT-4 Turbo" value="gpt-4-turbo" />
                </template>
                <template v-else-if="llmForm.provider === 'claude'">
                  <el-option label="Claude 3.5 Sonnet (推荐)" value="claude-3-5-sonnet-20241022" />
                  <el-option label="Claude 3 Opus" value="claude-3-opus-20240229" />
                  <el-option label="Claude 3 Haiku" value="claude-3-haiku-20240307" />
                </template>
                <template v-else-if="llmForm.provider === 'deepseek'">
                  <el-option label="DeepSeek Chat (推荐)" value="deepseek-chat" />
                  <el-option label="DeepSeek Coder" value="deepseek-coder" />
                </template>
                <template v-else-if="llmForm.provider === 'qwen'">
                  <el-option label="Qwen Turbo (推荐)" value="qwen-turbo" />
                  <el-option label="Qwen Plus" value="qwen-plus" />
                  <el-option label="Qwen Max" value="qwen-max" />
                </template>
                <template v-else-if="llmForm.provider === 'moonshot'">
                  <el-option label="Moonshot V1 8K (推荐)" value="moonshot-v1-8k" />
                  <el-option label="Moonshot V1 32K" value="moonshot-v1-32k" />
                  <el-option label="Moonshot V1 128K" value="moonshot-v1-128k" />
                </template>
                <template v-else-if="llmForm.provider === 'zhipu'">
                  <el-option label="GLM-4 Flash (推荐)" value="glm-4-flash" />
                  <el-option label="GLM-4" value="glm-4" />
                  <el-option label="GLM-4 Plus" value="glm-4-plus" />
                </template>
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="saveLLMConfig" :loading="isSavingLLM">
                保存配置
              </el-button>
            </el-form-item>
          </el-form>
          <div class="llm-hint">
            配置 API Key 后，可以使用真实的 LLM AI 生成更智能的转会策略
          </div>
        </div>
      </div>
    </el-card>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-select v-model="filters.region" placeholder="全部赛区" clearable style="width: 120px">
        <el-option label="全部赛区" value="" />
        <el-option label="LPL" value="LPL" />
        <el-option label="LCK" value="LCK" />
        <el-option label="LEC" value="LEC" />
        <el-option label="LCS" value="LCS" />
      </el-select>
      <el-select v-model="filters.personality" placeholder="全部人格" clearable style="width: 140px">
        <el-option label="全部人格" value="" />
        <el-option v-for="p in personalityTypes" :key="p.value" :label="p.name" :value="p.value" />
      </el-select>
      <el-input
        v-model="filters.search"
        placeholder="搜索球队..."
        style="width: 200px"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <el-button @click="loadData" :loading="isLoading">
        <el-icon><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <!-- 球队配置列表 -->
    <el-card class="config-card">
      <el-table
        :data="filteredProfiles"
        v-loading="isLoading"
        stripe
        style="width: 100%"
        max-height="600"
        @row-click="openEditDialog"
        row-class-name="clickable-row"
      >
        <el-table-column prop="team_name" label="球队" width="150" fixed>
          <template #default="{ row }">
            <div class="team-cell">
              <span class="team-name">{{ row.team_name }}</span>
              <span class="team-short" v-if="row.team_short_name">({{ row.team_short_name }})</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="赛区" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="getRegionTagType(row.region_name)">
              {{ row.region_name }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="GM 人格" width="120">
          <template #default="{ row }">
            <div class="personality-cell">
              <el-icon :class="'personality-icon-' + row.personality.toLowerCase()">
                <component :is="getPersonalityIcon(row.personality)" />
              </el-icon>
              <span>{{ row.personality_name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="预算" width="100">
          <template #default="{ row }">
            <span class="budget-value">{{ (row.budget_ratio * 100).toFixed(0) }}%</span>
          </template>
        </el-table-column>
        <el-table-column label="能力阈值" width="100" align="center">
          <template #default="{ row }">
            <el-tag size="small">{{ row.min_ability_threshold }}+</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="年龄范围" width="100" align="center">
          <template #default="{ row }">
            <span class="age-range">{{ row.preferred_age_min }}-{{ row.preferred_age_max }}</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="160" align="center" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click.stop="openEditDialog(row)">
              编辑
            </el-button>
            <el-button type="success" size="small" @click.stop="viewTeamStrategy(row)" :loading="loadingStrategyTeamId === row.team_id">
              策略
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 编辑对话框 -->
    <el-dialog
      v-model="editDialogVisible"
      :title="`配置 ${editingProfile?.team_name || ''} 的 AI GM`"
      width="700px"
      destroy-on-close
    >
      <el-form v-if="editingProfile" :model="editForm" label-width="120px" label-position="left">
        <!-- GM 人格 -->
        <el-form-item label="GM 人格">
          <el-select v-model="editForm.personality" style="width: 100%" @change="onPersonalityChange">
            <el-option
              v-for="p in personalityTypes"
              :key="p.value"
              :label="p.name"
              :value="p.value"
            >
              <div class="personality-option">
                <span class="option-name">{{ p.name }}</span>
                <span class="option-desc">{{ p.description }}</span>
              </div>
            </el-option>
          </el-select>
        </el-form-item>

        <!-- 风险偏好 -->
        <el-form-item label="风险偏好">
          <div class="slider-container">
            <el-slider
              v-model="editForm.riskTolerance"
              :min="0"
              :max="100"
              :step="5"
              show-stops
            />
            <div class="slider-labels">
              <span>保守</span>
              <span class="value-label">{{ editForm.riskTolerance }}%</span>
              <span>激进</span>
            </div>
          </div>
        </el-form-item>

        <!-- 预算分配比例 -->
        <el-form-item label="预算比例">
          <div class="slider-container">
            <el-slider
              v-model="editForm.budgetRatioPercent"
              :min="10"
              :max="100"
              :step="5"
              show-stops
            />
            <div class="slider-labels">
              <span>保守</span>
              <span class="value-label">使用 {{ editForm.budgetRatioPercent }}% 余额</span>
              <span>All-in</span>
            </div>
          </div>
        </el-form-item>

        <!-- 出售策略激进度 -->
        <el-form-item label="出售策略">
          <el-radio-group v-model="editForm.sellAggressiveness">
            <el-radio-button value="CONSERVATIVE">保守</el-radio-button>
            <el-radio-button value="NORMAL">正常</el-radio-button>
            <el-radio-button value="AGGRESSIVE">激进</el-radio-button>
          </el-radio-group>
          <div class="form-hint">
            {{ getSellAggressivenessHint(editForm.sellAggressiveness) }}
          </div>
        </el-form-item>

        <!-- 年龄偏好范围 -->
        <el-form-item label="年龄偏好">
          <div class="age-range-container">
            <el-input-number
              v-model="editForm.preferredAgeMin"
              :min="17"
              :max="35"
              controls-position="right"
              style="width: 100px"
            />
            <span class="range-separator">到</span>
            <el-input-number
              v-model="editForm.preferredAgeMax"
              :min="17"
              :max="35"
              controls-position="right"
              style="width: 100px"
            />
            <span class="range-hint">岁</span>
          </div>
        </el-form-item>

        <!-- 能力值门槛 -->
        <el-form-item label="能力值门槛">
          <div class="slider-container">
            <el-slider
              v-model="editForm.minAbilityThreshold"
              :min="50"
              :max="95"
              :step="5"
              :marks="{ 60: '60', 70: '70', 80: '80', 90: '90' }"
            />
            <div class="slider-labels">
              <span>低要求</span>
              <span class="value-label">最低能力 {{ editForm.minAbilityThreshold }}</span>
              <span>只要顶尖</span>
            </div>
          </div>
        </el-form-item>

        <!-- 溢价容忍度 -->
        <el-form-item label="溢价容忍">
          <div class="slider-container">
            <el-slider
              v-model="editForm.pricePremiumPercent"
              :min="50"
              :max="150"
              :step="5"
              :marks="{ 80: '8折', 100: '身价', 130: '溢价30%' }"
            />
            <div class="slider-labels">
              <span>折价买</span>
              <span class="value-label">{{ editForm.pricePremiumPercent }}%</span>
              <span>高价抢</span>
            </div>
          </div>
        </el-form-item>

        <!-- 位置优先级 -->
        <el-form-item label="位置优先级">
          <div class="position-priorities">
            <div v-for="pos in positions" :key="pos" class="position-item">
              <span class="position-label">{{ pos }}</span>
              <el-slider
                v-model="editForm.positionPriorities[pos]"
                :min="0"
                :max="100"
                :step="10"
                style="width: 200px"
              />
              <span class="priority-value">{{ editForm.positionPriorities[pos] }}</span>
            </div>
            <div class="form-hint">数值越高，越优先补强该位置</div>
          </div>
        </el-form-item>

        <!-- 选秀配置 -->
        <el-divider content-position="left">选秀配置</el-divider>

        <el-form-item label="卖签倾向">
          <div class="slider-container">
            <el-slider
              v-model="editForm.draftSellThresholdPercent"
              :min="0"
              :max="100"
              :step="5"
              show-stops
            />
            <div class="slider-labels">
              <span>保留签位</span>
              <span class="value-label">{{ editForm.draftSellThresholdPercent }}%</span>
              <span>积极出售</span>
            </div>
          </div>
          <div class="form-hint">影响在选秀权拍卖中挂牌出售的意愿</div>
        </el-form-item>

        <el-form-item label="竞拍激进度">
          <div class="slider-container">
            <el-slider
              v-model="editForm.draftBidAggressivenessPercent"
              :min="50"
              :max="200"
              :step="10"
              :marks="{ 100: '标准' }"
            />
            <div class="slider-labels">
              <span>保守</span>
              <span class="value-label">{{ editForm.draftBidAggressivenessPercent }}%</span>
              <span>激进</span>
            </div>
          </div>
          <div class="form-hint">影响竞拍选秀权时的出价金额和意愿</div>
        </el-form-item>

        <el-form-item label="选秀策略">
          <el-radio-group v-model="editForm.draftStrategy">
            <el-radio-button value="immediate">即战力</el-radio-button>
            <el-radio-button value="balanced">平衡</el-radio-button>
            <el-radio-button value="potential">潜力</el-radio-button>
            <el-radio-button value="custom">自定义</el-radio-button>
          </el-radio-group>
          <div class="form-hint">{{ getDraftStrategyHint(editForm.draftStrategy) }}</div>
        </el-form-item>

        <el-form-item label="能力-潜力权重" v-if="editForm.draftStrategy === 'custom'">
          <div class="slider-container">
            <el-slider
              v-model="editForm.draftAbilityWeightPercent"
              :min="0"
              :max="100"
              :step="5"
              :marks="{ 0: '纯潜力', 40: 'BPA', 70: '即战力', 100: '只看能力' }"
            />
            <div class="slider-labels">
              <span>Potential 100%</span>
              <span class="value-label">Ability {{ editForm.draftAbilityWeightPercent }}%</span>
              <span>Ability 100%</span>
            </div>
          </div>
        </el-form-item>

        <el-form-item label="年轻偏好">
          <div class="slider-container">
            <el-slider
              v-model="editForm.draftYoungBiasPercent"
              :min="-20"
              :max="20"
              :step="5"
              :marks="{ 0: '中立' }"
            />
            <div class="slider-labels">
              <span>偏好老将</span>
              <span class="value-label">{{ formatBias(editForm.draftYoungBiasPercent) }}</span>
              <span>偏好新秀</span>
            </div>
          </div>
          <div class="form-hint">对21岁以下球员的评分加成（正值）或惩罚（负值）</div>
        </el-form-item>

        <!-- 自定义提示词 -->
        <el-form-item label="自定义提示" v-if="editForm.personality === 'CUSTOM'">
          <el-input
            v-model="editForm.customPrompt"
            type="textarea"
            :rows="4"
            placeholder="输入自定义的 GM 行为描述，例如：专注于签约韩国选手，偏好高稳定性选手..."
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveProfile" :loading="isSaving">
          保存配置
        </el-button>
      </template>
    </el-dialog>

    <!-- AI 策略展示对话框 -->
    <el-dialog
      v-model="strategyDialogVisible"
      title="AI 策略生成结果"
      width="900px"
      destroy-on-close
    >
      <div v-if="aiStrategyStore.strategies.length > 0">
        <el-table :data="aiStrategyStore.strategies" max-height="500">
          <el-table-column prop="team_name" label="球队" width="120" />
          <el-table-column prop="overall_strategy" label="策略" width="100">
            <template #default="{ row }">
              <el-tag>{{ row.overall_strategy }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="目标" width="80" align="center">
            <template #default="{ row }">
              <span class="count-badge targets">{{ row.targets_count }}</span>
            </template>
          </el-table-column>
          <el-table-column label="出售" width="80" align="center">
            <template #default="{ row }">
              <span class="count-badge sells">{{ row.sell_count }}</span>
            </template>
          </el-table-column>
          <el-table-column label="补强位置" width="120">
            <template #default="{ row }">
              <el-tag v-for="pos in row.priority_positions" :key="pos" size="small" class="pos-tag">
                {{ pos }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="预算" width="100" align="right">
            <template #default="{ row }">
              {{ formatBudget(row.budget.total_budget) }}
            </template>
          </el-table-column>
          <el-table-column prop="reasoning" label="决策理由" min-width="200">
            <template #default="{ row }">
              <el-tooltip :content="row.reasoning" placement="top" :show-after="500">
                <span class="reasoning-text">{{ row.reasoning }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
        </el-table>
      </div>
      <el-empty v-else description="暂无策略数据" />
    </el-dialog>

    <!-- 单个球队策略详情对话框 -->
    <el-dialog
      v-model="teamStrategyDialogVisible"
      :title="`${currentTeamStrategy?.team_name || viewingTeamName || ''} 的 AI 转会策略`"
      width="1000px"
      destroy-on-close
    >
      <template #header="{ titleId, titleClass }">
        <div class="dialog-header-with-action">
          <span :id="titleId" :class="titleClass">{{ currentTeamStrategy?.team_name || viewingTeamName || '' }} 的 AI 转会策略</span>
          <el-button type="primary" size="small" @click="generateSingleTeamStrategy" :loading="isGeneratingSingleStrategy">
            <el-icon v-if="!isGeneratingSingleStrategy"><MagicStick /></el-icon>
            {{ currentTeamStrategy ? '重新生成' : '生成策略' }}
          </el-button>
        </div>
      </template>
      <div v-if="currentTeamStrategy" class="strategy-detail">
        <!-- 策略概览 -->
        <el-card class="strategy-overview" shadow="never">
          <div class="overview-header">
            <div class="strategy-type">
              <el-tag size="large" type="primary">{{ currentTeamStrategy.overall_strategy }}</el-tag>
              <span class="strategy-desc">{{ currentTeamStrategy.strategy_description }}</span>
            </div>
            <div class="strategy-meta">
              <span v-if="currentTeamStrategy.is_mock" class="mock-badge">规则 AI</span>
              <span v-else class="llm-badge">LLM AI</span>
              <span class="generated-time">{{ formatGeneratedTime(currentTeamStrategy.generated_at) }}</span>
            </div>
          </div>
          <div class="reasoning-box">
            <strong>决策理由：</strong>
            <p>{{ currentTeamStrategy.reasoning }}</p>
          </div>

          <!-- 分析步骤 -->
          <div v-if="currentTeamStrategy.analysis_steps && currentTeamStrategy.analysis_steps.length > 0" class="analysis-steps-section">
            <strong>AI 分析过程：</strong>
            <el-timeline>
              <el-timeline-item
                v-for="(step, index) in currentTeamStrategy.analysis_steps"
                :key="index"
                :type="getStepType(step.step_name)"
                :hollow="index === currentTeamStrategy.analysis_steps.length - 1"
              >
                <div class="analysis-step-item">
                  <div class="step-header">
                    <span class="step-name">{{ step.step_name }}</span>
                  </div>
                  <div class="step-content">
                    <div class="step-row">
                      <span class="step-label">数据：</span>
                      <span class="step-value">{{ step.data_used }}</span>
                    </div>
                    <div v-if="step.threshold" class="step-row">
                      <span class="step-label">阈值：</span>
                      <span class="step-value threshold">{{ step.threshold }}</span>
                    </div>
                    <div class="step-row">
                      <span class="step-label">结论：</span>
                      <span class="step-value result">{{ step.result }}</span>
                    </div>
                    <div class="step-row">
                      <span class="step-label">影响：</span>
                      <span class="step-value impact">{{ step.impact }}</span>
                    </div>
                  </div>
                </div>
              </el-timeline-item>
            </el-timeline>
          </div>
        </el-card>

        <!-- 预算分配 -->
        <el-card class="budget-card" shadow="never">
          <template #header>
            <span class="card-title">预算分配</span>
          </template>
          <div class="budget-grid">
            <div class="budget-item">
              <span class="budget-label">总预算</span>
              <span class="budget-value total">{{ formatBudget(currentTeamStrategy.budget_allocation.total_budget) }}</span>
            </div>
            <div class="budget-item">
              <span class="budget-label">转会费</span>
              <span class="budget-value transfer">{{ formatBudget(currentTeamStrategy.budget_allocation.transfer_spend) }}</span>
            </div>
            <div class="budget-item">
              <span class="budget-label">薪资预算</span>
              <span class="budget-value salary">{{ formatBudget(currentTeamStrategy.budget_allocation.salary_spend) }}</span>
            </div>
            <div class="budget-item">
              <span class="budget-label">预留资金</span>
              <span class="budget-value reserve">{{ formatBudget(currentTeamStrategy.budget_allocation.reserve) }}</span>
            </div>
          </div>
          <div class="priority-positions">
            <strong>优先补强位置：</strong>
            <el-tag v-for="pos in currentTeamStrategy.priority_positions" :key="pos" class="pos-tag-large">
              {{ pos }}
            </el-tag>
            <span v-if="currentTeamStrategy.priority_positions.length === 0" class="no-data">无</span>
          </div>
        </el-card>

        <!-- 目标选手 -->
        <el-card class="targets-card" shadow="never">
          <template #header>
            <div class="card-header-with-count">
              <span class="card-title">目标选手</span>
              <el-tag type="success">{{ currentTeamStrategy.targets.length }} 人</el-tag>
            </div>
          </template>
          <el-table v-if="currentTeamStrategy.targets.length > 0" :data="currentTeamStrategy.targets" max-height="300">
            <el-table-column prop="player_name" label="选手" width="120" />
            <el-table-column prop="position" label="位置" width="80" align="center">
              <template #default="{ row }">
                <el-tag size="small">{{ row.position }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="能力/潜力" width="100" align="center">
              <template #default="{ row }">
                <span class="ability-value">{{ row.ability }}</span>
                <span class="potential-value">/{{ row.potential }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="age" label="年龄" width="60" align="center" />
            <el-table-column label="身价" width="100" align="right">
              <template #default="{ row }">
                {{ formatBudget(row.market_value) }}
              </template>
            </el-table-column>
            <el-table-column label="最高出价" width="100" align="right">
              <template #default="{ row }">
                <span class="offer-value">{{ formatBudget(row.max_offer) }}</span>
              </template>
            </el-table-column>
            <el-table-column label="优先级" width="80" align="center">
              <template #default="{ row }">
                <el-rate v-model="row.priority" disabled :max="10" size="small" />
              </template>
            </el-table-column>
            <el-table-column prop="reasoning" label="签约理由" min-width="180">
              <template #default="{ row }">
                <el-tooltip :content="row.reasoning" placement="top">
                  <span class="reasoning-text">{{ row.reasoning }}</span>
                </el-tooltip>
              </template>
            </el-table-column>
          </el-table>
          <el-empty v-else description="暂无目标选手" :image-size="60" />
        </el-card>

        <!-- 出售列表 -->
        <el-card class="sell-card" shadow="never">
          <template #header>
            <div class="card-header-with-count">
              <span class="card-title">愿意出售</span>
              <el-tag type="danger">{{ currentTeamStrategy.willing_to_sell.length }} 人</el-tag>
            </div>
          </template>
          <el-table v-if="currentTeamStrategy.willing_to_sell.length > 0" :data="currentTeamStrategy.willing_to_sell" max-height="300">
            <el-table-column prop="player_name" label="选手" width="120" />
            <el-table-column prop="position" label="位置" width="80" align="center">
              <template #default="{ row }">
                <el-tag size="small">{{ row.position }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="ability" label="能力" width="60" align="center" />
            <el-table-column prop="age" label="年龄" width="60" align="center" />
            <el-table-column label="薪资" width="100" align="right">
              <template #default="{ row }">
                {{ formatBudget(row.salary) }}
              </template>
            </el-table-column>
            <el-table-column label="身价" width="100" align="right">
              <template #default="{ row }">
                {{ formatBudget(row.market_value) }}
              </template>
            </el-table-column>
            <el-table-column label="最低价" width="100" align="right">
              <template #default="{ row }">
                <span class="min-price">{{ formatBudget(row.min_price) }}</span>
              </template>
            </el-table-column>
            <el-table-column label="紧迫度" width="80" align="center">
              <template #default="{ row }">
                <el-progress :percentage="row.urgency * 10" :stroke-width="6" :show-text="false"
                  :color="row.urgency >= 7 ? '#f56c6c' : row.urgency >= 4 ? '#e6a23c' : '#67c23a'" />
              </template>
            </el-table-column>
            <el-table-column prop="reasoning" label="出售理由" min-width="180">
              <template #default="{ row }">
                <el-tooltip :content="row.reasoning" placement="top">
                  <span class="reasoning-text">{{ row.reasoning }}</span>
                </el-tooltip>
              </template>
            </el-table-column>
          </el-table>
          <el-empty v-else description="暂无出售意向" :image-size="60" />
        </el-card>
      </div>
      <el-empty v-else description="该球队暂无生成的策略，请先点击「生成 AI 策略」" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, reactive, watch } from 'vue'
import { ElMessage } from 'element-plus'
import {
  Search,
  Refresh,
  MagicStick,
  Trophy,
  Star,
  ScaleToOriginal,
  TrendCharts,
  RefreshRight,
  Edit,
  Connection,
} from '@element-plus/icons-vue'
import {
  aiTransferApi,
  type TeamGMProfileInfo,
  type PersonalityTypeInfo,
  type AITransferStrategy,
  type SellAggressivenessType,
  type LLMConfigInfo,
} from '@/api/tauri'
import { useAIStrategyStore } from '@/stores/useAIStrategyStore'

// 使用全局 store 管理 AI 策略生成状态
const aiStrategyStore = useAIStrategyStore()

// 数据
const profiles = ref<TeamGMProfileInfo[]>([])
const personalityTypes = ref<PersonalityTypeInfo[]>([])
const isLoading = ref(false)
const isSaving = ref(false)

// 单个球队策略查看
const teamStrategyDialogVisible = ref(false)
const currentTeamStrategy = ref<AITransferStrategy | null>(null)
const loadingStrategyTeamId = ref<number | null>(null)
const viewingTeamId = ref<number | null>(null)
const viewingTeamName = ref<string>('')
const isGeneratingSingleStrategy = ref(false)

// LLM 配置
const llmConfig = ref<LLMConfigInfo>({
  is_configured: false,
  provider: 'openai',
  model: 'gpt-4o-mini',
  base_url: null,
})
const llmForm = reactive({
  provider: 'openai',
  apiKey: '',
  model: 'gpt-4o-mini',
})
const isSavingLLM = ref(false)

// 监听 provider 变化，自动切换默认模型
watch(() => llmForm.provider, (newProvider) => {
  const defaultModels: Record<string, string> = {
    'openai': 'gpt-4o-mini',
    'claude': 'claude-3-5-sonnet-20241022',
    'deepseek': 'deepseek-chat',
    'qwen': 'qwen-turbo',
    'moonshot': 'moonshot-v1-8k',
    'zhipu': 'glm-4-flash',
  }
  llmForm.model = defaultModels[newProvider] || 'gpt-4o-mini'
})

// 位置列表
const positions = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']

// 筛选
const filters = reactive({
  region: '',
  personality: '',
  search: '',
})

// 编辑状态
const editDialogVisible = ref(false)
const strategyDialogVisible = ref(false)
const editingProfile = ref<TeamGMProfileInfo | null>(null)
const editForm = reactive({
  personality: 'BALANCED',
  riskTolerance: 50,
  customPrompt: '',
  budgetRatioPercent: 60,  // 0-100 显示，实际保存时转为 0.0-1.0
  sellAggressiveness: 'NORMAL' as SellAggressivenessType,
  preferredAgeMin: 18,
  preferredAgeMax: 30,
  minAbilityThreshold: 70,
  pricePremiumPercent: 100,  // 50-150 显示，实际保存时转为 0.5-1.5
  positionPriorities: {
    TOP: 50,
    JUG: 50,
    MID: 50,
    ADC: 50,
    SUP: 50,
  } as Record<string, number>,
  // 选秀配置
  draftSellThresholdPercent: 50,
  draftBidAggressivenessPercent: 100,
  draftStrategy: 'balanced' as 'immediate' | 'balanced' | 'potential' | 'custom',
  draftAbilityWeightPercent: 40,
  draftYoungBiasPercent: 0,
})

// 过滤后的配置
const filteredProfiles = computed(() => {
  return profiles.value.filter(p => {
    if (filters.region && p.region_name !== filters.region) return false
    if (filters.personality && p.personality !== filters.personality) return false
    if (filters.search && !p.team_name.toLowerCase().includes(filters.search.toLowerCase())) return false
    return true
  })
})

// ========== LLM 配置相关方法 ==========

// 加载 LLM 配置
const loadLLMConfig = async () => {
  try {
    llmConfig.value = await aiTransferApi.checkLLMConfig()
  } catch (e) {
    console.error('Failed to load LLM config:', e)
  }
}

// 保存 LLM 配置
const saveLLMConfig = async () => {
  if (!llmForm.apiKey.trim()) {
    ElMessage.warning('请输入 API Key')
    return
  }
  isSavingLLM.value = true
  try {
    await aiTransferApi.configureLLM(
      llmForm.provider,
      llmForm.apiKey,
      llmForm.model
    )
    ElMessage.success('LLM 配置保存成功')
    await loadLLMConfig()
    llmForm.apiKey = ''  // 清空输入的 key
  } catch (e) {
    console.error('Failed to save LLM config:', e)
    ElMessage.error('保存 LLM 配置失败')
  } finally {
    isSavingLLM.value = false
  }
}

// 清除 LLM 配置
const clearLLMConfig = async () => {
  try {
    await aiTransferApi.clearLLMConfig()
    ElMessage.success('LLM 配置已清除')
    await loadLLMConfig()
  } catch (e) {
    console.error('Failed to clear LLM config:', e)
    ElMessage.error('清除 LLM 配置失败')
  }
}

// ========== 数据加载 ==========

// 加载数据
const loadData = async () => {
  isLoading.value = true
  try {
    await aiTransferApi.initAITransferTables()
    await loadLLMConfig()
    personalityTypes.value = await aiTransferApi.getPersonalityTypes()
    profiles.value = await aiTransferApi.getAllGMProfiles()
  } catch (e) {
    console.error('Failed to load data:', e)
    ElMessage.error('加载数据失败')
  } finally {
    isLoading.value = false
  }
}

// 生成 AI 策略（使用全局 store，切换页面不会丢失进度）
const generateStrategies = async () => {
  try {
    const result = await aiStrategyStore.generateStrategies()
    strategyDialogVisible.value = true
    ElMessage.success(`已为 ${result.length} 支球队生成策略`)
  } catch (e) {
    console.error('Failed to generate strategies:', e)
    ElMessage.error('生成策略失败')
  }
}

// 人格改变时重置为默认值
const onPersonalityChange = () => {
  const p = personalityTypes.value.find(pt => pt.value === editForm.personality)
  if (!p) return

  // 根据人格类型重置相关默认值
  const defaults: Record<string, any> = {
    CHAMPIONSHIP: { budget: 80, ability: 85, age: [20, 30], premium: 130, draftSell: 20, draftBid: 150, draftAbility: 70, draftYoung: -10 },
    YOUTH_DEVELOPMENT: { budget: 40, ability: 65, age: [18, 22], premium: 90, draftSell: 40, draftBid: 80, draftAbility: 20, draftYoung: 20 },
    BALANCED: { budget: 60, ability: 75, age: [20, 28], premium: 100, draftSell: 50, draftBid: 100, draftAbility: 40, draftYoung: 0 },
    SPECULATOR: { budget: 50, ability: 70, age: [18, 25], premium: 85, draftSell: 70, draftBid: 90, draftAbility: 30, draftYoung: 10 },
    REBUILDING: { budget: 30, ability: 60, age: [18, 24], premium: 80, draftSell: 50, draftBid: 120, draftAbility: 10, draftYoung: 15 },
    CUSTOM: { budget: 60, ability: 70, age: [18, 30], premium: 100, draftSell: 50, draftBid: 100, draftAbility: 40, draftYoung: 0 },
  }

  const cfg = defaults[editForm.personality] || defaults.BALANCED
  editForm.budgetRatioPercent = cfg.budget
  editForm.minAbilityThreshold = cfg.ability
  editForm.preferredAgeMin = cfg.age[0]
  editForm.preferredAgeMax = cfg.age[1]
  editForm.pricePremiumPercent = cfg.premium
  editForm.draftSellThresholdPercent = cfg.draftSell
  editForm.draftBidAggressivenessPercent = cfg.draftBid
  editForm.draftAbilityWeightPercent = cfg.draftAbility
  editForm.draftYoungBiasPercent = cfg.draftYoung

  // 根据 ability 权重推断选秀策略
  if (cfg.draftAbility >= 70) {
    editForm.draftStrategy = 'immediate'
  } else if (cfg.draftAbility <= 30) {
    editForm.draftStrategy = 'potential'
  } else if (cfg.draftAbility === 40) {
    editForm.draftStrategy = 'balanced'
  } else {
    editForm.draftStrategy = 'custom'
  }
}

// 打开编辑对话框
const openEditDialog = (row: TeamGMProfileInfo) => {
  editingProfile.value = row
  editForm.personality = row.personality
  editForm.riskTolerance = row.risk_tolerance
  editForm.customPrompt = row.custom_prompt || ''
  editForm.budgetRatioPercent = Math.round(row.budget_ratio * 100)
  editForm.sellAggressiveness = row.sell_aggressiveness
  editForm.preferredAgeMin = row.preferred_age_min
  editForm.preferredAgeMax = row.preferred_age_max
  editForm.minAbilityThreshold = row.min_ability_threshold
  editForm.pricePremiumPercent = Math.round(row.price_premium_max * 100)
  editForm.positionPriorities = { ...row.position_priorities }

  // 加载选秀配置
  editForm.draftSellThresholdPercent = Math.round(row.draft_pick_sell_threshold * 100)
  editForm.draftBidAggressivenessPercent = Math.round(row.draft_pick_bid_aggressiveness * 100)
  editForm.draftAbilityWeightPercent = Math.round(row.draft_preference_ability_weight * 100)
  editForm.draftYoungBiasPercent = Math.round(row.draft_young_bias * 100)

  // 根据权重推断选秀策略
  const abilityWeight = row.draft_preference_ability_weight
  if (abilityWeight >= 0.7) {
    editForm.draftStrategy = 'immediate'
  } else if (abilityWeight <= 0.3) {
    editForm.draftStrategy = 'potential'
  } else if (Math.abs(abilityWeight - 0.4) < 0.05) {
    editForm.draftStrategy = 'balanced'
  } else {
    editForm.draftStrategy = 'custom'
  }

  editDialogVisible.value = true
}

// 保存配置
const saveProfile = async () => {
  if (!editingProfile.value) return

  // 计算最终的 ability_weight（根据策略）
  let finalAbilityWeight = editForm.draftAbilityWeightPercent / 100
  if (editForm.draftStrategy === 'immediate') {
    finalAbilityWeight = 0.7
  } else if (editForm.draftStrategy === 'potential') {
    finalAbilityWeight = 0.2
  } else if (editForm.draftStrategy === 'balanced') {
    finalAbilityWeight = 0.4
  }

  isSaving.value = true
  try {
    await aiTransferApi.updateTeamGMProfile(
      editingProfile.value.team_id,
      editForm.personality,
      editForm.personality === 'CUSTOM' ? editForm.customPrompt : null,
      editForm.riskTolerance,
      editForm.budgetRatioPercent / 100,  // 转换为 0.0-1.0
      editForm.sellAggressiveness,
      editForm.preferredAgeMin,
      editForm.preferredAgeMax,
      editForm.minAbilityThreshold,
      editForm.pricePremiumPercent / 100,  // 转换为 0.5-1.5
      editForm.positionPriorities,
      editForm.draftSellThresholdPercent / 100,  // 转换为 0.0-1.0
      editForm.draftBidAggressivenessPercent / 100,  // 转换为 0.5-2.0
      finalAbilityWeight,  // 转换为 0.0-1.0
      editForm.draftYoungBiasPercent / 100  // 转换为 -0.2 到 0.2
    )
    ElMessage.success('保存成功')
    editDialogVisible.value = false
    await loadData()
  } catch (e) {
    console.error('Failed to save profile:', e)
    ElMessage.error('保存失败')
  } finally {
    isSaving.value = false
  }
}

// 辅助函数
const formatRegion = (regionId: number) => {
  const map: Record<number, string> = { 1: 'LPL', 2: 'LCK', 3: 'LEC', 4: 'LCS' }
  return map[regionId] || `R${regionId}`
}

const getRegionTagType = (regionName: string) => {
  const types: Record<string, string> = { 'LPL': 'danger', 'LCK': 'primary', 'LEC': 'success', 'LCS': 'warning' }
  return types[regionName] || 'info'
}

const getPersonalityIcon = (personality: string) => {
  const icons: Record<string, any> = {
    CHAMPIONSHIP: Trophy,
    YOUTH_DEVELOPMENT: Star,
    BALANCED: ScaleToOriginal,
    SPECULATOR: TrendCharts,
    REBUILDING: RefreshRight,
    CUSTOM: Edit,
  }
  return icons[personality] || Edit
}

const getSellAggressivenessHint = (value: string) => {
  const hints: Record<string, string> = {
    CONSERVATIVE: '只出售低价值/老将，保持阵容稳定',
    NORMAL: '平衡出售，根据需求调整',
    AGGRESSIVE: '积极清洗阵容，快速换血',
  }
  return hints[value] || ''
}

const getDraftStrategyHint = (value: string) => {
  const hints: Record<string, string> = {
    immediate: '70% Ability + 30% Potential，选即战力球员',
    balanced: '40% Ability + 60% Potential，标准 BPA 策略',
    potential: '20% Ability + 80% Potential，选未来之星',
    custom: '自定义能力-潜力权重比例',
  }
  return hints[value] || ''
}

const formatBias = (value: number) => {
  if (value === 0) return '中立'
  if (value > 0) return `+${value}%`
  return `${value}%`
}

// 查看单个球队的 AI 策略
const viewTeamStrategy = async (row: TeamGMProfileInfo) => {
  loadingStrategyTeamId.value = row.team_id
  // 记录当前查看的球队信息（用于单独生成策略）
  viewingTeamId.value = row.team_id
  viewingTeamName.value = row.team_name
  try {
    const strategy = await aiTransferApi.getTeamAIStrategy(row.team_id)
    // 添加 team_name 到策略对象（后端可能没有返回）
    currentTeamStrategy.value = {
      ...strategy,
      team_name: row.team_name,
    }
    teamStrategyDialogVisible.value = true
  } catch (e) {
    console.error('Failed to load team strategy:', e)
    ElMessage.warning('该球队暂无生成的策略')
    currentTeamStrategy.value = null
    teamStrategyDialogVisible.value = true
  } finally {
    loadingStrategyTeamId.value = null
  }
}

// 为单个球队生成 AI 策略
const generateSingleTeamStrategy = async () => {
  if (!viewingTeamId.value) {
    ElMessage.warning('未选择球队')
    return
  }
  isGeneratingSingleStrategy.value = true
  try {
    const strategy = await aiTransferApi.generateLLMStrategy(viewingTeamId.value)
    currentTeamStrategy.value = {
      ...strategy,
      team_name: viewingTeamName.value || strategy.team_name,
    }
    ElMessage.success(`已为 ${viewingTeamName.value} 生成策略`)
  } catch (e) {
    console.error('Failed to generate single team strategy:', e)
    ElMessage.error('生成策略失败，请确认已配置 LLM')
  } finally {
    isGeneratingSingleStrategy.value = false
  }
}

// 格式化生成时间
const formatGeneratedTime = (time: string) => {
  if (!time) return '未知'
  try {
    const date = new Date(time)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return time
  }
}

const formatBudget = (value: number) => {
  if (value >= 10000) return `${(value / 10000).toFixed(1)}亿`
  if (value >= 1) return `${Math.round(value)}万`
  return `${value}万`
}

// 获取分析步骤的类型颜色
const getStepType = (stepName: string) => {
  if (stepName.includes('最终') || stepName.includes('决策')) return 'success'
  if (stepName.includes('阵容') || stepName.includes('评估')) return 'primary'
  if (stepName.includes('位置') || stepName.includes('分析')) return ''
  if (stepName.includes('财务') || stepName.includes('预算')) return 'warning'
  if (stepName.includes('荣誉') || stepName.includes('表现')) return 'info'
  return ''
}

// 初始化
onMounted(() => {
  loadData()
})
</script>

<style scoped>
.gm-config-view {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.progress-card {
  margin-bottom: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.progress-card :deep(.el-card__body) {
  padding: 16px 20px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  color: #fff;
}

.progress-title {
  font-size: 14px;
  font-weight: 500;
}

.progress-count {
  font-size: 16px;
  font-weight: 700;
}

.progress-card :deep(.el-progress-bar__outer) {
  background-color: rgba(255, 255, 255, 0.3);
}

.progress-card :deep(.el-progress-bar__inner) {
  background: linear-gradient(90deg, #fff 0%, #e8f5e9 100%);
}

.progress-card :deep(.el-progress__text) {
  color: #fff;
  font-weight: 600;
}

/* LLM 配置卡片 */
.llm-config-card {
  margin-bottom: 16px;
  border-radius: 12px;
  border: 1px solid #e4e7ed;
}

.llm-config-card :deep(.el-card__header) {
  padding: 12px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-bottom: none;
}

.llm-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.llm-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: white;
  font-weight: 600;
  font-size: 14px;
}

.llm-content {
  padding: 8px 0;
}

.llm-info {
  display: flex;
  align-items: center;
  gap: 24px;
  color: #606266;
}

.llm-info strong {
  color: #303133;
}

.llm-form {
  display: flex;
  flex-direction: column;
}

.llm-form :deep(.el-form-item) {
  margin-bottom: 8px;
}

.llm-hint {
  margin-top: 8px;
  color: #909399;
  font-size: 12px;
}

.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.config-card {
  border-radius: 12px;
}

.clickable-row {
  cursor: pointer;
}

.clickable-row:hover {
  background-color: #ecf5ff !important;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 4px;
}

.team-name {
  font-weight: 600;
  color: #303133;
}

.team-short {
  color: #909399;
  font-size: 12px;
}

.personality-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.personality-icon-championship {
  color: #e6a23c;
}

.personality-icon-youth_development {
  color: #67c23a;
}

.personality-icon-balanced {
  color: #409eff;
}

.personality-icon-speculator {
  color: #9b59b6;
}

.personality-icon-rebuilding {
  color: #f56c6c;
}

.personality-icon-custom {
  color: #606266;
}

.budget-value {
  font-weight: 600;
  color: #409eff;
}

.age-range {
  color: #606266;
}

/* 编辑对话框 */
.personality-option {
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.option-name {
  font-weight: 600;
}

.option-desc {
  font-size: 12px;
  color: #909399;
}

.slider-container {
  width: 100%;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.value-label {
  font-weight: 600;
  color: #303133;
}

.age-range-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.range-separator {
  color: #909399;
}

.range-hint {
  color: #909399;
  font-size: 13px;
}

.position-priorities {
  width: 100%;
}

.position-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.position-label {
  font-weight: 600;
  color: #303133;
  width: 40px;
}

.priority-value {
  font-weight: 600;
  color: #409eff;
  min-width: 30px;
}

.form-hint {
  margin-top: 8px;
  color: #909399;
  font-size: 12px;
}

/* 策略展示 */
.count-badge {
  display: inline-block;
  min-width: 24px;
  height: 24px;
  line-height: 24px;
  text-align: center;
  border-radius: 12px;
  font-weight: 600;
  font-size: 12px;
}

.count-badge.targets {
  background: #e1f3d8;
  color: #67c23a;
}

.count-badge.sells {
  background: #fef0f0;
  color: #f56c6c;
}

.pos-tag {
  margin-right: 4px;
  margin-bottom: 2px;
}

.reasoning-text {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  font-size: 13px;
  color: #606266;
}

/* 单个球队策略详情 */
.strategy-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.strategy-overview {
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e7ed 100%);
}

.overview-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.strategy-type {
  display: flex;
  align-items: center;
  gap: 12px;
}

.strategy-desc {
  color: #606266;
  font-size: 14px;
}

.strategy-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mock-badge {
  padding: 2px 8px;
  background: #909399;
  color: white;
  border-radius: 4px;
  font-size: 12px;
}

.llm-badge {
  padding: 2px 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 4px;
  font-size: 12px;
}

.generated-time {
  color: #909399;
  font-size: 12px;
}

.reasoning-box {
  background: white;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e4e7ed;
}

.reasoning-box strong {
  color: #303133;
}

.reasoning-box p {
  margin: 8px 0 0 0;
  color: #606266;
  line-height: 1.6;
}

.budget-card,
.targets-card,
.sell-card {
  border: 1px solid #e4e7ed;
}

.card-title {
  font-weight: 600;
  color: #303133;
}

.card-header-with-count {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.budget-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.budget-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.budget-label {
  color: #909399;
  font-size: 12px;
  margin-bottom: 4px;
}

.budget-value {
  font-size: 18px;
  font-weight: 700;
}

.budget-value.total {
  color: #303133;
}

.budget-value.transfer {
  color: #409eff;
}

.budget-value.salary {
  color: #67c23a;
}

.budget-value.reserve {
  color: #e6a23c;
}

.priority-positions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.pos-tag-large {
  margin-right: 4px;
}

.no-data {
  color: #909399;
  font-size: 13px;
}

.ability-value {
  font-weight: 600;
  color: #409eff;
}

.potential-value {
  color: #67c23a;
}

.offer-value {
  font-weight: 600;
  color: #409eff;
}

.min-price {
  font-weight: 600;
  color: #f56c6c;
}

/* 对话框头部带按钮 */
.dialog-header-with-action {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding-right: 32px;
}

.dialog-header-with-action span {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

/* 分析步骤样式 */
.analysis-steps-section {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px dashed #dcdfe6;
}

.analysis-steps-section > strong {
  display: block;
  margin-bottom: 12px;
  color: #606266;
  font-size: 14px;
}

.analysis-step-item {
  padding: 8px 0;
}

.step-header {
  margin-bottom: 8px;
}

.step-name {
  font-weight: 600;
  color: #303133;
  font-size: 14px;
}

.step-content {
  background: #f5f7fa;
  border-radius: 6px;
  padding: 10px 12px;
}

.step-row {
  display: flex;
  margin-bottom: 6px;
  font-size: 13px;
  line-height: 1.5;
}

.step-row:last-child {
  margin-bottom: 0;
}

.step-label {
  color: #909399;
  min-width: 48px;
  flex-shrink: 0;
}

.step-value {
  color: #606266;
  word-break: break-word;
}

.step-value.threshold {
  color: #e6a23c;
}

.step-value.result {
  color: #409eff;
  font-weight: 500;
}

.step-value.impact {
  color: #67c23a;
  font-weight: 500;
}
</style>
