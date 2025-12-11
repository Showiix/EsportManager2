-- =====================================================
-- 电竞比赛模拟器2 (EsportManager 2) 数据库设计
-- 数据库: MySQL 8.0
-- 字符集: utf8mb4
-- =====================================================

-- 创建数据库
CREATE DATABASE IF NOT EXISTS esport_manager_2
    DEFAULT CHARACTER SET utf8mb4
    DEFAULT COLLATE utf8mb4_unicode_ci;

USE esport_manager_2;

-- =====================================================
-- 1. 存档管理表
-- =====================================================

-- 存档表
CREATE TABLE saves (
    id CHAR(36) PRIMARY KEY COMMENT '存档UUID',
    name VARCHAR(100) NOT NULL COMMENT '存档名称',
    current_season INT UNSIGNED NOT NULL DEFAULT 1 COMMENT '当前赛季 (S1, S2...)',
    current_phase ENUM(
        'SPRING_REGULAR', 'SPRING_PLAYOFFS',
        'MSI', 'MADRID_MASTERS',
        'SUMMER_REGULAR', 'SUMMER_PLAYOFFS',
        'CLAUDE_INTERCONTINENTAL', 'WORLD_CHAMPIONSHIP',
        'SHANGHAI_MASTERS', 'ICP_INTERCONTINENTAL',
        'SUPER_INTERCONTINENTAL', 'TRANSFER_WINDOW',
        'DRAFT', 'SEASON_END'
    ) NOT NULL DEFAULT 'SPRING_REGULAR' COMMENT '当前阶段',
    phase_completed BOOLEAN NOT NULL DEFAULT FALSE COMMENT '当前阶段是否完成',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_updated_at (updated_at)
) ENGINE=InnoDB COMMENT='存档表';

-- =====================================================
-- 2. 基础数据表
-- =====================================================

-- 赛区表
CREATE TABLE regions (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL COMMENT '所属存档',
    code VARCHAR(10) NOT NULL COMMENT '赛区代码 (LPL/LCK/LEC/LCS)',
    name VARCHAR(50) NOT NULL COMMENT '赛区名称',
    full_name VARCHAR(100) COMMENT '赛区全称',
    UNIQUE KEY uk_save_code (save_id, code),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
) ENGINE=InnoDB COMMENT='赛区表';

-- 战队表
CREATE TABLE teams (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL COMMENT '所属存档',
    region_id BIGINT UNSIGNED NOT NULL COMMENT '所属赛区',
    name VARCHAR(100) NOT NULL COMMENT '战队名称',
    short_name VARCHAR(20) COMMENT '战队简称',
    power_rating DECIMAL(5,2) NOT NULL DEFAULT 50.00 COMMENT '战力值 (0-100)',
    total_matches INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '总场次',
    wins INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '胜场',
    win_rate DECIMAL(5,2) NOT NULL DEFAULT 0.00 COMMENT '胜率',
    annual_points INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '年度积分',
    cross_year_points INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '跨年度积分(洲际赛资格)',
    balance BIGINT NOT NULL DEFAULT 5000000 COMMENT '账户余额(万元)',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE,
    INDEX idx_save_region (save_id, region_id),
    INDEX idx_power_rating (power_rating),
    INDEX idx_annual_points (annual_points)
) ENGINE=InnoDB COMMENT='战队表';

-- =====================================================
-- 3. 选手系统表
-- =====================================================

-- 选手表
CREATE TABLE players (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL COMMENT '所属存档',
    team_id BIGINT UNSIGNED COMMENT '所属战队(NULL表示自由球员)',
    game_id VARCHAR(50) NOT NULL COMMENT '游戏ID',
    real_name VARCHAR(100) COMMENT '真实姓名',
    nationality VARCHAR(50) COMMENT '国籍',
    age TINYINT UNSIGNED NOT NULL COMMENT '年龄 (16-36)',
    ability TINYINT UNSIGNED NOT NULL COMMENT '能力值 (0-100)',
    potential TINYINT UNSIGNED NOT NULL COMMENT '潜力值上限 (0-100)',
    stability TINYINT UNSIGNED NOT NULL COMMENT '稳定性 (0-100)',
    tag ENUM('ORDINARY', 'NORMAL', 'GENIUS') NOT NULL DEFAULT 'NORMAL' COMMENT '标签',
    status ENUM('ACTIVE', 'RETIRED') NOT NULL DEFAULT 'ACTIVE' COMMENT '状态',
    position ENUM('TOP', 'JUG', 'MID', 'ADC', 'SUP') COMMENT '位置',
    salary BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '年薪(万元)',
    market_value BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '身价(万元)',
    contract_end_season INT UNSIGNED COMMENT '合同到期赛季',
    join_season INT UNSIGNED NOT NULL COMMENT '加入赛季',
    retire_season INT UNSIGNED COMMENT '退役赛季',
    is_starter BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否首发',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
    INDEX idx_save_team (save_id, team_id),
    INDEX idx_ability (ability),
    INDEX idx_status (status)
) ENGINE=InnoDB COMMENT='选手表';

