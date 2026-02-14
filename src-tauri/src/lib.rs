pub mod commands;
pub mod db;
pub mod engines;
pub mod errors;
pub mod models;
pub mod services;

use commands::{
    get_app_info, simulate_test_match,
    // 存档命令
    init_database, create_save, get_saves, load_save, delete_save, get_current_save_id, delete_database,
    get_default_game_config, create_save_with_config,
    // 队伍命令
    get_teams_by_region, get_all_teams, get_all_players, get_team, get_team_roster, get_team_starters, get_player, set_starter,
    get_all_team_rosters,
    update_player_market_value, update_all_market_values, update_player, update_team,
    // 游戏命令
    get_game_state, advance_phase, get_tournament_matches, get_standings,
    simulate_next_match, simulate_all_matches,
    // 游戏流程命令
    initialize_current_phase, complete_current_phase, start_new_season,
    fix_tournament_status,
    // 时间推进系统命令
    get_time_state, time_init_phase, complete_and_advance, fast_forward_to,
    time_simulate_all, time_simulate_next, time_start_new_season,
    // 荣誉命令
    get_honor_hall, get_team_honors, get_player_honors, get_season_honors, get_tournament_honors,
    get_team_champion_count, get_player_champion_count, get_player_mvp_count,
    get_team_honor_stats, get_player_honor_stats, get_champions_by_type,
    get_all_champions, get_all_mvps,
    // 荣誉殿堂命令
    get_international_champions, get_champion_detail, get_player_honor_rankings,
    get_team_honor_rankings, get_player_honor_detail, get_team_honor_detail,
    cleanup_duplicate_honors, regenerate_tournament_honors, regenerate_all_honors,
    get_hall_of_fame,
    // 选秀命令
    generate_draft_pool, run_draft_lottery, get_draft_order, get_available_draft_players,
    make_draft_pick, ai_auto_draft, get_draft_region_status,
    // 选手池管理命令
    get_draft_pool_players, add_draft_pool_players, update_draft_pool_player, delete_draft_pool_players,
    generate_rookies,
    // 选秀权拍卖命令
    get_draft_pick_prices, start_draft_auction, execute_auction_round, fast_forward_auction,
    get_auction_status, get_auction_events, finalize_auction,
    get_auction_wanted_requests,
    // 财务命令
    get_team_finance_summary, get_all_teams_finance, get_team_transactions,
    record_transaction, get_season_finance_report, pay_team_salaries,
    distribute_league_share, get_prize_pool_info, distribute_tournament_prizes,
    get_team_prize_details,
    // 查询命令
    get_all_regions, get_region_detail, get_season_tournaments, get_region_tournaments,
    get_tournament_detail, get_international_tournaments, get_tournaments_by_type, get_season_overview,
    search_teams, search_players,
    // 国际赛事命令
    create_msi_tournament, create_worlds_tournament, create_masters_tournament,
    create_super_tournament, create_icp_tournament, get_tournament_bracket, advance_bracket,
    get_swiss_round_status, generate_next_swiss_round,
    get_group_standings, generate_knockout_bracket, complete_tournament,
    get_msi_qualified_teams, regenerate_msi_bracket, fill_worlds_knockout_bracket,
    cleanup_duplicate_tournaments, get_shanghai_qualified_teams, regenerate_shanghai_bracket,
    regenerate_icp_bracket, generate_champion_prep_stage, generate_final_stage,
    // 比赛模拟命令
    simulate_match_detailed, simulate_all_matches_detailed, get_player_season_stats, get_match_prediction,
    get_match_lineups, update_match_result, update_match_teams, cancel_match,
    // 事件系统命令
    preview_season_settlement, execute_season_settlement, get_season_events,
    get_player_events, get_events_by_type, update_players_age,
    get_retiring_candidates, get_expiring_contracts,
    // 数据中心命令
    record_player_performance, batch_record_player_performance, record_championship,
    get_season_impact_ranking, get_position_ranking, get_player_stats,
    get_team_player_stats, clear_season_stats, get_player_impact_history,
    get_tournament_mvp_ranking, recalculate_yearly_scores, get_annual_awards_data,
    get_player_market_value_changes, get_player_season_history, get_player_contract_history,
    get_player_tournament_history, get_player_yearly_top_history,
    get_player_growth_logs,
    // 选手特性和状态命令
    get_player_traits, get_player_condition, get_player_full_detail,
    get_all_player_traits, get_trait_catalog, get_team_synergy,
    // 年度积分命令
    get_annual_points_ranking, get_team_points_detail, get_tournament_points, get_super_qualified_teams,
    // 比赛详情命令
    save_match_details, get_match_details, delete_match_details,
    // 开发工具命令
    dev_reassign_honors, dev_recalculate_annual_points, dev_sync_player_games_played,
    dev_recalculate_standings, dev_check_data_consistency, dev_reset_phase,
    dev_simulate_all_matches, dev_redistribute_prizes, dev_grant_funds,
    dev_reset_save, dev_get_game_status, dev_check_incomplete_matches, dev_force_complete_match,
    dev_migrate_loyalty_satisfaction, dev_recalculate_market_values, dev_fix_starters,
    // 转会系统命令
    start_transfer_window, execute_transfer_round, fast_forward_transfer,
    get_transfer_events, get_transfer_report, get_transfer_window_status,
    get_team_personality, update_team_personality, get_team_reputation,
    get_player_market_list,
    get_team_evaluations, get_team_position_needs,
    get_player_listing_evaluations, get_player_stay_evaluations,
    clear_evaluation_data,
    get_transfer_market_listings,
    get_current_transfer_window,
    get_transfer_window_by_season,
    get_transfer_bids_overview,
    get_player_bids,
    confirm_close_transfer_window,
    release_player,
    // 日志系统命令
    log_frontend_event, log_frontend_error, get_log_files, read_log_file, cleanup_logs,
    // 性能监控命令
    get_perf_records, get_perf_summary, toggle_perf_monitoring, clear_perf_records,
    // 版本系统命令
    get_current_meta, get_meta_history, get_all_meta_types, get_meta_detail,
    // 英雄/BP系统命令
    get_champion_list, get_champion_stats, get_draft_result, get_comp_stats, get_comp_matchups,
    get_meta_comp_effects, get_player_champion_mastery, get_player_champion_usage, get_team_comp_usage,
    // 应用状态
    AppState,
};

