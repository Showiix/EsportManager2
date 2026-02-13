use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::TransferEngine;
use crate::models::transfer::*;
use sqlx::Row;
use tauri::State;

use super::ReleasePlayerResult;

#[tauri::command]
pub async fn get_transfer_events(
    state: State<'_, AppState>,
    window_id: i64,
    round: Option<i64>,
    level: Option<String>,
) -> Result<CommandResult<Vec<TransferEvent>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let engine = TransferEngine::new();
    let events = engine.get_events(&pool, window_id, round, level.as_deref()).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(events))
}

#[tauri::command]
pub async fn get_transfer_report(
    state: State<'_, AppState>,
    window_id: i64,
) -> Result<CommandResult<TransferReport>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let engine = TransferEngine::new();
    let report = engine.generate_transfer_report(&pool, window_id).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(report))
}

#[tauri::command]
pub async fn get_player_market_list(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<PlayerContractInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let rows = sqlx::query(
        r#"
        SELECT
            p.id as player_id,
            p.game_id as player_name,
            p.position,
            p.age,
            p.ability,
            p.potential,
            p.team_id,
            t.name as team_name,
            t.short_name as team_short_name,
            r.short_name as region_code,
            p.salary,
            p.contract_end_season,
            p.join_season,
            p.market_value as base_market_value,
            p.calculated_market_value,
            p.satisfaction,
            p.loyalty,
            p.is_starter,
            p.status
        FROM players p
        LEFT JOIN teams t ON p.team_id = t.id
        LEFT JOIN regions r ON t.region_id = r.id
        WHERE p.save_id = ? AND p.status != 'RETIRED'
        ORDER BY p.ability DESC, p.potential DESC
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let players: Vec<PlayerContractInfo> = rows
        .iter()
        .map(|row| PlayerContractInfo {
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get("age"),
            ability: row.get("ability"),
            potential: row.get("potential"),
            team_id: row.get("team_id"),
            team_name: row.get("team_name"),
            team_short_name: row.get("team_short_name"),
            region_code: row.get("region_code"),
            salary: row.get("salary"),
            contract_end_season: row.get("contract_end_season"),
            join_season: row.get("join_season"),
            base_market_value: row.get("base_market_value"),
            calculated_market_value: row.get("calculated_market_value"),
            satisfaction: row.get("satisfaction"),
            loyalty: row.get("loyalty"),
            is_starter: row.get("is_starter"),
            status: row.get("status"),
        })
        .collect();

    Ok(CommandResult::ok(players))
}

#[tauri::command]
pub async fn get_transfer_market_listings(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferMarketData>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let window_row = sqlx::query(
        "SELECT id, season_id, status, current_round FROM transfer_windows WHERE save_id = ? ORDER BY season_id DESC, id DESC LIMIT 1"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (window_id, window_season, window_status, current_round) = match &window_row {
        Some(row) => (
            Some(row.get::<i64, _>("id")),
            row.get::<i64, _>("season_id"),
            Some(row.get::<String, _>("status")),
            Some(row.get::<i64, _>("current_round")),
        ),
        None => (None, current_season, None, None),
    };

    let listings = if let Some(wid) = window_id {
        let rows = sqlx::query(
            r#"
            SELECT
                pl.id as listing_id,
                pl.window_id,
                pl.listing_price,
                pl.min_accept_price,
                pl.status as listing_status,
                pl.listed_at,
                pl.sold_at,
                pl.actual_price,
                pl.player_id,
                p.game_id as player_name,
                p.position,
                p.age,
                p.ability,
                p.potential,
                p.calculated_market_value,
                pl.listed_by_team_id,
                lt.name as listed_by_team_name,
                lr.short_name as listed_by_region_code,
                pl.sold_to_team_id,
                st.name as sold_to_team_name,
                sr.short_name as sold_to_region_code
            FROM player_listings pl
            JOIN players p ON pl.player_id = p.id
            JOIN teams lt ON pl.listed_by_team_id = lt.id
            LEFT JOIN regions lr ON lt.region_id = lr.id
            LEFT JOIN teams st ON pl.sold_to_team_id = st.id
            LEFT JOIN regions sr ON st.region_id = sr.id
            WHERE pl.window_id = ? AND pl.status IN ('ACTIVE', 'SOLD')
            ORDER BY pl.status ASC, p.ability DESC
            "#
        )
        .bind(wid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter().map(|row| TransferMarketListingInfo {
            listing_id: row.get("listing_id"),
            window_id: row.get("window_id"),
            listing_price: row.get("listing_price"),
            min_accept_price: row.get("min_accept_price"),
            listing_status: row.get("listing_status"),
            listed_at: row.get("listed_at"),
            sold_at: row.get("sold_at"),
            actual_price: row.get("actual_price"),
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get("age"),
            ability: row.get("ability"),
            potential: row.get("potential"),
            calculated_market_value: row.get("calculated_market_value"),
            listed_by_team_id: row.get("listed_by_team_id"),
            listed_by_team_name: row.get("listed_by_team_name"),
            listed_by_region_code: row.get("listed_by_region_code"),
            sold_to_team_id: row.get("sold_to_team_id"),
            sold_to_team_name: row.get("sold_to_team_name"),
            sold_to_region_code: row.get("sold_to_region_code"),
        }).collect()
    } else {
        vec![]
    };

    let free_agent_rows = sqlx::query(
        r#"
        SELECT
            p.id as player_id,
            p.game_id as player_name,
            p.position,
            p.age,
            p.ability,
            p.potential,
            p.calculated_market_value,
            p.salary
        FROM players p
        WHERE p.save_id = ? AND p.team_id IS NULL AND p.status != 'RETIRED'
        ORDER BY p.ability DESC
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let free_agents: Vec<FreeAgentInfo> = free_agent_rows.iter().map(|row| FreeAgentInfo {
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        position: row.get("position"),
        age: row.get("age"),
        ability: row.get("ability"),
        potential: row.get("potential"),
        calculated_market_value: row.get("calculated_market_value"),
        salary: row.get("salary"),
    }).collect();

    Ok(CommandResult::ok(TransferMarketData {
        listings,
        free_agents,
        window_status,
        window_id,
        current_round,
        season_id: window_season,
    }))
}

#[tauri::command]
pub async fn release_player(
    state: State<'_, AppState>,
    player_id: i64,
) -> Result<CommandResult<ReleasePlayerResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let player = sqlx::query(
        r#"SELECT p.id, p.game_id, p.team_id, p.ability, p.age, p.potential, p.tag, p.position,
                  p.salary, p.calculated_market_value, p.status,
                  t.name as team_name, t.balance as team_balance
           FROM players p
           LEFT JOIN teams t ON p.team_id = t.id
           WHERE p.id = ? AND p.save_id = ?"#
    )
    .bind(player_id)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("查询选手失败: {}", e))?
    .ok_or("选手不存在")?;

    let team_id: Option<i64> = player.get("team_id");
    let team_id = team_id.ok_or("该选手没有所属战队，无法解约")?;
    let game_id: String = player.get("game_id");
    let ability: i64 = player.get("ability");
    let age: i64 = player.get("age");
    let potential: i64 = player.get("potential");
    let tag: String = player.get("tag");
    let position: String = player.get("position");
    let calculated_market_value: i64 = player.try_get("calculated_market_value").unwrap_or(0);
    let team_name: String = player.get("team_name");
    let team_balance: i64 = player.get("team_balance");

    let market_value = if calculated_market_value > 0 {
        calculated_market_value
    } else {
        crate::engines::MarketValueEngine::calculate_base_market_value(
            ability as u8, age as u8, potential as u8, &tag, &position
        ) as i64
    };

    let release_fee = market_value / 2;

    if team_balance < release_fee {
        return Ok(CommandResult::err(&format!(
            "余额不足：解约{}需要{}万，当前余额{}万",
            game_id, release_fee / 10000, team_balance / 10000
        )));
    }

    let season_id: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(season), 1) FROM game_time WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("查询赛季失败: {}", e))?;

    sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
        .bind(release_fee)
        .bind(team_id)
        .bind(&save_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("扣除解约金失败: {}", e))?;

    sqlx::query(
        "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, 'Penalty', ?, ?)"
    )
    .bind(&save_id)
    .bind(team_id)
    .bind(season_id)
    .bind(-release_fee)
    .bind(format!("解约{}，支付解约金{}万", game_id, release_fee / 10000))
    .execute(&pool)
    .await
    .map_err(|e| format!("记录解约交易失败: {}", e))?;

    sqlx::query(
        "UPDATE players SET team_id = NULL, is_starter = 0, satisfaction = MAX(satisfaction - 15, 0) WHERE id = ? AND save_id = ?"
    )
    .bind(player_id)
    .bind(&save_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("释放选手失败: {}", e))?;

    sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
        .bind(&save_id)
        .bind(player_id)
        .execute(&pool)
        .await
        .ok();

    let window: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM transfer_windows WHERE save_id = ? AND status = 'IN_PROGRESS' LIMIT 1"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("查询转会窗口失败: {}", e))?;

    if let Some((window_id,)) = window {
        sqlx::query(
            r#"INSERT INTO transfer_events (window_id, round, event_type, event_level, player_id, player_game_id, player_ability,
               from_team_id, from_team_name, transfer_fee, salary, contract_years, description)
               VALUES (?, 0, 'PLAYER_RELEASE', 'B', ?, ?, ?, ?, ?, ?, ?, 0, ?)"#
        )
        .bind(window_id)
        .bind(player_id)
        .bind(&game_id)
        .bind(ability)
        .bind(team_id)
        .bind(&team_name)
        .bind(release_fee)
        .bind(player.get::<i64, _>("salary"))
        .bind(format!("{}解约{}，支付解约金{}万", team_name, game_id, release_fee / 10000))
        .execute(&pool)
        .await
        .ok();
    }

    log::info!("解约: {}从{}解约，解约金{}万", game_id, team_name, release_fee / 10000);

    Ok(CommandResult::ok(ReleasePlayerResult {
        player_id,
        player_name: game_id,
        team_id,
        team_name,
        release_fee,
        remaining_balance: team_balance - release_fee,
    }))
}
