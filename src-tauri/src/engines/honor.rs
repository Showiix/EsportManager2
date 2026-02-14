use std::collections::HashMap;

use crate::models::honor::{Honor, HonorHallData, HonorStats, HonorType};
use crate::models::tournament_result::PlayerTournamentStats;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};

/// 荣誉引擎 - 统一管理所有荣誉记录
pub struct HonorEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallOfFameEntry {
    pub player_id: i64,
    pub player_name: String,
    pub position: String,
    pub region_id: Option<i64>,
    pub induction_season: i64,
    pub total_score: i64,
    pub tier: String,
    pub peak_ability: i64,
    pub career_seasons: i64,
    pub honors_json: String,
}

impl HonorEngine {
    pub fn new() -> Self {
        Self
    }

    // ========== 记录荣誉 ==========

    /// 记录战队冠军
    pub fn create_team_champion(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
    ) -> Honor {
        Honor::new_team_honor(
            save_id,
            HonorType::TeamChampion,
            season_id,
            tournament_id,
            tournament_name,
            tournament_type,
            team_id,
            team_name,
        )
    }

    /// 记录战队亚军
    pub fn create_team_runner_up(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
    ) -> Honor {
        Honor::new_team_honor(
            save_id,
            HonorType::TeamRunnerUp,
            season_id,
            tournament_id,
            tournament_name,
            tournament_type,
            team_id,
            team_name,
        )
    }

    /// 记录战队季军
    pub fn create_team_third(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
    ) -> Honor {
        Honor::new_team_honor(
            save_id,
            HonorType::TeamThird,
            season_id,
            tournament_id,
            tournament_name,
            tournament_type,
            team_id,
            team_name,
        )
    }

    /// 记录战队殿军
    pub fn create_team_fourth(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
    ) -> Honor {
        Honor::new_team_honor(
            save_id,
            HonorType::TeamFourth,
            season_id,
            tournament_id,
            tournament_name,
            tournament_type,
            team_id,
            team_name,
        )
    }