use services::logging_service::{init_logging, LoggingConfig};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    let log_config = if cfg!(debug_assertions) {
        LoggingConfig::development()
    } else {
        LoggingConfig::production()
    };

    if let Err(e) = init_logging(log_config) {
        eprintln!("日志系统初始化失败: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
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
            delete_database,
            get_default_game_config,
            create_save_with_config,
            // 队伍命令
            get_teams_by_region,
            get_all_teams,
            get_all_players,
            get_team,
            get_team_roster,
            get_team_starters,
            get_all_team_rosters,
            get_player,
            set_starter,
            update_player_market_value,
            update_all_market_values,
            update_player,
            update_team,
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
            start_new_season,
            fix_tournament_status,
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
            // 荣誉殿堂命令
            get_international_champions,
            get_champion_detail,
            get_player_honor_rankings,
            get_team_honor_rankings,
            get_player_honor_detail,
            get_team_honor_detail,
            cleanup_duplicate_honors,
            regenerate_tournament_honors,
            regenerate_all_honors,
            get_hall_of_fame,
            // 选秀命令
            generate_draft_pool,
            run_draft_lottery,
            get_draft_order,
            get_available_draft_players,
            make_draft_pick,
            ai_auto_draft,
            get_draft_region_status,
            // 选手池管理命令
            get_draft_pool_players,
            add_draft_pool_players,
            update_draft_pool_player,
            delete_draft_pool_players,
            generate_rookies,
            // 选秀权拍卖命令
            get_draft_pick_prices,
            start_draft_auction,
            execute_auction_round,
            fast_forward_auction,
            get_auction_status,
            get_auction_events,
            finalize_auction,
            get_auction_wanted_requests,
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
            get_team_prize_details,
            // 查询命令
            get_all_regions,
            get_region_detail,
            get_season_tournaments,
            get_region_tournaments,
            get_tournament_detail,
            get_international_tournaments,
            get_tournaments_by_type,
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
            complete_tournament,
            get_msi_qualified_teams,
            regenerate_msi_bracket,
            fill_worlds_knockout_bracket,
            cleanup_duplicate_tournaments,
            get_shanghai_qualified_teams,
            regenerate_shanghai_bracket,
            regenerate_icp_bracket,
            generate_champion_prep_stage,
            generate_final_stage,
            // 比赛模拟命令
            simulate_match_detailed,
            simulate_all_matches_detailed,
            get_player_season_stats,
            get_match_prediction,
            get_match_lineups,
            update_match_result,
            update_match_teams,
            cancel_match,
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
            get_player_impact_history,
            get_tournament_mvp_ranking,
            recalculate_yearly_scores,
            get_player_market_value_changes,
            get_player_season_history,
            get_player_contract_history,
            get_player_tournament_history,
            get_player_yearly_top_history,
            get_player_growth_logs,
            // 选手特性和状态命令
            get_player_traits,
            get_player_condition,
            get_player_full_detail,
            get_all_player_traits,
            get_trait_catalog,
            get_team_synergy,
            // 时间推进系统命令
            get_time_state,
            time_init_phase,
            complete_and_advance,
            fast_forward_to,
            time_simulate_all,
            time_simulate_next,
            time_start_new_season,
            // 年度积分命令
            get_annual_points_ranking,
            get_team_points_detail,
            get_tournament_points,
            get_super_qualified_teams,
            // 年度颁奖命令
            get_annual_awards_data,
            // 比赛详情持久化命令
            save_match_details,
            get_match_details,
            delete_match_details,
            // 开发工具命令
            dev_reassign_honors,
            dev_recalculate_annual_points,
            dev_sync_player_games_played,
            dev_recalculate_standings,
            dev_check_data_consistency,
            dev_reset_phase,
            dev_simulate_all_matches,
            dev_redistribute_prizes,
            dev_grant_funds,
            dev_reset_save,
            dev_get_game_status,
            dev_check_incomplete_matches,
            dev_force_complete_match,
            dev_migrate_loyalty_satisfaction,
            dev_recalculate_market_values,
            dev_fix_starters,
            // 转会系统命令
            start_transfer_window,
            execute_transfer_round,
            fast_forward_transfer,
            get_transfer_events,
            get_transfer_report,
            get_transfer_window_status,
            get_team_personality,
            update_team_personality,
            get_team_reputation,
            get_player_market_list,
            // 评估系统命令
            get_team_evaluations,
            get_team_position_needs,
            get_player_listing_evaluations,
            get_player_stay_evaluations,
            clear_evaluation_data,
            get_transfer_market_listings,
            get_current_transfer_window,
            get_transfer_window_by_season,
            get_transfer_bids_overview,
            get_player_bids,
            confirm_close_transfer_window,
            release_player,
            // 日志系统命令
            log_frontend_event,
            log_frontend_error,
            get_log_files,
            read_log_file,
            cleanup_logs,
            // 性能监控命令
            get_perf_records,
            get_perf_summary,
            toggle_perf_monitoring,
            clear_perf_records,
            // 版本系统命令
            get_current_meta,
            get_meta_history,
            get_all_meta_types,
            get_meta_detail,
            // 英雄/BP系统命令
            get_champion_list,
            get_champion_stats,
            get_draft_result,
            get_comp_stats,
            get_comp_matchups,
            get_meta_comp_effects,
            get_player_champion_mastery,
            get_player_champion_usage,
            get_team_comp_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
