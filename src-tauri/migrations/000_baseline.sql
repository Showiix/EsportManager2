-- ============================================
-- Baseline Schema (Version 0)
-- ============================================
-- This file represents the complete database schema as of 2026-02-26
-- All future migrations should be incremental SQL files (001_*.sql, 002_*.sql, etc.)
--
-- For new databases: This baseline is applied once
-- For existing databases: Marked as applied automatically
-- ============================================


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
    brand_value REAL NOT NULL DEFAULT 0,
    training_facility INTEGER NOT NULL DEFAULT 0,
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
    loyalty INTEGER NOT NULL DEFAULT 50,
    home_region_id INTEGER,
    region_loyalty INTEGER NOT NULL DEFAULT 70,
    growth_event_state TEXT DEFAULT '{}',
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
    original_team_id INTEGER,
    acquired_via TEXT DEFAULT 'LOTTERY',
    acquisition_price INTEGER DEFAULT 0,
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
    related_tournament_id INTEGER,
    related_player_id INTEGER,
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
    home_base_power REAL,
    away_base_power REAL,
    home_synergy_bonus REAL,
    away_synergy_bonus REAL,
    home_bp_bonus REAL,
    away_bp_bonus REAL,
    home_version_bonus REAL,
    away_version_bonus REAL,
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


-- Additional tables from incremental migrations

CREATE TABLE IF NOT EXISTS hall_of_fame (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                player_id INTEGER NOT NULL,
                player_name TEXT NOT NULL,
                position TEXT NOT NULL,
                region_id INTEGER,
                induction_season INTEGER NOT NULL,
                total_score INTEGER NOT NULL,
                tier TEXT NOT NULL,
                peak_ability INTEGER,
                career_seasons INTEGER,
                honors_json TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(save_id, player_id),
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (player_id) REFERENCES players(id)
            );

CREATE TABLE IF NOT EXISTS match_lineups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                match_id INTEGER NOT NULL,
                game_number INTEGER NOT NULL,
                team_id INTEGER NOT NULL,
                player_id INTEGER NOT NULL,
                position TEXT NOT NULL,
                is_substitution INTEGER DEFAULT 0,
                replaced_player_id INTEGER,
                substitution_reason TEXT,
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (match_id) REFERENCES matches(id),
                FOREIGN KEY (player_id) REFERENCES players(id)
            );

CREATE TABLE IF NOT EXISTS player_champion_mastery (
                save_id TEXT NOT NULL,
                player_id INTEGER NOT NULL,
                champion_id INTEGER NOT NULL,
                mastery_tier TEXT NOT NULL,
                games_played INTEGER NOT NULL DEFAULT 0,
                games_won INTEGER NOT NULL DEFAULT 0,
                UNIQUE(save_id, player_id, champion_id),
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (player_id) REFERENCES players(id)
            );

CREATE TABLE IF NOT EXISTS game_draft_results (
                save_id TEXT NOT NULL,
                match_id INTEGER NOT NULL,
                game_number INTEGER NOT NULL,
                bans_json TEXT NOT NULL,
                home_picks_json TEXT NOT NULL,
                away_picks_json TEXT NOT NULL,
                home_comp TEXT,
                away_comp TEXT,
                UNIQUE(save_id, match_id, game_number),
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (match_id) REFERENCES matches(id)
            );

CREATE TABLE IF NOT EXISTS player_growth_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                season_id INTEGER NOT NULL,
                player_id INTEGER NOT NULL,
                player_name TEXT NOT NULL,
                team_name TEXT NOT NULL DEFAULT '',
                age INTEGER NOT NULL,
                old_ability INTEGER NOT NULL,
                new_ability INTEGER NOT NULL,
                old_potential INTEGER NOT NULL,
                new_potential INTEGER NOT NULL,
                base_growth REAL NOT NULL DEFAULT 0,
                age_coeff REAL NOT NULL DEFAULT 1.0,
                playtime_coeff REAL NOT NULL DEFAULT 1.0,
                mentor_coeff REAL NOT NULL DEFAULT 1.0,
                synergy_coeff REAL NOT NULL DEFAULT 1.0,
                facility_coeff REAL NOT NULL DEFAULT 1.0,
                prodigy_mod REAL NOT NULL DEFAULT 1.0,
                perf_bonus REAL NOT NULL DEFAULT 0,
                fluctuation REAL NOT NULL DEFAULT 0,
                growth_event TEXT,
                description TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS ladder_tournament (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                season INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                event_name TEXT NOT NULL,
                edition INTEGER NOT NULL,
                total_rounds INTEGER DEFAULT 12,
                current_round INTEGER DEFAULT 0,
                status TEXT DEFAULT 'pending',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                UNIQUE(save_id, season, event_type),
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
            );

