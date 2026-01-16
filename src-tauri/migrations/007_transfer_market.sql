-- LLM 深度集成转会市场系统数据库迁移
-- 添加谈判、报价、市场状态相关表

-- ==================== 市场状态表 ====================

-- 转会市场整体状态表
CREATE TABLE IF NOT EXISTS transfer_market_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 阶段和轮次
    current_phase TEXT NOT NULL DEFAULT 'INITIALIZATION',
    -- INITIALIZATION/INTENTION_GENERATION/STRATEGY_GENERATION/RENEWAL_PROCESSING/
    -- DEPARTURE_ANNOUNCEMENT/INITIAL_BIDDING/NEGOTIATION_ROUNDS/LAST_CHANCE/FINALIZATION/COMPLETED
    current_round INTEGER NOT NULL DEFAULT 0,
    max_negotiation_rounds INTEGER NOT NULL DEFAULT 5,

    -- 自由球员池 (JSON 数组)
    free_agent_ids TEXT DEFAULT '[]',
    -- 进行中谈判 (JSON 数组)
    active_negotiation_ids TEXT DEFAULT '[]',
    -- 已完成转会 (JSON 数组)
    completed_transfer_ids TEXT DEFAULT '[]',

    -- 生成进度
    intentions_generated INTEGER NOT NULL DEFAULT 0,
    total_players INTEGER NOT NULL DEFAULT 0,
    strategies_generated INTEGER NOT NULL DEFAULT 0,
    total_teams INTEGER NOT NULL DEFAULT 0,

    -- 市场稳定性
    is_market_stable INTEGER NOT NULL DEFAULT 0,
    stable_rounds_count INTEGER NOT NULL DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id)
);

-- 球队市场状态表
CREATE TABLE IF NOT EXISTS team_market_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,

    -- 财务状况
    initial_balance INTEGER NOT NULL DEFAULT 0,
    remaining_budget INTEGER NOT NULL DEFAULT 0,
    spent_amount INTEGER NOT NULL DEFAULT 0,
    salary_increase INTEGER NOT NULL DEFAULT 0,

    -- 阵容状况
    roster_count INTEGER NOT NULL DEFAULT 0,
    min_roster_size INTEGER NOT NULL DEFAULT 5,
    max_roster_size INTEGER NOT NULL DEFAULT 10,

    -- 关联数据 (JSON 数组)
    pending_negotiation_ids TEXT DEFAULT '[]',
    completed_signing_ids TEXT DEFAULT '[]',
    departed_player_ids TEXT DEFAULT '[]',

    -- 策略状态
    strategy_generated INTEGER NOT NULL DEFAULT 0,
    strategy_id INTEGER,
    needs_emergency_signing INTEGER NOT NULL DEFAULT 0,

    -- 位置需求 (JSON 对象)
    position_needs TEXT DEFAULT '{}',

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id, team_id)
);

-- ==================== 谈判相关表 ====================

