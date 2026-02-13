use crate::models::TournamentStatus;

/// 获取赛区名称
pub(crate) fn get_region_name(region_id: u64) -> &'static str {
    match region_id {
        1 => "LPL",
        2 => "LCK",
        3 => "LEC",
        4 => "LCS",
        _ => "Unknown",
    }
}

/// 解析赛事状态（本地版本避免循环依赖）
pub(crate) fn parse_tournament_status_local(s: &str) -> TournamentStatus {
    match s {
        "Upcoming" => TournamentStatus::Upcoming,
        "InProgress" => TournamentStatus::InProgress,
        "Completed" => TournamentStatus::Completed,
        _ => TournamentStatus::Upcoming,
    }
}

/// 位置转排名数字
pub(crate) fn position_to_rank(position: &str) -> Option<u32> {
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
