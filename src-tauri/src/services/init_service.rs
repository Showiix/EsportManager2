use crate::db::{PlayerRepository, TeamRepository};
use crate::models::{Player, PlayerStatus, PlayerTag, Position, Team};
use rand::Rng;
use sqlx::{Pool, Sqlite};

/// 初始化服务 - 生成游戏初始数据
pub struct InitService;

/// 赛区配置
pub struct RegionConfig {
    pub id: u64,
    pub name: &'static str,
    pub short_name: &'static str,
    pub team_names: Vec<(&'static str, &'static str)>, // (name, short_name)
}

impl InitService {
    /// 获取四大赛区配置
    pub fn get_regions() -> Vec<RegionConfig> {
        vec![
            RegionConfig {
                id: 1,
                name: "LPL",
                short_name: "CN",
                team_names: vec![
                    ("Bilibili Gaming", "BLG"),
                    ("Top Esports", "TES"),
                    ("JD Gaming", "JDG"),
                    ("Weibo Gaming", "WBG"),
                    ("LNG Esports", "LNG"),
                    ("FunPlus Phoenix", "FPX"),
                    ("EDward Gaming", "EDG"),
                    ("Royal Never Give Up", "RNG"),
                    ("Invictus Gaming", "IG"),
                    ("ThunderTalk Gaming", "TT"),
                    ("Ninjas in Pyjamas", "NIP"),
                    ("Oh My God", "OMG"),
                    ("Anyone's Legend", "AL"),
                    ("Ultra Prime", "UP"),
                ],
            },
            RegionConfig {
                id: 2,
                name: "LCK",
                short_name: "KR",
                team_names: vec![
                    ("T1", "T1"),
                    ("Gen.G", "GEN"),
                    ("Hanwha Life Esports", "HLE"),
                    ("Dplus KIA", "DK"),
                    ("KT Rolster", "KT"),
                    ("DRX", "DRX"),
                    ("Kwangdong Freecs", "KDF"),
                    ("Liiv SANDBOX", "LSB"),
                    ("Nongshim RedForce", "NS"),
                    ("OK BRION", "BRO"),
                    ("Fearx", "FOX"),
                    ("BNK FearX", "BNK"),
                    ("DN Freecs", "DNF"),
                    ("Dankook Univ", "DKU"),
                ],
            },
            RegionConfig {
                id: 3,
                name: "LEC",
                short_name: "EU",
                team_names: vec![
                    ("G2 Esports", "G2"),
                    ("Fnatic", "FNC"),
                    ("MAD Lions KOI", "MAD"),
                    ("Team Vitality", "VIT"),
                    ("SK Gaming", "SK"),
                    ("Team BDS", "BDS"),
                    ("Karmine Corp", "KC"),
                    ("Rogue", "RGE"),
                    ("Excel Esports", "XL"),
                    ("Team Heretics", "TH"),
                    ("GIANTX", "GX"),
                    ("Astralis", "AST"),
                    ("Movistar Riders", "MRS"),
                    ("LDLC OL", "LDLC"),
                ],
            },
            RegionConfig {
                id: 4,
                name: "LCS",
                short_name: "NA",
                team_names: vec![
                    ("Cloud9", "C9"),
                    ("Team Liquid", "TL"),
                    ("FlyQuest", "FLY"),
                    ("100 Thieves", "100T"),
                    ("NRG Esports", "NRG"),
                    ("Dignitas", "DIG"),
                    ("Shopify Rebellion", "SR"),
                    ("Immortals", "IMT"),
                    ("Evil Geniuses", "EG"),
                    ("Golden Guardians", "GG"),
                    ("TSM", "TSM"),
                    ("Counter Logic Gaming", "CLG"),
                    ("OpTic Gaming", "OPT"),
                    ("Misfits Gaming", "MSF"),
                ],
            },
        ]
    }

    /// 生成初始队伍战力 (基于赛区强度)
    fn generate_team_power(region_id: u64, rank: usize) -> f64 {
        let mut rng = rand::thread_rng();

        // 基础战力 (赛区差异)
        let region_base = match region_id {
            1 => 78.0, // LPL
            2 => 77.0, // LCK
            3 => 72.0, // LEC
            4 => 70.0, // LCS
            _ => 70.0,
        };

        // 队伍排名差异 (顶级队伍更强)
        let rank_bonus = match rank {
            0..=1 => 10.0,
            2..=3 => 7.0,
            4..=5 => 4.0,
            6..=7 => 2.0,
            8..=10 => 0.0,
            _ => -2.0,
        };

        // 随机波动
        let random_factor: f64 = rng.gen_range(-3.0..3.0);

        (region_base + rank_bonus + random_factor).clamp(60.0, 95.0)
    }

    /// 生成队伍初始资金
    fn generate_team_balance(power: f64) -> i64 {
        let base = 3000i64; // 基础300万
        let power_bonus = ((power - 70.0) * 100.0).max(0.0) as i64;
        (base + power_bonus) * 10000 // 转换为实际金额
    }

