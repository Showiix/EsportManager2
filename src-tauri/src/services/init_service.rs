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

    /// 生成初始队伍战力 (基于赛区强度，缩放后)
    fn generate_team_power(region_id: u64, rank: usize) -> f64 {
        let mut rng = rand::thread_rng();

        // 基础战力 (赛区差异，缩放后)
        let region_base = match region_id {
            1 => 60.0, // LPL
            2 => 59.0, // LCK
            3 => 55.0, // LEC
            4 => 54.0, // LCS
            _ => 54.0,
        };

        // 队伍排名差异 (顶级队伍更强，缩放后)
        let rank_bonus = match rank {
            0..=1 => 7.0,
            2..=3 => 5.0,
            4..=5 => 3.0,
            6..=7 => 1.0,
            8..=10 => 0.0,
            _ => -1.0,
        };

        // 随机波动
        let random_factor: f64 = rng.gen_range(-2.0..2.0);

        (region_base + rank_bonus + random_factor).clamp(48.0, 72.0)
    }

    /// 获取队伍初始资金（固定值，单位：元）
    fn get_team_initial_balance(short_name: &str) -> i64 {
        let balance_wan = match short_name {
            // LPL
            "TES" => 8700, "BLG" => 8400, "JDG" => 8850, "WBG" => 8400,
            "RNG" => 8450, "FPX" => 8050, "LNG" => 8350, "TT" => 7350,
            "IG" => 8150, "UP" => 7700, "AL" => 7600, "NIP" => 8050,
            "MR" => 8000, "EDG" => 8550,
            // LCK
            "T1" => 8750, "DRX" => 7900, "DK" => 8250, "GEN" => 8400,
            "KT" => 8050, "KF" => 7650, "SB" => 7700, "BRO" => 7600,
            "NS" => 7550, "BNK" => 7350, "FX" => 7350, "LZ" => 8050,
            "HLE" => 8400, "AF" => 7900,
            // LEC
            "FNC" => 7600, "TH" => 7550, "MAD" => 7950, "G2" => 8200,
            "FAL" => 7500, "TW" => 7150, "AMB" => 7900, "MSF" => 7300,
            "WLF" => 7500, "NKE" => 7500, "AST" => 7150, "VIT" => 7450,
            "XL" => 7450, "SK" => 7000,
            // LCS
            "FQ" => 7250, "100T" => 7800, "C9" => 7900, "TL" => 7700,
            "NRG" => 7350, "DIG" => 7100, "EG" => 7750, "SR" => 7050,
            "TSM" => 7500, "EUB" => 6950, "SASY" => 6800, "IMT" => 7150,
            "CLG" => 7100, "LG" => 6600,
            _ => 7000,
        };
        balance_wan * 10000 // 万元转换为元
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

        // 根据选手属性计算独特的忠诚度和满意度
        let loyalty = Self::calculate_initial_loyalty(config.ability, config.potential, config.age, tag);
        let satisfaction = Self::calculate_initial_satisfaction(config.ability, config.potential, config.age, config.is_starter, tag);

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
            market_value: Self::calculate_market_value(config.ability, config.potential, config.age, tag, config.position),
            calculated_market_value: 0, // 初始化时为0，年度结算时计算
            contract_end_season: Some(current_season + rng.gen_range(1..4)),
            join_season: current_season,
            retire_season: None,
            is_starter: config.is_starter,
            loyalty,
            satisfaction,
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

                let ability = (ability_base.clamp(40.0, 72.0)) as u8;
                let potential = (ability as f64 + rng.gen_range(-5.0..15.0)).clamp(40.0, 75.0) as u8;
                let age = if is_starter {
                    rng.gen_range(19..26)
                } else {
                    rng.gen_range(17..28)
                };

                let tag = Self::determine_player_tag(ability, potential, age);
                let salary = Self::calculate_initial_salary(ability, potential, tag);

                // 根据选手属性计算独特的忠诚度和满意度
                let loyalty = Self::calculate_initial_loyalty(ability, potential, age, tag);
                let satisfaction = Self::calculate_initial_satisfaction(ability, potential, age, is_starter, tag);

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
                    market_value: Self::calculate_market_value(ability, potential, age, tag, *pos),
                    calculated_market_value: 0, // 初始化时为0，年度结算时计算
                    contract_end_season: Some(current_season + rng.gen_range(1..4)),
                    join_season: current_season,
                    retire_season: None,
                    is_starter,
                    loyalty,
                    satisfaction,
                });

                player_counter += 1;
            }
        }

        players
    }

    /// 判定选手标签 (Ordinary=平庸, Normal=一般, Genius=天才)
    /// 缩放后阈值：天才 ability≥68 且 potential≥67；平庸 ability≤59 或 potential≤59；其余为一般
    fn determine_player_tag(ability: u8, potential: u8, _age: u8) -> PlayerTag {
        if ability >= 68 && potential >= 67 {
            PlayerTag::Genius
        } else if ability <= 59 || potential <= 59 {
            PlayerTag::Ordinary
        } else {
            PlayerTag::Normal
        }
    }

    /// 计算初始忠诚度 (基于年龄、能力、潜力等因素)
    /// 返回值范围: 55-95
    ///
    /// 设计原则：大多数选手（70-80%）应该对球队有合理的忠诚度
    /// 只有少数选手会有较低的忠诚度想离队
    fn calculate_initial_loyalty(ability: u8, potential: u8, age: u8, tag: PlayerTag) -> u8 {
        let mut rng = rand::thread_rng();

        // 基础值提高到 72（大多数选手应该有较高忠诚度）
        let mut base: f64 = 72.0;

        // 年龄因素: 调整幅度减小，不让年轻选手过度受罚
        base += match age {
            17..=19 => -3.0,   // 年轻新秀，略有野心
            20..=22 => 0.0,    // 黄金期，正常
            23..=25 => 4.0,    // 成熟期，开始稳定
            26..=28 => 8.0,    // 老将，更看重稳定
            _ => 10.0,         // 高龄，非常忠诚
        };

        // 能力因素: 减小惩罚，顶级选手也可以忠诚（缩放后阈值）
        base += match ability {
            68..=100 => -2.0,  // 顶级选手，略有降低
            65..=67 => 0.0,    // 明星选手，正常
            61..=64 => 2.0,    // 优秀选手，略高
            54..=60 => 3.0,    // 普通选手，珍惜机会
            _ => 4.0,          // 替补选手，更加忠诚
        };

        // 潜力因素: 高潜力年轻人的惩罚减小（缩放后阈值）
        if age <= 21 && potential >= 68 {
            base -= 3.0;  // 只有超高潜力才略有影响
        }

        // 天赋因素: 减小差异
        base += match tag {
            PlayerTag::Genius => -2.0,  // 天才型略低
            PlayerTag::Normal => 0.0,
            PlayerTag::Ordinary => 2.0,  // 普通型略高
        };

        // 添加随机波动 (-6 到 +6)
        let random_factor: f64 = rng.gen_range(-6.0..6.0);
        base += random_factor;

        base.clamp(55.0, 95.0) as u8
    }

    /// 计算初始满意度 (基于能力、是否首发等因素)
    /// 返回值范围: 60-95
    ///
    /// 设计原则：大多数选手（70-80%）应该对当前处境较为满意
    /// 首发选手普遍满意，替补选手有机会稍微不满
    fn calculate_initial_satisfaction(ability: u8, potential: u8, age: u8, is_starter: bool, tag: PlayerTag) -> u8 {
        let mut rng = rand::thread_rng();

        // 基础值提高：首发 78，替补 68
        let mut base: f64 = if is_starter { 78.0 } else { 68.0 };

        // 首发/替补因素：减小惩罚
        if !is_starter {
            // 替补选手如果能力高，满意度会降低（觉得自己应该首发）（缩放后阈值）
            if ability >= 65 {
                base -= 5.0;  // 从 -8 减小到 -5
            } else if ability >= 61 {
                base -= 3.0;  // 从 -4 减小到 -3
            }
        }

        // 年龄因素: 年轻选手对替补更能接受
        if !is_starter {
            base += match age {
                17..=19 => 6.0,   // 年轻，可以接受做替补学习
                20..=22 => 3.0,   // 还年轻，但开始着急
                23..=25 => 0.0,   // 黄金期，想要上场
                _ => -2.0,        // 老了还是替补，不太满意
            };
        }

        // 潜力因素: 减小惩罚（缩放后阈值）
        if !is_starter && potential >= 67 && age <= 21 {
            base -= 3.0;  // 从 -5 减小到 -3
        }

        // 天赋因素: 减小差异
        base += match tag {
            PlayerTag::Genius => 2.0,   // 天才型选手反而更自信满足
            PlayerTag::Normal => 0.0,
            PlayerTag::Ordinary => -1.0, // 普通选手可能略有不安
        };

        // 能力因素: 高能力选手更自信（缩放后阈值）
        base += match ability {
            68..=100 => 5.0,   // 顶级选手，自信满满
            65..=67 => 3.0,
            61..=64 => 1.0,
            54..=60 => 0.0,    // 普通选手，正常
            _ => -2.0,         // 能力较低，可能有些焦虑
        };

        // 添加随机波动 (-5 到 +5)
        let random_factor: f64 = rng.gen_range(-5.0..5.0);
        base += random_factor;

        base.clamp(60.0, 95.0) as u8
    }

    /// 计算初始薪资（单位：元）
    /// 缩放后阈值，薪资基数约为原来的一半
    fn calculate_initial_salary(ability: u8, _potential: u8, tag: PlayerTag) -> u64 {
        // base 单位为万元（基数）
        let base = match ability {
            68..=100 => 75,
            65..=67 => 50,
            62..=64 => 35,
            60..=61 => 25,
            55..=59 => 18,
            _ => 10,
        };

        let tag_multiplier = match tag {
            PlayerTag::Genius => 1.5,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 0.8,
        };

        // 返回元
        ((base as f64) * tag_multiplier * 10000.0) as u64
    }

    /// 计算市场价值（单位：元）
    /// 使用与 Player::calculate_base_market_value 相同的公式
    /// 缩放后阈值
    fn calculate_market_value(ability: u8, potential: u8, age: u8, tag: PlayerTag, position: Position) -> u64 {
        // 基础身价系数（缩放后阈值）
        let multiplier = match ability {
            72..=100 => 25,  // 顶级选手
            68..=71 => 18,   // 世界级
            65..=67 => 10,   // 顶尖
            62..=64 => 6,    // 优秀
            60..=61 => 4,    // 合格首发
            55..=59 => 2,    // 替补级
            47..=54 => 1,    // 新人
            _ => 1,          // 青训
        };

        // 基础身价 = 能力值 × 系数
        let base = ability as u64 * multiplier;

        // 年龄因素
        let age_factor = match age {
            17..=19 => 1.5,  // 超新星溢价
            20..=22 => 1.3,  // 年轻潜力股
            23..=25 => 1.0,  // 黄金年龄
            26..=27 => 0.85, // 巅峰末期
            28..=29 => 0.7,  // 开始下滑
            _ => 0.5,        // 老将或太年轻
        };

        // 潜力加成
        let diff = potential.saturating_sub(ability);
        let potential_factor = if diff > 10 {
            1.25
        } else if diff >= 5 {
            1.1
        } else {
            1.0
        };

        // 天赋系数
        let tag_factor = tag.market_value_factor();

        // 位置系数
        let position_factor = position.market_value_factor();

        // 返回元
        ((base as f64) * age_factor * potential_factor * tag_factor * position_factor * 10000.0) as u64
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
                let balance = Self::get_team_initial_balance(short_name);

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

    /// 创建初始赛事（仅春季常规赛）
    /// 其他赛事（季后赛、夏季赛、国际赛事）通过时间推进引擎动态创建
    async fn create_initial_tournaments(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
        region_ids: &[u64],
    ) -> Result<(), String> {
        let region_names = ["LPL", "LCK", "LEC", "LCS"];
        let league_service = LeagueService::new();

        // 仅创建春季常规赛（四个赛区各一个）
        // 其他赛事通过 GameFlowService::initialize_phase 在时间推进时动态创建
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
        }

        // 注意：春季季后赛、夏季赛、国际赛事等将通过时间推进引擎动态创建
        // 参见 GameFlowService::initialize_phase (game_flow.rs)

        Ok(())
    }

    /// 创建初始选秀池（四大赛区各50人，写入 draft_pool 表）
    async fn create_initial_draft_pool(
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
        region_ids: &[u64],
    ) -> Result<(), String> {
        // 先清空该存档的所有选秀池数据（确保初始化时是干净的）
        sqlx::query("DELETE FROM draft_pool WHERE save_id = ?")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to clear draft pool: {}", e))?;

        // 赛区 ID 映射: region_ids[0]=LPL(1), region_ids[1]=LCK(2), region_ids[2]=LEC(3), region_ids[3]=LCS(4)
        for (idx, &region_id) in region_ids.iter().enumerate() {
            let config_region_id = (idx + 1) as u64; // 1=LPL, 2=LCK, 3=LEC, 4=LCS
            let draft_players = get_draft_pool(config_region_id);
            let nationality = get_region_nationality(config_region_id);

            for player_config in draft_players.iter() {
                sqlx::query(
                    r#"
                    INSERT INTO draft_pool (
                        save_id, region_id, game_id, real_name, nationality,
                        age, ability, potential, position, tag, status, created_season
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'available', ?)
                    "#
                )
                .bind(save_id)
                .bind(region_id as i64)
                .bind(player_config.game_id)
                .bind(player_config.real_name)
                .bind(nationality)
                .bind(player_config.age as i64)
                .bind(player_config.ability as i64)
                .bind(player_config.potential as i64)
                .bind(player_config.position)
                .bind(player_config.tag)
                .bind(current_season as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to insert draft pool player {}: {}", player_config.game_id, e))?;
            }
        }

        Ok(())
    }

    /// 迁移现有选手的忠诚度和满意度
    /// 根据选手属性重新计算并更新数据库
    pub async fn migrate_loyalty_satisfaction(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<u32, String> {
        use sqlx::Row;

        // 获取所有活跃选手
        let rows = sqlx::query(
            r#"
            SELECT id, ability, potential, age, tag, is_starter
            FROM players
            WHERE save_id = ? AND status = 'Active'
            "#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch players: {}", e))?;

        let mut updated_count = 0u32;

        for row in rows {
            let player_id: i64 = row.get("id");
            let ability: i64 = row.get("ability");
            let potential: i64 = row.get("potential");
            let age: i64 = row.get("age");
            let tag_str: String = row.get("tag");
            let is_starter: bool = row.get("is_starter");

            let ability = ability as u8;
            let potential = potential as u8;
            let age = age as u8;

            // 解析 tag
            let tag = match tag_str.as_str() {
                "Genius" => PlayerTag::Genius,
                "Normal" => PlayerTag::Normal,
                _ => PlayerTag::Ordinary,
            };

            // 计算新的忠诚度和满意度
            let loyalty = Self::calculate_initial_loyalty(ability, potential, age, tag);
            let satisfaction = Self::calculate_initial_satisfaction(ability, potential, age, is_starter, tag);

            // 更新数据库
            sqlx::query(
                "UPDATE players SET loyalty = ?, satisfaction = ? WHERE id = ?"
            )
            .bind(loyalty as i64)
            .bind(satisfaction as i64)
            .bind(player_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update player {}: {}", player_id, e))?;

            updated_count += 1;
        }

        Ok(updated_count)
    }
}

