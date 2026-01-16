//! 身价计算引擎
//!
//! 计算选手身价，包含荣誉系数计算

use std::collections::HashMap;

/// 赛事类型（用于荣誉加成计算）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TournamentType {
    /// 世界赛
    Worlds,
    /// MSI
    Msi,
    /// 春季赛
    SpringRegular,
    SpringPlayoffs,
    /// 夏季赛
    SummerRegular,
    SummerPlayoffs,
    /// 其他
    Other,
}

impl TournamentType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "worlds" | "world_championship" => TournamentType::Worlds,
            "msi" | "mid_season_invitational" => TournamentType::Msi,
            "springregular" | "spring_regular" => TournamentType::SpringRegular,
            "springplayoffs" | "spring_playoffs" => TournamentType::SpringPlayoffs,
            "summerregular" | "summer_regular" => TournamentType::SummerRegular,
            "summerplayoffs" | "summer_playoffs" => TournamentType::SummerPlayoffs,
            _ => TournamentType::Other,
        }
    }

    /// 是否是国际赛事
    pub fn is_international(&self) -> bool {
        matches!(self, TournamentType::Worlds | TournamentType::Msi)
    }

    /// 是否是赛区赛事
    pub fn is_regional(&self) -> bool {
        matches!(
            self,
            TournamentType::SpringRegular
                | TournamentType::SpringPlayoffs
                | TournamentType::SummerRegular
                | TournamentType::SummerPlayoffs
        )
    }
}

/// 荣誉类型（用于身价计算）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HonorCategory {
    // 冠军类
    WorldsChampion,
    WorldsRunnerUp,
    WorldsSemiFinal,
    MsiChampion,
    MsiRunnerUp,
    RegionalChampion,
    RegionalRunnerUp,

    // MVP类
    WorldsMvp,
    MsiMvp,
    RegionalMvp,

    // 最佳阵容
    AllProFirst,
    AllProSecond,
    AllProThird,

    // 新秀
    RookieOfTheYear,
}

impl HonorCategory {
    /// 获取荣誉加成值
    pub fn value_bonus(&self) -> f64 {
        match self {
            // 冠军类（永久）
            HonorCategory::WorldsChampion => 0.8,
            HonorCategory::WorldsRunnerUp => 0.4,
            HonorCategory::WorldsSemiFinal => 0.2,
            HonorCategory::MsiChampion => 0.5,
            HonorCategory::MsiRunnerUp => 0.25,
            HonorCategory::RegionalChampion => 0.3,
            HonorCategory::RegionalRunnerUp => 0.15,

            // MVP类
            HonorCategory::WorldsMvp => 0.4,
            HonorCategory::MsiMvp => 0.3,
            HonorCategory::RegionalMvp => 0.2,

            // 最佳阵容
            HonorCategory::AllProFirst => 0.25,
            HonorCategory::AllProSecond => 0.15,
            HonorCategory::AllProThird => 0.1,

            // 新秀
            HonorCategory::RookieOfTheYear => 0.2,
        }
    }

    /// 获取荣誉有效期（赛季数，0表示永久）
    pub fn duration_seasons(&self) -> u32 {
        match self {
            // 永久荣誉
            HonorCategory::WorldsChampion => 0,
            HonorCategory::WorldsRunnerUp => 0,
            HonorCategory::WorldsSemiFinal => 0,
            HonorCategory::MsiChampion => 0,
            HonorCategory::MsiRunnerUp => 0,
            HonorCategory::WorldsMvp => 0,
            HonorCategory::MsiMvp => 0,

            // 有时效的荣誉
            HonorCategory::RegionalChampion => 3,
            HonorCategory::RegionalRunnerUp => 2,
            HonorCategory::RegionalMvp => 2,
            HonorCategory::AllProFirst => 2,
            HonorCategory::AllProSecond => 2,
            HonorCategory::AllProThird => 2,
            HonorCategory::RookieOfTheYear => 3,
        }
    }

    /// 是否是永久荣誉
    pub fn is_permanent(&self) -> bool {
        self.duration_seasons() == 0
    }
}

/// 选手荣誉记录
#[derive(Debug, Clone)]
pub struct PlayerHonorRecord {
    pub category: HonorCategory,
    pub season_obtained: u32,
    pub tournament_name: String,
}

impl PlayerHonorRecord {
    pub fn new(category: HonorCategory, season_obtained: u32, tournament_name: &str) -> Self {
        Self {
            category,
            season_obtained,
            tournament_name: tournament_name.to_string(),
        }
    }

    /// 检查荣誉是否仍然有效
    pub fn is_valid(&self, current_season: u32) -> bool {
        if self.category.is_permanent() {
            return true;
        }
        let duration = self.category.duration_seasons();
        current_season <= self.season_obtained + duration
    }

    /// 获取当前有效的加成值（考虑衰减）
    pub fn current_bonus(&self, current_season: u32) -> f64 {
        if !self.is_valid(current_season) {
            return 0.0;
        }
        self.category.value_bonus()
    }
}

/// 身价计算引擎
pub struct MarketValueEngine;

