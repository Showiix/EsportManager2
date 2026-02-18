-- 选手赛季统计表
-- 用于数据中心统计选手每赛季的表现数据

CREATE TABLE IF NOT EXISTS player_season_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER,
    region_id TEXT,
    position TEXT NOT NULL,

    -- 比赛统计
    matches_played INTEGER NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0,

    -- 影响力统计
    total_impact REAL NOT NULL DEFAULT 0.0,
    avg_impact REAL NOT NULL DEFAULT 0.0,

    -- 发挥统计
    avg_performance REAL NOT NULL DEFAULT 0.0,
    best_performance REAL NOT NULL DEFAULT 0.0,
    worst_performance REAL NOT NULL DEFAULT 100.0,

    -- 稳定性评分
    consistency_score REAL NOT NULL DEFAULT 100.0,

    -- 冠军加成
    international_titles INTEGER NOT NULL DEFAULT 0,
    regional_titles INTEGER NOT NULL DEFAULT 0,
    champion_bonus REAL NOT NULL DEFAULT 0.0,

    -- 年度Top得分
    yearly_top_score REAL NOT NULL DEFAULT 0.0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
    UNIQUE(save_id, player_id, season_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_player_stats_save_season ON player_season_stats(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_player_stats_player ON player_season_stats(player_id);
CREATE INDEX IF NOT EXISTS idx_player_stats_team ON player_season_stats(team_id);
CREATE INDEX IF NOT EXISTS idx_player_stats_position ON player_season_stats(position);
CREATE INDEX IF NOT EXISTS idx_player_stats_yearly_score ON player_season_stats(yearly_top_score DESC);
CREATE INDEX IF NOT EXISTS idx_player_stats_avg_impact ON player_season_stats(avg_impact DESC);