CREATE INDEX IF NOT EXISTS idx_ladder_tournament_save_season ON ladder_tournament(save_id, season);

            CREATE TABLE IF NOT EXISTS ladder_rating (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                ladder_tournament_id INTEGER NOT NULL,
                player_id INTEGER NOT NULL,
                player_name TEXT NOT NULL,
                position TEXT NOT NULL,
                team_name TEXT,
                rating INTEGER DEFAULT 1200,
                games_played INTEGER DEFAULT 0,
                wins INTEGER DEFAULT 0,
                losses INTEGER DEFAULT 0,
                mvp_count INTEGER DEFAULT 0,
                total_influence REAL DEFAULT 0.0,
                avg_influence REAL DEFAULT 0.0,
                max_rating INTEGER DEFAULT 1200,
                final_rank INTEGER,
                round_data_json TEXT,
                UNIQUE(save_id, ladder_tournament_id, player_id),
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (ladder_tournament_id) REFERENCES ladder_tournament(id) ON DELETE CASCADE,
                FOREIGN KEY (player_id) REFERENCES players(id)
            );

CREATE INDEX IF NOT EXISTS idx_ladder_rating_tournament ON ladder_rating(ladder_tournament_id);

CREATE INDEX IF NOT EXISTS idx_ladder_rating_player ON ladder_rating(save_id, player_id);

            CREATE TABLE IF NOT EXISTS ladder_match (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                save_id TEXT NOT NULL,
                ladder_tournament_id INTEGER NOT NULL,
                round_number INTEGER NOT NULL,
                match_number INTEGER NOT NULL,
                blue_team_json TEXT NOT NULL,
                red_team_json TEXT NOT NULL,
                blue_avg_rating INTEGER,
                red_avg_rating INTEGER,
                blue_power REAL,
                red_power REAL,
                winner_side TEXT,
                mvp_player_id INTEGER,
                mvp_player_name TEXT,
                game_duration INTEGER,
                performances_json TEXT,
                draft_result_json TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                FOREIGN KEY (ladder_tournament_id) REFERENCES ladder_tournament(id) ON DELETE CASCADE
            );

CREATE INDEX IF NOT EXISTS idx_ladder_match_tournament_round ON ladder_match(ladder_tournament_id, round_number);

ALTER TABLE ladder_rating ADD COLUMN game_id TEXT DEFAULT '';

ALTER TABLE ladder_match ADD COLUMN rating_changes_json TEXT;

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
                );

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
                );

                CREATE TABLE IF NOT EXISTS loyalty_changes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    change_amount INTEGER NOT NULL,
                    reason TEXT NOT NULL,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

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
                );

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
                );

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
                );

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
                );

                CREATE TABLE IF NOT EXISTS draft_pick_wanted (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    season_id INTEGER NOT NULL,
                    region_id INTEGER NOT NULL,
                    auction_id INTEGER NOT NULL,
                    buyer_team_id INTEGER NOT NULL,
                    buyer_team_name TEXT NOT NULL,
                    target_position INTEGER NOT NULL,
                    offer_price INTEGER NOT NULL,
                    reason TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'ACTIVE',
                    holder_team_id INTEGER NOT NULL,
                    holder_team_name TEXT NOT NULL,
                    response_reason TEXT,
                    final_price INTEGER,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    resolved_at TEXT,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (auction_id) REFERENCES draft_pick_auctions(id) ON DELETE CASCADE,
                    FOREIGN KEY (buyer_team_id) REFERENCES teams(id) ON DELETE CASCADE,
                    FOREIGN KEY (holder_team_id) REFERENCES teams(id) ON DELETE CASCADE
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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
                );

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

