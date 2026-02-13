use crate::models::*;
use crate::models::tournament_result::PlayerTournamentStats;
use crate::models::match_game_detail::{MatchGameDetail, GamePlayerPerformance};
use sqlx::Row;

pub(crate) fn parse_season_phase(s: &str) -> SeasonPhase {
    match s {
        "SpringRegular" => SeasonPhase::SpringRegular,
        "SpringPlayoffs" => SeasonPhase::SpringPlayoffs,
        "Msi" => SeasonPhase::Msi,
        "MadridMasters" => SeasonPhase::MadridMasters,
        "SummerRegular" => SeasonPhase::SummerRegular,
        "SummerPlayoffs" => SeasonPhase::SummerPlayoffs,
        "ClaudeIntercontinental" => SeasonPhase::ClaudeIntercontinental,
        "WorldChampionship" => SeasonPhase::WorldChampionship,
        "ShanghaiMasters" => SeasonPhase::ShanghaiMasters,
        "IcpIntercontinental" => SeasonPhase::IcpIntercontinental,
        "SuperIntercontinental" => SeasonPhase::SuperIntercontinental,
        "AnnualAwards" => SeasonPhase::AnnualAwards,
        "TransferWindow" => SeasonPhase::TransferWindow,
        "Draft" => SeasonPhase::Draft,
        "SeasonEnd" => SeasonPhase::SeasonEnd,
        _ => SeasonPhase::SpringRegular,
    }
}

pub(crate) fn row_to_team(row: &sqlx::sqlite::SqliteRow) -> Team {
    Team {
        id: row.get::<i64, _>("id") as u64,
        region_id: row.get::<i64, _>("region_id") as u64,
        name: row.get("name"),
        short_name: row.get("short_name"),
        power_rating: row.get("power_rating"),
        total_matches: row.get::<i64, _>("total_matches") as u32,
        wins: row.get::<i64, _>("wins") as u32,
        win_rate: row.get("win_rate"),
        annual_points: row.get::<i64, _>("annual_points") as u32,
        cross_year_points: row.get::<i64, _>("cross_year_points") as u32,
        balance: row.get::<i64, _>("balance"),
        brand_value: row.get("brand_value"),
    }
}

pub(crate) fn row_to_player(row: &sqlx::sqlite::SqliteRow) -> Player {
    let pos_str: Option<String> = row.get("position");
    Player {
        id: row.get::<i64, _>("id") as u64,
        game_id: row.get("game_id"),
        real_name: row.get("real_name"),
        nationality: row.get("nationality"),
        age: row.get::<i64, _>("age") as u8,
        ability: row.get::<i64, _>("ability") as u8,
        potential: row.get::<i64, _>("potential") as u8,
        stability: row.get::<i64, _>("stability") as u8,
        tag: parse_player_tag(row.get("tag")),
        status: parse_player_status(row.get("status")),
        position: pos_str.as_deref().map(parse_position),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        salary: row.get::<i64, _>("salary") as u64,
        market_value: row.get::<i64, _>("market_value") as u64,
        calculated_market_value: row.try_get::<i64, _>("calculated_market_value").ok().map(|v| v as u64).unwrap_or(0),
        contract_end_season: row.get::<Option<i64>, _>("contract_end_season").map(|v| v as u32),
        join_season: row.get::<i64, _>("join_season") as u32,
        retire_season: row.get::<Option<i64>, _>("retire_season").map(|v| v as u32),
        is_starter: row.get("is_starter"),
        loyalty: row.try_get::<i64, _>("loyalty").ok().map(|v| v as u8).unwrap_or(50),
        satisfaction: row.try_get::<i64, _>("satisfaction").ok().map(|v| v as u8).unwrap_or(50),
        growth_accumulator: row.try_get::<f64, _>("growth_accumulator").unwrap_or(0.0),
    }
}