-- 选手赛季历史表 (记录每赛季的属性快照)
CREATE TABLE player_season_history (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    player_id BIGINT UNSIGNED NOT NULL,
    season INT UNSIGNED NOT NULL COMMENT '赛季',
    team_id BIGINT UNSIGNED COMMENT '当时所属战队',
    age TINYINT UNSIGNED NOT NULL,
    ability TINYINT UNSIGNED NOT NULL,
    potential TINYINT UNSIGNED NOT NULL,
    stability TINYINT UNSIGNED NOT NULL,
    salary BIGINT UNSIGNED NOT NULL,
    market_value BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE KEY uk_player_season (player_id, season)
) ENGINE=InnoDB COMMENT='选手赛季历史表';

-- =====================================================
-- 4. 赛事系统表
-- =====================================================

-- 赛季表
CREATE TABLE seasons (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL COMMENT '所属存档',
    season_number INT UNSIGNED NOT NULL COMMENT '赛季编号 (1=S1, 2=S2...)',
    status ENUM('UPCOMING', 'IN_PROGRESS', 'COMPLETED') NOT NULL DEFAULT 'UPCOMING',
    started_at TIMESTAMP NULL,
    ended_at TIMESTAMP NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE KEY uk_save_season (save_id, season_number)
) ENGINE=InnoDB COMMENT='赛季表';

-- 赛事表 (各类杯赛/联赛)
CREATE TABLE tournaments (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL COMMENT '所属存档',
    season_id BIGINT UNSIGNED NOT NULL COMMENT '所属赛季',
    type ENUM(
        'SPRING_REGULAR', 'SPRING_PLAYOFFS',
        'SUMMER_REGULAR', 'SUMMER_PLAYOFFS',
        'MSI', 'MADRID_MASTERS',
        'CLAUDE_INTERCONTINENTAL', 'WORLD_CHAMPIONSHIP',
        'SHANGHAI_MASTERS', 'ICP_INTERCONTINENTAL',
        'SUPER_INTERCONTINENTAL'
    ) NOT NULL COMMENT '赛事类型',
    name VARCHAR(100) NOT NULL COMMENT '赛事名称',
    region_id BIGINT UNSIGNED COMMENT '所属赛区(国际赛事为NULL)',
    status ENUM('UPCOMING', 'IN_PROGRESS', 'COMPLETED') NOT NULL DEFAULT 'UPCOMING',
    current_stage VARCHAR(50) COMMENT '当前阶段',
    current_round INT UNSIGNED COMMENT '当前轮次',
    started_at TIMESTAMP NULL,
    ended_at TIMESTAMP NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE SET NULL,
    INDEX idx_save_type (save_id, type)
) ENGINE=InnoDB COMMENT='赛事表';

-- 赛事参赛队伍表
CREATE TABLE tournament_participants (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    tournament_id BIGINT UNSIGNED NOT NULL,
    team_id BIGINT UNSIGNED NOT NULL,
    seed INT UNSIGNED COMMENT '种子排名',
    group_name VARCHAR(10) COMMENT '分组 (A/B/C...)',
    qualification_type ENUM('CHAMPION', 'RUNNER_UP', 'THIRD', 'QUALIFIED', 'INVITED') COMMENT '晋级方式',
    final_rank INT UNSIGNED COMMENT '最终排名',
    points_earned INT UNSIGNED DEFAULT 0 COMMENT '获得积分',
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE KEY uk_tournament_team (tournament_id, team_id)
) ENGINE=InnoDB COMMENT='赛事参赛队伍表';

-- =====================================================
-- 5. 比赛记录表
-- =====================================================

