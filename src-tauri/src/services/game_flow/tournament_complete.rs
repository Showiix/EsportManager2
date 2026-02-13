use crate::db::*;
use crate::engines::{FinancialEngine, PointsCalculationEngine};
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::helpers::*;
use super::GameFlowService;

impl GameFlowService {
    /// 获取季后赛前3名队伍（冠亚季军）
    pub(crate) async fn get_playoffs_top3(
        &self,
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<crate::models::Team>, String> {
        use crate::models::Team;
        let mut results: Vec<Team> = Vec::new();

        // 获取总决赛（GRAND_FINAL）
        let grand_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let (champion_id, runner_up_id) = if let Some(gf) = grand_final {
            let winner_id = gf.get::<Option<i64>, _>("winner_id");
            let home_id = gf.get::<i64, _>("home_team_id") as u64;
            let away_id = gf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                (Some(winner), Some(loser))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // 获取败者组决赛失败者（季军）
        let losers_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let third_id = if let Some(lf) = losers_final {
            let winner_id = lf.get::<Option<i64>, _>("winner_id");
            let home_id = lf.get::<i64, _>("home_team_id") as u64;
            let away_id = lf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                Some(loser)
            } else {
                None
            }
        } else {
            None
        };

        // 加载队伍详情
        if let Some(cid) = champion_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, cid).await {
                results.push(team);
            }
        }
        if let Some(rid) = runner_up_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, rid).await {
                results.push(team);
            }
        }
        if let Some(tid) = third_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, tid).await {
                results.push(team);
            }
        }

        log::debug!("tournament_id={}, found {} teams", tournament_id, results.len());
        Ok(results)
    }

    /// 获取季后赛前4名队伍 (用于ICP洲际赛)
    pub(crate) async fn get_playoffs_top4(
        &self,
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<crate::models::Team>, String> {
        use crate::models::Team;
        let mut results: Vec<Team> = Vec::new();

        log::debug!("开始获取 tournament_id={} 的前4名", tournament_id);

        // 获取总决赛（GRAND_FINAL）
        let grand_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let (champion_id, runner_up_id) = if let Some(gf) = grand_final {
            let winner_id = gf.get::<Option<i64>, _>("winner_id");
            let home_id = gf.get::<i64, _>("home_team_id") as u64;
            let away_id = gf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, loser);
                (Some(winner), Some(loser))
            } else {
                log::debug!("GRAND_FINAL 存在但无 winner_id");
                (None, None)
            }
        } else {
            log::debug!("未找到 GRAND_FINAL");
            (None, None)
        };

        // 获取败者组决赛失败者（季军）
        let losers_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let third_id = if let Some(lf) = losers_final {
            let winner_id = lf.get::<Option<i64>, _>("winner_id");
            let home_id = lf.get::<i64, _>("home_team_id") as u64;
            let away_id = lf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("LOSERS_FINAL: third={}", loser);
                Some(loser)
            } else {
                log::debug!("LOSERS_FINAL 存在但无 winner_id");
                None
            }
        } else {
            log::debug!("未找到 LOSERS_FINAL");
            None
        };

        // 获取败者组R3失败者（殿军/第4名）
        // 注意：季后赛的败者组结构是 R1 -> R2 -> R3(1场) -> FINAL
        // LOSERS_R3 只有1场比赛，败者是第4名
        let losers_r3 = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let fourth_id = if let Some(lr3) = losers_r3 {
            let winner_id = lr3.get::<Option<i64>, _>("winner_id");
            let home_id = lr3.get::<i64, _>("home_team_id") as u64;
            let away_id = lr3.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("LOSERS_R3: fourth={}", loser);
                Some(loser)
            } else {
                log::debug!("LOSERS_R3 存在但无 winner_id");
                None
            }
        } else {
            log::debug!("未找到 LOSERS_R3");
            None
        };

        // 加载队伍详情（按排名顺序）
        if let Some(cid) = champion_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, cid).await {
                results.push(team);
            }
        }
        if let Some(rid) = runner_up_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, rid).await {
                results.push(team);
            }
        }
        if let Some(tid) = third_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, tid).await {
                results.push(team);
            }
        }
        if let Some(fid) = fourth_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, fid).await {
                results.push(team);
            }
        }

        log::debug!("tournament_id={}, found {} teams", tournament_id, results.len());
        Ok(results)
    }

    /// 颁发赛事年度积分
    pub(crate) async fn award_tournament_points(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, u32)>, String> {
        let points_engine = PointsCalculationEngine::new();
        let mut awarded: Vec<(u64, u32)> = Vec::new();

        // 根据赛事类型获取排名结果
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        for (team_id, position) in &results {
            let points = points_engine.get_points(tournament_type, position);
            if points > 0 {
                // 保存积分明细（带去重检查）
                let (_, is_new) = PointsRepository::add_points_detail(
                    pool,
                    save_id,
                    season_id,
                    *team_id,
                    tournament_id,
                    points,
                    position_to_rank(position),
                )
                .await
                .map_err(|e| e.to_string())?;

                // 只有新记录才更新队伍的年度积分
                if is_new {
                    let mut team = TeamRepository::get_by_id(pool, *team_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    team.annual_points += points;
                    TeamRepository::update(pool, &team)
                        .await
                        .map_err(|e| e.to_string())?;

                    awarded.push((*team_id, points));
                    log::debug!("Awarded {} points to team {} for position {} in tournament {}",
                        points, team_id, position, tournament_id);
                } else {
                    log::debug!("Skipped duplicate points for team {} in tournament {}", team_id, tournament_id);
                }
            }
        }

        Ok(awarded)
    }

    /// 发放赛事奖金
    pub(crate) async fn distribute_tournament_prizes(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, u64)>, String> {
        // 幂等检查：如果该赛事奖金已发放过，跳过
        let existing: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM financial_transactions WHERE save_id = ? AND related_tournament_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to check existing prizes: {}", e))?;

        if existing.0 > 0 {
            log::debug!("奖金已发放过，跳过: tournament_id={}", tournament_id);
            return Ok(vec![]);
        }

        let financial_engine = FinancialEngine::new();
        let mut distributed: Vec<(u64, u64)> = Vec::new();

        // 获取赛事排名结果
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        for (team_id, position) in &results {
            let prize = financial_engine.calculate_prize_money(tournament_type, position);
            if prize > 0 {
                // 确定交易类型描述
                let transaction_type = if tournament_type.is_regional() {
                    "PlayoffBonus"
                } else {
                    "InternationalBonus"
                };

                let description = format!("{:?} - {} 奖金", tournament_type, position);

                // 记录财务交易（含 related_tournament_id 用于幂等检查）
                sqlx::query(
                    r#"
                    INSERT INTO financial_transactions (
                        save_id, team_id, season_id, transaction_type, amount, description, related_tournament_id
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(save_id)
                .bind(*team_id as i64)
                .bind(season_id as i64)
                .bind(transaction_type)
                .bind(prize as i64)
                .bind(&description)
                .bind(tournament_id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to record prize transaction: {}", e))?;

                // 更新队伍余额
                sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
                    .bind(prize as i64)
                    .bind(*team_id as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update team balance: {}", e))?;

                distributed.push((*team_id, prize));
                log::debug!("Awarded {} prize to team {} for position {} in tournament {}",
                    prize, team_id, position, tournament_id);
            }
        }

        log::debug!("Total {} prizes distributed for tournament {}", distributed.len(), tournament_id);
        Ok(distributed)
    }

    /// 更新冠军/亚军/季军队伍选手的统计数据
    pub(crate) async fn update_champion_player_stats(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
        is_international: bool,
    ) -> Result<(), String> {
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        for (team_id, position) in &results {
            let placement = position.as_str();
            if !matches!(placement, "CHAMPION" | "RUNNER_UP" | "THIRD") {
                continue;
            }

            log::debug!("Updating stats for {} team {} in tournament {}", placement, team_id, tournament_id);

            let players = sqlx::query(
                r#"
                SELECT id, game_id, position FROM players
                WHERE save_id = ? AND team_id = ? AND status = 'Active'
                "#
            )
            .bind(save_id)
            .bind(*team_id as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get team players: {}", e))?;

            for player_row in &players {
                let player_id: i64 = player_row.get("id");
                let player_name: String = player_row.get("game_id");
                let pos: String = player_row.get("position");

                let mut stats = PlayerStatsRepository::get_or_create(
                    pool,
                    save_id,
                    player_id,
                    &player_name,
                    season_id as i64,
                    Some(*team_id as i64),
                    None,
                    &pos
                )
                .await
                .map_err(|e| e.to_string())?;

                match placement {
                    "CHAMPION" => {
                        if is_international {
                            stats.international_titles += 1;
                        } else {
                            stats.regional_titles += 1;
                        }
                        stats.champion_bonus += if is_international { 3.0 } else { 1.0 };
                    }
                    "RUNNER_UP" => {
                        stats.champion_bonus += if is_international { 2.0 } else { 0.5 };
                    }
                    "THIRD" => {
                        stats.champion_bonus += if is_international { 1.0 } else { 0.25 };
                    }
                    _ => {}
                }

                stats.yearly_top_score = PlayerSeasonStatistics::calculate_yearly_top_score(
                    stats.avg_impact,
                    stats.avg_performance,
                    stats.consistency_score,
                    stats.games_played,
                    stats.champion_bonus,
                );
                stats.dominance_score = PlayerSeasonStatistics::calculate_dominance_score(
                    stats.best_performance,
                    stats.avg_impact,
                    stats.avg_performance,
                );

                PlayerStatsRepository::update(pool, &stats)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            log::debug!("Successfully updated {} stats for tournament {}", placement, tournament_id);
        }

        Ok(())
    }

    /// 获取赛事最终排名结果
    pub(crate) async fn get_tournament_final_results(
        &self,
        pool: &Pool<Sqlite>,
        _save_id: &str,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, String)>, String> {
        let mut results: Vec<(u64, String)> = Vec::new();

        match tournament_type {
            // 季后赛：从双败淘汰赛结果获取排名
            // 使用的 stage: WINNERS_R1, LOSERS_R1, WINNERS_FINAL, LOSERS_R2, LOSERS_R3, LOSERS_FINAL, GRAND_FINAL
            TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                // 获取总决赛 (GRAND_FINAL)
                let grand_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(final_match) = grand_final {
                    let winner_id = final_match.get::<Option<i64>, _>("winner_id");
                    let home_id = final_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = final_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let winner = winner as u64;
                        let runner_up = if winner == home_id { away_id } else { home_id };
                        results.push((winner, "CHAMPION".to_string()));
                        results.push((runner_up, "RUNNER_UP".to_string()));
                        log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                    }
                } else {
                    log::debug!("No GRAND_FINAL match found for tournament {}", tournament_id);
                }

                // 获取败者组决赛失败者（季军）
                let losers_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lf_match) = losers_final {
                    let winner_id = lf_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lf_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lf_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        // 败者组决赛的败者是季军
                        let third = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((third, "THIRD".to_string()));
                        log::debug!("LOSERS_FINAL loser (third): {}", third);
                    }
                }

                // 获取败者组第三轮失败者（殿军）
                let losers_r3 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lr3_match) = losers_r3 {
                    let winner_id = lr3_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr3_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr3_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        // 败者组R3的败者是殿军
                        let fourth = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((fourth, "FOURTH".to_string()));
                        log::debug!("LOSERS_R3 loser (fourth): {}", fourth);
                    }
                }

                // 获取败者组第二轮失败者（5-6名）
                let losers_r2 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R2' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr2_match in losers_r2 {
                    let winner_id = lr2_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr2_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr2_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "5TH_8TH".to_string()));
                        log::debug!("LOSERS_R2 loser (5th-8th): {}", loser);
                    }
                }

                // 获取败者组第一轮失败者（7-8名）
                let losers_r1 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R1' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr1_match in losers_r1 {
                    let winner_id = lr1_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr1_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr1_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "5TH_8TH".to_string()));
                        log::debug!("LOSERS_R1 loser (5th-8th): {}", loser);
                    }
                }

                log::debug!("Total results for tournament {}: {:?}", tournament_id, results);
            }

            // MSI - 双败赛制，需要单独处理
            TournamentType::Msi => {
                // 获取总决赛 (GRAND_FINAL)
                let grand_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(final_match) = grand_final {
                    let winner_id = final_match.get::<Option<i64>, _>("winner_id");
                    let home_id = final_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = final_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let winner = winner as u64;
                        let runner_up = if winner == home_id { away_id } else { home_id };
                        results.push((winner, "CHAMPION".to_string()));
                        results.push((runner_up, "RUNNER_UP".to_string()));
                        log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                    }
                }

                // 获取败者组决赛失败者（季军）
                let losers_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lf_match) = losers_final {
                    let winner_id = lf_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lf_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lf_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let third = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((third, "THIRD".to_string()));
                        log::debug!("LOSERS_FINAL loser (third): {}", third);
                    }
                }

                // 获取 LOSERS_R4 失败者（殿军）- MSI的败者组R4只有1场
                let losers_r4 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R4' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lr4_match) = losers_r4 {
                    let winner_id = lr4_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr4_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr4_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let fourth = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((fourth, "FOURTH".to_string()));
                        log::debug!("LOSERS_R4 loser (fourth): {}", fourth);
                    }
                }

                // 获取 LOSERS_R3 失败者（5-6名）
                let losers_r3 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr3_match in losers_r3 {
                    let winner_id = lr3_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr3_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr3_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "LOSERS_R2".to_string())); // 积分配置中是 LOSERS_R2
                        log::debug!("LOSERS_R3 loser: {}", loser);
                    }
                }

                // 获取 LOSERS_R2 失败者（7-8名）
                let losers_r2 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R2' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr2_match in losers_r2 {
                    let winner_id = lr2_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr2_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr2_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "LOSERS_R1".to_string())); // 积分配置中是 LOSERS_R1
                        log::debug!("LOSERS_R2 loser: {}", loser);
                    }
                }

                log::debug!("Total results: {:?}", results);
            }

            // 马德里大师赛/Claude洲际赛 (32队分组+东西半区淘汰)
            TournamentType::MadridMasters | TournamentType::ClaudeIntercontinental => {
                let all_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
                    ORDER BY stage DESC, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 找到总决赛 (GRAND_FINAL)
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "GRAND_FINAL" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                            log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                        }
                        break;
                    }
                }

                // 找到季军赛 (THIRD_PLACE) - 获取季军和殿军
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "THIRD_PLACE" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let loser = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "THIRD".to_string()));
                            results.push((loser, "FOURTH".to_string()));
                            log::debug!("THIRD_PLACE: third={}, fourth={}", winner, loser);
                        }
                        break;
                    }
                }

                // 东西半区半决赛失败者 (EAST_SEMI, WEST_SEMI) -> SEMI_LOSER
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "EAST_SEMI" || stage == "WEST_SEMI" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "SEMI_LOSER".to_string()));
                            log::debug!("{} loser (SEMI_LOSER): {}", stage, loser);
                        }
                    }
                }

                // 东西半区第一轮失败者 (EAST_R1, WEST_R1) -> QUARTER_LOSER
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "EAST_R1" || stage == "WEST_R1" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "QUARTER_LOSER".to_string()));
                            log::debug!("{} loser (QUARTER_LOSER): {}", stage, loser);
                        }
                    }
                }

                log::debug!("Total results: {:?}", results);
            }

            // 其他国际赛事 (标准淘汰赛制)
            TournamentType::WorldChampionship
            | TournamentType::ShanghaiMasters => {
                // 获取淘汰赛阶段的比赛结果
                let knockout_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
                    ORDER BY stage DESC, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 找到决赛
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "FINALS" || stage == "GRAND_FINALS" || stage == "GRAND_FINAL" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                        }
                        break;
                    }
                }

                // 找半决赛失败者
                let mut semi_losers: Vec<u64> = Vec::new();
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "SEMI_FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            semi_losers.push(loser);
                        }
                    }
                }

                if semi_losers.len() >= 2 {
                    results.push((semi_losers[0], "THIRD".to_string()));
                    results.push((semi_losers[1], "FOURTH".to_string()));
                } else if semi_losers.len() == 1 {
                    results.push((semi_losers[0], "THIRD".to_string()));
                }

                // 八强失败者
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "QUARTER_FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "QUARTER_FINAL".to_string()));
                        }
                    }
                }
            }

            // ICP洲际对抗赛 - 按赛区排名分配积分
            TournamentType::IcpIntercontinental => {
                // 1. 获取所有参赛队伍及其赛区
                let team_rows = sqlx::query(
                    r#"
                    SELECT DISTINCT t.id as team_id, t.region_id
                    FROM matches m
                    JOIN teams t ON t.id = m.home_team_id OR t.id = m.away_team_id
                    WHERE m.tournament_id = ?
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 收集所有参赛队伍ID
                let mut participant_team_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
                let mut team_region_map: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();

                for row in &team_rows {
                    let team_id: i64 = row.get("team_id");
                    let region_id: i64 = row.get("region_id");
                    participant_team_ids.insert(team_id as u64);
                    team_region_map.insert(team_id as u64, region_id as u64);
                }

                // 2. 从决赛和半决赛结果确定赛区排名
                let final_matches = sqlx::query(
                    r#"
                    SELECT m.stage, m.home_team_id, m.away_team_id, m.winner_id,
                           ht.region_id as home_region_id, at.region_id as away_region_id
                    FROM matches m
                    LEFT JOIN teams ht ON m.home_team_id = ht.id
                    LEFT JOIN teams at ON m.away_team_id = at.id
                    WHERE m.tournament_id = ? AND (m.stage LIKE 'ICP_FINAL%' OR m.stage LIKE 'ICP_SEMI%')
                    AND UPPER(m.status) = 'COMPLETED'
                    ORDER BY m.stage
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 统计决赛和半决赛的胜场
                let mut final_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
                let mut semi_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
                let mut final_regions: (Option<u64>, Option<u64>) = (None, None);
                let mut semi_regions: Vec<u64> = Vec::new();

                for row in &final_matches {
                    let stage: String = row.get("stage");
                    let winner_id: Option<i64> = row.get("winner_id");
                    let home_region_id: Option<i64> = row.get("home_region_id");
                    let away_region_id: Option<i64> = row.get("away_region_id");

                    if let (Some(winner), Some(home_region), Some(away_region)) = (winner_id, home_region_id, away_region_id) {
                        let home_team_id: Option<i64> = row.get("home_team_id");
                        let winner_region = if home_team_id == Some(winner) {
                            home_region as u64
                        } else {
                            away_region as u64
                        };

                        if stage.starts_with("ICP_FINAL") {
                            *final_wins.entry(winner_region).or_insert(0) += 1;
                            final_regions = (Some(home_region as u64), Some(away_region as u64));
                        } else if stage.starts_with("ICP_SEMI") {
                            *semi_wins.entry(winner_region).or_insert(0) += 1;
                            if !semi_regions.contains(&(home_region as u64)) {
                                semi_regions.push(home_region as u64);
                            }
                            if !semi_regions.contains(&(away_region as u64)) {
                                semi_regions.push(away_region as u64);
                            }
                        }
                    }
                }

                // 确定赛区排名
                let (champion_region, runner_up_region) = {
                    let mut sorted: Vec<_> = final_wins.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));

                    if sorted.len() >= 2 {
                        (*sorted[0].0, *sorted[1].0)
                    } else if sorted.len() == 1 {
                        if let (Some(r1), Some(r2)) = final_regions {
                            if *sorted[0].0 == r1 { (r1, r2) } else { (r2, r1) }
                        } else {
                            log::debug!("[ICP Points] 无法确定冠亚军赛区");
                            return Ok(results);
                        }
                    } else {
                        log::debug!("[ICP Points] 没有决赛结果");
                        return Ok(results);
                    }
                };

                // 第三、第四赛区是半决赛中未进入决赛的赛区
                let (third_region, fourth_region) = {
                    let losers: Vec<u64> = semi_regions.iter()
                        .filter(|&&r| r != champion_region && r != runner_up_region)
                        .copied()
                        .collect();

                    if losers.len() >= 2 {
                        // 根据半决赛胜场数排名
                        let r1_wins = semi_wins.get(&losers[0]).copied().unwrap_or(0);
                        let r2_wins = semi_wins.get(&losers[1]).copied().unwrap_or(0);
                        if r1_wins >= r2_wins {
                            (losers[0], losers[1])
                        } else {
                            (losers[1], losers[0])
                        }
                    } else if losers.len() == 1 {
                        (losers[0], 0)
                    } else {
                        (0, 0)
                    }
                };

                log::debug!("[ICP Points] 赛区排名: 冠军={}, 亚军={}, 季军={}, 殿军={}",
                    champion_region, runner_up_region, third_region, fourth_region);

                // 3. 获取每个赛区的所有队伍
                let all_region_teams = sqlx::query(
                    r#"
                    SELECT t.id as team_id, t.region_id
                    FROM teams t
                    WHERE t.region_id IN (?, ?, ?, ?)
                    "#
                )
                .bind(champion_region as i64)
                .bind(runner_up_region as i64)
                .bind(third_region as i64)
                .bind(fourth_region as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 按赛区分组
                let mut region_teams: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
                for row in &all_region_teams {
                    let team_id: i64 = row.get("team_id");
                    let region_id: i64 = row.get("region_id");
                    region_teams.entry(region_id as u64).or_default().push(team_id as u64);
                }

                // 4. 为每个队伍分配积分
                // 冠军赛区
                if let Some(teams) = region_teams.get(&champion_region) {
                    for &team_id in teams {
                        let position = if participant_team_ids.contains(&team_id) {
                            "FIRST_PARTICIPANT"
                        } else {
                            "FIRST_NON_PARTICIPANT"
                        };
                        results.push((team_id, position.to_string()));
                    }
                }

                // 亚军赛区
                if let Some(teams) = region_teams.get(&runner_up_region) {
                    for &team_id in teams {
                        let position = if participant_team_ids.contains(&team_id) {
                            "SECOND_PARTICIPANT"
                        } else {
                            "SECOND_NON_PARTICIPANT"
                        };
                        results.push((team_id, position.to_string()));
                    }
                }

                // 季军赛区
                if third_region > 0 {
                    if let Some(teams) = region_teams.get(&third_region) {
                        for &team_id in teams {
                            let position = if participant_team_ids.contains(&team_id) {
                                "THIRD_PARTICIPANT"
                            } else {
                                "THIRD_NON_PARTICIPANT"
                            };
                            results.push((team_id, position.to_string()));
                        }
                    }
                }

                // 殿军赛区
                if fourth_region > 0 {
                    if let Some(teams) = region_teams.get(&fourth_region) {
                        for &team_id in teams {
                            let position = if participant_team_ids.contains(&team_id) {
                                "FOURTH_PARTICIPANT"
                            } else {
                                "FOURTH_NON_PARTICIPANT"
                            };
                            results.push((team_id, position.to_string()));
                        }
                    }
                }

                log::debug!("[ICP Points] 共 {} 个队伍需要颁发积分", results.len());
            }

            // Super洲际邀请赛
            TournamentType::SuperIntercontinental => {
                let all_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND status = 'COMPLETED'
                    ORDER BY stage, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "GRAND_FINAL" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                        }
                        break;
                    }
                }

                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "THIRD_PLACE" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let loser = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "THIRD".to_string()));
                            results.push((loser, "FOURTH".to_string()));
                        }
                        break;
                    }
                }

                // FINALS_R1 败者（5-6名）
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "FINALS_R1" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "QUARTER_FINAL".to_string()));
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(results)
    }

}
