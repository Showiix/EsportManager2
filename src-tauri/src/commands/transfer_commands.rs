use crate::commands::save_commands::{AppState, CommandResult};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 转会市场挂牌信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferListingInfo {
    pub id: u64,
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub potential: u8,
    pub age: u8,
    pub team_id: u64,
    pub team_name: String,
    pub asking_price: u64,
    pub min_price: Option<u64>,
    pub market_value: u64,
    pub listing_type: String,
    pub status: String,
}

/// 自由球员信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FreeAgentInfo {
    pub id: u64,
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub potential: u8,
    pub age: u8,
    pub salary_demand: u64,
    pub reason: String,
    pub status: String,
}

/// 转会记录信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRecordInfo {
    pub id: u64,
    pub player_id: u64,
    pub player_name: String,
    pub from_team: Option<String>,
    pub to_team: Option<String>,
    pub transfer_type: String,
    pub transfer_fee: u64,
    pub new_salary: Option<u64>,
    pub contract_years: Option<u32>,
}

/// 获取转会市场列表
#[tauri::command]
pub async fn get_transfer_market(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<TransferListingInfo>>, String> {
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
        SELECT tl.*, p.game_id, p.position, p.ability, p.potential, p.age, p.market_value,
               t.name as team_name
        FROM transfer_listings tl
        JOIN players p ON tl.player_id = p.id
        JOIN teams t ON tl.team_id = t.id
        WHERE tl.save_id = ? AND tl.status = 'Active'
        ORDER BY p.ability DESC
        "#,
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let infos: Vec<TransferListingInfo> = rows
        .iter()
        .map(|row| TransferListingInfo {
            id: row.get::<i64, _>("id") as u64,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("game_id"),
            position: row.get("position"),
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            age: row.get::<i64, _>("age") as u8,
            team_id: row.get::<i64, _>("team_id") as u64,
            team_name: row.get("team_name"),
            asking_price: row.get::<i64, _>("asking_price") as u64,
            min_price: row.get::<Option<i64>, _>("min_price").map(|v| v as u64),
            market_value: row.get::<i64, _>("market_value") as u64,
            listing_type: row.get("listing_type"),
            status: row.get("status"),
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

/// 获取自由球员市场
#[tauri::command]
pub async fn get_free_agents(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<FreeAgentInfo>>, String> {
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
        SELECT fa.*, p.game_id, p.position, p.ability, p.potential, p.age
        FROM free_agents fa
        JOIN players p ON fa.player_id = p.id
        WHERE fa.save_id = ? AND fa.status = 'Available'
        ORDER BY p.ability DESC
        "#,
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let infos: Vec<FreeAgentInfo> = rows
        .iter()
        .map(|row| FreeAgentInfo {
            id: row.get::<i64, _>("id") as u64,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("game_id"),
            position: row.get("position"),
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            age: row.get::<i64, _>("age") as u8,
            salary_demand: row.get::<i64, _>("salary_demand") as u64,
            reason: row.get("reason"),
            status: row.get("status"),
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

/// 挂牌出售球员
#[tauri::command]
pub async fn list_player_for_transfer(
    state: State<'_, AppState>,
    player_id: u64,
    asking_price: u64,
    min_price: Option<u64>,
) -> Result<CommandResult<u64>, String> {
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

    // 获取球员信息
    let player_row = sqlx::query("SELECT team_id FROM players WHERE id = ?")
        .bind(player_id as i64)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let player_row = match player_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Player not found")),
    };

    let team_id: Option<i64> = player_row.get("team_id");
    let team_id = match team_id {
        Some(id) => id,
        None => return Ok(CommandResult::err("Player has no team")),
    };

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 创建挂牌
    let result = sqlx::query(
        r#"
        INSERT INTO transfer_listings (
            save_id, season_id, player_id, team_id, listing_type, asking_price, min_price, status
        ) VALUES (?, ?, ?, ?, 'ForSale', ?, ?, 'Active')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(player_id as i64)
    .bind(team_id)
    .bind(asking_price as i64)
    .bind(min_price.map(|v| v as i64))
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let listing_id: i64 = result.get("id");

    Ok(CommandResult::ok(listing_id as u64))
}

/// 撤销挂牌
#[tauri::command]
pub async fn cancel_transfer_listing(
    state: State<'_, AppState>,
    listing_id: u64,
) -> Result<CommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    sqlx::query("UPDATE transfer_listings SET status = 'Withdrawn' WHERE id = ?")
        .bind(listing_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(()))
}

/// 购买挂牌球员
#[tauri::command]
pub async fn buy_listed_player(
    state: State<'_, AppState>,
    listing_id: u64,
    buyer_team_id: u64,
    offer_price: u64,
) -> Result<CommandResult<TransferRecordInfo>, String> {
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

    // 获取挂牌信息
    let listing_row = sqlx::query(
        "SELECT * FROM transfer_listings WHERE id = ? AND status = 'Active'"
    )
    .bind(listing_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let listing_row = match listing_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Listing not found or not active")),
    };

    let min_price: Option<i64> = listing_row.get("min_price");
    let asking_price: i64 = listing_row.get("asking_price");

    // 检查报价是否足够
    let min_acceptable = min_price.unwrap_or(asking_price);
    if (offer_price as i64) < min_acceptable {
        return Ok(CommandResult::err("Offer too low"));
    }

    let player_id: i64 = listing_row.get("player_id");
    let seller_team_id: i64 = listing_row.get("team_id");

    // 检查买家资金
    let buyer_row = sqlx::query("SELECT balance FROM teams WHERE id = ?")
        .bind(buyer_team_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let buyer_balance: i64 = buyer_row.get("balance");

    if buyer_balance < offer_price as i64 {
        return Ok(CommandResult::err("Insufficient funds"));
    }

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 执行转会
    // 1. 更新球员所属队伍
    sqlx::query(
        "UPDATE players SET team_id = ?, contract_end_season = ?, is_starter = 0 WHERE id = ?"
    )
    .bind(buyer_team_id as i64)
    .bind(current_season + 3) // 3年合同
    .bind(player_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. 更新买家余额
    sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
        .bind(offer_price as i64)
        .bind(buyer_team_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. 更新卖家余额
    sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
        .bind(offer_price as i64)
        .bind(seller_team_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 4. 更新挂牌状态
    sqlx::query("UPDATE transfer_listings SET status = 'Sold' WHERE id = ?")
        .bind(listing_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 5. 记录转会
    let record_result = sqlx::query(
        r#"
        INSERT INTO transfer_records (
            save_id, season_id, player_id, from_team_id, to_team_id,
            transfer_type, transfer_fee, contract_years
        ) VALUES (?, ?, ?, ?, ?, 'Purchase', ?, 3)
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(player_id)
    .bind(seller_team_id)
    .bind(buyer_team_id as i64)
    .bind(offer_price as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let record_id: i64 = record_result.get("id");

    // 获取球员和队伍名称
    let player_row = sqlx::query("SELECT game_id FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let seller_row = sqlx::query("SELECT name FROM teams WHERE id = ?")
        .bind(seller_team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let buyer_row = sqlx::query("SELECT name FROM teams WHERE id = ?")
        .bind(buyer_team_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(TransferRecordInfo {
        id: record_id as u64,
        player_id: player_id as u64,
        player_name: player_row.get("game_id"),
        from_team: Some(seller_row.get("name")),
        to_team: Some(buyer_row.get("name")),
        transfer_type: "Purchase".to_string(),
        transfer_fee: offer_price,
        new_salary: None,
        contract_years: Some(3),
    }))
}

/// 签约自由球员
#[tauri::command]
pub async fn sign_free_agent(
    state: State<'_, AppState>,
    free_agent_id: u64,
    team_id: u64,
    salary: u64,
    contract_years: u32,
) -> Result<CommandResult<TransferRecordInfo>, String> {
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

    // 获取自由球员信息
    let agent_row = sqlx::query(
        "SELECT * FROM free_agents WHERE id = ? AND status = 'Available'"
    )
    .bind(free_agent_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let agent_row = match agent_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Free agent not available")),
    };

    let player_id: i64 = agent_row.get("player_id");
    let salary_demand: i64 = agent_row.get("salary_demand");

    // 检查薪资是否满足要求
    if (salary as i64) < (salary_demand as f64 * 0.8) as i64 {
        return Ok(CommandResult::err("Salary offer too low"));
    }

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 签约球员
    sqlx::query(
        r#"
        UPDATE players SET
            team_id = ?,
            salary = ?,
            contract_end_season = ?,
            status = 'Active',
            is_starter = 0
        WHERE id = ?
        "#,
    )
    .bind(team_id as i64)
    .bind(salary as i64)
    .bind(current_season + contract_years as i64)
    .bind(player_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 更新自由球员状态
    sqlx::query("UPDATE free_agents SET status = 'Signed' WHERE id = ?")
        .bind(free_agent_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 记录转会
    let record_result = sqlx::query(
        r#"
        INSERT INTO transfer_records (
            save_id, season_id, player_id, from_team_id, to_team_id,
            transfer_type, transfer_fee, new_salary, contract_years
        ) VALUES (?, ?, ?, NULL, ?, 'FreeAgent', 0, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(player_id)
    .bind(team_id as i64)
    .bind(salary as i64)
    .bind(contract_years as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let record_id: i64 = record_result.get("id");

    // 获取球员和队伍名称
    let player_row = sqlx::query("SELECT game_id FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let team_row = sqlx::query("SELECT name FROM teams WHERE id = ?")
        .bind(team_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(TransferRecordInfo {
        id: record_id as u64,
        player_id: player_id as u64,
        player_name: player_row.get("game_id"),
        from_team: None,
        to_team: Some(team_row.get("name")),
        transfer_type: "FreeAgent".to_string(),
        transfer_fee: 0,
        new_salary: Some(salary),
        contract_years: Some(contract_years),
    }))
}

/// 获取转会历史记录
#[tauri::command]
pub async fn get_transfer_history(
    state: State<'_, AppState>,
    season_id: Option<u64>,
) -> Result<CommandResult<Vec<TransferRecordInfo>>, String> {
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

    let query = if let Some(season) = season_id {
        sqlx::query(
            r#"
            SELECT tr.*, p.game_id as player_name,
                   ft.name as from_team_name, tt.name as to_team_name
            FROM transfer_records tr
            JOIN players p ON tr.player_id = p.id
            LEFT JOIN teams ft ON tr.from_team_id = ft.id
            LEFT JOIN teams tt ON tr.to_team_id = tt.id
            WHERE tr.save_id = ? AND tr.season_id = ?
            ORDER BY tr.id DESC
            "#,
        )
        .bind(&save_id)
        .bind(season as i64)
    } else {
        sqlx::query(
            r#"
            SELECT tr.*, p.game_id as player_name,
                   ft.name as from_team_name, tt.name as to_team_name
            FROM transfer_records tr
            JOIN players p ON tr.player_id = p.id
            LEFT JOIN teams ft ON tr.from_team_id = ft.id
            LEFT JOIN teams tt ON tr.to_team_id = tt.id
            WHERE tr.save_id = ?
            ORDER BY tr.id DESC
            "#,
        )
        .bind(&save_id)
    };

    let rows = query.fetch_all(&pool).await.map_err(|e| e.to_string())?;

    let infos: Vec<TransferRecordInfo> = rows
        .iter()
        .map(|row| TransferRecordInfo {
            id: row.get::<i64, _>("id") as u64,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("player_name"),
            from_team: row.get("from_team_name"),
            to_team: row.get("to_team_name"),
            transfer_type: row.get("transfer_type"),
            transfer_fee: row.get::<i64, _>("transfer_fee") as u64,
            new_salary: row.get::<Option<i64>, _>("new_salary").map(|v| v as u64),
            contract_years: row.get::<Option<i64>, _>("contract_years").map(|v| v as u32),
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

// ==================== AI 转会窗口系统 ====================

/// 转会窗口信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferWindowInfo {
    pub id: u64,
    pub season_id: u64,
    pub status: String,
    pub current_round: u32,
    pub total_rounds: u32,
    pub total_transfers: u32,
    pub total_fees: u64,
    pub free_agents_signed: u32,
    pub retirements: u32,
    pub contract_expires: u32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// 转会事件信息（用于新闻播报）
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferEventInfo {
    pub id: u64,
    pub round: u32,
    pub event_type: String,
    pub status: String,
    pub player_id: u64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub market_value: u64,
    pub from_team_id: Option<u64>,
    pub from_team_name: Option<String>,
    pub to_team_id: Option<u64>,
    pub to_team_name: Option<String>,
    pub transfer_fee: u64,
    pub new_salary: Option<u64>,
    pub contract_years: Option<u32>,
    pub contract_type: String,
    pub price_ratio: Option<f64>,
    pub headline: String,
    pub description: String,
    pub importance: String,
    pub competing_teams: Vec<u64>,
    pub was_bidding_war: bool,
    pub created_at: Option<String>,
}

/// 转会轮次摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRoundInfo {
    pub round: u32,
    pub round_name: String,
    pub events_count: u32,
    pub transfers_count: u32,
    pub total_fees: u64,
    pub summary: String,
    pub events: Vec<TransferEventInfo>,
}

/// 开始转会窗口
#[tauri::command]
pub async fn start_transfer_window(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferWindowInfo>, String> {
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 检查是否已有转会窗口
    let existing = sqlx::query(
        "SELECT id FROM transfer_windows WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(CommandResult::err("Transfer window already exists for this season"));
    }

    // 创建转会窗口
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        INSERT INTO transfer_windows (
            save_id, season_id, status, current_round, total_rounds,
            total_transfers, total_fees, free_agents_signed, retirements, contract_expires,
            started_at
        ) VALUES (?, ?, 'IN_PROGRESS', 0, 5, 0, 0, 0, 0, 0, ?)
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let window_id: i64 = result.get("id");

    Ok(CommandResult::ok(TransferWindowInfo {
        id: window_id as u64,
        season_id: current_season as u64,
        status: "IN_PROGRESS".to_string(),
        current_round: 0,
        total_rounds: 5,
        total_transfers: 0,
        total_fees: 0,
        free_agents_signed: 0,
        retirements: 0,
        contract_expires: 0,
        started_at: Some(now),
        completed_at: None,
    }))
}

/// 执行转会轮次
#[tauri::command]
pub async fn execute_transfer_round(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferRoundInfo>, String> {
    use crate::engines::TransferWindowEngine;
    use crate::models::{Player, Team, FreeAgent, TransferListing, get_round_name};
    use std::collections::HashMap;

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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 获取转会窗口状态
    let window_row = sqlx::query(
        "SELECT * FROM transfer_windows WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let window_row = match window_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("No active transfer window")),
    };

    let window_id: i64 = window_row.get("id");
    let current_round: i64 = window_row.get("current_round");
    let status: String = window_row.get("status");

    if status == "COMPLETED" {
        return Ok(CommandResult::err("Transfer window already completed"));
    }

    let next_round = current_round + 1;
    if next_round > 5 {
        return Ok(CommandResult::err("All rounds completed"));
    }

    // 加载所有球队
    let team_rows = sqlx::query(
        "SELECT * FROM teams WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let teams: Vec<Team> = team_rows.iter().map(|row| Team {
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
        balance: row.get("balance"),
    }).collect();

    // 加载所有选手（按球队分组）
    let player_rows = sqlx::query(
        "SELECT * FROM players WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    let mut all_players: Vec<Player> = Vec::new();

    for row in player_rows.iter() {
        // 解析 position
        let position_str: Option<String> = row.get("position");
        let position = position_str.map(|s| match s.to_uppercase().as_str() {
            "TOP" => crate::models::Position::Top,
            "JUG" | "JUNGLE" => crate::models::Position::Jug,
            "MID" => crate::models::Position::Mid,
            "ADC" | "BOT" => crate::models::Position::Adc,
            "SUP" | "SUPPORT" => crate::models::Position::Sup,
            _ => crate::models::Position::Mid,
        });

        // 解析 tag
        let tag_str: String = row.get("tag");
        let tag = match tag_str.to_uppercase().as_str() {
            "GENIUS" => crate::models::PlayerTag::Genius,
            "ORDINARY" => crate::models::PlayerTag::Ordinary,
            _ => crate::models::PlayerTag::Normal,
        };

        // 解析 status
        let status_str: String = row.get("status");
        let status = match status_str.to_uppercase().as_str() {
            "RETIRED" => crate::models::PlayerStatus::Retired,
            _ => crate::models::PlayerStatus::Active,
        };

        let player = Player {
            id: row.get::<i64, _>("id") as u64,
            game_id: row.get("game_id"),
            real_name: row.get("real_name"),
            nationality: row.get("nationality"),
            age: row.get::<i64, _>("age") as u8,
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            stability: row.get::<i64, _>("stability") as u8,
            tag,
            status,
            position,
            team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
            salary: row.get::<i64, _>("salary") as u64,
            market_value: row.get::<i64, _>("market_value") as u64,
            contract_end_season: row.get::<Option<i64>, _>("contract_end_season").map(|v| v as u32),
            join_season: row.get::<i64, _>("join_season") as u32,
            retire_season: row.get::<Option<i64>, _>("retire_season").map(|v| v as u32),
            is_starter: row.get::<i64, _>("is_starter") != 0,
        };

        if let Some(tid) = player.team_id {
            players_by_team.entry(tid).or_default().push(player.clone());
        }
        all_players.push(player);
    }

    // 创建转会引擎
    let mut engine = TransferWindowEngine::new(save_id.clone(), current_season as u64);

    // 加载自由球员
    let fa_rows = sqlx::query(
        "SELECT * FROM free_agents WHERE save_id = ? AND season_id = ? AND status = 'AVAILABLE'"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let free_agents: Vec<FreeAgent> = fa_rows.iter().map(|row| {
        let reason_str: String = row.get("reason");
        let reason = match reason_str.to_uppercase().as_str() {
            "RELEASED" => crate::models::FreeAgentReason::Released,
            "RETIRED_TEAM" => crate::models::FreeAgentReason::RetiredTeam,
            _ => crate::models::FreeAgentReason::ContractExpire,
        };

        let status_str: String = row.get("status");
        let fa_status = match status_str.to_uppercase().as_str() {
            "SIGNED" => crate::models::FreeAgentStatus::Signed,
            "RETIRED" => crate::models::FreeAgentStatus::Retired,
            _ => crate::models::FreeAgentStatus::Available,
        };

        FreeAgent {
            id: row.get::<i64, _>("id") as u64,
            save_id: row.get("save_id"),
            season_id: row.get::<i64, _>("season_id") as u64,
            player_id: row.get::<i64, _>("player_id") as u64,
            salary_demand: row.get::<i64, _>("salary_demand") as u64,
            reason,
            status: fa_status,
        }
    }).collect();

    // 加载挂牌球员
    let listing_rows = sqlx::query(
        "SELECT * FROM transfer_listings WHERE save_id = ? AND season_id = ? AND status = 'ACTIVE'"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let listings: Vec<TransferListing> = listing_rows.iter().map(|row| {
        let listing_type_str: String = row.get("listing_type");
        let listing_type = match listing_type_str.to_uppercase().as_str() {
            "LOAN_AVAILABLE" | "LOANAVAILABLE" => crate::models::ListingType::LoanAvailable,
            _ => crate::models::ListingType::ForSale,
        };

        let status_str: String = row.get("status");
        let listing_status = match status_str.to_uppercase().as_str() {
            "SOLD" => crate::models::ListingStatus::Sold,
            "WITHDRAWN" => crate::models::ListingStatus::Withdrawn,
            _ => crate::models::ListingStatus::Active,
        };

        TransferListing {
            id: row.get::<i64, _>("id") as u64,
            save_id: row.get("save_id"),
            season_id: row.get::<i64, _>("season_id") as u64,
            player_id: row.get::<i64, _>("player_id") as u64,
            team_id: row.get::<i64, _>("team_id") as u64,
            listing_type,
            asking_price: row.get::<i64, _>("asking_price") as u64,
            min_price: row.get::<Option<i64>, _>("min_price").map(|v| v as u64),
            status: listing_status,
        }
    }).collect();

    // 初始化引擎
    engine.initialize(&teams, &players_by_team, free_agents, listings);

    // 执行对应轮次
    let events = match next_round {
        1 => engine.execute_round_1_contracts_and_retirements(&mut all_players, &teams),
        2 => engine.execute_round_2_free_agents(&teams, &players_by_team),
        3 => engine.execute_round_3_financial_clearance(&teams, &players_by_team),
        4 => engine.execute_round_4_reinforcement(&teams, &players_by_team),
        5 => engine.execute_round_5_finalize(&teams),
        _ => vec![],
    };

    // 保存事件到数据库
    for event in &events {
        let competing_json = serde_json::to_string(&event.competing_teams).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO transfer_events (
                save_id, season_id, round, event_type, status,
                player_id, player_name, position, age, ability, potential, market_value,
                from_team_id, from_team_name, to_team_id, to_team_name,
                transfer_fee, new_salary, contract_years, contract_type,
                price_ratio, headline, description, importance,
                competing_teams, was_bidding_war, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event.save_id)
        .bind(event.season_id as i64)
        .bind(event.round as i64)
        .bind(format!("{:?}", event.event_type).to_uppercase())
        .bind(format!("{:?}", event.status).to_uppercase())
        .bind(event.player_id as i64)
        .bind(&event.player_name)
        .bind(&event.position)
        .bind(event.age as i64)
        .bind(event.ability as i64)
        .bind(event.potential as i64)
        .bind(event.market_value as i64)
        .bind(event.from_team_id.map(|v| v as i64))
        .bind(&event.from_team_name)
        .bind(event.to_team_id.map(|v| v as i64))
        .bind(&event.to_team_name)
        .bind(event.transfer_fee as i64)
        .bind(event.new_salary.map(|v| v as i64))
        .bind(event.contract_years.map(|v| v as i64))
        .bind(format!("{:?}", event.contract_type).to_uppercase())
        .bind(event.price_ratio)
        .bind(&event.headline)
        .bind(&event.description)
        .bind(format!("{:?}", event.importance).to_uppercase())
        .bind(&competing_json)
        .bind(if event.was_bidding_war { 1i64 } else { 0i64 })
        .bind(&event.created_at)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 根据事件类型更新数据库
        match event.event_type {
            crate::models::TransferEventType::Retirement => {
                sqlx::query(
                    "UPDATE players SET status = 'Retired', retire_season = ? WHERE id = ?"
                )
                .bind(current_season)
                .bind(event.player_id as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            crate::models::TransferEventType::ContractExpire => {
                // 添加到自由球员池
                sqlx::query(
                    r#"
                    INSERT INTO free_agents (save_id, season_id, player_id, salary_demand, reason, status)
                    VALUES (?, ?, ?, ?, 'CONTRACT_EXPIRE', 'AVAILABLE')
                    "#
                )
                .bind(&save_id)
                .bind(current_season)
                .bind(event.player_id as i64)
                .bind(event.new_salary.unwrap_or(50) as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

                // 清除球员球队
                sqlx::query("UPDATE players SET team_id = NULL WHERE id = ?")
                    .bind(event.player_id as i64)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            crate::models::TransferEventType::FreeAgent => {
                if let Some(to_team_id) = event.to_team_id {
                    // 更新球员
                    sqlx::query(
                        r#"
                        UPDATE players SET
                            team_id = ?,
                            salary = ?,
                            contract_end_season = ?,
                            status = 'Active',
                            is_starter = 0
                        WHERE id = ?
                        "#
                    )
                    .bind(to_team_id as i64)
                    .bind(event.new_salary.unwrap_or(50) as i64)
                    .bind(current_season + event.contract_years.unwrap_or(2) as i64)
                    .bind(event.player_id as i64)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;

                    // 更新自由球员状态
                    sqlx::query(
                        "UPDATE free_agents SET status = 'SIGNED' WHERE player_id = ? AND save_id = ?"
                    )
                    .bind(event.player_id as i64)
                    .bind(&save_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
            crate::models::TransferEventType::Purchase => {
                if let (Some(from_team_id), Some(to_team_id)) = (event.from_team_id, event.to_team_id) {
                    // 更新球员
                    sqlx::query(
                        r#"
                        UPDATE players SET
                            team_id = ?,
                            salary = ?,
                            contract_end_season = ?,
                            is_starter = 0
                        WHERE id = ?
                        "#
                    )
                    .bind(to_team_id as i64)
                    .bind(event.new_salary.unwrap_or(50) as i64)
                    .bind(current_season + event.contract_years.unwrap_or(2) as i64)
                    .bind(event.player_id as i64)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;

                    // 更新买家余额
                    sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
                        .bind(event.transfer_fee as i64)
                        .bind(to_team_id as i64)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 更新卖家余额
                    sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
                        .bind(event.transfer_fee as i64)
                        .bind(from_team_id as i64)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // 生成轮次摘要
    let summary = engine.generate_round_summary(next_round as u32);

    // 更新转会窗口状态
    let new_status = if next_round >= 5 { "COMPLETED" } else { "IN_PROGRESS" };
    let completed_at = if next_round >= 5 { Some(chrono::Utc::now().to_rfc3339()) } else { None };

    sqlx::query(
        r#"
        UPDATE transfer_windows SET
            current_round = ?,
            status = ?,
            total_transfers = total_transfers + ?,
            total_fees = total_fees + ?,
            free_agents_signed = free_agents_signed + ?,
            retirements = retirements + ?,
            contract_expires = contract_expires + ?,
            completed_at = COALESCE(?, completed_at)
        WHERE id = ?
        "#
    )
    .bind(next_round)
    .bind(new_status)
    .bind(summary.transfers_count as i64)
    .bind(summary.total_fees as i64)
    .bind(events.iter().filter(|e| e.event_type == crate::models::TransferEventType::FreeAgent).count() as i64)
    .bind(events.iter().filter(|e| e.event_type == crate::models::TransferEventType::Retirement).count() as i64)
    .bind(events.iter().filter(|e| e.event_type == crate::models::TransferEventType::ContractExpire).count() as i64)
    .bind(&completed_at)
    .bind(window_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 构造返回事件信息
    let event_infos: Vec<TransferEventInfo> = events.iter().map(|e| TransferEventInfo {
        id: 0,
        round: e.round,
        event_type: format!("{:?}", e.event_type).to_uppercase(),
        status: format!("{:?}", e.status).to_uppercase(),
        player_id: e.player_id,
        player_name: e.player_name.clone(),
        position: e.position.clone(),
        age: e.age,
        ability: e.ability,
        potential: e.potential,
        market_value: e.market_value,
        from_team_id: e.from_team_id,
        from_team_name: e.from_team_name.clone(),
        to_team_id: e.to_team_id,
        to_team_name: e.to_team_name.clone(),
        transfer_fee: e.transfer_fee,
        new_salary: e.new_salary,
        contract_years: e.contract_years,
        contract_type: format!("{:?}", e.contract_type).to_uppercase(),
        price_ratio: e.price_ratio,
        headline: e.headline.clone(),
        description: e.description.clone(),
        importance: format!("{:?}", e.importance).to_uppercase(),
        competing_teams: e.competing_teams.clone(),
        was_bidding_war: e.was_bidding_war,
        created_at: e.created_at.clone(),
    }).collect();

    Ok(CommandResult::ok(TransferRoundInfo {
        round: next_round as u32,
        round_name: get_round_name(next_round as u32).to_string(),
        events_count: summary.events_count,
        transfers_count: summary.transfers_count,
        total_fees: summary.total_fees,
        summary: summary.summary,
        events: event_infos,
    }))
}

/// 快进完成所有转会
#[tauri::command]
pub async fn fast_forward_transfers(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferWindowInfo>, String> {
    // 循环执行所有剩余轮次
    loop {
        let result = execute_transfer_round(state.clone()).await?;
        match result {
            CommandResult { success: true, data: Some(round_info), .. } => {
                if round_info.round >= 5 {
                    break;
                }
            }
            CommandResult { success: false, error: Some(err), .. } => {
                if err.contains("completed") || err.contains("All rounds") {
                    break;
                }
                return Ok(CommandResult::err(err));
            }
            _ => break,
        }
    }

    // 返回最终状态
    get_transfer_window_status(state).await
}

/// 获取转会窗口状态
#[tauri::command]
pub async fn get_transfer_window_status(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferWindowInfo>, String> {
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 获取转会窗口
    let window_row = sqlx::query(
        "SELECT * FROM transfer_windows WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match window_row {
        Some(row) => Ok(CommandResult::ok(TransferWindowInfo {
            id: row.get::<i64, _>("id") as u64,
            season_id: row.get::<i64, _>("season_id") as u64,
            status: row.get("status"),
            current_round: row.get::<i64, _>("current_round") as u32,
            total_rounds: row.get::<i64, _>("total_rounds") as u32,
            total_transfers: row.get::<i64, _>("total_transfers") as u32,
            total_fees: row.get::<i64, _>("total_fees") as u64,
            free_agents_signed: row.get::<i64, _>("free_agents_signed") as u32,
            retirements: row.get::<i64, _>("retirements") as u32,
            contract_expires: row.get::<i64, _>("contract_expires") as u32,
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
        })),
        None => Ok(CommandResult::err("No transfer window found")),
    }
}

/// 获取转会事件列表
#[tauri::command]
pub async fn get_transfer_events(
    state: State<'_, AppState>,
    round: Option<u32>,
) -> Result<CommandResult<Vec<TransferEventInfo>>, String> {
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let rows = if let Some(r) = round {
        sqlx::query(
            "SELECT * FROM transfer_events WHERE save_id = ? AND season_id = ? AND round = ? ORDER BY id"
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(r as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            "SELECT * FROM transfer_events WHERE save_id = ? AND season_id = ? ORDER BY round, id"
        )
        .bind(&save_id)
        .bind(current_season)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let events: Vec<TransferEventInfo> = rows.iter().map(|row| {
        let competing_str: String = row.get("competing_teams");
        let competing_teams: Vec<u64> = serde_json::from_str(&competing_str).unwrap_or_default();

        TransferEventInfo {
            id: row.get::<i64, _>("id") as u64,
            round: row.get::<i64, _>("round") as u32,
            event_type: row.get("event_type"),
            status: row.get("status"),
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get::<i64, _>("age") as u8,
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            market_value: row.get::<i64, _>("market_value") as u64,
            from_team_id: row.get::<Option<i64>, _>("from_team_id").map(|v| v as u64),
            from_team_name: row.get("from_team_name"),
            to_team_id: row.get::<Option<i64>, _>("to_team_id").map(|v| v as u64),
            to_team_name: row.get("to_team_name"),
            transfer_fee: row.get::<i64, _>("transfer_fee") as u64,
            new_salary: row.get::<Option<i64>, _>("new_salary").map(|v| v as u64),
            contract_years: row.get::<Option<i64>, _>("contract_years").map(|v| v as u32),
            contract_type: row.get::<String, _>("contract_type"),
            price_ratio: row.get("price_ratio"),
            headline: row.get("headline"),
            description: row.get("description"),
            importance: row.get("importance"),
            competing_teams,
            was_bidding_war: row.get::<i64, _>("was_bidding_war") != 0,
            created_at: row.get("created_at"),
        }
    }).collect();

    Ok(CommandResult::ok(events))
}