CREATE TABLE IF NOT EXISTS schema_migrations (
                migration_name TEXT PRIMARY KEY,
                applied_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

CREATE TABLE IF NOT EXISTS transfer_bids (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_id INTEGER NOT NULL,
                    round INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    player_name TEXT NOT NULL,
                    player_ability INTEGER NOT NULL,
                    player_age INTEGER NOT NULL,
                    player_position TEXT,
                    from_team_id INTEGER,
                    from_team_name TEXT,
                    bid_team_id INTEGER NOT NULL,
                    bid_team_name TEXT NOT NULL,
                    bid_team_region_id INTEGER,
                    offered_salary INTEGER NOT NULL,
                    contract_years INTEGER NOT NULL,
                    transfer_fee INTEGER NOT NULL DEFAULT 0,
                    signing_bonus INTEGER NOT NULL DEFAULT 0,
                    match_score REAL NOT NULL,
                    willingness REAL NOT NULL,
                    is_winner INTEGER NOT NULL DEFAULT 0,
                    reject_reason TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

CREATE TABLE IF NOT EXISTS player_contracts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    player_id INTEGER NOT NULL,
                    team_id INTEGER NOT NULL,
                    contract_type TEXT NOT NULL DEFAULT 'INITIAL',
                    total_salary INTEGER NOT NULL DEFAULT 0,
                    annual_salary INTEGER NOT NULL DEFAULT 0,
                    contract_years INTEGER NOT NULL DEFAULT 1,
                    start_season INTEGER NOT NULL,
                    end_season INTEGER NOT NULL,
                    transfer_fee INTEGER NOT NULL DEFAULT 0,
                    signing_bonus INTEGER NOT NULL DEFAULT 0,
                    is_active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES players(id),
                    FOREIGN KEY (team_id) REFERENCES teams(id)
                );

                INSERT INTO player_contracts (save_id, player_id, team_id, contract_type, total_salary, annual_salary, contract_years, start_season, end_season, is_active)
                SELECT
                    p.save_id,
                    p.id,
                    p.team_id,
                    'INITIAL',
                    p.salary,
                    CASE
                        WHEN p.contract_end_season - COALESCE(p.join_season, s.current_season) > 0
                        THEN p.salary / (p.contract_end_season - COALESCE(p.join_season, s.current_season))
                        ELSE p.salary
                    END,
                    CASE
                        WHEN p.contract_end_season - COALESCE(p.join_season, s.current_season) > 0
                        THEN p.contract_end_season - COALESCE(p.join_season, s.current_season)
                        ELSE 1
                    END,
                    COALESCE(p.join_season, s.current_season),
                    p.contract_end_season,
                    1
                FROM players p
                JOIN saves s ON p.save_id = s.id
                WHERE p.status = 'Active' AND p.team_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS draft_pool (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    save_id TEXT NOT NULL,
                    region_id INTEGER NOT NULL,
                    game_id TEXT NOT NULL,
                    real_name TEXT,
                    nationality TEXT,
                    age INTEGER NOT NULL,
                    ability INTEGER NOT NULL,
                    potential INTEGER NOT NULL,
                    position TEXT NOT NULL,
                    tag TEXT NOT NULL DEFAULT 'Normal',
                    status TEXT NOT NULL DEFAULT 'available',
                    drafted_season INTEGER,
                    drafted_by_team_id INTEGER,
                    created_season INTEGER NOT NULL,
                    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
                );

                INSERT INTO draft_pool (save_id, region_id, game_id, real_name, nationality, age, ability, potential, position, tag, status, created_season)
                SELECT save_id, region_id, game_id, real_name, nationality, age, ability, potential, position, tag, 'available', season_id
                FROM draft_players
                WHERE is_picked = 0;
