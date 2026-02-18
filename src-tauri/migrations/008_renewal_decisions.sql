-- 续约决策持久化表
-- 存储每个赛季的续约结果，以便刷新后仍可查看

CREATE TABLE IF NOT EXISTS renewal_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 选手信息
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,

    -- 球队信息
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,

    -- 球队决策
    team_wants_renewal INTEGER NOT NULL DEFAULT 1,
    team_rejection_reason TEXT,

    -- 报价条件
    offered_salary INTEGER NOT NULL DEFAULT 0,  -- 万/年
    offered_years INTEGER NOT NULL DEFAULT 1,

    -- 选手决策
    player_accepts INTEGER NOT NULL DEFAULT 1,
    player_rejection_reason TEXT,

    -- 最终结果
    renewal_successful INTEGER NOT NULL DEFAULT 0,
    final_salary INTEGER,  -- 万/年
    final_years INTEGER,

    -- AI 分析（JSON）
    team_analysis TEXT DEFAULT '[]',
    player_analysis TEXT DEFAULT '[]',
    summary TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_renewal_decisions_save_season ON renewal_decisions(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_renewal_decisions_player ON renewal_decisions(player_id);
CREATE INDEX IF NOT EXISTS idx_renewal_decisions_team ON renewal_decisions(team_id);
CREATE INDEX IF NOT EXISTS idx_renewal_decisions_result ON renewal_decisions(renewal_successful);
