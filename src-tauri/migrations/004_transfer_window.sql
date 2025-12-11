-- AI 转会系统数据库迁移
-- 添加转会窗口和转会事件相关表

-- 转会窗口状态表
CREATE TABLE IF NOT EXISTS transfer_windows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'PREPARING',  -- PREPARING/IN_PROGRESS/COMPLETED
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL DEFAULT 5,

    -- 统计数据
    total_transfers INTEGER NOT NULL DEFAULT 0,
    total_fees INTEGER NOT NULL DEFAULT 0,
    free_agents_signed INTEGER NOT NULL DEFAULT 0,
    retirements INTEGER NOT NULL DEFAULT 0,
    contract_expires INTEGER NOT NULL DEFAULT 0,

    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id)
);

-- 转会事件表（用于新闻播报）
CREATE TABLE IF NOT EXISTS transfer_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    round INTEGER NOT NULL,

    -- 事件类型
    event_type TEXT NOT NULL,  -- FREE_AGENT/PURCHASE/RETIREMENT/CONTRACT_EXPIRE
    status TEXT NOT NULL DEFAULT 'PENDING',  -- PENDING/COMPLETED/FAILED

    -- 选手信息
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    position TEXT,
    age INTEGER,
    ability INTEGER,
    potential INTEGER,
    market_value INTEGER,

    -- 转会双方
    from_team_id INTEGER,
    from_team_name TEXT,
    to_team_id INTEGER,
    to_team_name TEXT,

    -- 财务信息
    transfer_fee INTEGER NOT NULL DEFAULT 0,
    new_salary INTEGER,
    contract_years INTEGER,
    contract_type TEXT DEFAULT 'STANDARD',  -- ROOKIE/STANDARD

    -- 身价相关
    price_ratio REAL,  -- 转会费/身价 比例，用于显示溢价/折扣

    -- 新闻信息
    headline TEXT,
    description TEXT,
    importance TEXT NOT NULL DEFAULT 'NORMAL',  -- BREAKING/MAJOR/NORMAL/MINOR

    -- 竞争情况（自由球员签约时）
    competing_teams TEXT,  -- JSON 数组存储竞争球队ID
    was_bidding_war INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (to_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 转会轮次摘要表
CREATE TABLE IF NOT EXISTS transfer_round_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    round INTEGER NOT NULL,
    round_name TEXT NOT NULL,

    -- 轮次统计
    events_count INTEGER NOT NULL DEFAULT 0,
    transfers_count INTEGER NOT NULL DEFAULT 0,
    total_fees INTEGER NOT NULL DEFAULT 0,

    summary TEXT,  -- 轮次总结文本

    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id, round)
);

-- 球队转会计划表（AI 决策用）
CREATE TABLE IF NOT EXISTS team_transfer_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,

    -- 财务状况
    balance INTEGER NOT NULL,
    financial_status TEXT NOT NULL,  -- WEALTHY/HEALTHY/TIGHT/DEFICIT/BANKRUPT
    transfer_budget INTEGER NOT NULL,
    salary_space INTEGER NOT NULL,
    current_total_salary INTEGER NOT NULL,

    -- 阵容状况
    roster_count INTEGER NOT NULL,
    avg_ability REAL NOT NULL,
    avg_age REAL NOT NULL,

    -- 位置需求 (JSON)
    position_needs TEXT,  -- {"TOP": 80, "JUG": 50, ...}

    -- 策略
    strategy TEXT NOT NULL,  -- AGGRESSIVE_BUY/PASSIVE/MUST_SELL/FORCE_CLEAR
    ambition TEXT NOT NULL,  -- CHAMPIONSHIP/PLAYOFF/REBUILD

    -- 标记
    must_sign INTEGER DEFAULT 0,  -- 阵容不足5人
    must_clear INTEGER DEFAULT 0,  -- 阵容超过10人

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id, team_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_transfer_windows_save ON transfer_windows(save_id);
CREATE INDEX IF NOT EXISTS idx_transfer_windows_status ON transfer_windows(status);

CREATE INDEX IF NOT EXISTS idx_transfer_events_save_season ON transfer_events(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_round ON transfer_events(round);
CREATE INDEX IF NOT EXISTS idx_transfer_events_type ON transfer_events(event_type);
CREATE INDEX IF NOT EXISTS idx_transfer_events_importance ON transfer_events(importance);
CREATE INDEX IF NOT EXISTS idx_transfer_events_player ON transfer_events(player_id);

CREATE INDEX IF NOT EXISTS idx_transfer_plans_save_season ON team_transfer_plans(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_plans_team ON team_transfer_plans(team_id);
CREATE INDEX IF NOT EXISTS idx_transfer_plans_strategy ON team_transfer_plans(strategy);
