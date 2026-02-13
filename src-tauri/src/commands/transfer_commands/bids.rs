use crate::commands::save_commands::{AppState, CommandResult};
use crate::models::transfer::*;
use sqlx::Row;
use tauri::State;

#[tauri::command]
pub async fn get_transfer_bids_overview(
    state: State<'_, AppState>,
    window_id: Option<i64>,
    season_id: Option<i64>,
    round: Option<i64>,
) -> Result<CommandResult<BidOverview>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let resolved_window_id = if let Some(wid) = window_id {
        wid
    } else if let Some(sid) = season_id {
        let current_save = state.current_save_id.read().await;
        let save_id = match current_save.as_ref() {
            Some(id) => id.clone(),
            None => return Ok(CommandResult::err("No save loaded")),
        };
        let row = sqlx::query("SELECT id FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1")
            .bind(&save_id).bind(sid)
            .fetch_optional(&pool).await.map_err(|e| e.to_string())?;
        match row {
            Some(r) => r.get::<i64, _>("id"),
            None => return Ok(CommandResult::ok(BidOverview {
                window_id: 0,
                round,
                total_players: 0,
                total_bids: 0,
                successful_signings: 0,
                failed_signings: 0,
                avg_bids_per_player: 0.0,
                player_analyses: vec![],
            })),
        }
    } else {
        return Ok(CommandResult::err("需要提供 window_id 或 season_id"));
    };

    let rows = if let Some(r) = round {
        sqlx::query(
            r#"SELECT id, window_id, round, player_id, player_name, player_ability, player_age,
                      player_position, from_team_id, from_team_name,
                      bid_team_id, bid_team_name, bid_team_region_id,
                      offered_salary, contract_years, transfer_fee, signing_bonus,
                      match_score, willingness, is_winner, reject_reason
               FROM transfer_bids
               WHERE window_id = ? AND round = ?
               ORDER BY player_id, match_score DESC"#
        )
        .bind(resolved_window_id)
        .bind(r)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            r#"SELECT id, window_id, round, player_id, player_name, player_ability, player_age,
                      player_position, from_team_id, from_team_name,
                      bid_team_id, bid_team_name, bid_team_region_id,
                      offered_salary, contract_years, transfer_fee, signing_bonus,
                      match_score, willingness, is_winner, reject_reason
               FROM transfer_bids
               WHERE window_id = ?
               ORDER BY round, player_id, match_score DESC"#
        )
        .bind(resolved_window_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let mut player_map: std::collections::HashMap<(i64, i64), Vec<TransferBid>> = std::collections::HashMap::new();

    for row in &rows {
        let bid = TransferBid {
            id: row.get("id"),
            window_id: row.get("window_id"),
            round: row.get("round"),
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            player_ability: row.get("player_ability"),
            player_age: row.get("player_age"),
            player_position: row.get("player_position"),
            from_team_id: row.get("from_team_id"),
            from_team_name: row.get("from_team_name"),
            bid_team_id: row.get("bid_team_id"),
            bid_team_name: row.get("bid_team_name"),
            bid_team_region_id: row.get("bid_team_region_id"),
            offered_salary: row.get("offered_salary"),
            contract_years: row.get("contract_years"),
            transfer_fee: row.get("transfer_fee"),
            signing_bonus: row.get("signing_bonus"),
            match_score: row.get("match_score"),
            willingness: row.get("willingness"),
            is_winner: row.get::<i32, _>("is_winner") != 0,
            reject_reason: row.get("reject_reason"),
        };
        let key = (bid.player_id, bid.round);
        player_map.entry(key).or_default().push(bid);
    }

    let mut player_analyses: Vec<PlayerBidAnalysis> = Vec::new();
    let mut total_bids: i64 = 0;
    let mut successful: i64 = 0;
    let mut failed: i64 = 0;

    for ((_pid, _rnd), bids) in &player_map {
        let first = &bids[0];
        let winner = bids.iter().find(|b| b.is_winner);
        let outcome = if winner.is_some() { "signed" } else if bids.is_empty() { "no_bids" } else { "no_willing_team" };

        if winner.is_some() {
            successful += 1;
        } else {
            failed += 1;
        }

        total_bids += bids.len() as i64;

        player_analyses.push(PlayerBidAnalysis {
            player_id: first.player_id,
            player_name: first.player_name.clone(),
            player_ability: first.player_ability,
            player_age: first.player_age,
            player_position: first.player_position.clone(),
            from_team_id: first.from_team_id,
            from_team_name: first.from_team_name.clone(),
            round: first.round,
            total_bids: bids.len() as i64,
            bids: bids.clone(),
            winner_team_name: winner.map(|w| w.bid_team_name.clone()),
            outcome: outcome.to_string(),
        });
    }

    player_analyses.sort_by(|a, b| b.player_ability.cmp(&a.player_ability));

    let total_players = player_analyses.len() as i64;
    let avg_bids_per_player = if total_players > 0 {
        total_bids as f64 / total_players as f64
    } else {
        0.0
    };

    Ok(CommandResult::ok(BidOverview {
        window_id: resolved_window_id,
        round,
        total_players,
        total_bids,
        successful_signings: successful,
        failed_signings: failed,
        avg_bids_per_player,
        player_analyses,
    }))
}

#[tauri::command]
pub async fn get_player_bids(
    state: State<'_, AppState>,
    window_id: i64,
    player_id: i64,
) -> Result<CommandResult<PlayerBidAnalysis>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let rows = sqlx::query(
        r#"SELECT id, window_id, round, player_id, player_name, player_ability, player_age,
                  player_position, from_team_id, from_team_name,
                  bid_team_id, bid_team_name, bid_team_region_id,
                  offered_salary, contract_years, transfer_fee, signing_bonus,
                  match_score, willingness, is_winner, reject_reason
           FROM transfer_bids
           WHERE window_id = ? AND player_id = ?
           ORDER BY round, match_score DESC"#
    )
    .bind(window_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(CommandResult::err("未找到该选手的竞价记录"));
    }

    let bids: Vec<TransferBid> = rows.iter().map(|row| TransferBid {
        id: row.get("id"),
        window_id: row.get("window_id"),
        round: row.get("round"),
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        player_ability: row.get("player_ability"),
        player_age: row.get("player_age"),
        player_position: row.get("player_position"),
        from_team_id: row.get("from_team_id"),
        from_team_name: row.get("from_team_name"),
        bid_team_id: row.get("bid_team_id"),
        bid_team_name: row.get("bid_team_name"),
        bid_team_region_id: row.get("bid_team_region_id"),
        offered_salary: row.get("offered_salary"),
        contract_years: row.get("contract_years"),
        transfer_fee: row.get("transfer_fee"),
        signing_bonus: row.get("signing_bonus"),
        match_score: row.get("match_score"),
        willingness: row.get("willingness"),
        is_winner: row.get::<i32, _>("is_winner") != 0,
        reject_reason: row.get("reject_reason"),
    }).collect();

    let first = &bids[0];
    let winner_team_name = bids.iter().find(|b| b.is_winner).map(|w| w.bid_team_name.clone());
    let outcome = if winner_team_name.is_some() { "signed" } else { "no_willing_team" };

    Ok(CommandResult::ok(PlayerBidAnalysis {
        player_id: first.player_id,
        player_name: first.player_name.clone(),
        player_ability: first.player_ability,
        player_age: first.player_age,
        player_position: first.player_position.clone(),
        from_team_id: first.from_team_id,
        from_team_name: first.from_team_name.clone(),
        round: first.round,
        total_bids: bids.len() as i64,
        bids,
        winner_team_name,
        outcome: outcome.to_string(),
    }))
}
