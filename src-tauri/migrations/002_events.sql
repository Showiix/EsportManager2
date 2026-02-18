-- 事件系统数据库迁移
-- 添加游戏事件记录表

-- 游戏事件表
CREATE TABLE IF NOT EXISTS game_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    player_id INTEGER,
    team_id INTEGER,
    description TEXT NOT NULL,
    details TEXT,
    phase TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE SET NULL,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 自由球员表
CREATE TABLE IF NOT EXISTS free_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    salary_demand INTEGER NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'AVAILABLE',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- 转会市场挂牌表
CREATE TABLE IF NOT EXISTS transfer_listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    listing_type TEXT NOT NULL DEFAULT 'FOR_SALE',
    asking_price INTEGER NOT NULL,
    min_price INTEGER,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
);

-- 选秀池表
CREATE TABLE IF NOT EXISTS draft_pool (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    position TEXT NOT NULL,
    tag TEXT NOT NULL,
    is_picked INTEGER NOT NULL DEFAULT 0,
    picked_by_team_id INTEGER,
    pick_order INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE,
    FOREIGN KEY (picked_by_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 选秀顺位表
CREATE TABLE IF NOT EXISTS draft_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    pick_order INTEGER NOT NULL,
    original_rank INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id, team_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_game_events_save_season ON game_events(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_game_events_type ON game_events(event_type);
CREATE INDEX IF NOT EXISTS idx_game_events_player ON game_events(player_id);
CREATE INDEX IF NOT EXISTS idx_free_agents_save_season ON free_agents(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_free_agents_status ON free_agents(status);
CREATE INDEX IF NOT EXISTS idx_transfer_listings_status ON transfer_listings(status);
CREATE INDEX IF NOT EXISTS idx_draft_pool_save_season ON draft_pool(save_id, season_id);
