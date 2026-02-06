use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::DraftEngine;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

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
    pub original_team_id: Option<u64>,
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

/// 生成选秀球员池（从 draft_pool 表随机抽取）
#[tauri::command]
pub async fn generate_draft_pool(
    state: State<'_, AppState>,
    region_id: u64,
    pool_size: Option<u32>,
) -> Result<CommandResult<Vec<DraftPlayerInfo>>, String> {
    let pool_size = pool_size.unwrap_or(14) as i64;
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

    // 先清空该赛区当前赛季的旧选秀池数据（只清除未被选中的）
    sqlx::query(
        "DELETE FROM draft_players WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 从 draft_pool 中随机抽取 pool_size 名 available 的选手
    let available_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM draft_pool WHERE save_id = ? AND region_id = ? AND status = 'available'"
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if available_count == 0 {
        return Ok(CommandResult::err("选秀池为空，没有可用的选秀选手"));
    }

    if available_count < pool_size {
        return Ok(CommandResult::err(format!(
            "选秀池人数不足，当前仅有 {} 人，需要至少 {} 人",
            available_count, pool_size
        )));
    }

    let pool_rows = sqlx::query(
        r#"
        SELECT id, game_id, real_name, nationality, age, ability, potential, position, tag
        FROM draft_pool
        WHERE save_id = ? AND region_id = ? AND status = 'available'
        ORDER BY RANDOM()
        LIMIT ?
        "#
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .bind(pool_size)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 按综合评分排名（决定顺位价值）
    let mut draft_players: Vec<DraftPlayerInfo> = pool_rows
        .iter()
        .map(|row| {
            DraftPlayerInfo {
                id: 0,
                game_id: row.get("game_id"),
                real_name: row.get("real_name"),
                nationality: row.get("nationality"),
                age: row.get::<i64, _>("age") as u8,
                ability: row.get::<i64, _>("ability") as u8,
                potential: row.get::<i64, _>("potential") as u8,
                position: row.get("position"),
                tag: row.get("tag"),
                draft_rank: 0,
                is_picked: false,
            }
        })
        .collect();

    draft_players.sort_by(|a, b| {
        let score_a = a.ability as f64 * 0.4 + a.potential as f64 * 0.6;
        let score_b = b.ability as f64 * 0.4 + b.potential as f64 * 0.6;
        score_b.partial_cmp(&score_a).unwrap()
    });

    // 更新排名
    for (i, player) in draft_players.iter_mut().enumerate() {
        player.draft_rank = (i + 1) as u32;
    }

    // 保存到 draft_players 表
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
                original_team_id: None,
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
        SELECT do.team_id, do.original_team_id, do.summer_rank, do.draft_position, do.lottery_result, t.name as team_name
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
        .map(|row| {
            let original_team_id: Option<i64> = row.get("original_team_id");
            DraftOrderInfo {
                team_id: row.get::<i64, _>("team_id") as u64,
                team_name: row.get("team_name"),
                original_team_id: original_team_id.map(|id| id as u64),
                summer_rank: row.get::<i64, _>("summer_rank") as u32,
                draft_position: row.get::<i64, _>("draft_position") as u32,
                lottery_result: row.get("lottery_result"),
            }
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

    // 同步更新 draft_pool 状态：标记为 drafted
    let draft_game_id: String = player_row.get("game_id");
    sqlx::query(
        r#"
        UPDATE draft_pool
        SET status = 'drafted', drafted_season = ?, drafted_by_team_id = ?
        WHERE save_id = ? AND region_id = ? AND game_id = ? AND status = 'available'
        "#
    )
    .bind(current_season)
    .bind(team_id as i64)
    .bind(&save_id)
    .bind(region_id)
    .bind(&draft_game_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 根据赛区计算 region_loyalty 默认值
    // LPL(1)=75-90, LCK(2)=55-75, LEC(3)=45-65, LCS(4)=40-60
    let region_loyalty: i64 = match region_id {
        1 => 75 + (rand::random::<u8>() % 16) as i64,  // LPL: 75-90
        2 => 55 + (rand::random::<u8>() % 21) as i64,  // LCK: 55-75
        3 => 45 + (rand::random::<u8>() % 21) as i64,  // LEC: 45-65
        4 => 40 + (rand::random::<u8>() % 21) as i64,  // LCS: 40-60
        _ => 60,
    };

    // 创建正式球员
    let new_player_id: i64 = sqlx::query(
        r#"
        INSERT INTO players (
            save_id, game_id, real_name, nationality, age, ability, potential, stability,
            tag, status, position, team_id, salary, market_value, contract_end_season,
            join_season, is_starter, home_region_id, region_loyalty
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'Active', ?, ?, ?, ?, ?, ?, 0, ?, ?)
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
    .bind(current_season + 2) // 2年新秀合同
    .bind(current_season)
    .bind(region_id)  // home_region_id = 选秀赛区
    .bind(region_loyalty)
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

    // 记录选秀合同历史
    sqlx::query(
        r#"INSERT INTO player_contracts (save_id, player_id, team_id, contract_type, total_salary, annual_salary, contract_years, start_season, end_season, is_active)
           VALUES (?, ?, ?, 'DRAFT', ?, ?, 2, ?, ?, 1)"#
    )
    .bind(&save_id)
    .bind(new_player_id)
    .bind(team_id as i64)
    .bind(20i64)   // total_salary = 20
    .bind(10i64)   // annual_salary = 20 / 2 = 10
    .bind(current_season)
    .bind(current_season + 2)
    .execute(&pool)
    .await
    .map_err(|e| format!("记录选秀合同失败: {}", e))?;

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
        "SELECT team_id, draft_position FROM draft_orders WHERE save_id = ? AND season_id = ? AND region_id = ? ORDER BY draft_position"
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
        let _pick_number: i64 = order_row.get("draft_position");

        // 重新获取锁
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

        // 获取球队阵容
        let roster = crate::db::PlayerRepository::get_by_team(&pool, team_id as u64)
            .await
            .unwrap_or_default();

        // 查询所有可选球员（简化版：只获取评分所需字段）
        let player_rows = sqlx::query(
            "SELECT id, ability, potential, age, position FROM draft_players WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0"
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        // 构建简化的选秀球员列表用于评分
        let mut player_scores: Vec<(i64, f64)> = player_rows
            .iter()
            .map(|row| {
                // 构建简化的 DraftPlayer 用于评分
                let draft_player = crate::models::DraftPlayer {
                    id: row.get::<i64, _>("id") as u64,
                    save_id: save_id.clone(),
                    season_id: current_season as u64,
                    region_id,
                    draft_rank: 0,
                    game_id: String::new(),
                    real_name: None,
                    nationality: None,
                    age: row.get::<i64, _>("age") as u8,
                    ability: row.get::<i64, _>("ability") as u8,
                    potential: row.get::<i64, _>("potential") as u8,
                    tag: crate::models::PlayerTag::Normal,
                    position: row.get::<Option<String>, _>("position")
                        .and_then(|s| match s.to_uppercase().as_str() {
                            "TOP" => Some(crate::models::Position::Top),
                            "JUG" | "JUNGLE" => Some(crate::models::Position::Jug),
                            "MID" | "MIDDLE" => Some(crate::models::Position::Mid),
                            "ADC" | "BOT" => Some(crate::models::Position::Adc),
                            "SUP" | "SUPPORT" => Some(crate::models::Position::Sup),
                            _ => None,
                        }),
                    is_picked: false,
                    picked_by_team_id: None,
                };

                let score = crate::services::DraftAIService::calculate_player_score(
                    &draft_player,
                    &crate::services::DraftAIService::calculate_position_needs(&roster),
                    &roster,
                );

                (row.get::<i64, _>("id"), score)
            })
            .collect();

        // 按分数排序
        player_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_player_id = player_scores.first().map(|(id, _)| *id);

        // 释放锁
        drop(guard);
        drop(current_save);

        if let Some(player_id) = best_player_id {
            // 调用 make_draft_pick 执行选秀
            let result = make_draft_pick(state.clone(), team_id as u64, player_id as u64).await?;
            if let Some(pick_info) = result.data {
                picks.push(pick_info);
            }
        }
    }

    Ok(CommandResult::ok(picks))
}

// ========================================
// 选手池管理命令 (draft_pool 表 CRUD)
// ========================================

/// 选手池选手信息（对应 draft_pool 表）
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftPoolPlayer {
    pub id: u64,
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub position: String,
    pub tag: String,
    pub status: String,
}

/// 新增选手池选手的输入结构
#[derive(Debug, Serialize, Deserialize)]
pub struct NewDraftPoolPlayer {
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub position: String,
    pub tag: String,
}

/// 读取选秀池选手（draft_pool 表中 available 的记录）
#[tauri::command]
pub async fn get_draft_pool_players(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<DraftPoolPlayer>>, String> {
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
        SELECT id, game_id, real_name, nationality, age, ability, potential, position, tag, status
        FROM draft_pool
        WHERE save_id = ? AND region_id = ? AND status = 'available'
        ORDER BY id
        "#,
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let players: Vec<DraftPoolPlayer> = rows
        .iter()
        .map(|row| DraftPoolPlayer {
            id: row.get::<i64, _>("id") as u64,
            game_id: row.get("game_id"),
            real_name: row.get("real_name"),
            nationality: row.get("nationality"),
            age: row.get::<i64, _>("age") as u8,
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            position: row.get("position"),
            tag: row.get("tag"),
            status: row.get("status"),
        })
        .collect();

    Ok(CommandResult::ok(players))
}

/// 批量添加选手到选秀池
#[tauri::command]
pub async fn add_draft_pool_players(
    state: State<'_, AppState>,
    region_id: u64,
    players: Vec<NewDraftPoolPlayer>,
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

    // 获取当前赛季作为 created_season
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let mut count: u64 = 0;
    for p in &players {
        sqlx::query(
            r#"
            INSERT INTO draft_pool (
                save_id, region_id, game_id, real_name, nationality,
                age, ability, potential, position, tag, status, created_season
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'available', ?)
            "#,
        )
        .bind(&save_id)
        .bind(region_id as i64)
        .bind(&p.game_id)
        .bind(&p.real_name)
        .bind(&p.nationality)
        .bind(p.age as i64)
        .bind(p.ability as i64)
        .bind(p.potential as i64)
        .bind(&p.position)
        .bind(&p.tag)
        .bind(current_season)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    Ok(CommandResult::ok(count))
}

/// 编辑选秀池中的单个选手
#[tauri::command]
pub async fn update_draft_pool_player(
    state: State<'_, AppState>,
    player_id: u64,
    game_id: String,
    ability: u8,
    potential: u8,
    position: String,
    tag: String,
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

    let result = sqlx::query(
        r#"
        UPDATE draft_pool
        SET game_id = ?, ability = ?, potential = ?, position = ?, tag = ?
        WHERE id = ? AND status = 'available'
        "#,
    )
    .bind(&game_id)
    .bind(ability as i64)
    .bind(potential as i64)
    .bind(&position)
    .bind(&tag)
    .bind(player_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Ok(CommandResult::err("选手不存在或已被选中"));
    }

    Ok(CommandResult::ok(()))
}

/// 删除选秀池选手（支持单个/批量/清空）
#[tauri::command]
pub async fn delete_draft_pool_players(
    state: State<'_, AppState>,
    region_id: u64,
    player_ids: Option<Vec<u64>>,
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

    let rows_affected = match player_ids {
        Some(ids) => {
            // 删除指定 id 的选手
            let mut total: u64 = 0;
            for id in ids {
                let result = sqlx::query(
                    "DELETE FROM draft_pool WHERE id = ? AND save_id = ? AND region_id = ? AND status = 'available'"
                )
                .bind(id as i64)
                .bind(&save_id)
                .bind(region_id as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
                total += result.rows_affected();
            }
            total
        }
        None => {
            // 清空该赛区所有 available 的记录
            let result = sqlx::query(
                "DELETE FROM draft_pool WHERE save_id = ? AND region_id = ? AND status = 'available'"
            )
            .bind(&save_id)
            .bind(region_id as i64)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
            result.rows_affected()
        }
    };

    Ok(CommandResult::ok(rows_affected))
}
