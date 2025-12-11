use crate::db::{PlayerRepository, TeamRepository, TournamentRepository, MatchRepository, StandingRepository};
use crate::models::{Player, PlayerStatus, PlayerTag, Position, Team, Tournament, TournamentType, TournamentStatus, LeagueStanding};
use crate::services::player_data::{get_team_players, PlayerConfig};
use crate::services::draft_pool_data::{get_draft_pool, get_region_nationality};
use crate::services::league_service::LeagueService;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
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
                    ("Top Esports", "TES"),
                    ("Bilibili Gaming", "BLG"),
                    ("JD Gaming", "JDG"),
                    ("Weibo Gaming", "WBG"),
                    ("Royal Never Give Up", "RNG"),
                    ("FunPlus Phoenix", "FPX"),
                    ("LNG Esports", "LNG"),
                    ("ThunderTalk Gaming", "TT"),
                    ("Invictus Gaming", "IG"),
                    ("Ultra Prime", "UP"),
                    ("Anyone's Legend", "AL"),
                    ("Ninjas in Pyjamas", "NIP"),
                    ("Mercury Rising", "MR"),
                    ("EDward Gaming", "EDG"),
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
                    ("DRX", "DRX"),
                    ("DWG KIA", "DK"),
                    ("KT Rolster", "KT"),
                    ("Kwangdong Freecs", "KF"),
                    ("Liiv SANDBOX", "SB"),
                    ("OK BRION", "BRO"),
                    ("Nongshim RedForce", "NS"),
                    ("BNK FearX", "BNK"),
                    ("FearX", "FX"),
                    ("Longzhu Gaming", "LZ"),
                    ("Afreeca Freecs", "AF"),
                ],
            },
            RegionConfig {
                id: 3,
                name: "LEC",
                short_name: "EU",
                team_names: vec![
                    ("Fnatic", "FNC"),
                    ("Team Heretics", "TH"),
                    ("MAD Lions", "MAD"),
                    ("G2 Esports", "G2"),
                    ("Falcons", "FAL"),
                    ("Team Whales", "TW"),
                    ("AmBear", "AMB"),
                    ("Misfits Gaming", "MSF"),
                    ("Team Wolf", "WLF"),
                    ("Nike Esports", "NKE"),
                    ("Astralis", "AST"),
                    ("Team Vitality", "VIT"),
                    ("Excel Esports", "XL"),
                    ("SK Gaming", "SK"),
                ],
            },
            RegionConfig {
                id: 4,
                name: "LCS",
                short_name: "NA",
                team_names: vec![
                    ("Frost Quake", "FQ"),
                    ("100 Thieves", "100T"),
                    ("Cloud9", "C9"),
                    ("Team Liquid", "TL"),
                    ("NRG Esports", "NRG"),
                    ("Dignitas", "DIG"),
                    ("Evil Geniuses", "EG"),
                    ("Shopify Rebellion", "SR"),
                    ("TSM", "TSM"),
                    ("EU-Bear", "EUB"),
                    ("SA-SY", "SASY"),
                    ("Immortals", "IMT"),
                    ("Counter Logic Gaming", "CLG"),
                    ("Logic Gaming", "LG"),
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

    /// 生成选手（使用真实数据或随机生成）
    fn generate_players_for_team(
        team_id: u64,
        team_short_name: &str,
        team_power: f64,
        current_season: u32,
    ) -> Vec<Player> {
        let real_players = get_team_players(team_short_name);

        if !real_players.is_empty() {
            // 使用真实选手数据
            Self::create_players_from_config(team_id, &real_players, current_season)
        } else {
            // 回退到随机生成
            Self::generate_random_players(team_id, team_power, current_season)
        }
    }

    /// 从配置创建选手
    fn create_players_from_config(
        team_id: u64,
        configs: &[PlayerConfig],
        current_season: u32,
    ) -> Vec<Player> {
        let mut rng = rand::thread_rng();

        configs.iter().map(|config| {
            let tag = Self::determine_player_tag(config.ability, config.potential, config.age);
            let salary = Self::calculate_initial_salary(config.ability, config.potential, tag);

            Player {
                id: 0,
                game_id: config.game_id.to_string(),
                real_name: config.real_name.map(|s| s.to_string()),
                nationality: Some(config.nationality.to_string()),
                age: config.age,
                ability: config.ability,
                potential: config.potential,
                stability: Player::calculate_stability(config.age),
                tag,
                status: PlayerStatus::Active,
                position: Some(config.position),
                team_id: Some(team_id),
                salary,
                market_value: Self::calculate_market_value(config.ability, config.potential, config.age),
                contract_end_season: Some(current_season + rng.gen_range(1..4)),
                join_season: current_season,
                retire_season: None,
                is_starter: config.is_starter,
            }
        }).collect()
    }

    /// 从配置创建单个选手
    fn create_player_from_config(
        team_id: u64,
        config: &PlayerConfig,
        current_season: u32,
    ) -> Player {
        let mut rng = StdRng::from_entropy();
        let tag = Self::determine_player_tag(config.ability, config.potential, config.age);
        let salary = Self::calculate_initial_salary(config.ability, config.potential, tag);

        Player {
            id: 0,
            game_id: config.game_id.to_string(),
            real_name: config.real_name.map(|s| s.to_string()),
            nationality: Some(config.nationality.to_string()),
            age: config.age,
            ability: config.ability,
            potential: config.potential,
            stability: Player::calculate_stability(config.age),
            tag,
            status: PlayerStatus::Active,
            position: Some(config.position),
            team_id: Some(team_id),
            salary,
            market_value: Self::calculate_market_value(config.ability, config.potential, config.age),
            contract_end_season: Some(current_season + rng.gen_range(1..4)),
            join_season: current_season,
            retire_season: None,
            is_starter: config.is_starter,
        }
    }

    /// 随机生成选手（备用方案）
    fn generate_random_players(
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
                    id: 0,
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

    /// 计算初始薪资（返回实际金额，单位：元）
    fn calculate_initial_salary(ability: u8, _potential: u8, tag: PlayerTag) -> u64 {
        // base 单位为万元
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

        // 转换为实际金额（乘以10000）
        (((base as f64) * tag_multiplier) as u64) * 10000
    }

    /// 计算市场价值（返回实际金额，单位：元）
    fn calculate_market_value(ability: u8, potential: u8, age: u8) -> u64 {
        // base 单位为万元
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

        // 转换为实际金额（乘以10000）
        (((base as f64) * age_factor * potential_bonus) as u64) * 10000
    }

    /// 初始化所有数据
    pub async fn initialize_game_data(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
    ) -> Result<(), String> {
        let regions = Self::get_regions();
        let mut region_ids: Vec<u64> = Vec::new();

        for region in &regions {
            // 创建赛区并获取实际的 ID
            let result = sqlx::query(
                "INSERT INTO regions (save_id, name, short_name, team_count) VALUES (?, ?, ?, ?)"
            )
            .bind(save_id)
            .bind(region.name)
            .bind(region.short_name)
            .bind(14i64)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // 获取实际插入的 region_id
            let actual_region_id = result.last_insert_rowid() as u64;
            region_ids.push(actual_region_id);

            // 创建队伍
            for (rank, (name, short_name)) in region.team_names.iter().enumerate() {
                let power = Self::generate_team_power(region.id, rank);
                let balance = Self::generate_team_balance(power);

                let team = Team {
                    id: 0,
                    region_id: actual_region_id,  // 使用实际的 region_id
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
                let player_configs = get_team_players(short_name);
                let mut rng = StdRng::from_entropy();

                if !player_configs.is_empty() {
                    // 使用配置数据创建选手
                    for config in &player_configs {
                        let player = Self::create_player_from_config(team_id, config, current_season);
                        let player_id = PlayerRepository::create(pool, save_id, &player)
                            .await
                            .map_err(|e| e.to_string())?;

                        // 保存配置中的特性
                        for trait_type in config.traits {
                            let trait_str = serde_json::to_string(trait_type)
                                .map(|s| s.trim_matches('"').to_string())
                                .unwrap_or_else(|_| format!("{:?}", trait_type).to_lowercase());

                            sqlx::query(
                                "INSERT INTO player_traits (save_id, player_id, trait_type) VALUES (?, ?, ?)"
                            )
                            .bind(save_id)
                            .bind(player_id as i64)
                            .bind(&trait_str)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("Failed to insert trait for player {}: {}", player_id, e))?;
                        }

                        // 初始化状态因子
                        let form_cycle = rng.gen_range(0.0..100.0);
                        sqlx::query(
                            r#"
                            INSERT INTO player_form_factors (
                                save_id, player_id, form_cycle, momentum,
                                last_performance, last_match_won, games_since_rest
                            ) VALUES (?, ?, ?, 0, 0.0, 1, 0)
                            "#
                        )
                        .bind(save_id)
                        .bind(player_id as i64)
                        .bind(form_cycle)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("Failed to init form factors for player {}: {}", player_id, e))?;
                    }
                } else {
                    // 回退到随机生成选手
                    let players = Self::generate_random_players(team_id, power, current_season);
                    for player in players {
                        let player_id = PlayerRepository::create(pool, save_id, &player)
                            .await
                            .map_err(|e| e.to_string())?;

                        // 初始化状态因子
                        let form_cycle = rng.gen_range(0.0..100.0);
                        sqlx::query(
                            r#"
                            INSERT INTO player_form_factors (
                                save_id, player_id, form_cycle, momentum,
                                last_performance, last_match_won, games_since_rest
                            ) VALUES (?, ?, ?, 0, 0.0, 1, 0)
                            "#
                        )
                        .bind(save_id)
                        .bind(player_id as i64)
                        .bind(form_cycle)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("Failed to init form factors for player {}: {}", player_id, e))?;
                    }
                }
            }
        }

        // 创建初始赛事（春季常规赛）
        Self::create_initial_tournaments(pool, save_id, current_season, &region_ids).await?;

        // 创建初始选秀池（四大赛区各50人）
        Self::create_initial_draft_pool(pool, save_id, current_season, &region_ids).await?;

        Ok(())
    }

    /// 创建初始赛事（完整赛季）
    async fn create_initial_tournaments(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
        region_ids: &[u64],
    ) -> Result<(), String> {
        let region_names = ["LPL", "LCK", "LEC", "LCS"];
        let league_service = LeagueService::new();

        // 1. 创建各赛区的联赛赛事
        for (idx, &region_id) in region_ids.iter().enumerate() {
            let region_name = region_names.get(idx).unwrap_or(&"LPL");

            // 春季常规赛 (进行中，需要生成赛程)
            let spring_regular = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                name: format!("S{} {} 春季赛", current_season, region_name),
                tournament_type: TournamentType::SpringRegular,
                season_id: current_season as u64,
                region_id: Some(region_id),
                status: TournamentStatus::InProgress,
                current_stage: Some("regular".to_string()),
                current_round: Some(1),
            };

            let spring_regular_id = TournamentRepository::create(pool, save_id, &spring_regular)
                .await
                .map_err(|e| e.to_string())?;

            // 获取赛区队伍并生成春季赛赛程
            let teams = TeamRepository::get_by_region(pool, save_id, region_id)
                .await
                .map_err(|e| e.to_string())?;

            let spring_matches = league_service.generate_regular_schedule(spring_regular_id, &teams);
            MatchRepository::create_batch(pool, save_id, &spring_matches)
                .await
                .map_err(|e| e.to_string())?;

            // 初始化积分榜 (每个队伍的初始积分为0)
            let initial_standings: Vec<LeagueStanding> = teams.iter().enumerate().map(|(idx, team)| {
                LeagueStanding {
                    id: 0,
                    tournament_id: spring_regular_id,
                    team_id: team.id,
                    rank: Some((idx + 1) as u32),
                    matches_played: 0,
                    wins: 0,
                    losses: 0,
                    points: 0,
                    games_won: 0,
                    games_lost: 0,
                    game_diff: 0,
                }
            }).collect();
            StandingRepository::upsert_batch(pool, save_id, &initial_standings)
                .await
                .map_err(|e| e.to_string())?;

            // 春季季后赛 (待开始)
            let spring_playoffs = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                name: format!("S{} {} 春季季后赛", current_season, region_name),
                tournament_type: TournamentType::SpringPlayoffs,
                season_id: current_season as u64,
                region_id: Some(region_id),
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };
            TournamentRepository::create(pool, save_id, &spring_playoffs)
                .await
                .map_err(|e| e.to_string())?;

            // 夏季常规赛 (待开始)
            let summer_regular = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                name: format!("S{} {} 夏季赛", current_season, region_name),
                tournament_type: TournamentType::SummerRegular,
                season_id: current_season as u64,
                region_id: Some(region_id),
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };
            TournamentRepository::create(pool, save_id, &summer_regular)
                .await
                .map_err(|e| e.to_string())?;

            // 夏季季后赛 (待开始)
            let summer_playoffs = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                name: format!("S{} {} 夏季季后赛", current_season, region_name),
                tournament_type: TournamentType::SummerPlayoffs,
                season_id: current_season as u64,
                region_id: Some(region_id),
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };
            TournamentRepository::create(pool, save_id, &summer_playoffs)
                .await
                .map_err(|e| e.to_string())?;
        }

        // 2. 创建国际赛事 (无赛区，待开始)
        let international_tournaments = vec![
            (TournamentType::Msi, format!("S{} MSI季中冠军赛", current_season)),
            (TournamentType::MadridMasters, format!("S{} 马德里大师赛", current_season)),
            (TournamentType::ClaudeIntercontinental, format!("S{} Claude洲际赛", current_season)),
            (TournamentType::WorldChampionship, format!("S{} 全球总决赛", current_season)),
            (TournamentType::ShanghaiMasters, format!("S{} 上海大师赛", current_season)),
            (TournamentType::IcpIntercontinental, format!("S{} ICP四赛区洲际对抗赛", current_season)),
            (TournamentType::SuperIntercontinental, format!("S{} Super洲际年度邀请赛", current_season)),
        ];

        for (tournament_type, name) in international_tournaments {
            let tournament = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                name,
                tournament_type,
                season_id: current_season as u64,
                region_id: None, // 国际赛事无赛区
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };
            TournamentRepository::create(pool, save_id, &tournament)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 创建初始选秀池（四大赛区各50人）
    async fn create_initial_draft_pool(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
        region_ids: &[u64],
    ) -> Result<(), String> {
        // 赛区 ID 映射: region_ids[0]=LPL(1), region_ids[1]=LCK(2), region_ids[2]=LEC(3), region_ids[3]=LCS(4)
        for (idx, &region_id) in region_ids.iter().enumerate() {
            let config_region_id = (idx + 1) as u64; // 1=LPL, 2=LCK, 3=LEC, 4=LCS
            let draft_players = get_draft_pool(config_region_id);
            let nationality = get_region_nationality(config_region_id);

            for (rank, player_config) in draft_players.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO draft_players (
                        save_id, season_id, region_id, game_id, real_name, nationality,
                        age, ability, potential, position, tag, draft_rank, is_picked
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
                    "#
                )
                .bind(save_id)
                .bind(current_season as i64)
                .bind(region_id as i64)
                .bind(player_config.game_id)
                .bind(player_config.real_name)
                .bind(nationality)
                .bind(player_config.age as i64)
                .bind(player_config.ability as i64)
                .bind(player_config.potential as i64)
                .bind(player_config.position)
                .bind(player_config.tag)
                .bind((rank + 1) as i64) // draft_rank 从 1 开始
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to insert draft player {}: {}", player_config.game_id, e))?;
            }
        }

        Ok(())
    }
}

