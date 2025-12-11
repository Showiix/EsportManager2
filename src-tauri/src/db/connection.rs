use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 数据库连接管理器
pub struct DatabaseManager {
    /// SQLite连接池
    pool: Arc<RwLock<Option<Pool<Sqlite>>>>,
    /// 数据库文件路径
    db_path: PathBuf,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            db_path,
        }
    }

    /// 初始化数据库连接
    pub async fn init(&self) -> Result<(), DatabaseError> {
        // 确保目录存在
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::Io(e.to_string()))?;
        }

        // 构建连接字符串
        let db_url = format!("sqlite:{}?mode=rwc", self.db_path.display());

        // 创建连接池
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        // 运行迁移
        self.run_migrations(&pool).await?;

        // 存储连接池
        let mut guard = self.pool.write().await;
        *guard = Some(pool);

        Ok(())
    }

    /// 获取连接池
    pub async fn get_pool(&self) -> Result<Pool<Sqlite>, DatabaseError> {
        let guard = self.pool.read().await;
        guard.clone().ok_or(DatabaseError::NotInitialized)
    }

    /// 关闭数据库连接
    pub async fn close(&self) {
        let mut guard = self.pool.write().await;
        if let Some(pool) = guard.take() {
            pool.close().await;
        }
    }

    /// 运行数据库迁移
    async fn run_migrations(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 创建基础表结构
        sqlx::query(SCHEMA_SQL)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        Ok(())
    }
}

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database not initialized")]
    NotInitialized,

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// 数据库模式定义
const SCHEMA_SQL: &str = r#"
-- 存档表
CREATE TABLE IF NOT EXISTS saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    current_season INTEGER NOT NULL DEFAULT 1,
    current_phase TEXT NOT NULL DEFAULT 'SpringRegular',
    phase_completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 赛区表
CREATE TABLE IF NOT EXISTS regions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    name TEXT NOT NULL,
    short_name TEXT NOT NULL,
    team_count INTEGER NOT NULL DEFAULT 14,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- 队伍表
CREATE TABLE IF NOT EXISTS teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    region_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    short_name TEXT,
    power_rating REAL NOT NULL DEFAULT 70.0,
    total_matches INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    win_rate REAL NOT NULL DEFAULT 0.0,
    annual_points INTEGER NOT NULL DEFAULT 0,
    cross_year_points INTEGER NOT NULL DEFAULT 0,
    balance INTEGER NOT NULL DEFAULT 5000000,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id)
);

-- 选手表
CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    real_name TEXT,
    nationality TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    stability INTEGER NOT NULL,
    tag TEXT NOT NULL DEFAULT 'Regular',
    status TEXT NOT NULL DEFAULT 'Active',
    position TEXT NOT NULL,
    team_id INTEGER,
    salary INTEGER NOT NULL DEFAULT 0,
    market_value INTEGER NOT NULL DEFAULT 0,
    contract_end_season INTEGER,
    join_season INTEGER,
    retire_season INTEGER,
    is_starter INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 赛事表
CREATE TABLE IF NOT EXISTS tournaments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER,
    name TEXT NOT NULL,
    tournament_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Scheduled',
    start_date TEXT,
    end_date TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- 比赛表
CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    tournament_id INTEGER NOT NULL,
    stage TEXT NOT NULL,
    round INTEGER,
    match_order INTEGER,
    format TEXT NOT NULL DEFAULT 'Bo3',
    home_team_id INTEGER NOT NULL,
    away_team_id INTEGER NOT NULL,
    home_score INTEGER NOT NULL DEFAULT 0,
    away_score INTEGER NOT NULL DEFAULT 0,
    winner_id INTEGER,
    status TEXT NOT NULL DEFAULT 'Scheduled',
    played_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (home_team_id) REFERENCES teams(id),
    FOREIGN KEY (away_team_id) REFERENCES teams(id)
);

