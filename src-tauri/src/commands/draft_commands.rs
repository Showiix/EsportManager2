use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::DraftEngine;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// 选秀球员信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftPlayerInfo {
    pub id: u64,
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub position: String,
    pub tag: String,
    pub draft_rank: u32,
    pub is_picked: bool,
}

/// 选秀顺位信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftOrderInfo {
    pub team_id: u64,
    pub team_name: String,
    pub summer_rank: u32,
    pub draft_position: u32,
    pub lottery_result: Option<String>,
}

/// 选秀结果信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftPickInfo {
    pub pick_number: u32,
    pub team_id: u64,
    pub team_name: String,
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub potential: u8,
}

/// 生成选秀球员池
#[tauri::command]
pub async fn generate_draft_pool(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftPlayerInfo>>, String> {
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

    // 生成选秀球员
    let mut rng = StdRng::from_entropy();
    let positions = vec!["Top", "Jungle", "Mid", "Bot", "Support"];
    let tags = vec!["Normal", "Genius", "Ordinary"];

    let mut draft_players = Vec::new();
    for i in 0..20 {
        let position = positions[i % 5];
        let tag = tags[rng.gen_range(0..3)];
        let ability = 55 + rng.gen_range(0..25);
        let potential = ability + rng.gen_range(5..20);

        draft_players.push(DraftPlayerInfo {
            id: 0,
            game_id: format!("Rookie_{}_{}_{}", region_id, current_season, i + 1),
            real_name: None,
            nationality: Some("CN".to_string()),
            age: 17 + rng.gen_range(0..3),
            ability,
            potential: potential.min(99),
            position: position.to_string(),
            tag: tag.to_string(),
            draft_rank: (i + 1) as u32,
            is_picked: false,
        });
    }

    // 按潜力排序
    draft_players.sort_by(|a, b| {
        let score_a = a.ability as f64 * 0.4 + a.potential as f64 * 0.6;
        let score_b = b.ability as f64 * 0.4 + b.potential as f64 * 0.6;
        score_b.partial_cmp(&score_a).unwrap()
    });

    // 更新排名
    for (i, player) in draft_players.iter_mut().enumerate() {
        player.draft_rank = (i + 1) as u32;
    }

    // 保存到数据库
    for player in &mut draft_players {
        let result = sqlx::query(
            r#"
            INSERT INTO draft_players (
                save_id, season_id, region_id, game_id, real_name, nationality,
                age, ability, potential, position, tag, draft_rank, is_picked
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
            RETURNING id
            "#,
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .bind(&player.game_id)
        .bind(&player.real_name)
        .bind(&player.nationality)
        .bind(player.age as i64)
        .bind(player.ability as i64)
        .bind(player.potential as i64)
        .bind(&player.position)
        .bind(&player.tag)
        .bind(player.draft_rank as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

        player.id = result.get::<i64, _>("id") as u64;
    }

    Ok(CommandResult::ok(draft_players))
}

/// 执行选秀抽签
#[tauri::command]
pub async fn run_draft_lottery(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftOrderInfo>>, String> {
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

    // 获取赛区队伍及其排名
    let team_rows = sqlx::query(
        "SELECT id, name, power_rating FROM teams WHERE save_id = ? AND region_id = ? ORDER BY power_rating ASC"
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 构建排名 (team_id, team_name, summer_rank)
    let rankings: Vec<(u64, String, u32)> = team_rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            (id as u64, name, (team_rows.len() - idx) as u32)
        })
        .collect();

    // 转换为引擎需要的格式 (team_id, summer_rank)
    let teams_for_draft: Vec<(u64, u32)> = rankings.iter().map(|(id, _, rank)| (*id, *rank)).collect();

    // 使用DraftEngine执行抽签
    let mut draft_engine = DraftEngine::new();
    let draft_orders = draft_engine.generate_draft_order(&save_id, current_season as u64, region_id, &teams_for_draft);

    // 保存到数据库
    for order in &draft_orders {
        sqlx::query(
            r#"
            INSERT INTO draft_orders (
                save_id, season_id, region_id, team_id, summer_rank, draft_position, lottery_result
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .bind(order.team_id as i64)
        .bind(order.summer_rank as i64)
        .bind(order.draft_position as i64)
        .bind(&order.lottery_result)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 构建返回信息
    let infos: Vec<DraftOrderInfo> = draft_orders
        .into_iter()
        .map(|o| {
            let team_name = rankings
                .iter()
                .find(|(id, _, _)| *id == o.team_id)
                .map(|(_, name, _)| name.clone())
                .unwrap_or_default();
            DraftOrderInfo {
                team_id: o.team_id,
                team_name,
                summer_rank: o.summer_rank,
                draft_position: o.draft_position,
                lottery_result: o.lottery_result,
            }
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

/// 获取选秀顺位
#[tauri::command]
pub async fn get_draft_order(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftOrderInfo>>, String> {
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

    let rows = sqlx::query(
        r#"
        SELECT do.team_id, do.summer_rank, do.draft_position, do.lottery_result, t.name as team_name
        FROM draft_orders do
        JOIN teams t ON do.team_id = t.id
        WHERE do.save_id = ? AND do.season_id = ? AND do.region_id = ?
        ORDER BY do.draft_position ASC
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let infos: Vec<DraftOrderInfo> = rows
        .iter()
        .map(|row| DraftOrderInfo {
            team_id: row.get::<i64, _>("team_id") as u64,
            team_name: row.get("team_name"),
            summer_rank: row.get::<i64, _>("summer_rank") as u32,
            draft_position: row.get::<i64, _>("draft_position") as u32,
            lottery_result: row.get("lottery_result"),
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

/// 获取可选秀球员
#[tauri::command]
pub async fn get_available_draft_players(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftPlayerInfo>>, String> {
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

    let rows = sqlx::query(
        r#"
        SELECT * FROM draft_players
        WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0
        ORDER BY draft_rank ASC
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let infos: Vec<DraftPlayerInfo> = rows
        .iter()
        .map(|row| DraftPlayerInfo {
            id: row.get::<i64, _>("id") as u64,
            game_id: row.get("game_id"),
            real_name: row.get("real_name"),
            nationality: row.get("nationality"),
            age: row.get::<i64, _>("age") as u8,
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            position: row.get("position"),
            tag: row.get("tag"),
            draft_rank: row.get::<i64, _>("draft_rank") as u32,
            is_picked: row.get::<i64, _>("is_picked") != 0,
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

/// 执行选秀选人
#[tauri::command]
pub async fn make_draft_pick(
    state: State<'_, AppState>,
    team_id: u64,
    draft_player_id: u64,
) -> Result<CommandResult<DraftPickInfo>, String> {
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

    // 获取选秀球员信息
    let player_row = sqlx::query("SELECT * FROM draft_players WHERE id = ? AND is_picked = 0")
        .bind(draft_player_id as i64)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let player_row = match player_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Draft player not available")),
    };

    let region_id: i64 = player_row.get("region_id");

    // 获取当前选秀轮次
    let pick_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM draft_results WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let pick_number = (pick_count + 1) as u32;

    // 标记球员已被选中
    sqlx::query("UPDATE draft_players SET is_picked = 1, picked_by_team_id = ? WHERE id = ?")
        .bind(team_id as i64)
        .bind(draft_player_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 创建正式球员
    let new_player_id: i64 = sqlx::query(
        r#"
        INSERT INTO players (
            save_id, game_id, real_name, nationality, age, ability, potential, stability,
            tag, status, position, team_id, salary, market_value, contract_end_season,
            join_season, is_starter
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'Active', ?, ?, ?, ?, ?, ?, 0)
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(player_row.get::<String, _>("game_id"))
    .bind(player_row.get::<Option<String>, _>("real_name"))
    .bind(player_row.get::<Option<String>, _>("nationality"))
    .bind(player_row.get::<i64, _>("age"))
    .bind(player_row.get::<i64, _>("ability"))
    .bind(player_row.get::<i64, _>("potential"))
    .bind(80i64) // stability
    .bind(player_row.get::<String, _>("tag"))
    .bind(player_row.get::<String, _>("position"))
    .bind(team_id as i64)
    .bind(20i64) // 新秀合同薪资
    .bind(50i64) // 初始市场价值
    .bind(current_season + 3) // 3年新秀合同
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    // 记录选秀结果
    sqlx::query(
        r#"
        INSERT INTO draft_results (
            save_id, season_id, region_id, draft_player_id, team_id, pick_number, player_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id)
    .bind(draft_player_id as i64)
    .bind(team_id as i64)
    .bind(pick_number as i64)
    .bind(new_player_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 获取队伍名称
    let team_row = sqlx::query("SELECT name FROM teams WHERE id = ?")
        .bind(team_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(DraftPickInfo {
        pick_number,
        team_id,
        team_name: team_row.get("name"),
        player_id: new_player_id as u64,
        player_name: player_row.get::<String, _>("game_id"),
        position: player_row.get("position"),
        ability: player_row.get::<i64, _>("ability") as u8,
        potential: player_row.get::<i64, _>("potential") as u8,
    }))
}

/// AI自动选秀
#[tauri::command]
pub async fn ai_auto_draft(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftPickInfo>>, String> {
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

    // 获取选秀顺位
    let order_rows = sqlx::query(
        "SELECT team_id FROM draft_orders WHERE save_id = ? AND season_id = ? AND region_id = ? ORDER BY draft_position"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 释放锁
    drop(guard);
    drop(current_save);

    let mut picks = Vec::new();

    for order_row in order_rows {
        let team_id: i64 = order_row.get("team_id");

        // 重新获取锁查询最佳球员
        let guard = state.db.read().await;
        let db = match guard.as_ref() {
            Some(db) => db,
            None => continue,
        };

        let pool = match db.get_pool().await {
            Ok(p) => p,
            Err(_) => continue,
        };

        let current_save = state.current_save_id.read().await;
        let save_id = match current_save.as_ref() {
            Some(id) => id.clone(),
            None => continue,
        };

        // 获取最佳可用球员
        let best_player = sqlx::query(
            r#"
            SELECT id FROM draft_players
            WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0
            ORDER BY (ability * 0.4 + potential * 0.6) DESC
            LIMIT 1
            "#,
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();

        // 释放锁
        drop(guard);
        drop(current_save);

        if let Some(player_row) = best_player {
            let draft_player_id: i64 = player_row.get("id");

            // 调用make_draft_pick
            let result = make_draft_pick(state.clone(), team_id as u64, draft_player_id as u64).await?;
            if let Some(pick_info) = result.data {
                picks.push(pick_info);
            }
        }
    }

    Ok(CommandResult::ok(picks))
}
