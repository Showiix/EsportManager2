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

        // 运行增量迁移
        self.run_incremental_migrations(pool).await?;

        Ok(())
    }

    /// 运行增量迁移
    async fn run_incremental_migrations(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 迁移1: 为 financial_transactions 表添加 related_tournament_id 和 related_player_id 字段
        // SQLite 不支持 ADD COLUMN IF NOT EXISTS，所以需要先检查
        let columns: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM pragma_table_info('financial_transactions')"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        let column_names: Vec<&str> = columns.iter().map(|c| c.0.as_str()).collect();

        if !column_names.contains(&"related_tournament_id") {
            sqlx::query("ALTER TABLE financial_transactions ADD COLUMN related_tournament_id INTEGER")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !column_names.contains(&"related_player_id") {
            sqlx::query("ALTER TABLE financial_transactions ADD COLUMN related_player_id INTEGER")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 迁移2: 为 players 表添加 loyalty 字段
        let player_columns: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM pragma_table_info('players')"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        let player_column_names: Vec<&str> = player_columns.iter().map(|c| c.0.as_str()).collect();

        if !player_column_names.contains(&"loyalty") {
            sqlx::query("ALTER TABLE players ADD COLUMN loyalty INTEGER NOT NULL DEFAULT 50")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 迁移: 为 players 表添加 calculated_market_value 字段
        if !player_column_names.contains(&"calculated_market_value") {
            sqlx::query("ALTER TABLE players ADD COLUMN calculated_market_value INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 迁移: 为 players 表添加 home_region_id 和 region_loyalty 字段（跨赛区转会偏好）
        if !player_column_names.contains(&"home_region_id") {
            sqlx::query("ALTER TABLE players ADD COLUMN home_region_id INTEGER")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 初始化现有选手的 home_region_id（根据当前球队所属赛区）
            sqlx::query(r#"
                UPDATE players
                SET home_region_id = (
                    SELECT t.region_id FROM teams t WHERE t.id = players.team_id
                )
                WHERE team_id IS NOT NULL AND home_region_id IS NULL
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !player_column_names.contains(&"region_loyalty") {
            sqlx::query("ALTER TABLE players ADD COLUMN region_loyalty INTEGER NOT NULL DEFAULT 70")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 根据赛区设置默认 region_loyalty
            // LPL (region_id=1): 80, LCK (region_id=2): 65, LEC (region_id=3): 55, LCS (region_id=4): 50
            sqlx::query(r#"
                UPDATE players SET region_loyalty =
                    CASE
                        WHEN home_region_id = 1 THEN 75 + ABS(RANDOM() % 16)  -- LPL: 75-90
                        WHEN home_region_id = 2 THEN 55 + ABS(RANDOM() % 21)  -- LCK: 55-75
                        WHEN home_region_id = 3 THEN 45 + ABS(RANDOM() % 21)  -- LEC: 45-65
                        WHEN home_region_id = 4 THEN 40 + ABS(RANDOM() % 21)  -- LCS: 40-60
                        ELSE 60
                    END
                WHERE home_region_id IS NOT NULL
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 迁移3: 运行 005_player_satisfaction.sql 的表创建
        self.run_satisfaction_tables_migration(pool).await?;

        // 迁移4: 运行 006_draft_auction.sql 的表创建
        self.run_draft_auction_tables_migration(pool).await?;

        // 迁移5: 运行 007_transfer_market.sql 的表创建
        self.run_transfer_market_tables_migration(pool).await?;

        // 迁移6: 运行 009_llm_task_log.sql 的表创建
        self.run_llm_task_log_migration(pool).await?;

        // 迁移7: 运行 010_transfer_system.sql 的表创建
        self.run_transfer_system_migration(pool).await?;

        Ok(())
    }

    /// 运行满意度系统相关表的迁移
    async fn run_satisfaction_tables_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='player_season_status'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建选手赛季状态表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS player_season_status (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    satisfaction INTEGER NOT NULL DEFAULT 70,
                    wants_to_leave BOOLEAN NOT NULL DEFAULT FALSE,
                    departure_reasons TEXT DEFAULT '[]',
                    games_as_starter INTEGER DEFAULT 0,
                    total_games INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(save_id, season_id, player_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球队赛季表现表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_season_performance (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    team_id INTEGER NOT NULL,
                    final_rank INTEGER,
                    made_playoffs BOOLEAN DEFAULT FALSE,
                    playoff_result TEXT,
                    international_result TEXT,
                    consecutive_no_playoffs INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(save_id, season_id, team_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建忠诚度变化记录表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS loyalty_changes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    change_amount INTEGER NOT NULL,
                    reason TEXT NOT NULL,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_season_status_lookup ON player_season_status(save_id, season_id, player_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_season_status_departure ON player_season_status(save_id, season_id, wants_to_leave)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_season_performance_lookup ON team_season_performance(save_id, season_id, team_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_loyalty_changes_player ON loyalty_changes(save_id, player_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        Ok(())
    }

    /// 运行选秀权拍卖系统相关表的迁移
    async fn run_draft_auction_tables_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='draft_pick_auctions'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建选秀权拍卖主表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS draft_pick_auctions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    region_id INTEGER NOT NULL,
                    status TEXT NOT NULL DEFAULT 'PREPARING',
                    current_round INTEGER NOT NULL DEFAULT 0,
                    total_rounds INTEGER NOT NULL DEFAULT 3,
                    total_auctions INTEGER NOT NULL DEFAULT 0,
                    successful_auctions INTEGER NOT NULL DEFAULT 0,
                    total_revenue INTEGER NOT NULL DEFAULT 0,
                    total_commission INTEGER NOT NULL DEFAULT 0,
                    started_at TEXT,
                    completed_at TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(save_id, season_id, region_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建选秀权挂牌表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS draft_pick_listings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    region_id INTEGER NOT NULL,
                    auction_id INTEGER NOT NULL,
                    seller_team_id INTEGER NOT NULL,
                    seller_team_name TEXT NOT NULL,
                    draft_position INTEGER NOT NULL,
                    starting_price INTEGER NOT NULL,
                    current_price INTEGER NOT NULL,
                    min_increment INTEGER NOT NULL,
                    status TEXT NOT NULL DEFAULT 'PENDING',
                    buyer_team_id INTEGER,
                    buyer_team_name TEXT,
                    final_price INTEGER,
                    commission_fee INTEGER,
                    seller_revenue INTEGER,
                    current_bid_round INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    sold_at TEXT,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (auction_id) REFERENCES draft_pick_auctions(id) ON DELETE CASCADE,
                    FOREIGN KEY (seller_team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (buyer_team_id) REFERENCES teams(id) ON DELETE SET NULL
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建竞拍出价记录表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS draft_pick_bids (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    listing_id INTEGER NOT NULL,
                    bidder_team_id INTEGER NOT NULL,
                    bidder_team_name TEXT NOT NULL,
                    bid_amount INTEGER NOT NULL,
                    bid_round INTEGER NOT NULL,
                    status TEXT NOT NULL DEFAULT 'ACTIVE',
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (listing_id) REFERENCES draft_pick_listings(id) ON DELETE CASCADE,
                    FOREIGN KEY (bidder_team_id) REFERENCES teams(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建拍卖事件表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS draft_pick_auction_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    auction_id INTEGER NOT NULL,
                    listing_id INTEGER,
                    event_type TEXT NOT NULL,
                    team_id INTEGER,
                    team_name TEXT,
                    draft_position INTEGER,
                    amount INTEGER,
                    headline TEXT NOT NULL,
                    description TEXT NOT NULL,
                    importance TEXT NOT NULL DEFAULT 'NORMAL',
                    round INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (auction_id) REFERENCES draft_pick_auctions(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_draft_pick_auctions_save_season_region ON draft_pick_auctions(save_id, season_id, region_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_draft_pick_listings_auction ON draft_pick_listings(auction_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_draft_pick_listings_status ON draft_pick_listings(status)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_draft_pick_bids_listing ON draft_pick_bids(listing_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_draft_pick_auction_events_auction ON draft_pick_auction_events(auction_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 检查 draft_orders 表是否需要添加新字段
        let order_columns: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM pragma_table_info('draft_orders')"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        let order_column_names: Vec<&str> = order_columns.iter().map(|c| c.0.as_str()).collect();

        if !order_column_names.contains(&"original_team_id") {
            sqlx::query("ALTER TABLE draft_orders ADD COLUMN original_team_id INTEGER")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !order_column_names.contains(&"acquired_via") {
            sqlx::query("ALTER TABLE draft_orders ADD COLUMN acquired_via TEXT DEFAULT 'LOTTERY'")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !order_column_names.contains(&"acquisition_price") {
            sqlx::query("ALTER TABLE draft_orders ADD COLUMN acquisition_price INTEGER DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 迁移：创建双向评估系统相关表
        self.run_evaluation_tables_migration(pool).await?;

        Ok(())
    }

    /// 运行双向评估系统表的迁移
    async fn run_evaluation_tables_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查 team_season_evaluations 表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='team_season_evaluations'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建战队赛季评估表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_season_evaluations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    window_id INTEGER NOT NULL,
                    team_id INTEGER NOT NULL,
                    team_name TEXT NOT NULL,
                    season_id INTEGER NOT NULL,

                    -- 战绩评估
                    current_rank INTEGER,
                    last_season_rank INTEGER,
                    rank_trend TEXT,
                    rank_change INTEGER,

                    -- 阵容评估
                    roster_power REAL,
                    roster_age_avg REAL,
                    roster_salary_total INTEGER,
                    budget_remaining INTEGER,
                    roster_count INTEGER,

                    -- 评估结论
                    stability_score INTEGER,
                    urgency_level TEXT,
                    strategy TEXT,
                    strategy_reason TEXT,

                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (window_id) REFERENCES transfer_windows(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建位置需求表（买人列表）
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_position_needs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    evaluation_id INTEGER NOT NULL,
                    position TEXT NOT NULL,

                    -- 当前状况
                    current_starter_id INTEGER,
                    current_starter_name TEXT,
                    current_starter_ability INTEGER,
                    current_starter_age INTEGER,

                    -- 需求描述
                    need_level TEXT,
                    min_ability_target INTEGER,
                    max_salary_budget INTEGER,
                    prefer_young INTEGER,
                    reason TEXT,

                    FOREIGN KEY (evaluation_id) REFERENCES team_season_evaluations(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建挂牌评估表（卖人列表）
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_listing_evaluations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    evaluation_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    player_name TEXT NOT NULL,
                    position TEXT,

                    -- 选手状况
                    ability INTEGER,
                    age INTEGER,
                    salary INTEGER,

                    -- 保护因素
                    has_recent_honor INTEGER,
                    honor_details TEXT,
                    season_influence_rank INTEGER,

                    -- 挂牌决策
                    should_list INTEGER,
                    list_reason TEXT,
                    protect_reason TEXT,
                    suggested_price INTEGER,

                    FOREIGN KEY (evaluation_id) REFERENCES team_season_evaluations(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建选手赛季评估表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS player_season_evaluations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    window_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    player_name TEXT NOT NULL,
                    team_id INTEGER NOT NULL,
                    team_name TEXT NOT NULL,

                    -- 选手属性
                    ability INTEGER,
                    age INTEGER,
                    salary INTEGER,
                    satisfaction INTEGER,
                    loyalty INTEGER,

                    -- 评估因素得分
                    team_rank_score REAL,
                    team_trend_score REAL,
                    teammate_score REAL,
                    salary_score REAL,
                    honor_score REAL,
                    satisfaction_score REAL,

                    -- 评估结论
                    stay_score REAL,
                    wants_to_leave INTEGER,
                    leave_reason TEXT,

                    -- 市场估值
                    estimated_market_salary INTEGER,
                    salary_gap INTEGER,

                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (window_id) REFERENCES transfer_windows(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_season_evaluations_save ON team_season_evaluations(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_season_evaluations_team ON team_season_evaluations(team_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_position_needs_evaluation ON team_position_needs(evaluation_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_listing_evaluations_evaluation ON team_listing_evaluations(evaluation_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_season_evaluations_save ON player_season_evaluations(save_id, window_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_season_evaluations_player ON player_season_evaluations(player_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_season_evaluations_team ON player_season_evaluations(team_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        Ok(())
    }

    /// 运行 LLM 转会市场系统相关表的迁移
    async fn run_transfer_market_tables_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查 transfer_market_states 表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='transfer_market_states'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建转会市场整体状态表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS transfer_market_states (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    current_phase TEXT NOT NULL DEFAULT 'INITIALIZATION',
                    current_round INTEGER NOT NULL DEFAULT 0,
                    max_negotiation_rounds INTEGER NOT NULL DEFAULT 5,
                    free_agent_ids TEXT DEFAULT '[]',
                    active_negotiation_ids TEXT DEFAULT '[]',
                    completed_transfer_ids TEXT DEFAULT '[]',
                    intentions_generated INTEGER NOT NULL DEFAULT 0,
                    total_players INTEGER NOT NULL DEFAULT 0,
                    strategies_generated INTEGER NOT NULL DEFAULT 0,
                    total_teams INTEGER NOT NULL DEFAULT 0,
                    is_market_stable INTEGER NOT NULL DEFAULT 0,
                    stable_rounds_count INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(save_id, season_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球队市场状态表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_market_states (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    team_id INTEGER NOT NULL,
                    team_name TEXT NOT NULL,
                    initial_balance INTEGER NOT NULL DEFAULT 0,
                    remaining_budget INTEGER NOT NULL DEFAULT 0,
                    spent_amount INTEGER NOT NULL DEFAULT 0,
                    salary_increase INTEGER NOT NULL DEFAULT 0,
                    roster_count INTEGER NOT NULL DEFAULT 0,
                    min_roster_size INTEGER NOT NULL DEFAULT 5,
                    max_roster_size INTEGER NOT NULL DEFAULT 10,
                    pending_negotiation_ids TEXT DEFAULT '[]',
                    completed_signing_ids TEXT DEFAULT '[]',
                    departed_player_ids TEXT DEFAULT '[]',
                    strategy_generated INTEGER NOT NULL DEFAULT 0,
                    strategy_id INTEGER,
                    needs_emergency_signing INTEGER NOT NULL DEFAULT 0,
                    position_needs TEXT DEFAULT '{}',
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    UNIQUE(save_id, season_id, team_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建谈判记录表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS negotiations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    player_name TEXT NOT NULL,
                    player_position TEXT,
                    player_ability INTEGER,
                    from_team_id INTEGER,
                    from_team_name TEXT,
                    status TEXT NOT NULL DEFAULT 'OPEN',
                    current_round INTEGER NOT NULL DEFAULT 0,
                    max_rounds INTEGER NOT NULL DEFAULT 5,
                    competing_team_ids TEXT DEFAULT '[]',
                    final_team_id INTEGER,
                    final_team_name TEXT,
                    final_salary INTEGER,
                    final_years INTEGER,
                    final_starter INTEGER,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
                    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE SET NULL,
                    FOREIGN KEY (final_team_id) REFERENCES teams(id) ON DELETE SET NULL
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建报价表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS offers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    negotiation_id INTEGER NOT NULL,
                    from_team_id INTEGER NOT NULL,
                    from_team_name TEXT NOT NULL,
                    to_player_id INTEGER NOT NULL,
                    round INTEGER NOT NULL,
                    salary_offer INTEGER NOT NULL DEFAULT 0,
                    contract_years INTEGER NOT NULL DEFAULT 1,
                    guarantee_starter INTEGER NOT NULL DEFAULT 0,
                    signing_bonus INTEGER NOT NULL DEFAULT 0,
                    transfer_fee INTEGER NOT NULL DEFAULT 0,
                    status TEXT NOT NULL DEFAULT 'PENDING',
                    offer_reasoning TEXT,
                    analysis_steps TEXT DEFAULT '[]',
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (negotiation_id) REFERENCES negotiations(id) ON DELETE CASCADE,
                    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (to_player_id) REFERENCES players(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建报价回应表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS offer_responses (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    offer_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    response_type TEXT NOT NULL,
                    counter_salary INTEGER,
                    counter_years INTEGER,
                    counter_starter INTEGER,
                    reasoning TEXT,
                    analysis_steps TEXT DEFAULT '[]',
                    responded_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建市场事件表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS transfer_market_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    event_type TEXT NOT NULL,
                    phase TEXT NOT NULL,
                    round INTEGER NOT NULL DEFAULT 0,
                    player_id INTEGER,
                    player_name TEXT,
                    team_id INTEGER,
                    team_name TEXT,
                    secondary_team_id INTEGER,
                    secondary_team_name TEXT,
                    amount INTEGER,
                    title TEXT NOT NULL,
                    description TEXT,
                    ai_analysis TEXT,
                    importance INTEGER NOT NULL DEFAULT 2,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE SET NULL,
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
                    FOREIGN KEY (secondary_team_id) REFERENCES teams(id) ON DELETE SET NULL
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球队转会策略表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS team_transfer_strategies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    team_id INTEGER NOT NULL,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    overall_strategy TEXT,
                    strategy_description TEXT,
                    reasoning TEXT,
                    targets TEXT DEFAULT '[]',
                    willing_to_sell TEXT DEFAULT '[]',
                    priority_positions TEXT DEFAULT '[]',
                    total_budget INTEGER NOT NULL DEFAULT 0,
                    transfer_spend INTEGER NOT NULL DEFAULT 0,
                    salary_spend INTEGER NOT NULL DEFAULT 0,
                    reserve INTEGER NOT NULL DEFAULT 0,
                    analysis_steps TEXT DEFAULT '[]',
                    is_mock INTEGER NOT NULL DEFAULT 1,
                    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(team_id, save_id, season_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_market_states_save ON transfer_market_states(save_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_market_save_season ON team_market_states(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_negotiations_save_season ON negotiations(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_negotiations_status ON negotiations(status)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_offers_negotiation ON offers(negotiation_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_market_events_save_season ON transfer_market_events(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_strategies_save_season ON team_transfer_strategies(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建 AI 球队策略表（简化版，存储 JSON）
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS ai_transfer_strategies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    team_id INTEGER NOT NULL,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    strategy_json TEXT NOT NULL,
                    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(team_id, save_id, season_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_strategies_save_season ON ai_transfer_strategies(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 确保 ai_transfer_strategies 表存在（即使其他表已存在）
        let ai_strategy_tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='ai_transfer_strategies'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if ai_strategy_tables.is_empty() {
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS ai_transfer_strategies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    team_id INTEGER NOT NULL,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    strategy_json TEXT NOT NULL,
                    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(team_id, save_id, season_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_strategies_save_season ON ai_transfer_strategies(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 检查 player_transfer_strategies 表是否需要扩展字段
        let strategy_columns: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM pragma_table_info('player_transfer_strategies')"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        let strategy_column_names: Vec<&str> = strategy_columns.iter().map(|c| c.0.as_str()).collect();

        // 添加新字段（如果不存在）
        if !strategy_column_names.contains(&"wants_to_leave") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN wants_to_leave INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"decision_confidence") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN decision_confidence INTEGER NOT NULL DEFAULT 50")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"departure_reasons") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN departure_reasons TEXT DEFAULT '[]'")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"leave_reasoning") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN leave_reasoning TEXT")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"preferred_teams") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN preferred_teams TEXT DEFAULT '[]'")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"team_preference_reasoning") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN team_preference_reasoning TEXT")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"expected_salary") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN expected_salary INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"expected_min_salary") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN expected_min_salary INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"expected_years") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN expected_years INTEGER NOT NULL DEFAULT 1")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"requires_starter") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN requires_starter INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"analysis_data") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN analysis_data TEXT")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"analysis_steps") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN analysis_steps TEXT DEFAULT '[]'")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        if !strategy_column_names.contains(&"is_mock") {
            sqlx::query("ALTER TABLE player_transfer_strategies ADD COLUMN is_mock INTEGER NOT NULL DEFAULT 1")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // 创建续约决策表（用于持久化续约结果）
        let renewal_tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='renewal_decisions'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if renewal_tables.is_empty() {
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS renewal_decisions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    player_name TEXT NOT NULL,
                    team_id INTEGER NOT NULL,
                    team_name TEXT NOT NULL,
                    team_wants_renewal INTEGER NOT NULL DEFAULT 1,
                    team_rejection_reason TEXT,
                    offered_salary INTEGER NOT NULL DEFAULT 0,
                    offered_years INTEGER NOT NULL DEFAULT 1,
                    player_accepts INTEGER NOT NULL DEFAULT 1,
                    player_rejection_reason TEXT,
                    renewal_successful INTEGER NOT NULL DEFAULT 0,
                    final_salary INTEGER,
                    final_years INTEGER,
                    team_analysis TEXT DEFAULT '[]',
                    player_analysis TEXT DEFAULT '[]',
                    summary TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
                    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_renewal_decisions_save_season ON renewal_decisions(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_renewal_decisions_result ON renewal_decisions(renewal_successful)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        Ok(())
    }

    /// 运行 LLM 任务日志表的迁移
    async fn run_llm_task_log_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='llm_task_log'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建 LLM 任务日志表
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS llm_task_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    task_type TEXT NOT NULL,
                    entity_id INTEGER NOT NULL,
                    entity_type TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'pending',
                    attempt_count INTEGER NOT NULL DEFAULT 0,
                    max_attempts INTEGER NOT NULL DEFAULT 3,
                    error_msg TEXT,
                    last_error_at TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    completed_at TEXT,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    UNIQUE(save_id, season_id, task_type, entity_id)
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_llm_task_status ON llm_task_log(save_id, season_id, status)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_llm_task_type ON llm_task_log(save_id, season_id, task_type)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_llm_task_entity ON llm_task_log(entity_type, entity_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_llm_task_updated ON llm_task_log(updated_at)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            log::info!("✅ LLM 任务日志表创建成功");
        }

        Ok(())
    }

    /// 运行转会系统表的迁移 (010_transfer_system.sql)
    async fn run_transfer_system_migration(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查 team_personality_configs 表是否已存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='team_personality_configs'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if tables.is_empty() {
            // 创建 AI 球队性格配置表
            sqlx::query(r#"
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
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球队声望缓存表
            sqlx::query(r#"
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
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球员挂牌表
            sqlx::query(r#"
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
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建球员冷却期记录表
            sqlx::query(r#"
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
                )
            "#)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_personality_save ON team_personality_configs(save_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_reputation_team ON team_reputation_cache(team_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_reputation_season ON team_reputation_cache(save_id, season_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_listings_window ON player_listings(window_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_listings_status ON player_listings(status)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_listings_player ON player_listings(player_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_cooldowns_window ON player_cooldowns(window_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            sqlx::query("CREATE INDEX IF NOT EXISTS idx_player_cooldowns_player ON player_cooldowns(player_id, team_id)")
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            log::info!("✅ 转会系统表创建成功");
        }

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
    calculated_market_value INTEGER NOT NULL DEFAULT 0,
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
    current_stage TEXT,
    current_round INTEGER,
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
    home_team_id INTEGER,
    away_team_id INTEGER,
    home_score INTEGER NOT NULL DEFAULT 0,
    away_score INTEGER NOT NULL DEFAULT 0,
    winner_id INTEGER,
    status TEXT NOT NULL DEFAULT 'Scheduled',
    played_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id)
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
CREATE TABLE IF NOT EXISTS annual_points_detail (
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
    team_id INTEGER NOT NULL,
    listing_type TEXT NOT NULL DEFAULT 'FOR_SALE',
    asking_price INTEGER NOT NULL,
    min_price INTEGER,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 自由球员表
CREATE TABLE IF NOT EXISTS free_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    salary_demand INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT 'CONTRACT_EXPIRE',
    status TEXT NOT NULL DEFAULT 'AVAILABLE',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
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

-- 荣誉记录表 (完整版)
CREATE TABLE IF NOT EXISTS honors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    honor_type TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER,
    tournament_type TEXT,
    tournament_name TEXT,
    team_id INTEGER,
    team_name TEXT,
    player_id INTEGER,
    player_name TEXT,
    position TEXT,
    stats_json TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id)
);

-- 赛事结果表 (记录冠亚季殿军)
CREATE TABLE IF NOT EXISTS tournament_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    tournament_type TEXT NOT NULL,
    tournament_name TEXT NOT NULL,
    champion_team_id INTEGER NOT NULL,
    champion_team_name TEXT NOT NULL,
    runner_up_team_id INTEGER NOT NULL,
    runner_up_team_name TEXT NOT NULL,
    third_team_id INTEGER,
    third_team_name TEXT,
    fourth_team_id INTEGER,
    fourth_team_name TEXT,
    final_match_id INTEGER,
    final_score TEXT,
    total_matches INTEGER,
    total_games INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    UNIQUE(save_id, tournament_id)
);

-- 选手赛事统计表 (用于MVP计算)
CREATE TABLE IF NOT EXISTS player_tournament_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    tournament_type TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,
    position TEXT NOT NULL,
    games_played INTEGER NOT NULL DEFAULT 0,
    games_won INTEGER NOT NULL DEFAULT 0,
    total_impact REAL NOT NULL DEFAULT 0.0,
    avg_impact REAL NOT NULL DEFAULT 0.0,
    max_impact REAL NOT NULL DEFAULT 0.0,
    avg_performance REAL NOT NULL DEFAULT 0.0,
    best_performance REAL NOT NULL DEFAULT 0.0,
    game_mvp_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (player_id) REFERENCES players(id),
    UNIQUE(save_id, tournament_id, player_id)
);

-- 选手状态因子表 (用于动态计算 condition)
CREATE TABLE IF NOT EXISTS player_form_factors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    form_cycle REAL NOT NULL DEFAULT 50.0,
    momentum INTEGER NOT NULL DEFAULT 0,
    last_performance REAL NOT NULL DEFAULT 0.0,
    last_match_won INTEGER NOT NULL DEFAULT 0,
    games_since_rest INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE(save_id, player_id)
);

-- 选手特性表
CREATE TABLE IF NOT EXISTS player_traits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    trait_type TEXT NOT NULL,
    acquired_season INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE(save_id, player_id, trait_type)
);

-- 选手赛季统计表 (数据中心)
-- 注意：player_id 不设置外键约束，因为统计数据可能来自不同来源
CREATE TABLE IF NOT EXISTS player_season_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER,
    region_id TEXT,
    position TEXT NOT NULL,
    matches_played INTEGER NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0,
    total_impact REAL NOT NULL DEFAULT 0.0,
    avg_impact REAL NOT NULL DEFAULT 0.0,
    avg_performance REAL NOT NULL DEFAULT 0.0,
    best_performance REAL NOT NULL DEFAULT 0.0,
    worst_performance REAL NOT NULL DEFAULT 100.0,
    consistency_score REAL NOT NULL DEFAULT 100.0,
    international_titles INTEGER NOT NULL DEFAULT 0,
    regional_titles INTEGER NOT NULL DEFAULT 0,
    champion_bonus REAL NOT NULL DEFAULT 0.0,
    yearly_top_score REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, player_id, season_id)
);

-- 比赛每局详情表
CREATE TABLE IF NOT EXISTS match_games (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    match_id INTEGER NOT NULL,
    game_number INTEGER NOT NULL,
    winner_team_id INTEGER NOT NULL,
    loser_team_id INTEGER NOT NULL,
    duration_minutes INTEGER,
    mvp_player_id INTEGER,
    key_player_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_team_id) REFERENCES teams(id),
    FOREIGN KEY (loser_team_id) REFERENCES teams(id),
    FOREIGN KEY (mvp_player_id) REFERENCES players(id),
    FOREIGN KEY (key_player_id) REFERENCES players(id)
);

-- 每局选手表现表
CREATE TABLE IF NOT EXISTS game_player_performances (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL DEFAULT '',
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL DEFAULT '',
    position TEXT NOT NULL,
    base_ability REAL NOT NULL DEFAULT 0.0,
    condition_bonus REAL NOT NULL DEFAULT 0.0,
    stability_noise REAL NOT NULL DEFAULT 0.0,
    actual_ability REAL NOT NULL,
    impact_score REAL NOT NULL,
    mvp_score REAL NOT NULL DEFAULT 0.0,
    is_mvp INTEGER NOT NULL DEFAULT 0,
    is_key_player INTEGER NOT NULL DEFAULT 0,
    kills INTEGER,
    deaths INTEGER,
    assists INTEGER,
    cs INTEGER,
    gold INTEGER,
    damage_dealt INTEGER,
    damage_taken INTEGER,
    vision_score INTEGER,
    traits_json TEXT,
    activated_traits_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (game_id) REFERENCES match_games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (team_id) REFERENCES teams(id)
);

-- 转会窗口表
CREATE TABLE IF NOT EXISTS transfer_windows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL DEFAULT 5,
    total_transfers INTEGER NOT NULL DEFAULT 0,
    total_fees INTEGER NOT NULL DEFAULT 0,
    free_agents_signed INTEGER NOT NULL DEFAULT 0,
    retirements INTEGER NOT NULL DEFAULT 0,
    contract_expires INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id)
);

-- 转会事件表
CREATE TABLE IF NOT EXISTS transfer_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    round INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    status TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    position TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    market_value INTEGER NOT NULL,
    from_team_id INTEGER,
    from_team_name TEXT,
    to_team_id INTEGER,
    to_team_name TEXT,
    transfer_fee INTEGER NOT NULL DEFAULT 0,
    new_salary INTEGER,
    contract_years INTEGER,
    contract_type TEXT NOT NULL DEFAULT 'RENEWAL',
    price_ratio REAL,
    headline TEXT NOT NULL,
    description TEXT NOT NULL,
    importance TEXT NOT NULL DEFAULT 'NORMAL',
    competing_teams TEXT,
    was_bidding_war INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- 身价变化记录表
CREATE TABLE IF NOT EXISTS market_value_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    old_value INTEGER NOT NULL,
    new_value INTEGER NOT NULL,
    change_amount INTEGER NOT NULL,
    change_percent REAL NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- 选手转会策略表
CREATE TABLE IF NOT EXISTS player_transfer_strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    strategy_json TEXT NOT NULL,
    generated_at TEXT NOT NULL,
    UNIQUE(player_id, save_id, season_id),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_match_games_match ON match_games(match_id);
CREATE INDEX IF NOT EXISTS idx_match_games_save ON match_games(save_id);
CREATE INDEX IF NOT EXISTS idx_game_player_performances_game ON game_player_performances(game_id);
CREATE INDEX IF NOT EXISTS idx_game_player_performances_player ON game_player_performances(player_id);
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
CREATE INDEX IF NOT EXISTS idx_honors_player ON honors(player_id);
CREATE INDEX IF NOT EXISTS idx_honors_season ON honors(season_id);
CREATE INDEX IF NOT EXISTS idx_honors_type ON honors(honor_type);
CREATE INDEX IF NOT EXISTS idx_honors_tournament_type ON honors(tournament_type);
CREATE INDEX IF NOT EXISTS idx_tournament_results_save ON tournament_results(save_id);
CREATE INDEX IF NOT EXISTS idx_tournament_results_tournament ON tournament_results(tournament_id);
CREATE INDEX IF NOT EXISTS idx_player_tournament_stats_tournament ON player_tournament_stats(tournament_id);
CREATE INDEX IF NOT EXISTS idx_player_tournament_stats_player ON player_tournament_stats(player_id);
CREATE INDEX IF NOT EXISTS idx_player_tournament_stats_impact ON player_tournament_stats(avg_impact DESC);
CREATE INDEX IF NOT EXISTS idx_player_form_factors ON player_form_factors(player_id);
CREATE INDEX IF NOT EXISTS idx_player_traits ON player_traits(player_id);
CREATE INDEX IF NOT EXISTS idx_player_season_stats_save ON player_season_stats(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_player_season_stats_player ON player_season_stats(player_id);
CREATE INDEX IF NOT EXISTS idx_player_season_stats_yearly ON player_season_stats(yearly_top_score DESC);
CREATE INDEX IF NOT EXISTS idx_transfer_listings_status ON transfer_listings(status);
CREATE INDEX IF NOT EXISTS idx_transfer_listings_save ON transfer_listings(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_free_agents_status ON free_agents(status);
CREATE INDEX IF NOT EXISTS idx_free_agents_save ON free_agents(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_windows_save ON transfer_windows(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_save ON transfer_events(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_transfer_events_round ON transfer_events(round);
CREATE INDEX IF NOT EXISTS idx_player_transfer_strategies_save ON player_transfer_strategies(save_id, season_id);
CREATE INDEX IF NOT EXISTS idx_player_transfer_strategies_player ON player_transfer_strategies(player_id);
"#;
