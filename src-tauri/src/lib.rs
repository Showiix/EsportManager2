pub mod commands;
pub mod db;
pub mod engines;
pub mod models;
pub mod services;

use commands::{
    get_app_info, simulate_test_match,
    // 存档命令
    init_database, create_save, get_saves, load_save, delete_save, get_current_save_id,
    // 队伍命令
    get_teams_by_region, get_all_teams, get_team, get_team_roster, get_team_starters, get_player, set_starter,
    update_player_market_value, update_all_market_values,
    // 游戏命令
    get_game_state, advance_phase, get_tournament_matches, get_standings,
    simulate_next_match, simulate_all_matches,
    // 游戏流程命令
    initialize_current_phase, complete_current_phase, run_season_settlement, start_new_season,
    // 荣誉命令
    get_honor_hall, get_team_honors, get_player_honors, get_season_honors, get_tournament_honors,
    get_team_champion_count, get_player_champion_count, get_player_mvp_count,
    get_team_honor_stats, get_player_honor_stats, get_champions_by_type,
    get_all_champions, get_all_mvps,
    // 选秀命令
    generate_draft_pool, run_draft_lottery, get_draft_order, get_available_draft_players,
    make_draft_pick, ai_auto_draft,
    // 转会命令
    get_transfer_market, get_free_agents, list_player_for_transfer, cancel_transfer_listing,
    buy_listed_player, sign_free_agent, get_transfer_history,
    // AI 转会窗口命令
    start_transfer_window, execute_transfer_round, fast_forward_transfers,
    get_transfer_window_status, get_transfer_events,
    // 财务命令
    get_team_finance_summary, get_all_teams_finance, get_team_transactions,
    record_transaction, get_season_finance_report, pay_team_salaries,
    distribute_league_share, get_prize_pool_info, distribute_tournament_prizes,
    // 查询命令
    get_all_regions, get_region_detail, get_season_tournaments, get_region_tournaments,
    get_tournament_detail, get_international_tournaments, get_season_overview,
    search_teams, search_players,
    // 国际赛事命令
    create_msi_tournament, create_worlds_tournament, create_masters_tournament,
    create_super_tournament, create_icp_tournament, get_tournament_bracket, advance_bracket,
    get_swiss_round_status, generate_next_swiss_round,
    get_group_standings, generate_knockout_bracket,
    // 比赛模拟命令
    simulate_match_detailed, get_player_season_stats, get_match_prediction,
    // 事件系统命令
    preview_season_settlement, execute_season_settlement, get_season_events,
    get_player_events, get_events_by_type, update_players_age,
    get_retiring_candidates, get_expiring_contracts,
    // 数据中心命令
    record_player_performance, batch_record_player_performance, record_championship,
    get_season_impact_ranking, get_position_ranking, get_player_stats,
    get_team_player_stats, clear_season_stats,
    // 选手特性和状态命令
    get_player_traits, get_player_condition, get_player_full_detail,
    // 应用状态
    AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 基础命令
            get_app_info,
            simulate_test_match,
            // 存档命令
            init_database,
            create_save,
            get_saves,
            load_save,
            delete_save,
            get_current_save_id,
            // 队伍命令
            get_teams_by_region,
            get_all_teams,
            get_team,
            get_team_roster,
            get_team_starters,
            get_player,
            set_starter,
            update_player_market_value,
            update_all_market_values,
            // 游戏命令
            get_game_state,
            advance_phase,
            get_tournament_matches,
            get_standings,
            simulate_next_match,
            simulate_all_matches,
            // 游戏流程命令
            initialize_current_phase,
            complete_current_phase,
            run_season_settlement,
            start_new_season,
            // 荣誉命令
            get_honor_hall,
            get_team_honors,
            get_player_honors,
            get_season_honors,
            get_tournament_honors,
            get_team_champion_count,
            get_player_champion_count,
            get_player_mvp_count,
            get_team_honor_stats,
            get_player_honor_stats,
            get_champions_by_type,
            get_all_champions,
            get_all_mvps,
            // 选秀命令
            generate_draft_pool,
            run_draft_lottery,
            get_draft_order,
            get_available_draft_players,
            make_draft_pick,
            ai_auto_draft,
            // 转会命令
            get_transfer_market,
            get_free_agents,
            list_player_for_transfer,
            cancel_transfer_listing,
            buy_listed_player,
            sign_free_agent,
            get_transfer_history,
            // AI 转会窗口命令
            start_transfer_window,
            execute_transfer_round,
            fast_forward_transfers,
            get_transfer_window_status,
            get_transfer_events,
            // 财务命令
            get_team_finance_summary,
            get_all_teams_finance,
            get_team_transactions,
            record_transaction,
            get_season_finance_report,
            pay_team_salaries,
            distribute_league_share,
            get_prize_pool_info,
            distribute_tournament_prizes,
            // 查询命令
            get_all_regions,
            get_region_detail,
            get_season_tournaments,
            get_region_tournaments,
            get_tournament_detail,
            get_international_tournaments,
            get_season_overview,
            search_teams,
            search_players,
            // 国际赛事命令
            create_msi_tournament,
            create_worlds_tournament,
            create_masters_tournament,
            create_super_tournament,
            create_icp_tournament,
            get_tournament_bracket,
            advance_bracket,
            get_swiss_round_status,
            generate_next_swiss_round,
            get_group_standings,
            generate_knockout_bracket,
            // 比赛模拟命令
            simulate_match_detailed,
            get_player_season_stats,
            get_match_prediction,
            // 事件系统命令
            preview_season_settlement,
            execute_season_settlement,
            get_season_events,
            get_player_events,
            get_events_by_type,
            update_players_age,
            get_retiring_candidates,
            get_expiring_contracts,
            // 数据中心命令
            record_player_performance,
            batch_record_player_performance,
            record_championship,
            get_season_impact_ranking,
            get_position_ranking,
            get_player_stats,
            get_team_player_stats,
            clear_season_stats,
            // 选手特性和状态命令
            get_player_traits,
            get_player_condition,
            get_player_full_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