-- 谈判记录表
CREATE TABLE IF NOT EXISTS negotiations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 选手信息
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    player_position TEXT,
    player_ability INTEGER,

    -- 原球队（自由球员为 NULL）
    from_team_id INTEGER,
    from_team_name TEXT,

    -- 状态
    status TEXT NOT NULL DEFAULT 'OPEN',
    -- OPEN/ACCEPTED/REJECTED/EXPIRED/WITHDRAWN
    current_round INTEGER NOT NULL DEFAULT 0,
    max_rounds INTEGER NOT NULL DEFAULT 5,

    -- 竞争球队 (JSON 数组)
    competing_team_ids TEXT DEFAULT '[]',

    -- 最终结果
    final_team_id INTEGER,
    final_team_name TEXT,
    final_salary INTEGER,
    final_years INTEGER,
    final_starter INTEGER,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (final_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 报价表
CREATE TABLE IF NOT EXISTS offers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    negotiation_id INTEGER NOT NULL,

    -- 报价方
    from_team_id INTEGER NOT NULL,
    from_team_name TEXT NOT NULL,
    to_player_id INTEGER NOT NULL,

    -- 报价轮次
    round INTEGER NOT NULL,

    -- 报价内容
    salary_offer INTEGER NOT NULL DEFAULT 0,  -- 万/年
    contract_years INTEGER NOT NULL DEFAULT 1,
    guarantee_starter INTEGER NOT NULL DEFAULT 0,
    signing_bonus INTEGER NOT NULL DEFAULT 0,  -- 万
    transfer_fee INTEGER NOT NULL DEFAULT 0,   -- 万

    -- 状态
    status TEXT NOT NULL DEFAULT 'PENDING',
    -- PENDING/ACCEPTED/REJECTED/COUNTERED/WITHDRAWN/EXPIRED

    -- AI 分析
    offer_reasoning TEXT,
    analysis_steps TEXT DEFAULT '[]',  -- JSON 数组

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (negotiation_id) REFERENCES negotiations(id) ON DELETE CASCADE,
    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (to_player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- 报价回应表
CREATE TABLE IF NOT EXISTS offer_responses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    offer_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,

    -- 回应类型
    response_type TEXT NOT NULL,
    -- ACCEPT/REJECT/COUNTER/WAIT

    -- 还价内容（当 response_type = COUNTER 时）
    counter_salary INTEGER,
    counter_years INTEGER,
    counter_starter INTEGER,

    -- AI 分析
    reasoning TEXT,
    analysis_steps TEXT DEFAULT '[]',  -- JSON 数组

    responded_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- ==================== 市场事件表 ====================

-- 转会市场事件表（扩展版）
CREATE TABLE IF NOT EXISTS transfer_market_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 事件类型
    event_type TEXT NOT NULL,
    -- CONTRACT_EXPIRED/PLAYER_RETIRED/TRANSFER_REQUESTED/RENEWAL_SUCCESSFUL/RENEWAL_FAILED/
    -- OFFER_MADE/OFFER_ACCEPTED/OFFER_REJECTED/COUNTER_OFFER/OFFER_RAISED/TEAM_WITHDREW/
    -- SIGNING_COMPLETED/TRADE_COMPLETED/EMERGENCY_SIGNING

    -- 发生阶段和轮次
    phase TEXT NOT NULL,
    round INTEGER NOT NULL DEFAULT 0,

    -- 相关选手
    player_id INTEGER,
    player_name TEXT,

    -- 相关球队
    team_id INTEGER,
    team_name TEXT,
    secondary_team_id INTEGER,
    secondary_team_name TEXT,

    -- 金额
    amount INTEGER,

    -- 事件内容
    title TEXT NOT NULL,
    description TEXT,
    ai_analysis TEXT,

    -- 重要程度 (1-5)
    importance INTEGER NOT NULL DEFAULT 2,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE SET NULL,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (secondary_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- ==================== 选手意愿策略存储表 ====================

-- 选手转会策略表（扩展现有功能）
CREATE TABLE IF NOT EXISTS player_transfer_strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 离队意愿
    wants_to_leave INTEGER NOT NULL DEFAULT 0,
    decision_confidence INTEGER NOT NULL DEFAULT 50,
    departure_reasons TEXT DEFAULT '[]',  -- JSON 数组
    leave_reasoning TEXT,

    -- 偏好球队
    preferred_teams TEXT DEFAULT '[]',  -- JSON 数组
    team_preference_reasoning TEXT,

    -- 期望条件
    expected_salary INTEGER NOT NULL DEFAULT 0,  -- 万/年
    expected_min_salary INTEGER NOT NULL DEFAULT 0,
    expected_years INTEGER NOT NULL DEFAULT 1,
    requires_starter INTEGER NOT NULL DEFAULT 0,

    -- 分析数据
    analysis_data TEXT,  -- JSON 对象
    analysis_steps TEXT DEFAULT '[]',  -- JSON 数组

    -- 元数据
    is_mock INTEGER NOT NULL DEFAULT 1,
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(player_id, save_id, season_id)
);

-- 球队转会策略表（扩展现有功能）
CREATE TABLE IF NOT EXISTS team_transfer_strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id INTEGER NOT NULL,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 策略概述
    overall_strategy TEXT,
    strategy_description TEXT,
    reasoning TEXT,

    -- 目标和出售
    targets TEXT DEFAULT '[]',  -- JSON 数组
    willing_to_sell TEXT DEFAULT '[]',  -- JSON 数组
    priority_positions TEXT DEFAULT '[]',  -- JSON 数组

    -- 预算分配
    total_budget INTEGER NOT NULL DEFAULT 0,
    transfer_spend INTEGER NOT NULL DEFAULT 0,
    salary_spend INTEGER NOT NULL DEFAULT 0,
    reserve INTEGER NOT NULL DEFAULT 0,

    -- 分析步骤
    analysis_steps TEXT DEFAULT '[]',  -- JSON 数组

    -- 元数据
    is_mock INTEGER NOT NULL DEFAULT 1,
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(team_id, save_id, season_id)
);

-- ==================== 创建索引 ====================

-- 市场状态索引
CREATE INDEX IF NOT EXISTS idx_market_states_save ON transfer_market_states(save_id);
CREATE INDEX IF NOT EXISTS idx_market_states_phase ON transfer_market_states(current_phase);

-- 球队市场状态索引
CREATE INDEX IF NOT EXISTS idx_team_market_save_season ON team_market_states(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_team_market_team ON team_market_states(team_id);

-- 谈判索引
CREATE INDEX IF NOT EXISTS idx_negotiations_save_season ON negotiations(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_negotiations_player ON negotiations(player_id);
CREATE INDEX IF NOT EXISTS idx_negotiations_status ON negotiations(status);
CREATE INDEX IF NOT EXISTS idx_negotiations_from_team ON negotiations(from_team_id);

-- 报价索引
CREATE INDEX IF NOT EXISTS idx_offers_negotiation ON offers(negotiation_id);
CREATE INDEX IF NOT EXISTS idx_offers_team ON offers(from_team_id);
CREATE INDEX IF NOT EXISTS idx_offers_player ON offers(to_player_id);
CREATE INDEX IF NOT EXISTS idx_offers_status ON offers(status);
CREATE INDEX IF NOT EXISTS idx_offers_round ON offers(round);

-- 回应索引
CREATE INDEX IF NOT EXISTS idx_responses_offer ON offer_responses(offer_id);
CREATE INDEX IF NOT EXISTS idx_responses_player ON offer_responses(player_id);

-- 事件索引
CREATE INDEX IF NOT EXISTS idx_market_events_save_season ON transfer_market_events(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_market_events_type ON transfer_market_events(event_type);
CREATE INDEX IF NOT EXISTS idx_market_events_phase ON transfer_market_events(phase);
CREATE INDEX IF NOT EXISTS idx_market_events_player ON transfer_market_events(player_id);
CREATE INDEX IF NOT EXISTS idx_market_events_team ON transfer_market_events(team_id);
CREATE INDEX IF NOT EXISTS idx_market_events_importance ON transfer_market_events(importance);

-- 选手策略索引
CREATE INDEX IF NOT EXISTS idx_player_strategies_save_season ON player_transfer_strategies(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_player_strategies_player ON player_transfer_strategies(player_id);
CREATE INDEX IF NOT EXISTS idx_player_strategies_wants_leave ON player_transfer_strategies(wants_to_leave);

-- 球队策略索引
CREATE INDEX IF NOT EXISTS idx_team_strategies_save_season ON team_transfer_strategies(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_team_strategies_team ON team_transfer_strategies(team_id);