    /// 记录选手冠军荣誉（冠军队成员）
    pub fn create_player_champion(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::PlayerChampion,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            team_id,
            team_name,
            player_id,
            player_name,
            position,
            None,
        )
    }

    /// 记录选手亚军荣誉（亚军队成员）
    pub fn create_player_runner_up(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::PlayerRunnerUp,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            team_id,
            team_name,
            player_id,
            player_name,
            position,
            None,
        )
    }

    /// 记录选手季军荣誉（季军队成员）
    pub fn create_player_third(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::PlayerThird,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            team_id,
            team_name,
            player_id,
            player_name,
            position,
            None,
        )
    }

    /// 记录选手殿军荣誉（殿军队成员）
    pub fn create_player_fourth(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::PlayerFourth,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            team_id,
            team_name,
            player_id,
            player_name,
            position,
            None,
        )
    }

    /// 记录赛事MVP
    pub fn create_tournament_mvp(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        player_stats: &PlayerTournamentStats,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::TournamentMvp,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            player_stats.team_id,
            &player_stats.team_name,
            player_stats.player_id,
            &player_stats.player_name,
            &player_stats.position,
            Some(player_stats.to_honor_stats()),
        )
    }

    /// 记录决赛MVP
    pub fn create_finals_mvp(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        team_id: u64,
        team_name: &str,
        player_id: u64,
        player_name: &str,
        position: &str,
        stats: HonorStats,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::FinalsMvp,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            team_id,
            team_name,
            player_id,
            player_name,
            position,
            Some(stats),
        )
    }

    /// 记录常规赛MVP
    pub fn create_regular_season_mvp(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        player_stats: &PlayerTournamentStats,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::RegularSeasonMvp,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            player_stats.team_id,
            &player_stats.team_name,
            player_stats.player_id,
            &player_stats.player_name,
            &player_stats.position,
            Some(player_stats.to_honor_stats()),
        )
    }

    /// 记录季后赛FMVP
    pub fn create_playoffs_fmvp(
        &self,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_name: &str,
        tournament_type: &str,
        player_stats: &PlayerTournamentStats,
    ) -> Honor {
        Honor::new_player_honor(
            save_id,
            HonorType::PlayoffsFmvp,
            season_id,
            Some(tournament_id),
            tournament_name,
            tournament_type,
            player_stats.team_id,
            &player_stats.team_name,
            player_stats.player_id,
            &player_stats.player_name,
            &player_stats.position,
            Some(player_stats.to_honor_stats()),
        )
    }

    // ========== MVP计算 ==========

    /// 计算赛事MVP（累计影响力最高的选手）
    ///
    /// # Arguments
    /// * `player_performances` - 所有比赛中选手的表现数据
    ///   格式: Vec<(player_id, player_name, team_id, team_name, position, impact_score, is_winner, is_game_mvp)>
    pub fn calculate_tournament_mvp(
        &self,
        player_performances: &[(u64, String, u64, String, String, f64, bool, bool)],
    ) -> Option<PlayerTournamentStats> {
        let mut stats_map: HashMap<u64, PlayerTournamentStats> = HashMap::new();

        for (player_id, player_name, team_id, team_name, position, impact, is_winner, is_mvp) in
            player_performances
        {
            let entry = stats_map.entry(*player_id).or_insert_with(|| {
                PlayerTournamentStats::new(
                    String::new(),
                    0,
                    0,
                    String::new(),
                    *player_id,
                    player_name.clone(),
                    *team_id,
                    team_name.clone(),
                    position.clone(),
                )
            });

            entry.total_impact += impact;
            entry.games_played += 1;
            if *is_winner {
                entry.games_won += 1;
            }
            if *is_mvp {
                entry.game_mvp_count += 1;
            }
        }

        // 找出累计影响力最高的选手
        stats_map.into_values().max_by(|a, b| {
            a.total_impact
                .partial_cmp(&b.total_impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// 计算决赛MVP（决赛中胜方影响力最高的选手）
    ///
    /// # Arguments
    /// * `finals_performances` - 决赛中选手的表现数据
    ///   格式: Vec<(player_id, player_name, team_id, team_name, position, impact_score, is_winner)>
    ///
    /// # Returns
    /// * 只返回胜方队伍中影响力最高的选手
    pub fn calculate_finals_mvp(
        &self,
        finals_performances: &[(u64, String, u64, String, String, f64, bool)],
    ) -> Option<(u64, String, u64, String, String, HonorStats)> {
        let mut stats_map: HashMap<u64, (String, u64, String, String, f64, u32, u32)> =
            HashMap::new();

        // 只统计胜方选手的表现
        for (player_id, player_name, team_id, team_name, position, impact, is_winner) in
            finals_performances
        {
            // 只有胜方选手才计入FMVP候选
            if !is_winner {
                continue;
            }

            let entry = stats_map.entry(*player_id).or_insert_with(|| {
                (
                    player_name.clone(),
                    *team_id,
                    team_name.clone(),
                    position.clone(),
                    0.0,
                    0,
                    0,
                )
            });

            entry.4 += impact; // total_impact
            entry.5 += 1; // games_played
            entry.6 += 1; // wins (胜方选手参与的局都是胜场)
        }

        // 找出累计影响力最高的胜方选手
        stats_map
            .into_iter()
            .max_by(|a, b| {
                a.1 .4
                    .partial_cmp(&b.1 .4)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(
                |(player_id, (name, team_id, team_name, position, impact, games, wins))| {
                    (
                        player_id,
                        name,
                        team_id,
                        team_name,
                        position,
                        HonorStats {
                            total_impact: impact,
                            mvp_count: 0,
                            games_played: games,
                            wins,
                            avg_performance: if games > 0 {
                                impact / games as f64
                            } else {
                                0.0
                            },
                        },
                    )
                },
            )
    }

    // ========== 数据聚合 ==========

    /// 构建荣誉殿堂数据
    pub fn build_honor_hall(&self, all_honors: Vec<Honor>) -> HonorHallData {
        let mut hall = HonorHallData::default();

        for honor in all_honors {
            // 收集冠军
            if honor.honor_type == HonorType::TeamChampion {
                hall.champions.push(honor.clone());

                // 按赛事类型分组
                hall.champions_by_type
                    .entry(honor.tournament_type.clone())
                    .or_default()
                    .push(honor.clone());
            }

            // 收集MVP
            if honor.honor_type.is_mvp_honor() {
                hall.mvps.push(honor);
            }
        }

        // 按时间倒序排列
        hall.champions
            .sort_by(|a, b| b.created_at.cmp(&a.created_at));
        hall.mvps.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        hall
    }

    /// 统计战队冠军数量
    pub fn count_team_champions(&self, honors: &[Honor], team_id: u64) -> u32 {
        honors
            .iter()
            .filter(|h| h.honor_type == HonorType::TeamChampion && h.team_id == Some(team_id))
            .count() as u32
    }

    /// 统计选手冠军数量
    pub fn count_player_champions(&self, honors: &[Honor], player_id: u64) -> u32 {
        honors
            .iter()
            .filter(|h| h.honor_type == HonorType::PlayerChampion && h.player_id == Some(player_id))
            .count() as u32
    }

    /// 统计选手MVP数量
    pub fn count_player_mvps(&self, honors: &[Honor], player_id: u64) -> u32 {
        honors
            .iter()
            .filter(|h| h.honor_type.is_mvp_honor() && h.player_id == Some(player_id))
            .count() as u32
    }

    /// 获取战队所有荣誉
    pub fn get_team_honors(&self, honors: &[Honor], team_id: u64) -> Vec<Honor> {
        honors
            .iter()
            .filter(|h| h.team_id == Some(team_id))
            .cloned()
            .collect()
    }

    /// 获取选手所有荣誉
    pub fn get_player_honors(&self, honors: &[Honor], player_id: u64) -> Vec<Honor> {
        honors
            .iter()
            .filter(|h| h.player_id == Some(player_id))
            .cloned()
            .collect()
    }

    /// 获取赛季所有荣誉
    pub fn get_season_honors(&self, honors: &[Honor], season_id: u64) -> Vec<Honor> {
        honors
            .iter()
            .filter(|h| h.season_id == season_id)
            .cloned()
            .collect()
    }

    /// 获取赛事所有荣誉
    pub fn get_tournament_honors(&self, honors: &[Honor], tournament_id: u64) -> Vec<Honor> {
        honors
            .iter()
            .filter(|h| h.tournament_id == Some(tournament_id))
            .cloned()
            .collect()
    }

    pub async fn evaluate_hall_of_fame(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        season_id: i64,
    ) -> Result<Option<HallOfFameEntry>, String> {
        let player_row = sqlx::query(
            "SELECT game_id, position, home_region_id, age, join_season, ability FROM players WHERE save_id = ? AND id = ? LIMIT 1",
        )
        .bind(save_id)
        .bind(player_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询选手基础信息失败: {}", e))?;

        let player_row = match player_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let player_name: String = player_row.get("game_id");
        let position: String = player_row.get("position");
        let region_id: Option<i64> = player_row.try_get("home_region_id").unwrap_or(None);
        let age: i64 = player_row.get("age");
        let join_season: Option<i64> = player_row.try_get("join_season").unwrap_or(None);
        let current_ability: i64 = player_row.get("ability");

        let honor_rows = sqlx::query(
            "SELECT honor_type, tournament_type FROM honors WHERE save_id = ? AND player_id = ?",
        )
        .bind(save_id)
        .bind(player_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询选手荣誉失败: {}", e))?;

        let mut super_champion_count = 0i64;
        let mut international_champion_count = 0i64;
        let mut league_champion_count = 0i64;
        let mut mvp_count = 0i64;
        let mut annual_mvp_count = 0i64;
        let mut annual_top3_count = 0i64;
        let mut annual_top20_count = 0i64;

        for row in &honor_rows {
            let honor_type: String = row.get("honor_type");
            let tournament_type: String = row.try_get("tournament_type").unwrap_or_default();

            match honor_type.as_str() {
                "PLAYER_CHAMPION" | "PlayerChampion" => match tournament_type.as_str() {
                    "SuperIntercontinental" => super_champion_count += 1,
                    "Msi"
                    | "WorldChampionship"
                    | "MadridMasters"
                    | "ShanghaiMasters"
                    | "ClaudeIntercontinental"
                    | "IcpIntercontinental" => international_champion_count += 1,
                    "SpringPlayoff" | "SummerPlayoff" => league_champion_count += 1,
                    _ => {}
                },
                "TOURNAMENT_MVP" | "TournamentMvp" => mvp_count += 1,
                "FINALS_MVP" | "FinalsMvp" => mvp_count += 1,
                "REGULAR_SEASON_MVP" | "RegularSeasonMvp" => mvp_count += 1,
                "PLAYOFFS_FMVP" | "PlayoffsFmvp" => mvp_count += 1,
                "ANNUAL_MVP" | "AnnualMvp" => annual_mvp_count += 1,
                "ANNUAL_ALL_PRO_1ST" | "AnnualAllPro1st" => annual_top3_count += 1,
                "ANNUAL_ALL_PRO_2ND" | "AnnualAllPro2nd" => annual_top3_count += 1,
                "ANNUAL_ALL_PRO_3RD" | "AnnualAllPro3rd" => annual_top3_count += 1,
                "ANNUAL_TOP20" | "AnnualTop20" => annual_top20_count += 1,
                _ => {}
            }
        }

        let career_seasons = (age - 17).max(0);
        let career_bonus = if career_seasons >= 8 { 5 } else { 0 };

        let peak_stats_row = sqlx::query(
            "SELECT MAX(best_performance) as max_perf FROM player_season_stats WHERE save_id = ? AND player_id = ?",
        )
        .bind(save_id)
        .bind(player_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("查询选手历史巅峰能力失败: {}", e))?;

        let historical_peak = peak_stats_row
            .try_get::<Option<f64>, _>("max_perf")
            .unwrap_or(None)
            .unwrap_or(0.0)
            .round() as i64;
        let peak_ability = current_ability.max(historical_peak);
        let peak_bonus = if peak_ability >= 70 { 5 } else { 0 };

        let super_score = super_champion_count * 20;
        let international_score = international_champion_count * 15;
        let league_score = league_champion_count * 10;
        let mvp_score = mvp_count * 8;
        let annual_mvp_score = annual_mvp_count * 15;
        let annual_top3_score = annual_top3_count * 10;
        let annual_top20_score = annual_top20_count * 5;

        let total_score = super_score
            + international_score
            + league_score
            + mvp_score
            + annual_mvp_score
            + annual_top3_score
            + annual_top20_score
            + career_bonus
            + peak_bonus;

        let tier = if total_score >= 300 {
            "Legend"
        } else if total_score >= 200 {
            "HallOfFame"
        } else {
            return Ok(None);
        };

        let honors_json = serde_json::json!({
            "super_champion_count": super_champion_count,
            "international_champion_count": international_champion_count,
            "league_champion_count": league_champion_count,
            "mvp_count": mvp_count,
            "annual_mvp_count": annual_mvp_count,
            "annual_top3_count": annual_top3_count,
            "annual_top20_count": annual_top20_count,
            "career_bonus": career_bonus,
            "peak_bonus": peak_bonus,
            "super_score": super_score,
            "international_score": international_score,
            "league_score": league_score,
            "mvp_score": mvp_score,
            "annual_mvp_score": annual_mvp_score,
            "annual_top3_score": annual_top3_score,
            "annual_top20_score": annual_top20_score,
            "join_season": join_season
        })
        .to_string();

        Ok(Some(HallOfFameEntry {
            player_id,
            player_name,
            position,
            region_id,
            induction_season: season_id,
            total_score,
            tier: tier.to_string(),
            peak_ability,
            career_seasons,
            honors_json,
        }))
    }
}

impl Default for HonorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_team_champion() {
        let engine = HonorEngine::new();
        let honor =
            engine.create_team_champion("save1", 1, 100, "S1 MSI季中邀请赛", "msi", 1, "T1");

        assert_eq!(honor.honor_type, HonorType::TeamChampion);
        assert_eq!(honor.team_id, Some(1));
        assert_eq!(honor.team_name, Some("T1".to_string()));
        assert_eq!(honor.tournament_type, "msi");
    }

    #[test]
    fn test_calculate_tournament_mvp() {
        let engine = HonorEngine::new();

        let performances = vec![
            (
                1,
                "Faker".to_string(),
                1,
                "T1".to_string(),
                "MID".to_string(),
                15.0,
                true,
                true,
            ),
            (
                1,
                "Faker".to_string(),
                1,
                "T1".to_string(),
                "MID".to_string(),
                12.0,
                true,
                false,
            ),
            (
                2,
                "Chovy".to_string(),
                2,
                "GEN".to_string(),
                "MID".to_string(),
                10.0,
                false,
                false,
            ),
            (
                2,
                "Chovy".to_string(),
                2,
                "GEN".to_string(),
                "MID".to_string(),
                8.0,
                false,
                true,
            ),
        ];

        let mvp = engine.calculate_tournament_mvp(&performances);
        assert!(mvp.is_some());

        let mvp = mvp.unwrap();
        assert_eq!(mvp.player_id, 1);
        assert_eq!(mvp.player_name, "Faker");
        assert_eq!(mvp.total_impact, 27.0);
        assert_eq!(mvp.games_played, 2);
        assert_eq!(mvp.games_won, 2);
        assert_eq!(mvp.game_mvp_count, 1);
    }

    #[test]
    fn test_count_honors() {
        let engine = HonorEngine::new();

        let honors = vec![
            Honor::new_team_honor(
                "save1",
                HonorType::TeamChampion,
                1,
                1,
                "MSI",
                "msi",
                1,
                "T1",
            ),
            Honor::new_team_honor(
                "save1",
                HonorType::TeamChampion,
                1,
                2,
                "Worlds",
                "worlds",
                1,
                "T1",
            ),
            Honor::new_team_honor(
                "save1",
                HonorType::TeamChampion,
                1,
                3,
                "Spring",
                "spring",
                2,
                "GEN",
            ),
            Honor::new_player_honor(
                "save1",
                HonorType::TournamentMvp,
                1,
                Some(1),
                "MSI",
                "msi",
                1,
                "T1",
                1,
                "Faker",
                "MID",
                None,
            ),
            Honor::new_player_honor(
                "save1",
                HonorType::FinalsMvp,
                1,
                Some(2),
                "Worlds",
                "worlds",
                1,
                "T1",
                1,
                "Faker",
                "MID",
                None,
            ),
        ];

        assert_eq!(engine.count_team_champions(&honors, 1), 2);
        assert_eq!(engine.count_team_champions(&honors, 2), 1);
        assert_eq!(engine.count_player_mvps(&honors, 1), 2);
    }
}
