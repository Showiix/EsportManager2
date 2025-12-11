-- 初始化数据库迁移
-- SQLite版本 (本地存储)

-- 存档表
CREATE TABLE IF NOT EXISTS saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    current_season INTEGER NOT NULL DEFAULT 1,
    current_phase TEXT NOT NULL DEFAULT 'SPRING_REGULAR',
    phase_completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 赛区表
CREATE TABLE IF NOT EXISTS regions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, code)
);

-- 战队表
CREATE TABLE IF NOT EXISTS teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    region_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    short_name TEXT,
    power_rating REAL NOT NULL DEFAULT 50.0,
    total_matches INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    win_rate REAL NOT NULL DEFAULT 0.0,
    annual_points INTEGER NOT NULL DEFAULT 0,
    cross_year_points INTEGER NOT NULL DEFAULT 0,
    balance INTEGER NOT NULL DEFAULT 5000000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE
);

-- 选手表
CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    team_id INTEGER,
    game_id TEXT NOT NULL,
    real_name TEXT,
    nationality TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    stability INTEGER NOT NULL,
    tag TEXT NOT NULL DEFAULT 'NORMAL',
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    position TEXT,
    salary INTEGER NOT NULL DEFAULT 0,
    market_value INTEGER NOT NULL DEFAULT 0,
    contract_end_season INTEGER,
    join_season INTEGER NOT NULL,
    retire_season INTEGER,
    is_starter INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 赛季表
CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_number INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'UPCOMING',
    started_at TEXT,
    ended_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_number)
);

-- 赛事表
CREATE TABLE IF NOT EXISTS tournaments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_type TEXT NOT NULL,
    name TEXT NOT NULL,
    region_id INTEGER,
    status TEXT NOT NULL DEFAULT 'UPCOMING',
    current_stage TEXT,
    current_round INTEGER,
    started_at TEXT,
    ended_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE SET NULL
);

-- 比赛表
CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    tournament_id INTEGER NOT NULL,
    stage TEXT NOT NULL,
    round INTEGER,
    match_order INTEGER,
    format TEXT NOT NULL DEFAULT 'BO3',
    home_team_id INTEGER NOT NULL,
    away_team_id INTEGER NOT NULL,
    home_score INTEGER NOT NULL DEFAULT 0,
    away_score INTEGER NOT NULL DEFAULT 0,
    winner_id INTEGER,
    status TEXT NOT NULL DEFAULT 'SCHEDULED',
    scheduled_at TEXT,
    played_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (home_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (away_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 比赛小局表
CREATE TABLE IF NOT EXISTS match_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id INTEGER NOT NULL,
    game_number INTEGER NOT NULL,
    home_power REAL NOT NULL,
    away_power REAL NOT NULL,
    home_performance REAL NOT NULL,
    away_performance REAL NOT NULL,
    winner_id INTEGER NOT NULL,
    duration_minutes INTEGER,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE(match_id, game_number)
);

-- 联赛积分榜表
CREATE TABLE IF NOT EXISTS league_standings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
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
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
);

-- 荣誉表
CREATE TABLE IF NOT EXISTS honors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    honor_type TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
);

-- 选手荣誉关联表
CREATE TABLE IF NOT EXISTS player_honors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    honor_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    FOREIGN KEY (honor_id) REFERENCES honors(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE(honor_id, player_id)
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
    transfer_fee INTEGER DEFAULT 0,
    new_salary INTEGER,
    contract_years INTEGER,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (to_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- 球队财务表
CREATE TABLE IF NOT EXISTS team_finances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    opening_balance INTEGER NOT NULL,
    closing_balance INTEGER NOT NULL,
    total_income INTEGER NOT NULL DEFAULT 0,
    total_expense INTEGER NOT NULL DEFAULT 0,
    financial_status TEXT NOT NULL,
    salary_cap_used INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    UNIQUE(team_id, season_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_teams_save_region ON teams(save_id, region_id);
CREATE INDEX IF NOT EXISTS idx_players_save_team ON players(save_id, team_id);
CREATE INDEX IF NOT EXISTS idx_matches_tournament ON matches(tournament_id, stage);
CREATE INDEX IF NOT EXISTS idx_standings_points ON league_standings(points DESC, game_diff DESC);