    /// 生成选手
    fn generate_players_for_team(
        team_id: u64,
        team_power: f64,
        current_season: u32,
    ) -> Vec<Player> {
        let mut rng = rand::thread_rng();
        let positions = [
            Position::Top,
            Position::Jug,
            Position::Mid,
            Position::Adc,
            Position::Sup,
        ];

        let mut players = Vec::new();
        let mut player_counter = 1u64;

        // 每个位置2名选手 (首发+替补)
        for pos in &positions {
            for is_starter in [true, false] {
                let ability_base = if is_starter {
                    team_power + rng.gen_range(-5.0..5.0)
                } else {
                    team_power - 5.0 + rng.gen_range(-5.0..3.0)
                };

                let ability = (ability_base.clamp(50.0, 99.0)) as u8;
                let potential = (ability as f64 + rng.gen_range(-5.0..15.0)).clamp(50.0, 99.0) as u8;
                let age = if is_starter {
                    rng.gen_range(19..26)
                } else {
                    rng.gen_range(17..28)
                };

                let tag = Self::determine_player_tag(ability, potential, age);
                let salary = Self::calculate_initial_salary(ability, potential, tag);

                players.push(Player {
                    id: 0, // 由数据库生成
                    game_id: format!("Player_{}_{}", team_id, player_counter),
                    real_name: Some(format!("选手{}", player_counter)),
                    nationality: Some("CN".to_string()),
                    age,
                    ability,
                    potential,
                    stability: Player::calculate_stability(age),
                    tag,
                    status: PlayerStatus::Active,
                    position: Some(*pos),
                    team_id: Some(team_id),
                    salary,
                    market_value: Self::calculate_market_value(ability, potential, age),
                    contract_end_season: Some(current_season + rng.gen_range(1..4)),
                    join_season: current_season,
                    retire_season: None,
                    is_starter,
                });

                player_counter += 1;
            }
        }

        players
    }

    /// 判定选手标签 (Ordinary=普通, Normal=正常, Genius=天才)
    fn determine_player_tag(ability: u8, potential: u8, age: u8) -> PlayerTag {
        if ability >= 90 || (age <= 20 && potential >= 90) {
            PlayerTag::Genius
        } else if ability >= 75 || potential >= 80 {
            PlayerTag::Normal
        } else {
            PlayerTag::Ordinary
        }
    }

    /// 计算初始薪资
    fn calculate_initial_salary(ability: u8, _potential: u8, tag: PlayerTag) -> u64 {
        let base = match ability {
            90..=100 => 150,
            85..=89 => 100,
            80..=84 => 70,
            75..=79 => 50,
            70..=74 => 35,
            _ => 20,
        };

        let tag_multiplier = match tag {
            PlayerTag::Genius => 1.5,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 0.8,
        };

        ((base as f64) * tag_multiplier) as u64
    }

    /// 计算市场价值
    fn calculate_market_value(ability: u8, potential: u8, age: u8) -> u64 {
        let base = match ability {
            90..=100 => 500,
            85..=89 => 350,
            80..=84 => 250,
            75..=79 => 180,
            70..=74 => 120,
            65..=69 => 80,
            _ => 50,
        };

        // 年龄因素
        let age_factor = match age {
            17..=20 => 1.3,
            21..=24 => 1.2,
            25..=27 => 1.0,
            28..=30 => 0.8,
            _ => 0.5,
        };

        // 潜力加成
        let potential_bonus = if potential > ability + 10 {
            1.4
        } else if potential > ability + 5 {
            1.2
        } else {
            1.0
        };

        ((base as f64) * age_factor * potential_bonus) as u64
    }

    /// 初始化所有数据
    pub async fn initialize_game_data(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
    ) -> Result<(), String> {
        let regions = Self::get_regions();

        for region in regions {
            // 创建赛区
            sqlx::query(
                "INSERT INTO regions (save_id, name, short_name, team_count) VALUES (?, ?, ?, ?)"
            )
            .bind(save_id)
            .bind(region.name)
            .bind(region.short_name)
            .bind(14i64)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // 创建队伍
            for (rank, (name, short_name)) in region.team_names.iter().enumerate() {
                let power = Self::generate_team_power(region.id, rank);
                let balance = Self::generate_team_balance(power);

                let team = Team {
                    id: 0,
                    region_id: region.id,
                    name: name.to_string(),
                    short_name: Some(short_name.to_string()),
                    power_rating: power,
                    total_matches: 0,
                    wins: 0,
                    win_rate: 0.0,
                    annual_points: 0,
                    cross_year_points: 0,
                    balance,
                };

                let team_id = TeamRepository::create(pool, save_id, &team)
                    .await
                    .map_err(|e| e.to_string())?;

                // 创建选手
                let players = Self::generate_players_for_team(team_id, power, current_season);
                for player in players {
                    PlayerRepository::create(pool, save_id, &player)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }
}

