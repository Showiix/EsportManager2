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

    // 大师赛/洲际赛
    MastersChampion,
    IntercontinentalChampion,

    // 其他名次
    OtherRunnerUp,
    ThirdPlace,
    FourthPlace,
    RegularSeasonFirst,

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

    // 年度奖项
    AnnualMvp,
    AnnualTop2to3,
    AnnualTop4to5,
    AnnualTop6to10,
    AnnualTop11to20,
    AnnualAllPro1st,
    AnnualAllPro2nd,
    AnnualAllPro3rd,
    AnnualMostConsistent,
    AnnualMostDominant,
    AnnualRookie,
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

            // 大师赛/洲际赛
            HonorCategory::MastersChampion => 0.30,
            HonorCategory::IntercontinentalChampion => 0.25,

            // 其他名次
            HonorCategory::OtherRunnerUp => 0.10,
            HonorCategory::ThirdPlace => 0.06,
            HonorCategory::FourthPlace => 0.04,
            HonorCategory::RegularSeasonFirst => 0.08,

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

            // 年度奖项
            HonorCategory::AnnualMvp => 0.35,
            HonorCategory::AnnualTop2to3 => 0.18,
            HonorCategory::AnnualTop4to5 => 0.15,
            HonorCategory::AnnualTop6to10 => 0.10,
            HonorCategory::AnnualTop11to20 => 0.05,
            HonorCategory::AnnualAllPro1st => 0.15,
            HonorCategory::AnnualAllPro2nd => 0.10,
            HonorCategory::AnnualAllPro3rd => 0.06,
            HonorCategory::AnnualMostConsistent => 0.08,
            HonorCategory::AnnualMostDominant => 0.12,
            HonorCategory::AnnualRookie => 0.12,
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
            HonorCategory::MastersChampion => 2,
            HonorCategory::IntercontinentalChampion => 2,
            HonorCategory::OtherRunnerUp => 2,
            HonorCategory::ThirdPlace => 1,
            HonorCategory::FourthPlace => 1,
            HonorCategory::RegularSeasonFirst => 1,
            HonorCategory::RegionalMvp => 2,
            HonorCategory::AllProFirst => 2,
            HonorCategory::AllProSecond => 2,
            HonorCategory::AllProThird => 2,
            HonorCategory::RookieOfTheYear => 3,

            // 年度奖项
            HonorCategory::AnnualMvp => 3,
            HonorCategory::AnnualTop2to3 => 2,
            HonorCategory::AnnualTop4to5 => 2,
            HonorCategory::AnnualTop6to10 => 1,
            HonorCategory::AnnualTop11to20 => 1,
            HonorCategory::AnnualAllPro1st => 2,
            HonorCategory::AnnualAllPro2nd => 2,
            HonorCategory::AnnualAllPro3rd => 1,
            HonorCategory::AnnualMostConsistent => 1,
            HonorCategory::AnnualMostDominant => 2,
            HonorCategory::AnnualRookie => 2,
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
        let base = self.category.value_bonus();
        if self.category.is_permanent() {
            // 永久荣誉每5赛季衰减10%，最低保留50%
            let years_ago = current_season.saturating_sub(self.season_obtained);
            let decay = 1.0 - (years_ago as f64 * 0.02).min(0.5);
            return base * decay;
        }
        // 有时效荣誉：最后一个有效赛季半衰
        let duration = self.category.duration_seasons();
        if current_season == self.season_obtained + duration {
            base * 0.5
        } else {
            base
        }
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

        // 基础系数 1.0 + 荣誉加成，上限 4.0
        (1.0 + total_bonus).min(4.0)
    }

    /// 从数据库荣誉类型字符串转换为 HonorCategory
    pub fn parse_honor_category(
        honor_type: &str,
        tournament_type: &str,
        tournament_name: &str,
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
                    _ => {
                        // 通过 tournament_name 区分大师赛/洲际赛
                        let name_lower = tournament_name.to_lowercase();
                        if name_lower.contains("大师赛") || name_lower.contains("masters") {
                            Some(HonorCategory::MastersChampion)
                        } else if name_lower.contains("洲际") || name_lower.contains("intercontinental")
                            || name_lower.contains("icp") || name_lower.contains("super")
                            || name_lower.contains("claude")
                        {
                            Some(HonorCategory::IntercontinentalChampion)
                        } else {
                            Some(HonorCategory::RegionalChampion)
                        }
                    }
                }
            }
            "teamrunnerup" | "team_runner_up" | "playerrunnerup" | "player_runner_up" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsRunnerUp),
                    TournamentType::Msi => Some(HonorCategory::MsiRunnerUp),
                    TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                        Some(HonorCategory::RegionalRunnerUp)
                    }
                    _ => Some(HonorCategory::OtherRunnerUp),
                }
            }
            "teamthird" | "team_third" | "playerthird" | "player_third" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsSemiFinal),
                    _ => Some(HonorCategory::ThirdPlace),
                }
            }
            "teamfourth" | "team_fourth" | "playerfourth" | "player_fourth" => {
                Some(HonorCategory::FourthPlace)
            }
            "regularseasonmvp" | "regular_season_mvp" | "playoffsfmvp" | "playoffs_fmvp" => {
                Some(HonorCategory::RegionalMvp)
            }
            "regularseasonfirst" | "regular_season_first" => {
                Some(HonorCategory::RegularSeasonFirst)
            }
            "tournamentmvp" | "tournament_mvp" | "finalsmvp" | "finals_mvp" => {
                match tt {
                    TournamentType::Worlds => Some(HonorCategory::WorldsMvp),
                    TournamentType::Msi => Some(HonorCategory::MsiMvp),
                    TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                        Some(HonorCategory::RegionalMvp)
                    }
                    _ => Some(HonorCategory::RegionalMvp),
                }
            }
            "allprofirst" | "all_pro_first" => Some(HonorCategory::AllProFirst),
            "allprosecond" | "all_pro_second" => Some(HonorCategory::AllProSecond),
            "allprothird" | "all_pro_third" => Some(HonorCategory::AllProThird),
            "rookieoftheyear" | "rookie_of_the_year" => Some(HonorCategory::RookieOfTheYear),
            // 年度奖项
            "annualmvp" | "annual_mvp" => Some(HonorCategory::AnnualMvp),
            "annualtop20" | "annual_top20" => {
                // 从 tournament_name 提取排名（如 "年度Top5"）
                let rank = tournament_name
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(20);
                match rank {
                    1 => Some(HonorCategory::AnnualMvp),
                    2..=3 => Some(HonorCategory::AnnualTop2to3),
                    4..=5 => Some(HonorCategory::AnnualTop4to5),
                    6..=10 => Some(HonorCategory::AnnualTop6to10),
                    _ => Some(HonorCategory::AnnualTop11to20),
                }
            }
            "annualallpro1st" | "annual_all_pro_1st" => Some(HonorCategory::AnnualAllPro1st),
            "annualallpro2nd" | "annual_all_pro_2nd" => Some(HonorCategory::AnnualAllPro2nd),
            "annualallpro3rd" | "annual_all_pro_3rd" => Some(HonorCategory::AnnualAllPro3rd),
            "annualmostconsistent" | "annual_most_consistent" => Some(HonorCategory::AnnualMostConsistent),
            "annualmostdominant" | "annual_most_dominant" => Some(HonorCategory::AnnualMostDominant),
            "annualrookie" | "annual_rookie" => Some(HonorCategory::AnnualRookie),
            // 兼容旧存档
            "annualbesttop" | "annual_best_top" | "annualbestjungle" | "annual_best_jungle"
            | "annualbestmid" | "annual_best_mid" | "annualbestadc" | "annual_best_adc"
            | "annualbestsupport" | "annual_best_support" => Some(HonorCategory::AnnualAllPro1st),
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

    // ==================== 基础身价计算 ====================

    /// 能力系数 — 分段线性插值，消除断层
    pub fn ability_multiplier(ability: u8) -> f64 {
        let a = ability as f64;
        match ability {
            72..=100 => 25.0 + (a - 72.0) * (60.0 - 25.0) / (100.0 - 72.0),
            68..=71 => 18.0 + (a - 68.0) * (25.0 - 18.0) / (72.0 - 68.0),
            65..=67 => 10.0 + (a - 65.0) * (18.0 - 10.0) / (68.0 - 65.0),
            62..=64 => 6.0 + (a - 62.0) * (10.0 - 6.0) / (65.0 - 62.0),
            60..=61 => 4.0 + (a - 60.0) * (6.0 - 4.0) / (62.0 - 60.0),
            55..=59 => 2.0 + (a - 55.0) * (4.0 - 2.0) / (60.0 - 55.0),
            47..=54 => 1.0 + (a - 47.0) * (2.0 - 1.0) / (55.0 - 47.0),
            _ => 1.0,
        }
    }

    /// 年龄系数 — 分段线性插值，消除突变
    pub fn age_factor(age: u8) -> f64 {
        let a = age as f64;
        if a <= 19.0 { 1.5 }
        else if a <= 22.0 { 1.5 - (a - 19.0) * (1.5 - 1.3) / 3.0 }
        else if a <= 25.0 { 1.3 - (a - 22.0) * (1.3 - 1.0) / 3.0 }
        else if a <= 27.0 { 1.0 - (a - 25.0) * (1.0 - 0.85) / 2.0 }
        else if a <= 29.0 { 0.85 - (a - 27.0) * (0.85 - 0.7) / 2.0 }
        else if a <= 32.0 { 0.7 - (a - 29.0) * (0.7 - 0.5) / 3.0 }
        else { 0.5 }
    }

    /// 潜力系数
    pub fn potential_factor(ability: u8, potential: u8) -> f64 {
        let diff = potential.saturating_sub(ability);
        if diff > 10 { 1.25 }
        else if diff >= 5 { 1.1 }
        else { 1.0 }
    }

    /// 天赋标签系数（字符串版本）
    pub fn tag_factor(tag: &str) -> f64 {
        match tag.to_uppercase().as_str() {
            "GENIUS" => 1.2,
            "NORMAL" => 1.0,
            _ => 0.9,
        }
    }

    /// 天赋标签系数（枚举版本）
    pub fn tag_factor_enum(tag: &crate::models::PlayerTag) -> f64 {
        tag.market_value_factor()
    }

    /// 位置系数（字符串版本）
    pub fn position_factor(position: &str) -> f64 {
        match position.to_uppercase().as_str() {
            "MID" => 1.2,
            "ADC" | "BOT" => 1.15,
            "JUG" | "JUNGLE" => 1.1,
            "TOP" => 1.0,
            "SUP" | "SUPPORT" => 0.9,
            _ => 1.0,
        }
    }

    /// 位置系数（枚举版本）
    pub fn position_factor_enum(pos: &crate::models::Position) -> f64 {
        pos.market_value_factor()
    }

    /// 赛区系数
    pub fn region_factor(region_code: &str) -> f64 {
        match region_code.to_uppercase().as_str() {
            "LPL" => 1.3,
            "LCK" => 1.2,
            "LEC" => 1.0,
            "LCS" => 0.9,
            _ => 0.8,
        }
    }

    /// 基础身价（不含荣誉和赛区），字符串参数版本
    pub fn calculate_base_market_value(
        ability: u8, age: u8, potential: u8, tag: &str, position: &str,
    ) -> u64 {
        let base = ability as f64 * Self::ability_multiplier(ability);
        (base * Self::age_factor(age)
            * Self::potential_factor(ability, potential)
            * Self::tag_factor(tag)
            * Self::position_factor(position)
            * 10000.0) as u64
    }

    /// 基础身价（不含荣誉和赛区），枚举参数版本
    pub fn calculate_base_market_value_enum(
        ability: u8, age: u8, potential: u8,
        tag: &crate::models::PlayerTag, position: &crate::models::Position,
    ) -> u64 {
        let base = ability as f64 * Self::ability_multiplier(ability);
        (base * Self::age_factor(age)
            * Self::potential_factor(ability, potential)
            * Self::tag_factor_enum(tag)
            * Self::position_factor_enum(position)
            * 10000.0) as u64
    }

    /// 完整身价（基础 × 荣誉 × 赛区）
    pub fn calculate_full_market_value(
        base_value: u64, honor_factor: f64, region_code: &str,
    ) -> u64 {
        let region_f = Self::region_factor(region_code);
        (base_value as f64 * honor_factor.clamp(1.0, 4.0) * region_f) as u64
    }

    /// 统一薪资估算
    pub fn estimate_salary(market_value: u64, ability: u8, age: u8) -> u64 {
        let ratio = if ability >= 72 { 0.12 }
            else if ability >= 65 { 0.11 }
            else { 0.10 };
        let age_adj = if age <= 22 { 0.9 } else if age >= 28 { 0.95 } else { 1.0 };
        (market_value as f64 * ratio * age_adj) as u64
    }

    /// 赛季表现系数 (0.9 ~ 1.15)
    pub fn performance_factor(
        avg_impact: f64, consistency_score: f64, games_played: u32,
    ) -> f64 {
        if games_played < 10 { return 1.0; }
        let impact_bonus = (avg_impact / 20.0).clamp(-0.05, 0.05);
        let stability_bonus = ((consistency_score - 60.0) / 200.0).clamp(-0.02, 0.03);
        (1.0 + impact_bonus + stability_bonus).clamp(0.9, 1.15)
    }

    /// 特性影响身价系数 (0.9 ~ 1.15)
    pub fn trait_factor(traits: &[crate::engines::TraitType]) -> f64 {
        let mut f: f64 = 1.0;
        for t in traits {
            f += match t {
                crate::engines::TraitType::Clutch => 0.05,
                crate::engines::TraitType::TeamLeader => 0.04,
                crate::engines::TraitType::Consistent => 0.03,
                crate::engines::TraitType::Explosive => 0.02,
                crate::engines::TraitType::Tilter => -0.03,
                crate::engines::TraitType::Fragile => -0.02,
                _ => 0.0,
            };
        }
        f.clamp(0.9, 1.15)
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
        // 上限 4.0
        assert!((factor - 4.0).abs() < 0.01);
    }
}
