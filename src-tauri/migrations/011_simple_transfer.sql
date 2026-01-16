-- 简化版转会系统数据表
-- 移除 LLM 和 GM 人格系统

-- 转会市场状态表
CREATE TABLE IF NOT EXISTS simple_transfer_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    phase TEXT NOT NULL DEFAULT 'MarketAnalysis',
    current_round INTEGER NOT NULL DEFAULT 0,
    max_rounds INTEGER NOT NULL DEFAULT 5,
    no_signing_rounds INTEGER NOT NULL DEFAULT 0,
    free_agents_count INTEGER NOT NULL DEFAULT 0,
    willing_to_transfer_count INTEGER NOT NULL DEFAULT 0,
    active_negotiations INTEGER NOT NULL DEFAULT 0,
    completed_signings INTEGER NOT NULL DEFAULT 0,
    is_initialized INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, season_id)
);

-- 选手转会信息表
CREATE TABLE IF NOT EXISTS simple_player_transfer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    position TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    team_id INTEGER,
    team_name TEXT,
    salary INTEGER NOT NULL DEFAULT 0,
    market_value INTEGER NOT NULL DEFAULT 0,
    contract_end_season INTEGER,
    satisfaction INTEGER NOT NULL DEFAULT 70,
    loyalty INTEGER NOT NULL DEFAULT 50,
    intent TEXT NOT NULL DEFAULT 'StayNeutral',
    departure_reason TEXT NOT NULL DEFAULT 'None',
    status TEXT NOT NULL DEFAULT 'Contracted',
    expected_salary INTEGER NOT NULL DEFAULT 50,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, season_id, player_id)
);

-- 球队转会策略表
CREATE TABLE IF NOT EXISTS simple_team_strategy (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,
    region_id INTEGER NOT NULL,
    strategy_type TEXT NOT NULL DEFAULT 'Maintaining',
    avg_ability REAL NOT NULL DEFAULT 0,
    roster_count INTEGER NOT NULL DEFAULT 0,
    budget INTEGER NOT NULL DEFAULT 0,
    salary_cap_space INTEGER NOT NULL DEFAULT 0,
    position_needs_json TEXT,  -- JSON 格式存储位置需求
    targets_json TEXT,         -- JSON 格式存储引援目标
    sell_candidates_json TEXT, -- JSON 格式存储出售候选
    summary TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, season_id, team_id)
);

-- 转会报价表
CREATE TABLE IF NOT EXISTS simple_transfer_offer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    from_team_id INTEGER NOT NULL,
    from_team_name TEXT NOT NULL,
    salary_offer INTEGER NOT NULL,
    contract_years INTEGER NOT NULL DEFAULT 2,
    transfer_fee INTEGER NOT NULL DEFAULT 0,
    is_starter_promised INTEGER NOT NULL DEFAULT 0,
    round INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 续约结果表
CREATE TABLE IF NOT EXISTS simple_renewal_result (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,
    success INTEGER NOT NULL DEFAULT 0,
    new_salary INTEGER,
    new_years INTEGER,
    failure_reason TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 转会事件表
CREATE TABLE IF NOT EXISTS simple_transfer_event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    round INTEGER NOT NULL DEFAULT 0,
    player_id INTEGER,
    player_name TEXT,
    from_team_id INTEGER,
    from_team_name TEXT,
    to_team_id INTEGER,
    to_team_name TEXT,
    amount INTEGER,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_simple_transfer_state_save_season ON simple_transfer_state(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_simple_player_transfer_save_season ON simple_player_transfer(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_simple_player_transfer_team ON simple_player_transfer(team_id);
CREATE INDEX IF NOT EXISTS idx_simple_team_strategy_save_season ON simple_team_strategy(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_simple_transfer_offer_player ON simple_transfer_offer(player_id);
CREATE INDEX IF NOT EXISTS idx_simple_transfer_event_save_season ON simple_transfer_event(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_simple_transfer_event_round ON simple_transfer_event(round);
