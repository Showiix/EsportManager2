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
