-- 荣誉引擎数据库迁移
-- 更新荣誉表以支持完整的荣誉系统

-- 删除旧的荣誉表（如果存在）
DROP TABLE IF EXISTS player_honors;
DROP TABLE IF EXISTS honors;

-- 创建新的荣誉表
CREATE TABLE IF NOT EXISTS honors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    honor_type TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    tournament_name TEXT NOT NULL,
    tournament_type TEXT NOT NULL,

    -- 获得者（战队或选手）
    team_id INTEGER,
    team_name TEXT,
    player_id INTEGER,
    player_name TEXT,
    position TEXT,

    -- MVP统计数据（JSON格式）
    stats_json TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE SET NULL
);

-- 创建索引优化查询
CREATE INDEX IF NOT EXISTS idx_honors_save ON honors(save_id);
CREATE INDEX IF NOT EXISTS idx_honors_team ON honors(team_id);
CREATE INDEX IF NOT EXISTS idx_honors_player ON honors(player_id);
CREATE INDEX IF NOT EXISTS idx_honors_season ON honors(season_id);
CREATE INDEX IF NOT EXISTS idx_honors_tournament ON honors(tournament_id);
CREATE INDEX IF NOT EXISTS idx_honors_type ON honors(honor_type);
CREATE INDEX IF NOT EXISTS idx_honors_tournament_type ON honors(tournament_type);

-- 复合索引：按赛季和类型查询
CREATE INDEX IF NOT EXISTS idx_honors_season_type ON honors(season_id, honor_type);

-- 复合索引：按战队和类型查询
CREATE INDEX IF NOT EXISTS idx_honors_team_type ON honors(team_id, honor_type);

-- 复合索引：按选手和类型查询
CREATE INDEX IF NOT EXISTS idx_honors_player_type ON honors(player_id, honor_type);
