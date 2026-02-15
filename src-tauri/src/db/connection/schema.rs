/// 数据库模式定义
pub(crate) const SCHEMA_SQL: &str = r#"
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
    satisfaction INTEGER NOT NULL DEFAULT 50,
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
    perf_history TEXT NOT NULL DEFAULT '',
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
    home_power REAL,
    away_power REAL,
    home_meta_power REAL,
    away_meta_power REAL,
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
    save_id TEXT NOT NULL DEFAULT '',
    season_id INTEGER NOT NULL DEFAULT 0,
    window_id INTEGER,
    round INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'NORMAL',
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    player_ability INTEGER NOT NULL DEFAULT 0,
    from_team_id INTEGER,
    from_team_name TEXT,
    to_team_id INTEGER,
    to_team_name TEXT,
    transfer_fee INTEGER NOT NULL DEFAULT 0,
    salary INTEGER NOT NULL DEFAULT 0,
    contract_years INTEGER,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (window_id) REFERENCES transfer_windows(id),
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

-- Meta 版本表
CREATE TABLE IF NOT EXISTS meta_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    meta_type TEXT NOT NULL,
    meta_name TEXT NOT NULL,
    weight_top REAL NOT NULL DEFAULT 1.0,
    weight_jug REAL NOT NULL DEFAULT 1.0,
    weight_mid REAL NOT NULL DEFAULT 1.0,
    weight_adc REAL NOT NULL DEFAULT 1.0,
    weight_sup REAL NOT NULL DEFAULT 1.0,
    UNIQUE(save_id, season_id),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
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
CREATE INDEX IF NOT EXISTS idx_meta_versions_save ON meta_versions(save_id, season_id);

-- 性能优化：补充缺失索引
CREATE INDEX IF NOT EXISTS idx_honors_save_id ON honors(save_id);
CREATE INDEX IF NOT EXISTS idx_financial_transactions_team_season ON financial_transactions(team_id, season_id);
CREATE INDEX IF NOT EXISTS idx_financial_transactions_save_season ON financial_transactions(save_id, season_id);
"#;
