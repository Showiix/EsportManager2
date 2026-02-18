-- LLM 任务日志表
-- 追踪所有 LLM 调用的状态，支持并发执行和失败重试

CREATE TABLE IF NOT EXISTS llm_task_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,

    -- 任务标识
    task_type TEXT NOT NULL,  -- 'intention' | 'strategy' | 'renewal' | 'free_market' | 'poaching'
    entity_id INTEGER NOT NULL,  -- player_id 或 team_id
    entity_type TEXT NOT NULL,  -- 'player' | 'team'

    -- 任务状态
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'running' | 'success' | 'failed'
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    -- 错误信息
    error_msg TEXT,
    last_error_at TEXT,

    -- 时间戳
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,

    -- 唯一约束：同一存档/赛季/任务类型/实体只能有一条记录
    UNIQUE(save_id, season_id, task_type, entity_id)
);

-- 索引：快速查询失败任务
CREATE INDEX IF NOT EXISTS idx_llm_task_status ON llm_task_log(save_id, season_id, status);
CREATE INDEX IF NOT EXISTS idx_llm_task_type ON llm_task_log(save_id, season_id, task_type);
CREATE INDEX IF NOT EXISTS idx_llm_task_entity ON llm_task_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_llm_task_updated ON llm_task_log(updated_at);