impl MarketValueEngine {
    /// 计算荣誉系数
    ///
    /// 参数：
    /// - honors: 选手的荣誉列表
    /// - current_season: 当前赛季
    ///
    /// 返回值：荣誉系数 (1.0 ~ 3.0)
    pub fn calculate_honor_factor(honors: &[PlayerHonorRecord], current_season: u32) -> f64 {
        let mut total_bonus = 0.0;

        for honor in honors {
            total_bonus += honor.current_bonus(current_season);
        }

        // 基础系数 1.0 + 荣誉加成，上限 3.0
        (1.0 + total_bonus).min(3.0)
    }

    /// 从数据库荣誉类型字符串转换为 HonorCategory
    pub fn parse_honor_category(
        honor_type: &str,
        tournament_type: &str,
    ) -> Option<HonorCategory> {
        let tt = TournamentType::from_str(tournament_type);

        match honor_type.to_lowercase().as_str() {
            "teamchampion" | "team_champion" | "playerchampion" | "player_champion" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsChampion),
                    TournamentType::Msi => Some(HonorCategory::MsiChampion),
                    TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                        Some(HonorCategory::RegionalChampion)
                    }
                    _ => None,
                }
            }
            "teamrunnerup" | "team_runner_up" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsRunnerUp),
                    TournamentType::Msi => Some(HonorCategory::MsiRunnerUp),
                    TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                        Some(HonorCategory::RegionalRunnerUp)
                    }
                    _ => None,
                }
            }
            "teamthird" | "team_third" | "teamfourth" | "team_fourth" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsSemiFinal),
                    _ => None,
                }
            }
            "tournamentmvp" | "tournament_mvp" | "finalsmvp" | "finals_mvp" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsMvp),
                    TournamentType::Msi => Some(HonorCategory::MsiMvp),
                    TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                        Some(HonorCategory::RegionalMvp)
                    }
                    _ => None,
                }
            }
            "regularseasonmvp" | "regular_season_mvp" | "playoffsfmvp" | "playoffs_fmvp" => {
                Some(HonorCategory::RegionalMvp)
            }
            "allprofirst" | "all_pro_first" => Some(HonorCategory::AllProFirst),
            "allprosecond" | "all_pro_second" => Some(HonorCategory::AllProSecond),
            "allprothird" | "all_pro_third" => Some(HonorCategory::AllProThird),
            "rookieoftheyear" | "rookie_of_the_year" => Some(HonorCategory::RookieOfTheYear),
            _ => None,
        }
    }

    /// 批量计算多个选手的荣誉系数
    pub fn calculate_honor_factors_batch(
        player_honors: &HashMap<u64, Vec<PlayerHonorRecord>>,
        current_season: u32,
    ) -> HashMap<u64, f64> {
        player_honors
            .iter()
            .map(|(player_id, honors)| {
                (*player_id, Self::calculate_honor_factor(honors, current_season))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honor_factor_calculation() {
        let honors = vec![
            PlayerHonorRecord::new(HonorCategory::WorldsChampion, 1, "2024 Worlds"),
            PlayerHonorRecord::new(HonorCategory::RegionalChampion, 1, "2024 LPL Spring"),
            PlayerHonorRecord::new(HonorCategory::RegionalMvp, 1, "2024 LPL Spring MVP"),
        ];

        let factor = MarketValueEngine::calculate_honor_factor(&honors, 1);
        // 0.8 + 0.3 + 0.2 = 1.3, 总系数 = 1 + 1.3 = 2.3
        assert!((factor - 2.3).abs() < 0.01);
    }

    #[test]
    fn test_honor_factor_with_decay() {
        let honors = vec![
            PlayerHonorRecord::new(HonorCategory::RegionalChampion, 1, "2024 LPL Spring"),
        ];

        // 赛季1：有效
        let factor1 = MarketValueEngine::calculate_honor_factor(&honors, 1);
        assert!((factor1 - 1.3).abs() < 0.01);

        // 赛季4：有效期3赛季，已过期
        let factor4 = MarketValueEngine::calculate_honor_factor(&honors, 5);
        assert!((factor4 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_honor_factor_cap() {
        // 模拟一个多冠王
        let honors = vec![
            PlayerHonorRecord::new(HonorCategory::WorldsChampion, 1, "2021 Worlds"),
            PlayerHonorRecord::new(HonorCategory::WorldsChampion, 2, "2022 Worlds"),
            PlayerHonorRecord::new(HonorCategory::WorldsChampion, 3, "2023 Worlds"),
            PlayerHonorRecord::new(HonorCategory::MsiChampion, 1, "2021 MSI"),
            PlayerHonorRecord::new(HonorCategory::MsiChampion, 2, "2022 MSI"),
            PlayerHonorRecord::new(HonorCategory::WorldsMvp, 3, "2023 Worlds MVP"),
        ];

        let factor = MarketValueEngine::calculate_honor_factor(&honors, 3);
        // 0.8*3 + 0.5*2 + 0.4 = 2.4 + 1.0 + 0.4 = 3.8, 但上限是 3.0
        assert!((factor - 3.0).abs() < 0.01);
    }
}
