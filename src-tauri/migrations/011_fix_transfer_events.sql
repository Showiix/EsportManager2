-- ============================================
-- 修复转会事件表结构
-- 版本: 011
-- 日期: 2026-02-09
-- 说明: 重建 transfer_events 表以匹配最新代码结构
-- ============================================

-- 1. 备份现有数据（如果表存在且有数据）
CREATE TABLE IF NOT EXISTS transfer_events_backup AS
SELECT * FROM transfer_events WHERE 1=0;

-- 2. 删除旧表
DROP TABLE IF EXISTS transfer_events;

-- 3. 创建新表（与 010_transfer_system.sql 保持一致）
CREATE TABLE transfer_events (
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

-- 4. 创建索引
CREATE INDEX IF NOT EXISTS idx_transfer_events_window_id ON transfer_events(window_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_player_id ON transfer_events(player_id);

-- 5. 清理备份表
DROP TABLE IF EXISTS transfer_events_backup;
