use sqlx::Row;

use super::msi::MsiTeamInfo;

/// 辅助函数：根据ID获取队伍
pub(super) async fn get_teams_by_ids(pool: &sqlx::SqlitePool, ids: &[u64]) -> Result<Vec<crate::models::Team>, String> {
    let mut teams = Vec::new();
    for id in ids {
        let row = sqlx::query(
            "SELECT id, region_id, name, short_name, power_rating, total_matches, wins, win_rate, annual_points, cross_year_points, balance FROM teams WHERE id = ?"
        )
        .bind(*id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            teams.push(crate::models::Team {
                id: r.get::<i64, _>("id") as u64,
                region_id: r.get::<i64, _>("region_id") as u64,
                name: r.get("name"),
                short_name: r.get("short_name"),
                power_rating: r.get("power_rating"),
                total_matches: r.get::<i64, _>("total_matches") as u32,
                wins: r.get::<i64, _>("wins") as u32,
                win_rate: r.get("win_rate"),
                annual_points: r.get::<i64, _>("annual_points") as u32,
                cross_year_points: r.get::<i64, _>("cross_year_points") as u32,
                balance: r.get("balance"),
                brand_value: r.get("brand_value"),
            });
        }
    }
    Ok(teams)
}

/// 辅助函数：确定胜者晋级的下一场比赛
pub(super) fn determine_next_matches(stage: &str, match_order: u32) -> Vec<(String, u32, bool)> {
    match stage {
        // MSI双败制
        "QUALIFIER_R1" => vec![("LOSERS_R1".to_string(), match_order, true)],
        "CHALLENGER_R1" => vec![("LOSERS_R2".to_string(), match_order, true)],
        "LOSERS_R1" => vec![("LOSERS_R2".to_string(), match_order, false)],
        "LOSERS_R2" => vec![("LOSERS_R3".to_string(), match_order, false)],
        "WINNERS_R1" => vec![
            ("WINNERS_FINAL".to_string(), 1, match_order == 1),
            ("LOSERS_R3".to_string(), match_order, true),
        ],
        "LOSERS_R3" => vec![("LOSERS_R4".to_string(), 1, match_order == 1)],
        "WINNERS_FINAL" => vec![("GRAND_FINAL".to_string(), 1, true)],
        "LOSERS_R4" => vec![("LOSERS_FINAL".to_string(), 1, false)],
        "LOSERS_FINAL" => vec![("GRAND_FINAL".to_string(), 1, false)],

        // 大师赛淘汰赛
        "EAST_R1" => vec![("EAST_SEMI".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        "WEST_R1" => vec![("WEST_SEMI".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        "EAST_SEMI" => vec![("EAST_FINAL".to_string(), 1, match_order == 1)],
        "WEST_SEMI" => vec![("WEST_FINAL".to_string(), 1, match_order == 1)],
        "EAST_FINAL" => vec![("GRAND_FINAL".to_string(), 1, true)],
        "WEST_FINAL" => vec![("GRAND_FINAL".to_string(), 1, false)],

        // 世界赛淘汰赛
        "QUARTER_FINAL" => vec![("SEMI_FINAL".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        // 世界赛淘汰赛 - 胜者只进决赛，败者进季军赛（由 determine_loser_next_match 处理）
        "SEMI_FINAL" => vec![("GRAND_FINAL".to_string(), 1, match_order == 1)],

        // Super赛事第二阶段 - 定位赛胜者进入冠军预备战胜者组
        "CHALLENGER_POSITIONING" => vec![("PREP_WINNERS".to_string(), 1, match_order == 1)], // 定位赛1胜者=home, 定位赛2胜者=away
        // Super赛事第二阶段 - 晋级赛胜者进入冠军预备战败者组
        "CHALLENGER_PROMOTION" => vec![("PREP_LOSERS".to_string(), 1, match_order == 1)], // 晋级赛1胜者=home, 晋级赛2胜者=away

        // Super赛事第三阶段 - 败者组胜者进入败者组决赛
        "PREP_LOSERS" => vec![("PREP_LOSERS_FINAL".to_string(), 1, false)], // 败者组胜者进入败者组决赛 away

        // Super赛事第四阶段 - 首轮胜者进入次轮
        "FINALS_R1" => vec![("FINALS_R2".to_string(), match_order, false)], // 首轮胜者进入对应次轮的away (home已有传奇组队伍)
        // Super赛事第四阶段 - 次轮胜者进入总决赛
        "FINALS_R2" => vec![("GRAND_FINAL".to_string(), 1, match_order == 1)], // 次轮1胜者=home, 次轮2胜者=away

        _ => vec![],
    }
}

/// 确定败者的下一场比赛
pub(super) fn determine_loser_next_match(stage: &str, match_order: u32) -> Vec<(String, u32, bool)> {
    match stage {
        // MSI双败制 - 败者去向
        "CHALLENGER_R1" => vec![("LOSERS_R1".to_string(), match_order, false)], // 挑战者组败者进入败者组R1的away
        "WINNERS_R1" => vec![("LOSERS_R3".to_string(), match_order, true)],     // 胜者组R1败者进入败者组R3
        "WINNERS_FINAL" => vec![("LOSERS_FINAL".to_string(), 1, true)],         // 胜者组决赛败者进入败者组决赛

        // Super赛事 - 定位赛败者进入晋级赛（away位置）
        "CHALLENGER_POSITIONING" => vec![("CHALLENGER_PROMOTION".to_string(), match_order, false)], // 定位赛败者进入对应晋级赛的away
        // Super赛事 - 胜者组败者进入败者组决赛（home位置）
        "PREP_WINNERS" => vec![("PREP_LOSERS_FINAL".to_string(), 1, true)], // 胜者组败者进入败者组决赛 home
        // Super赛事第四阶段 - 次轮败者进入季军赛
        "FINALS_R2" => vec![("THIRD_PLACE".to_string(), 1, match_order == 1)], // 次轮1败者=home, 次轮2败者=away

        // 马德里大师赛半区决赛败者进入季军赛
        "EAST_FINAL" => vec![("THIRD_PLACE".to_string(), 1, true)],             // 东半区决赛败者进入季军赛home
        "WEST_FINAL" => vec![("THIRD_PLACE".to_string(), 1, false)],            // 西半区决赛败者进入季军赛away

        // 世界赛半决赛败者进入季军赛
        "SEMI_FINAL" => vec![("THIRD_PLACE".to_string(), 1, match_order != 1)],

        _ => vec![],
    }
}

/// 位置转排名数字
pub(super) fn position_to_rank(position: &str) -> Option<u32> {
    match position {
        "CHAMPION" => Some(1),
        "RUNNER_UP" => Some(2),
        "THIRD" => Some(3),
        "FOURTH" => Some(4),
        "5TH_8TH" | "QUARTER_FINAL" => Some(5),
        // ICP积分位置
        "FIRST_PARTICIPANT" | "FIRST_NON_PARTICIPANT" => Some(1),
        "SECOND_PARTICIPANT" | "SECOND_NON_PARTICIPANT" => Some(2),
        "THIRD_PARTICIPANT" | "THIRD_NON_PARTICIPANT" => Some(3),
        "FOURTH_PARTICIPANT" | "FOURTH_NON_PARTICIPANT" => Some(4),
        _ => None,
    }
}

/// 获取阶段显示名称
pub(super) fn get_stage_display_name(stage: &str) -> String {
    match stage {
        "QUALIFIER_R1" => "资格赛".to_string(),
        "CHALLENGER_R1" => "挑战者组".to_string(),
        "LOSERS_R1" | "LOSERS_R2" | "LOSERS_R3" | "LOSERS_R4" => "败者组".to_string(),
        "LOSERS_FINAL" => "败者组决赛".to_string(),
        "WINNERS_R1" => "胜者组首轮".to_string(),
        "WINNERS_FINAL" => "胜者组决赛".to_string(),
        "GRAND_FINAL" => "总决赛".to_string(),
        "SWISS_R1" | "SWISS_R2" | "SWISS_R3" | "SWISS_R4" | "SWISS_R5" => "瑞士轮".to_string(),
        "QUARTER_FINAL" => "八强赛".to_string(),
        "SEMI_FINAL" => "半决赛".to_string(),
        "THIRD_PLACE" => "季军赛".to_string(),
        "EAST_R1" | "WEST_R1" => "淘汰赛首轮".to_string(),
        "EAST_SEMI" | "WEST_SEMI" => "半决赛".to_string(),
        "EAST_FINAL" | "WEST_FINAL" => "半区决赛".to_string(),
        s if s.starts_with("GROUP_") => format!("{}组", &s[6..]),
        s if s.starts_with("FIGHTER_GROUP_") => format!("Fighter {}组", &s[14..]),
        _ => stage.to_string(),
    }
}

/// 获取队伍信息的辅助函数
pub(super) async fn get_team_info(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    team_id: u64,
    region_id: Option<u64>,
    region_name: &Option<String>,
) -> Result<Option<MsiTeamInfo>, String> {
    let row = sqlx::query(
        "SELECT id, name, short_name FROM teams WHERE id = ?"
    )
    .bind(team_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| {
        let name: String = r.get("name");
        let short_name: Option<String> = r.get("short_name");
        MsiTeamInfo {
            team_id,
            team_name: name.clone(),
            short_name: short_name.unwrap_or(name),
            region_id: region_id.unwrap_or(0),
            region_name: region_name.clone().unwrap_or_default(),
        }
    }))
}
