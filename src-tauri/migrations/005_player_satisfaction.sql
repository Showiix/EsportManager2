-- 005_player_satisfaction.sql
-- 选手满意度、忠诚度和球队赛季表现系统

-- 选手赛季状态表
CREATE TABLE IF NOT EXISTS player_season_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,

    -- 满意度 (0-100)
    satisfaction INTEGER NOT NULL DEFAULT 70,

    -- 离队意愿
    wants_to_leave BOOLEAN NOT NULL DEFAULT FALSE,
    departure_reasons TEXT DEFAULT '[]',  -- JSON数组

    -- 上赛季数据（用于计算）
    games_as_starter INTEGER DEFAULT 0,
    total_games INTEGER DEFAULT 0,

    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(save_id, season_id, player_id)
);

-- 球队赛季表现表
CREATE TABLE IF NOT EXISTS team_season_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,

    -- 战绩数据
    final_rank INTEGER,
    made_playoffs BOOLEAN DEFAULT FALSE,
    playoff_result TEXT,  -- CHAMPION/RUNNER_UP/SEMI/QUARTER/NULL
    international_result TEXT,  -- MSI_CHAMPION/WORLDS_CHAMPION/...

    -- 连续统计
    consecutive_no_playoffs INTEGER DEFAULT 0,

    created_at TEXT DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(save_id, season_id, team_id)
);

-- 选手忠诚度变化记录表
CREATE TABLE IF NOT EXISTS loyalty_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    change_amount INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_player_season_status_lookup
ON player_season_status(save_id, season_id, player_id);

CREATE INDEX IF NOT EXISTS idx_player_season_status_departure
ON player_season_status(save_id, season_id, wants_to_leave);

CREATE INDEX IF NOT EXISTS idx_team_season_performance_lookup
ON team_season_performance(save_id, season_id, team_id);

CREATE INDEX IF NOT EXISTS idx_loyalty_changes_player
ON loyalty_changes(save_id, player_id);