-- 联赛积分榜表
CREATE TABLE IF NOT EXISTS league_standings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    tournament_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    rank INTEGER,
    matches_played INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    points INTEGER NOT NULL DEFAULT 0,
    games_won INTEGER NOT NULL DEFAULT 0,
    games_lost INTEGER NOT NULL DEFAULT 0,
    game_diff INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (team_id) REFERENCES teams(id),
    UNIQUE(tournament_id, team_id)
);

-- 年度积分明细表
CREATE TABLE IF NOT EXISTS annual_points_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    points INTEGER NOT NULL,
    final_rank INTEGER,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id)
);

-- 全球排名表
CREATE TABLE IF NOT EXISTS global_rankings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    global_rank INTEGER NOT NULL,
    total_points INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    UNIQUE(save_id, season_id, team_id)
);

-- 选秀球员表
CREATE TABLE IF NOT EXISTS draft_players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,
    game_id TEXT NOT NULL,
    real_name TEXT,
    nationality TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    position TEXT NOT NULL,
    tag TEXT NOT NULL DEFAULT 'Rookie',
    draft_rank INTEGER NOT NULL,
    is_picked INTEGER NOT NULL DEFAULT 0,
    picked_by_team_id INTEGER,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- 选秀顺位表
CREATE TABLE IF NOT EXISTS draft_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    summer_rank INTEGER NOT NULL,
    draft_position INTEGER NOT NULL,
    lottery_result TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 选秀结果表
CREATE TABLE IF NOT EXISTS draft_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,
    draft_player_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    pick_number INTEGER NOT NULL,
    player_id INTEGER,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (draft_player_id) REFERENCES draft_players(id),
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 转会记录表
CREATE TABLE IF NOT EXISTS transfer_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    from_team_id INTEGER,
    to_team_id INTEGER,
    transfer_type TEXT NOT NULL,
    transfer_fee INTEGER,
    salary INTEGER,
    contract_years INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- 转会市场挂牌表
CREATE TABLE IF NOT EXISTS transfer_listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    selling_team_id INTEGER NOT NULL,
    asking_price INTEGER NOT NULL,
    listed_at INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (selling_team_id) REFERENCES teams(id)
);

-- 自由球员表
CREATE TABLE IF NOT EXISTS free_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    expected_salary INTEGER NOT NULL,
    available_from INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- 财务交易记录表
CREATE TABLE IF NOT EXISTS financial_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL,
    amount INTEGER NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 赛季财务报告表
CREATE TABLE IF NOT EXISTS team_season_finances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    salary_expense INTEGER NOT NULL DEFAULT 0,
    prize_income INTEGER NOT NULL DEFAULT 0,
    sponsorship_income INTEGER NOT NULL DEFAULT 0,
    transfer_income INTEGER NOT NULL DEFAULT 0,
    operating_cost INTEGER NOT NULL DEFAULT 0,
    net_profit INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    UNIQUE(save_id, season_id, team_id)
);

-- 荣誉记录表
CREATE TABLE IF NOT EXISTS honors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    player_id INTEGER,
    tournament_id INTEGER NOT NULL,
    honor_type TEXT NOT NULL,
    description TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_teams_region ON teams(region_id);
CREATE INDEX IF NOT EXISTS idx_teams_save ON teams(save_id);
CREATE INDEX IF NOT EXISTS idx_players_team ON players(team_id);
CREATE INDEX IF NOT EXISTS idx_players_save ON players(save_id);
CREATE INDEX IF NOT EXISTS idx_matches_tournament ON matches(tournament_id);
CREATE INDEX IF NOT EXISTS idx_matches_save ON matches(save_id);
CREATE INDEX IF NOT EXISTS idx_standings_tournament ON league_standings(tournament_id);
CREATE INDEX IF NOT EXISTS idx_global_rankings_season ON global_rankings(season_id);
CREATE INDEX IF NOT EXISTS idx_draft_players_season ON draft_players(season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_records_season ON transfer_records(season_id);
CREATE INDEX IF NOT EXISTS idx_honors_team ON honors(team_id);
"#;