-- 比赛表 (Match = 一场BO系列赛)
CREATE TABLE matches (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    tournament_id BIGINT UNSIGNED NOT NULL COMMENT '所属赛事',
    stage VARCHAR(50) NOT NULL COMMENT '阶段 (GROUP/KNOCKOUT/FINAL等)',
    round INT UNSIGNED COMMENT '轮次',
    match_order INT UNSIGNED COMMENT '场次序号',
    format ENUM('BO1', 'BO3', 'BO5') NOT NULL DEFAULT 'BO3' COMMENT '赛制',
    home_team_id BIGINT UNSIGNED NOT NULL COMMENT '主队',
    away_team_id BIGINT UNSIGNED NOT NULL COMMENT '客队',
    home_score TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '主队得分',
    away_score TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '客队得分',
    winner_id BIGINT UNSIGNED COMMENT '获胜队伍',
    status ENUM('SCHEDULED', 'IN_PROGRESS', 'COMPLETED') NOT NULL DEFAULT 'SCHEDULED',
    scheduled_at TIMESTAMP NULL COMMENT '预定时间',
    played_at TIMESTAMP NULL COMMENT '实际比赛时间',
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (home_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (away_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_id) REFERENCES teams(id) ON DELETE SET NULL,
    INDEX idx_tournament_stage (tournament_id, stage),
    INDEX idx_team_matches (home_team_id),
    INDEX idx_team_matches_away (away_team_id)
) ENGINE=InnoDB COMMENT='比赛表';

-- 比赛小局表 (Game = 一小局)
CREATE TABLE match_games (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    match_id BIGINT UNSIGNED NOT NULL COMMENT '所属比赛',
    game_number TINYINT UNSIGNED NOT NULL COMMENT '局数 (1/2/3/4/5)',
    home_power DECIMAL(5,2) NOT NULL COMMENT '主队战力值',
    away_power DECIMAL(5,2) NOT NULL COMMENT '客队战力值',
    home_performance DECIMAL(5,2) NOT NULL COMMENT '主队发挥值',
    away_performance DECIMAL(5,2) NOT NULL COMMENT '客队发挥值',
    winner_id BIGINT UNSIGNED NOT NULL COMMENT '获胜队伍',
    duration_minutes INT UNSIGNED COMMENT '比赛时长(分钟)',
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE KEY uk_match_game (match_id, game_number)
) ENGINE=InnoDB COMMENT='比赛小局表';

-- 选手比赛表现表
CREATE TABLE player_match_stats (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    match_game_id BIGINT UNSIGNED NOT NULL COMMENT '所属小局',
    player_id BIGINT UNSIGNED NOT NULL COMMENT '选手',
    team_id BIGINT UNSIGNED NOT NULL COMMENT '所属队伍',
    base_ability TINYINT UNSIGNED NOT NULL COMMENT '基础能力',
    form_bonus TINYINT COMMENT '状态加成',
    performance DECIMAL(5,2) NOT NULL COMMENT '实际发挥值',
    contribution DECIMAL(5,2) NOT NULL COMMENT '贡献值 (能力值/5)',
    FOREIGN KEY (match_game_id) REFERENCES match_games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    INDEX idx_player_stats (player_id)
) ENGINE=InnoDB COMMENT='选手比赛表现表';

-- =====================================================
-- 6. 积分系统表
-- =====================================================

-- 联赛积分榜表
CREATE TABLE league_standings (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    tournament_id BIGINT UNSIGNED NOT NULL COMMENT '所属联赛',
    team_id BIGINT UNSIGNED NOT NULL,
    rank INT UNSIGNED COMMENT '排名',
    matches_played INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已比场次',
    wins INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '胜场(大比分)',
    losses INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '负场',
    points INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '积分',
    games_won INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '小局胜场',
    games_lost INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '小局负场',
    game_diff INT NOT NULL DEFAULT 0 COMMENT '净胜小局',
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE KEY uk_tournament_team (tournament_id, team_id),
    INDEX idx_points (points DESC, game_diff DESC)
) ENGINE=InnoDB COMMENT='联赛积分榜表';

-- 年度积分明细表
CREATE TABLE annual_points_detail (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL COMMENT '赛季',
    team_id BIGINT UNSIGNED NOT NULL COMMENT '战队',
    tournament_id BIGINT UNSIGNED NOT NULL COMMENT '赛事',
    points INT UNSIGNED NOT NULL COMMENT '获得积分',
    final_rank INT UNSIGNED COMMENT '赛事最终排名',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    INDEX idx_team_season (team_id, season_id)
) ENGINE=InnoDB COMMENT='年度积分明细表';

-- 全球积分排名表
CREATE TABLE global_rankings (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    team_id BIGINT UNSIGNED NOT NULL,
    global_rank INT UNSIGNED NOT NULL COMMENT '全球排名',
    total_points INT UNSIGNED NOT NULL COMMENT '总积分',
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE KEY uk_save_season_team (save_id, season_id, team_id),
    INDEX idx_global_rank (global_rank)
) ENGINE=InnoDB COMMENT='全球积分排名表';

-- =====================================================
-- 7. 荣誉系统表
-- =====================================================

-- 荣誉表
CREATE TABLE honors (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL COMMENT '赛季',
    tournament_id BIGINT UNSIGNED NOT NULL COMMENT '赛事',
    team_id BIGINT UNSIGNED NOT NULL COMMENT '获得战队',
    honor_type ENUM('CHAMPION', 'RUNNER_UP', 'THIRD', 'FOURTH', 'MVP', 'BEST_PLAYER') NOT NULL COMMENT '荣誉类型',
    description VARCHAR(255) COMMENT '描述',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    INDEX idx_team_honors (team_id)
) ENGINE=InnoDB COMMENT='荣誉表';

-- 选手荣誉关联表
CREATE TABLE player_honors (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    honor_id BIGINT UNSIGNED NOT NULL,
    player_id BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (honor_id) REFERENCES honors(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE KEY uk_honor_player (honor_id, player_id)
) ENGINE=InnoDB COMMENT='选手荣誉关联表';

-- =====================================================
-- 8. 选秀系统表
-- =====================================================

-- 选秀池表
CREATE TABLE draft_pool (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL COMMENT '选秀赛季',
    region_id BIGINT UNSIGNED NOT NULL COMMENT '赛区',
    draft_rank TINYINT UNSIGNED NOT NULL COMMENT '选秀排名 (1=状元, 2=榜眼, 3=探花...)',
    game_id VARCHAR(50) NOT NULL COMMENT '游戏ID',
    real_name VARCHAR(100) COMMENT '真实姓名',
    nationality VARCHAR(50) COMMENT '国籍',
    age TINYINT UNSIGNED NOT NULL COMMENT '年龄',
    ability TINYINT UNSIGNED NOT NULL COMMENT '能力值',
    potential TINYINT UNSIGNED NOT NULL COMMENT '潜力值',
    tag ENUM('ORDINARY', 'NORMAL', 'GENIUS') NOT NULL DEFAULT 'NORMAL',
    position ENUM('TOP', 'JUG', 'MID', 'ADC', 'SUP') COMMENT '位置',
    is_picked BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否已被选中',
    picked_by_team_id BIGINT UNSIGNED COMMENT '被哪支队伍选中',
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE,
    FOREIGN KEY (picked_by_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    INDEX idx_region_season (region_id, season_id)
) ENGINE=InnoDB COMMENT='选秀池表';

-- 选秀顺位表
CREATE TABLE draft_order (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    region_id BIGINT UNSIGNED NOT NULL,
    team_id BIGINT UNSIGNED NOT NULL COMMENT '队伍',
    summer_rank INT UNSIGNED NOT NULL COMMENT '夏季赛排名',
    draft_position INT UNSIGNED NOT NULL COMMENT '选秀顺位',
    lottery_result VARCHAR(50) COMMENT '抽签结果描述',
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE KEY uk_season_region_team (season_id, region_id, team_id)
) ENGINE=InnoDB COMMENT='选秀顺位表';

-- 选秀结果表
CREATE TABLE draft_results (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    region_id BIGINT UNSIGNED NOT NULL,
    draft_pool_id BIGINT UNSIGNED NOT NULL COMMENT '选秀池球员',
    team_id BIGINT UNSIGNED NOT NULL COMMENT '选中队伍',
    pick_number INT UNSIGNED NOT NULL COMMENT '第几顺位',
    player_id BIGINT UNSIGNED COMMENT '创建的选手ID',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE,
    FOREIGN KEY (draft_pool_id) REFERENCES draft_pool(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE SET NULL
) ENGINE=InnoDB COMMENT='选秀结果表';

-- =====================================================
-- 9. 转会系统表
-- =====================================================

-- 转会记录表
CREATE TABLE transfer_records (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL COMMENT '转会赛季',
    player_id BIGINT UNSIGNED NOT NULL COMMENT '转会选手',
    from_team_id BIGINT UNSIGNED COMMENT '原球队(NULL=自由球员/新秀)',
    to_team_id BIGINT UNSIGNED COMMENT '新球队(NULL=退役/解约)',
    transfer_type ENUM('FREE_AGENT', 'PURCHASE', 'SALE', 'RETIREMENT', 'LOAN', 'DRAFT', 'CONTRACT_EXPIRE') NOT NULL,
    transfer_fee BIGINT UNSIGNED DEFAULT 0 COMMENT '转会费(万元)',
    new_salary BIGINT UNSIGNED COMMENT '新薪资(万元)',
    contract_years INT UNSIGNED COMMENT '合同年限',
    description VARCHAR(500) COMMENT '转会描述',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (from_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    FOREIGN KEY (to_team_id) REFERENCES teams(id) ON DELETE SET NULL,
    INDEX idx_season_transfers (season_id),
    INDEX idx_player_transfers (player_id)
) ENGINE=InnoDB COMMENT='转会记录表';

-- 转会市场挂牌表
CREATE TABLE transfer_listings (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    player_id BIGINT UNSIGNED NOT NULL COMMENT '挂牌选手',
    team_id BIGINT UNSIGNED NOT NULL COMMENT '所属球队',
    listing_type ENUM('FOR_SALE', 'LOAN_AVAILABLE') NOT NULL DEFAULT 'FOR_SALE',
    asking_price BIGINT UNSIGNED NOT NULL COMMENT '要价(万元)',
    min_price BIGINT UNSIGNED COMMENT '最低接受价',
    status ENUM('ACTIVE', 'SOLD', 'WITHDRAWN') NOT NULL DEFAULT 'ACTIVE',
    listed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    INDEX idx_active_listings (status, save_id)
) ENGINE=InnoDB COMMENT='转会市场挂牌表';

-- 自由球员市场表
CREATE TABLE free_agents (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    player_id BIGINT UNSIGNED NOT NULL,
    salary_demand BIGINT UNSIGNED NOT NULL COMMENT '期望年薪(万元)',
    reason ENUM('CONTRACT_EXPIRE', 'RELEASED', 'RETIRED_TEAM') NOT NULL COMMENT '原因',
    status ENUM('AVAILABLE', 'SIGNED', 'RETIRED') NOT NULL DEFAULT 'AVAILABLE',
    available_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    INDEX idx_available (status, save_id)
) ENGINE=InnoDB COMMENT='自由球员市场表';

-- =====================================================
-- 10. 财务系统表
-- =====================================================

-- 球队财务表
CREATE TABLE team_finances (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    team_id BIGINT UNSIGNED NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    opening_balance BIGINT NOT NULL COMMENT '期初余额(万元)',
    closing_balance BIGINT NOT NULL COMMENT '期末余额(万元)',
    total_income BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '总收入',
    total_expense BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '总支出',
    financial_status ENUM('WEALTHY', 'HEALTHY', 'TIGHT', 'DEFICIT', 'BANKRUPT') NOT NULL COMMENT '财务状态',
    salary_cap_used BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '薪资使用额',
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    UNIQUE KEY uk_team_season (team_id, season_id)
) ENGINE=InnoDB COMMENT='球队财务表';

-- 财务明细表
CREATE TABLE financial_transactions (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    team_id BIGINT UNSIGNED NOT NULL,
    season_id BIGINT UNSIGNED NOT NULL,
    type ENUM(
        'LEAGUE_SHARE',          -- 联盟分成
        'RANKING_BONUS',         -- 排名奖金
        'PLAYOFF_BONUS',         -- 季后赛奖金
        'INTERNATIONAL_BONUS',   -- 国际赛奖金
        'MERCHANDISE',           -- 周边收入
        'TRANSFER_IN',           -- 转会收入
        'SALARY',                -- 选手薪资
        'TRANSFER_OUT',          -- 转会支出
        'OPERATING_COST',        -- 运营成本
        'FACILITY',              -- 设施投资
        'PENALTY',               -- 违约金
        'LUXURY_TAX'             -- 奢侈税
    ) NOT NULL COMMENT '交易类型',
    amount BIGINT NOT NULL COMMENT '金额(正=收入,负=支出)',
    description VARCHAR(255) COMMENT '描述',
    related_player_id BIGINT UNSIGNED COMMENT '关联选手',
    related_tournament_id BIGINT UNSIGNED COMMENT '关联赛事',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (related_player_id) REFERENCES players(id) ON DELETE SET NULL,
    FOREIGN KEY (related_tournament_id) REFERENCES tournaments(id) ON DELETE SET NULL,
    INDEX idx_team_season (team_id, season_id)
) ENGINE=InnoDB COMMENT='财务明细表';

-- =====================================================
-- 11. 抽签记录表
-- =====================================================

-- 抽签记录表
CREATE TABLE draw_records (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    tournament_id BIGINT UNSIGNED NOT NULL,
    draw_type ENUM(
        'GROUP_STAGE',           -- 小组赛分组
        'KNOCKOUT_BRACKET',      -- 淘汰赛对阵
        'SEED_POSITION',         -- 种子位置
        'DRAFT_ORDER'            -- 选秀顺位
    ) NOT NULL COMMENT '抽签类型',
    stage VARCHAR(50) COMMENT '阶段',
    draw_data JSON NOT NULL COMMENT '抽签结果数据',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
) ENGINE=InnoDB COMMENT='抽签记录表';

-- =====================================================
-- 12. 系统配置表
-- =====================================================

-- 积分配置表
CREATE TABLE points_config (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    tournament_type ENUM(
        'SPRING_PLAYOFFS', 'SUMMER_PLAYOFFS',
        'MSI', 'MADRID_MASTERS',
        'CLAUDE_INTERCONTINENTAL', 'WORLD_CHAMPIONSHIP',
        'SHANGHAI_MASTERS', 'ICP_INTERCONTINENTAL',
        'SUPER_INTERCONTINENTAL'
    ) NOT NULL,
    rank_position VARCHAR(50) NOT NULL COMMENT '名次/位置',
    points INT UNSIGNED NOT NULL COMMENT '积分',
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE KEY uk_save_type_rank (save_id, tournament_type, rank_position)
) ENGINE=InnoDB COMMENT='积分配置表';

-- 财务配置表
CREATE TABLE financial_config (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    config_key VARCHAR(100) NOT NULL COMMENT '配置键',
    config_value BIGINT NOT NULL COMMENT '配置值',
    description VARCHAR(255) COMMENT '描述',
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE KEY uk_save_key (save_id, config_key)
) ENGINE=InnoDB COMMENT='财务配置表';

-- =====================================================
-- 13. 历史交锋记录表
-- =====================================================

-- 战队历史交锋统计表
CREATE TABLE head_to_head_stats (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    save_id CHAR(36) NOT NULL,
    team1_id BIGINT UNSIGNED NOT NULL,
    team2_id BIGINT UNSIGNED NOT NULL,
    total_matches INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '总交手次数',
    team1_wins INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'team1胜场',
    team2_wins INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'team2胜场',
    team1_games INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'team1小局胜',
    team2_games INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'team2小局胜',
    last_match_id BIGINT UNSIGNED COMMENT '最近一次交手',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (team1_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (team2_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (last_match_id) REFERENCES matches(id) ON DELETE SET NULL,
    UNIQUE KEY uk_teams (save_id, team1_id, team2_id),
    CHECK (team1_id < team2_id)  -- 确保team1_id始终小于team2_id,避免重复
) ENGINE=InnoDB COMMENT='战队历史交锋统计表';

-- =====================================================
-- 初始化数据
-- =====================================================

-- 插入默认积分配置(示例)
-- INSERT INTO points_config (save_id, tournament_type, rank_position, points) VALUES
-- ('存档ID', 'SPRING_PLAYOFFS', 'CHAMPION', 12),
-- ('存档ID', 'SPRING_PLAYOFFS', 'RUNNER_UP', 10),
-- ... 其他配置

-- =====================================================
-- 索引优化建议
-- =====================================================

-- 复合索引用于常见查询
-- 1. 查询某赛区某赛季的所有比赛
-- CREATE INDEX idx_matches_region_season ON matches(tournament_id, stage);

-- 2. 查询某选手的所有比赛表现
-- CREATE INDEX idx_player_performance ON player_match_stats(player_id, match_game_id);

-- 3. 查询某队伍的财务历史
-- CREATE INDEX idx_team_finance_history ON financial_transactions(team_id, season_id, created_at);
