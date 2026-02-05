-- ============================================
-- 转会系统数据库迁移
-- 版本: 010
-- 日期: 2026-02-05
-- ============================================

-- 转会期表
CREATE TABLE IF NOT EXISTS transfer_windows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    current_round INTEGER DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 转会事件表
CREATE TABLE IF NOT EXISTS transfer_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    window_id INTEGER NOT NULL,
    round INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'C',
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    player_ability INTEGER NOT NULL DEFAULT 0,
    from_team_id INTEGER,
    from_team_name TEXT,
    to_team_id INTEGER,
    to_team_name TEXT,
    transfer_fee INTEGER DEFAULT 0,
    salary INTEGER NOT NULL DEFAULT 0,
    contract_years INTEGER NOT NULL DEFAULT 1,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (window_id) REFERENCES transfer_windows(id),
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- AI球队性格配置表
CREATE TABLE IF NOT EXISTS team_personality_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id INTEGER NOT NULL UNIQUE,
    save_id TEXT NOT NULL,
    personality TEXT NOT NULL DEFAULT 'BALANCED',
    short_term_focus REAL DEFAULT 0.5,
    long_term_focus REAL DEFAULT 0.5,
    risk_tolerance REAL DEFAULT 0.5,
    youth_preference REAL DEFAULT 0.5,
    star_chasing REAL DEFAULT 0.5,
    bargain_hunting REAL DEFAULT 0.5,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 球队声望缓存表
CREATE TABLE IF NOT EXISTS team_reputation_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id INTEGER NOT NULL,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    overall INTEGER NOT NULL DEFAULT 30,
    historical INTEGER NOT NULL DEFAULT 30,
    recent INTEGER NOT NULL DEFAULT 30,
    international INTEGER NOT NULL DEFAULT 0,
    calculated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 球员挂牌表
CREATE TABLE IF NOT EXISTS player_listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL,
    window_id INTEGER NOT NULL,
    listed_by_team_id INTEGER NOT NULL,
    listing_price INTEGER,
    min_accept_price INTEGER,
    status TEXT DEFAULT 'ACTIVE',
    listed_at TEXT NOT NULL DEFAULT (datetime('now')),
    sold_at TEXT,
    sold_to_team_id INTEGER,
    actual_price INTEGER,
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (window_id) REFERENCES transfer_windows(id),
    FOREIGN KEY (listed_by_team_id) REFERENCES teams(id)
);

-- 球员冷却期记录表
CREATE TABLE IF NOT EXISTS player_cooldowns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    window_id INTEGER NOT NULL,
    cooldown_until_round INTEGER NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (window_id) REFERENCES transfer_windows(id)
);

-- ============================================
-- 索引
-- ============================================

-- 转会事件查询索引
CREATE INDEX IF NOT EXISTS idx_transfer_events_window ON transfer_events(window_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_round ON transfer_events(window_id, round);
CREATE INDEX IF NOT EXISTS idx_transfer_events_level ON transfer_events(level);
CREATE INDEX IF NOT EXISTS idx_transfer_events_player ON transfer_events(player_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_to_team ON transfer_events(to_team_id);

-- 转会期查询索引
CREATE INDEX IF NOT EXISTS idx_transfer_windows_save ON transfer_windows(save_id);
CREATE INDEX IF NOT EXISTS idx_transfer_windows_season ON transfer_windows(save_id, season_id);

-- 球员挂牌查询索引
CREATE INDEX IF NOT EXISTS idx_player_listings_window ON player_listings(window_id);
CREATE INDEX IF NOT EXISTS idx_player_listings_status ON player_listings(status);
CREATE INDEX IF NOT EXISTS idx_player_listings_player ON player_listings(player_id);

-- 球队声望查询索引
CREATE INDEX IF NOT EXISTS idx_team_reputation_team ON team_reputation_cache(team_id);
CREATE INDEX IF NOT EXISTS idx_team_reputation_season ON team_reputation_cache(save_id, season_id);

-- 球队性格查询索引
CREATE INDEX IF NOT EXISTS idx_team_personality_save ON team_personality_configs(save_id);

-- 冷却期查询索引
CREATE INDEX IF NOT EXISTS idx_player_cooldowns_window ON player_cooldowns(window_id);
CREATE INDEX IF NOT EXISTS idx_player_cooldowns_player ON player_cooldowns(player_id, team_id);