pub(crate) fn row_to_match(row: &sqlx::sqlite::SqliteRow) -> Match {
    Match {
        id: row.get::<i64, _>("id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        stage: row.get("stage"),
        round: row.get::<Option<i64>, _>("round").map(|v| v as u32),
        match_order: row.get::<Option<i64>, _>("match_order").map(|v| v as u32),
        format: parse_match_format(row.get("format")),
        // NULL 表示队伍待定，转换为 0
        home_team_id: row.get::<Option<i64>, _>("home_team_id").unwrap_or(0) as u64,
        away_team_id: row.get::<Option<i64>, _>("away_team_id").unwrap_or(0) as u64,
        home_score: row.get::<i64, _>("home_score") as u8,
        away_score: row.get::<i64, _>("away_score") as u8,
        winner_id: row.get::<Option<i64>, _>("winner_id").map(|v| v as u64),
        status: parse_match_status(row.get("status")),
    }
}

pub(crate) fn row_to_tournament(row: &sqlx::sqlite::SqliteRow) -> Tournament {
    Tournament {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_type: parse_tournament_type(row.get("tournament_type")),
        name: row.get("name"),
        region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
        status: parse_tournament_status(row.get("status")),
        current_stage: row.get("current_stage"),
        current_round: row.get::<Option<i64>, _>("current_round").map(|v| v as u32),
    }
}

pub(crate) fn row_to_standing(row: &sqlx::sqlite::SqliteRow) -> LeagueStanding {
    LeagueStanding {
        id: row.get::<i64, _>("id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        team_id: row.get::<i64, _>("team_id") as u64,
        rank: row.get::<Option<i64>, _>("rank").map(|v| v as u32),
        matches_played: row.get::<i64, _>("matches_played") as u32,
        wins: row.get::<i64, _>("wins") as u32,
        losses: row.get::<i64, _>("losses") as u32,
        points: row.get::<i64, _>("points") as u32,
        games_won: row.get::<i64, _>("games_won") as u32,
        games_lost: row.get::<i64, _>("games_lost") as u32,
        game_diff: row.get::<i64, _>("game_diff") as i32,
    }
}

pub(crate) fn parse_player_tag(s: &str) -> PlayerTag {
    match s {
        "Ordinary" => PlayerTag::Ordinary,
        "Normal" => PlayerTag::Normal,
        "Genius" => PlayerTag::Genius,
        _ => PlayerTag::Normal,
    }
}

pub(crate) fn parse_player_status(s: &str) -> PlayerStatus {
    match s {
        "Active" => PlayerStatus::Active,
        "Retired" => PlayerStatus::Retired,
        _ => PlayerStatus::Active,
    }
}

pub(crate) fn parse_position(s: &str) -> Position {
    // 处理 Some(Position) 格式和纯 Position 格式
    let s = s.trim_start_matches("Some(").trim_end_matches(")");
    match s {
        "Top" => Position::Top,
        "Jug" => Position::Jug,
        "Mid" => Position::Mid,
        "Adc" => Position::Adc,
        "Sup" => Position::Sup,
        _ => Position::Mid,
    }
}

pub(crate) fn parse_match_format(s: &str) -> MatchFormat {
    match s {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    }
}

pub(crate) fn parse_match_status(s: &str) -> MatchStatus {
    match s {
        "Scheduled" | "SCHEDULED" => MatchStatus::Scheduled,
        "InProgress" | "IN_PROGRESS" => MatchStatus::InProgress,
        "Completed" | "COMPLETED" => MatchStatus::Completed,
        _ => MatchStatus::Scheduled,
    }
}

pub(crate) fn parse_tournament_type(s: &str) -> TournamentType {
    match s {
        "SpringRegular" => TournamentType::SpringRegular,
        "SpringPlayoffs" => TournamentType::SpringPlayoffs,
        "SummerRegular" => TournamentType::SummerRegular,
        "SummerPlayoffs" => TournamentType::SummerPlayoffs,
        "Msi" => TournamentType::Msi,
        "MadridMasters" => TournamentType::MadridMasters,
        "ClaudeIntercontinental" => TournamentType::ClaudeIntercontinental,
        "WorldChampionship" => TournamentType::WorldChampionship,
        "ShanghaiMasters" => TournamentType::ShanghaiMasters,
        "IcpIntercontinental" => TournamentType::IcpIntercontinental,
        "SuperIntercontinental" => TournamentType::SuperIntercontinental,
        _ => TournamentType::SpringRegular,
    }
}

pub(crate) fn parse_tournament_status(s: &str) -> TournamentStatus {
    match s {
        "Upcoming" => TournamentStatus::Upcoming,
        "InProgress" => TournamentStatus::InProgress,
        "Completed" => TournamentStatus::Completed,
        _ => TournamentStatus::Upcoming,
    }
}

pub(crate) fn parse_honor_type(s: &str) -> HonorType {
    match s {
        "TEAM_CHAMPION" => HonorType::TeamChampion,
        "TEAM_RUNNER_UP" => HonorType::TeamRunnerUp,
        "TEAM_THIRD" => HonorType::TeamThird,
        "TEAM_FOURTH" => HonorType::TeamFourth,
        "REGULAR_SEASON_FIRST" => HonorType::RegularSeasonFirst,
        "TOURNAMENT_MVP" => HonorType::TournamentMvp,
        "FINALS_MVP" => HonorType::FinalsMvp,
        "REGULAR_SEASON_MVP" => HonorType::RegularSeasonMvp,
        "PLAYOFFS_FMVP" => HonorType::PlayoffsFmvp,
        "PLAYER_CHAMPION" => HonorType::PlayerChampion,
        "PLAYER_RUNNER_UP" => HonorType::PlayerRunnerUp,
        "PLAYER_THIRD" => HonorType::PlayerThird,
        "PLAYER_FOURTH" => HonorType::PlayerFourth,
        "ANNUAL_MVP" => HonorType::AnnualMvp,
        "ANNUAL_TOP20" => HonorType::AnnualTop20,
        "ANNUAL_ALL_PRO_1ST" => HonorType::AnnualAllPro1st,
        "ANNUAL_ALL_PRO_2ND" => HonorType::AnnualAllPro2nd,
        "ANNUAL_ALL_PRO_3RD" => HonorType::AnnualAllPro3rd,
        "ANNUAL_MOST_CONSISTENT" => HonorType::AnnualMostConsistent,
        "ANNUAL_MOST_DOMINANT" => HonorType::AnnualMostDominant,
        "ANNUAL_ROOKIE" => HonorType::AnnualRookie,
        // 兼容旧存档
        "ANNUAL_BEST_TOP" | "ANNUAL_BEST_JUNGLE" | "ANNUAL_BEST_MID"
        | "ANNUAL_BEST_ADC" | "ANNUAL_BEST_SUPPORT" => HonorType::AnnualAllPro1st,
        _ => HonorType::TeamChampion,
    }
}

pub(crate) fn honor_type_to_db_string(honor_type: &HonorType) -> &'static str {
    match honor_type {
        HonorType::TeamChampion => "TEAM_CHAMPION",
        HonorType::TeamRunnerUp => "TEAM_RUNNER_UP",
        HonorType::TeamThird => "TEAM_THIRD",
        HonorType::TeamFourth => "TEAM_FOURTH",
        HonorType::RegularSeasonFirst => "REGULAR_SEASON_FIRST",
        HonorType::TournamentMvp => "TOURNAMENT_MVP",
        HonorType::FinalsMvp => "FINALS_MVP",
        HonorType::RegularSeasonMvp => "REGULAR_SEASON_MVP",
        HonorType::PlayoffsFmvp => "PLAYOFFS_FMVP",
        HonorType::PlayerChampion => "PLAYER_CHAMPION",
        HonorType::PlayerRunnerUp => "PLAYER_RUNNER_UP",
        HonorType::PlayerThird => "PLAYER_THIRD",
        HonorType::PlayerFourth => "PLAYER_FOURTH",
        HonorType::AnnualMvp => "ANNUAL_MVP",
        HonorType::AnnualTop20 => "ANNUAL_TOP20",
        HonorType::AnnualAllPro1st => "ANNUAL_ALL_PRO_1ST",
        HonorType::AnnualAllPro2nd => "ANNUAL_ALL_PRO_2ND",
        HonorType::AnnualAllPro3rd => "ANNUAL_ALL_PRO_3RD",
        HonorType::AnnualMostConsistent => "ANNUAL_MOST_CONSISTENT",
        HonorType::AnnualMostDominant => "ANNUAL_MOST_DOMINANT",
        HonorType::AnnualRookie => "ANNUAL_ROOKIE",
    }
}

pub(crate) fn row_to_honor(row: &sqlx::sqlite::SqliteRow) -> Honor {
    let stats_json: Option<String> = row.get("stats_json");
    let stats = stats_json.and_then(|json| serde_json::from_str(&json).ok());

    Honor {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        honor_type: parse_honor_type(row.get("honor_type")),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<Option<i64>, _>("tournament_id").map(|v| v as u64),
        tournament_name: row.get("tournament_name"),
        tournament_type: row.get("tournament_type"),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        team_name: row.get("team_name"),
        player_id: row.get::<Option<i64>, _>("player_id").map(|v| v as u64),
        player_name: row.get("player_name"),
        position: row.get("position"),
        stats,
        created_at: row.get("created_at"),
    }
}

pub(crate) fn parse_event_type(s: &str) -> EventType {
    match s {
        "PlayerGrowth" => EventType::PlayerGrowth,
        "PlayerDecline" => EventType::PlayerDecline,
        "PlayerRetirement" => EventType::PlayerRetirement,
        "RookieGeneration" => EventType::RookieGeneration,
        "ContractExpire" => EventType::ContractExpire,
        "PlayerAging" => EventType::PlayerAging,
        "SeasonSettlement" => EventType::SeasonSettlement,
        _ => EventType::SeasonSettlement,
    }
}

pub(crate) fn row_to_game_event(row: &sqlx::sqlite::SqliteRow) -> GameEvent {
    GameEvent {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        event_type: parse_event_type(row.get("event_type")),
        player_id: row.get::<Option<i64>, _>("player_id").map(|v| v as u64),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        description: row.get("description"),
        details: row.get("details"),
        phase: row.get("phase"),
    }
}

pub(crate) fn row_to_player_stats(row: &sqlx::sqlite::SqliteRow) -> PlayerSeasonStatistics {
    PlayerSeasonStatistics {
        id: Some(row.get::<i64, _>("id")),
        save_id: row.get("save_id"),
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        season_id: row.get("season_id"),
        team_id: row.get("team_id"),
        region_id: row.get("region_id"),
        position: row.get("position"),
        matches_played: row.get("matches_played"),
        games_played: row.get("games_played"),
        total_impact: row.get("total_impact"),
        avg_impact: row.get("avg_impact"),
        avg_performance: row.get("avg_performance"),
        best_performance: row.get("best_performance"),
        worst_performance: row.get("worst_performance"),
        consistency_score: row.get("consistency_score"),
        international_titles: row.get("international_titles"),
        regional_titles: row.get("regional_titles"),
        champion_bonus: row.get("champion_bonus"),
        yearly_top_score: row.get("yearly_top_score"),
        dominance_score: row.try_get("dominance_score").unwrap_or(0.0),
    }
}

pub(crate) fn row_to_tournament_result(row: &sqlx::sqlite::SqliteRow) -> TournamentResult {
    TournamentResult {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        tournament_type: row.get("tournament_type"),
        tournament_name: row.get("tournament_name"),
        champion_team_id: row.get::<i64, _>("champion_team_id") as u64,
        champion_team_name: row.get("champion_team_name"),
        runner_up_team_id: row.get::<i64, _>("runner_up_team_id") as u64,
        runner_up_team_name: row.get("runner_up_team_name"),
        third_team_id: row.get::<Option<i64>, _>("third_team_id").map(|v| v as u64),
        third_team_name: row.get("third_team_name"),
        fourth_team_id: row.get::<Option<i64>, _>("fourth_team_id").map(|v| v as u64),
        fourth_team_name: row.get("fourth_team_name"),
        final_match_id: row.get::<Option<i64>, _>("final_match_id").map(|v| v as u64),
        final_score: row.get("final_score"),
        total_matches: row.get::<Option<i64>, _>("total_matches").map(|v| v as u32),
        total_games: row.get::<Option<i64>, _>("total_games").map(|v| v as u32),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_player_tournament_stats(row: &sqlx::sqlite::SqliteRow) -> PlayerTournamentStats {
    PlayerTournamentStats {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        tournament_type: row.get("tournament_type"),
        player_id: row.get::<i64, _>("player_id") as u64,
        player_name: row.get("player_name"),
        team_id: row.get::<i64, _>("team_id") as u64,
        team_name: row.get("team_name"),
        position: row.get("position"),
        games_played: row.get::<i64, _>("games_played") as u32,
        games_won: row.get::<i64, _>("games_won") as u32,
        total_impact: row.get("total_impact"),
        avg_impact: row.get("avg_impact"),
        max_impact: row.get("max_impact"),
        avg_performance: row.get("avg_performance"),
        best_performance: row.get("best_performance"),
        game_mvp_count: row.get::<i64, _>("game_mvp_count") as u32,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// 将数据库行转换为 MatchGameDetail
pub(crate) fn row_to_match_game_detail(row: &sqlx::sqlite::SqliteRow) -> MatchGameDetail {
    MatchGameDetail {
        id: row.get("id"),
        save_id: row.get("save_id"),
        match_id: row.get("match_id"),
        game_number: row.get("game_number"),
        winner_team_id: row.get("winner_team_id"),
        loser_team_id: row.get("loser_team_id"),
        duration_minutes: row.get("duration_minutes"),
        mvp_player_id: row.get("mvp_player_id"),
        key_player_id: row.get("key_player_id"),
        home_power: row.get("home_power"),
        away_power: row.get("away_power"),
        home_meta_power: row.get("home_meta_power"),
        away_meta_power: row.get("away_meta_power"),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_game_player_performance(row: &sqlx::sqlite::SqliteRow) -> GamePlayerPerformance {
    GamePlayerPerformance {
        id: row.get("id"),
        save_id: row.get("save_id"),
        game_id: row.get("game_id"),
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        team_id: row.get("team_id"),
        team_name: row.get("team_name"),
        position: row.get("position"),
        base_ability: row.get("base_ability"),
        condition_bonus: row.get("condition_bonus"),
        stability_noise: row.get("stability_noise"),
        actual_ability: row.get("actual_ability"),
        impact_score: row.get("impact_score"),
        mvp_score: row.get("mvp_score"),
        is_mvp: row.get::<i32, _>("is_mvp") != 0,
        is_key_player: row.get::<i32, _>("is_key_player") != 0,
        kills: row.get("kills"),
        deaths: row.get("deaths"),
        assists: row.get("assists"),
        cs: row.get("cs"),
        gold: row.get("gold"),
        damage_dealt: row.get("damage_dealt"),
        damage_taken: row.get("damage_taken"),
        vision_score: row.get("vision_score"),
        traits_json: row.get("traits_json"),
        activated_traits_json: row.get("activated_traits_json"),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_player_season_status(row: &sqlx::sqlite::SqliteRow) -> PlayerSeasonStatus {
    let reasons_json: String = row.get("departure_reasons");
    let departure_reasons: Vec<DepartureReason> = serde_json::from_str(&reasons_json)
        .unwrap_or_default();

    PlayerSeasonStatus {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        player_id: row.get::<i64, _>("player_id") as u64,
        satisfaction: row.get::<i32, _>("satisfaction") as u8,
        wants_to_leave: row.get("wants_to_leave"),
        departure_reasons,
        games_as_starter: row.get::<i32, _>("games_as_starter") as u32,
        total_games: row.get::<i32, _>("total_games") as u32,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn row_to_team_season_performance(row: &sqlx::sqlite::SqliteRow) -> TeamSeasonPerformance {
    TeamSeasonPerformance {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        team_id: row.get::<i64, _>("team_id") as u64,
        final_rank: row.get::<Option<i32>, _>("final_rank").map(|r| r as u32),
        made_playoffs: row.get("made_playoffs"),
        playoff_result: row.get("playoff_result"),
        international_result: row.get("international_result"),
        consecutive_no_playoffs: row.get::<i32, _>("consecutive_no_playoffs") as u32,
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_loyalty_change(row: &sqlx::sqlite::SqliteRow) -> LoyaltyChange {
    LoyaltyChange {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        player_id: row.get::<i64, _>("player_id") as u64,
        change_amount: row.get("change_amount"),
        reason: row.get("reason"),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn row_to_llm_task_log(row: &sqlx::sqlite::SqliteRow) -> LLMTaskLog {
    let task_type_str: String = row.get("task_type");
    let status_str: String = row.get("status");

    LLMTaskLog {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        task_type: TaskType::from_str(&task_type_str).unwrap_or(TaskType::Intention),
        entity_id: row.get::<i64, _>("entity_id") as u64,
        entity_type: row.get("entity_type"),
        status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
        attempt_count: row.get::<i64, _>("attempt_count") as u32,
        max_attempts: row.get::<i64, _>("max_attempts") as u32,
        error_msg: row.get("error_msg"),
        last_error_at: row.get("last_error_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        completed_at: row.get("completed_at"),
    }
}
