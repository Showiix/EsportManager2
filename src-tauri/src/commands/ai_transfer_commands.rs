//! AI 转会系统 Tauri 命令接口
//!
//! 提供以下功能：
//! - GM 人格配置的 CRUD 操作
//! - AI 策略生成和查询
//! - 策略展示信息获取

use crate::commands::ApiResponse;
use crate::commands::save_commands::AppState;
use crate::db::{TeamRepository, PlayerRepository, PointsRepository};
use crate::models::{
    AIStrategyInfo, AITransferStrategy, GMPersonality, Player,
    PlayerStatus, Team, TeamGMProfile, TeamGMProfileInfo,
};
use crate::services::AITransferService;
use crate::engines::transfer::FreeAgentInfo;
use sqlx::Row;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use serde::Serialize;

/// AI 策略生成进度事件
#[derive(Clone, Serialize)]
pub struct StrategyGenerationProgress {
    pub current: usize,
    pub total: usize,
    pub team_name: String,
    pub status: String,  // "generating", "success", "failed", "completed"
}

// ==================== GM 人格配置命令 ====================

/// 获取所有球队的 GM 人格配置
#[tauri::command]
pub async fn get_all_gm_profiles(state: State<'_, AppState>) -> Result<ApiResponse<Vec<TeamGMProfileInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 查询所有球队
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    // 查询所有赛区，建立 region_id -> short_name 的映射
    let region_rows = sqlx::query("SELECT id, short_name FROM regions WHERE save_id = ?")
        .bind(&save_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let region_map: HashMap<u64, String> = region_rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let short_name: String = row.get("short_name");
            (id as u64, short_name)
        })
        .collect();

    // 查询已有的 GM 配置
    let existing_profiles: Vec<_> = sqlx::query(
        "SELECT * FROM team_gm_profiles WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let profile_map: HashMap<u64, _> = existing_profiles
        .into_iter()
        .map(|row| {
            let team_id: i64 = row.get("team_id");
            (team_id as u64, row)
        })
        .collect();

    // 构建响应
    let mut result = Vec::new();
    for team in teams {
        let (personality, custom_prompt, risk_tolerance, budget_ratio, sell_aggressiveness,
             preferred_age_min, preferred_age_max, min_ability_threshold, price_premium_max, position_priorities,
             draft_pick_sell_threshold, draft_pick_bid_aggressiveness, draft_preference_ability_weight, draft_young_bias)
        = if let Some(row) = profile_map.get(&team.id) {
            let personality_str: String = row.get("personality");
            let personality = match personality_str.to_uppercase().as_str() {
                "CHAMPIONSHIP" => GMPersonality::Championship,
                "YOUTH_DEVELOPMENT" | "YOUTHDEVELOPMENT" => GMPersonality::YouthDevelopment,
                "BALANCED" => GMPersonality::Balanced,
                "SPECULATOR" => GMPersonality::Speculator,
                "REBUILDING" => GMPersonality::Rebuilding,
                "CUSTOM" => GMPersonality::Custom,
                _ => GMPersonality::Balanced,
            };

            let sell_agg_str: String = row.get("sell_aggressiveness");
            let sell_aggressiveness = match sell_agg_str.to_uppercase().as_str() {
                "CONSERVATIVE" => crate::models::SellAggressiveness::Conservative,
                "AGGRESSIVE" => crate::models::SellAggressiveness::Aggressive,
                _ => crate::models::SellAggressiveness::Normal,
            };

            let position_priorities_json: String = row.get("position_priorities");
            let position_priorities: HashMap<String, u8> = serde_json::from_str(&position_priorities_json)
                .unwrap_or_else(|_| {
                    let mut map = HashMap::new();
                    map.insert("TOP".to_string(), 50);
                    map.insert("JUG".to_string(), 50);
                    map.insert("MID".to_string(), 50);
                    map.insert("ADC".to_string(), 50);
                    map.insert("SUP".to_string(), 50);
                    map
                });

            (
                personality,
                row.get::<Option<String>, _>("custom_prompt"),
                row.get::<i64, _>("risk_tolerance") as u8,
                row.get::<f64, _>("budget_ratio"),
                sell_aggressiveness,
                row.get::<i64, _>("preferred_age_min") as u8,
                row.get::<i64, _>("preferred_age_max") as u8,
                row.get::<i64, _>("min_ability_threshold") as u8,
                row.get::<f64, _>("price_premium_max"),
                position_priorities,
                row.get::<f64, _>("draft_pick_sell_threshold"),
                row.get::<f64, _>("draft_pick_bid_aggressiveness"),
                row.get::<f64, _>("draft_preference_ability_weight"),
                row.get::<f64, _>("draft_young_bias"),
            )
        } else {
            // 默认值
            let personality = GMPersonality::Balanced;
            let mut pos_priorities = HashMap::new();
            pos_priorities.insert("TOP".to_string(), 50);
            pos_priorities.insert("JUG".to_string(), 50);
            pos_priorities.insert("MID".to_string(), 50);
            pos_priorities.insert("ADC".to_string(), 50);
            pos_priorities.insert("SUP".to_string(), 50);

            (
                personality,
                None,
                50u8,
                personality.default_budget_ratio(),
                crate::models::SellAggressiveness::Normal,
                18u8,
                30u8,
                personality.target_ability_threshold(),
                personality.price_premium_tolerance(),
                pos_priorities,
                0.5f64,
                1.0f64,
                personality.default_ability_weight(),
                0.0f64,
            )
        };

        result.push(TeamGMProfileInfo {
            team_id: team.id,
            team_name: team.name.clone(),
            team_short_name: team.short_name.clone(),
            region_id: team.region_id,
            region_name: region_map.get(&team.region_id).cloned().unwrap_or_else(|| format!("R{}", team.region_id)),
            personality,
            personality_name: personality.name().to_string(),
            personality_description: personality.description().to_string(),
            custom_prompt,
            risk_tolerance,
            budget_ratio,
            sell_aggressiveness,
            preferred_age_min,
            preferred_age_max,
            min_ability_threshold,
            price_premium_max,
            position_priorities,
            draft_pick_sell_threshold,
            draft_pick_bid_aggressiveness,
            draft_preference_ability_weight,
            draft_young_bias,
        });
    }

    Ok(ApiResponse::success(result))
}

/// 获取单个球队的 GM 人格配置
#[tauri::command]
pub async fn get_team_gm_profile(
    team_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<TeamGMProfileInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 查询球队信息
    let team = TeamRepository::get_by_id(&pool, team_id).await
        .map_err(|e| format!("球队不存在: {}", e))?;

    // 查询赛区名称
    let region_name: String = sqlx::query_scalar("SELECT short_name FROM regions WHERE id = ?")
        .bind(team.region_id as i64)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| format!("R{}", team.region_id));

    // 查询 GM 配置
    let profile_row = sqlx::query(
        "SELECT * FROM team_gm_profiles WHERE save_id = ? AND team_id = ?"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (personality, custom_prompt, risk_tolerance, budget_ratio, sell_aggressiveness,
         preferred_age_min, preferred_age_max, min_ability_threshold, price_premium_max, position_priorities,
         draft_pick_sell_threshold, draft_pick_bid_aggressiveness, draft_preference_ability_weight, draft_young_bias)
    = if let Some(row) = profile_row {
        let personality_str: String = row.get("personality");
        let personality = match personality_str.to_uppercase().as_str() {
            "CHAMPIONSHIP" => GMPersonality::Championship,
            "YOUTH_DEVELOPMENT" | "YOUTHDEVELOPMENT" => GMPersonality::YouthDevelopment,
            "BALANCED" => GMPersonality::Balanced,
            "SPECULATOR" => GMPersonality::Speculator,
            "REBUILDING" => GMPersonality::Rebuilding,
            "CUSTOM" => GMPersonality::Custom,
            _ => GMPersonality::Balanced,
        };

        let sell_agg_str: String = row.get("sell_aggressiveness");
        let sell_aggressiveness = match sell_agg_str.to_uppercase().as_str() {
            "CONSERVATIVE" => crate::models::SellAggressiveness::Conservative,
            "AGGRESSIVE" => crate::models::SellAggressiveness::Aggressive,
            _ => crate::models::SellAggressiveness::Normal,
        };

        let position_priorities_json: String = row.get("position_priorities");
        let position_priorities: HashMap<String, u8> = serde_json::from_str(&position_priorities_json)
            .unwrap_or_else(|_| {
                let mut map = HashMap::new();
                map.insert("TOP".to_string(), 50);
                map.insert("JUG".to_string(), 50);
                map.insert("MID".to_string(), 50);
                map.insert("ADC".to_string(), 50);
                map.insert("SUP".to_string(), 50);
                map
            });

        (
            personality,
            row.get::<Option<String>, _>("custom_prompt"),
            row.get::<i64, _>("risk_tolerance") as u8,
            row.get::<f64, _>("budget_ratio"),
            sell_aggressiveness,
            row.get::<i64, _>("preferred_age_min") as u8,
            row.get::<i64, _>("preferred_age_max") as u8,
            row.get::<i64, _>("min_ability_threshold") as u8,
            row.get::<f64, _>("price_premium_max"),
            position_priorities,
            row.get::<f64, _>("draft_pick_sell_threshold"),
            row.get::<f64, _>("draft_pick_bid_aggressiveness"),
            row.get::<f64, _>("draft_preference_ability_weight"),
            row.get::<f64, _>("draft_young_bias"),
        )
    } else {
        let personality = GMPersonality::Balanced;
        let mut pos_priorities = HashMap::new();
        pos_priorities.insert("TOP".to_string(), 50);
        pos_priorities.insert("JUG".to_string(), 50);
        pos_priorities.insert("MID".to_string(), 50);
        pos_priorities.insert("ADC".to_string(), 50);
        pos_priorities.insert("SUP".to_string(), 50);

        (
            personality,
            None,
            50u8,
            personality.default_budget_ratio(),
            crate::models::SellAggressiveness::Normal,
            18u8,
            30u8,
            personality.target_ability_threshold(),
            personality.price_premium_tolerance(),
            pos_priorities,
            0.5f64,
            1.0f64,
            personality.default_ability_weight(),
            0.0f64,
        )
    };

    Ok(ApiResponse::success(TeamGMProfileInfo {
        team_id: team.id,
        team_name: team.name.clone(),
        team_short_name: team.short_name.clone(),
        region_id: team.region_id,
        region_name,
        personality,
        personality_name: personality.name().to_string(),
        personality_description: personality.description().to_string(),
        custom_prompt,
        risk_tolerance,
        budget_ratio,
        sell_aggressiveness,
        preferred_age_min,
        preferred_age_max,
        min_ability_threshold,
        price_premium_max,
        position_priorities,
        draft_pick_sell_threshold,
        draft_pick_bid_aggressiveness,
        draft_preference_ability_weight,
        draft_young_bias,
    }))
}

/// 更新球队 GM 人格配置
#[tauri::command]
pub async fn update_team_gm_profile(
    team_id: u64,
    personality: String,
    custom_prompt: Option<String>,
    risk_tolerance: u8,
    budget_ratio: f64,
    sell_aggressiveness: String,
    preferred_age_min: u8,
    preferred_age_max: u8,
    min_ability_threshold: u8,
    price_premium_max: f64,
    position_priorities: HashMap<String, u8>,
    draft_pick_sell_threshold: f64,
    draft_pick_bid_aggressiveness: f64,
    draft_preference_ability_weight: f64,
    draft_young_bias: f64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let position_priorities_json = serde_json::to_string(&position_priorities).unwrap_or_default();

    // 使用 UPSERT 语法
    sqlx::query(
        r#"
        INSERT INTO team_gm_profiles (
            team_id, save_id, personality, custom_prompt, risk_tolerance,
            budget_ratio, sell_aggressiveness, preferred_age_min, preferred_age_max,
            min_ability_threshold, price_premium_max, position_priorities,
            draft_pick_sell_threshold, draft_pick_bid_aggressiveness,
            draft_preference_ability_weight, draft_young_bias,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(team_id, save_id) DO UPDATE SET
            personality = excluded.personality,
            custom_prompt = excluded.custom_prompt,
            risk_tolerance = excluded.risk_tolerance,
            budget_ratio = excluded.budget_ratio,
            sell_aggressiveness = excluded.sell_aggressiveness,
            preferred_age_min = excluded.preferred_age_min,
            preferred_age_max = excluded.preferred_age_max,
            min_ability_threshold = excluded.min_ability_threshold,
            price_premium_max = excluded.price_premium_max,
            position_priorities = excluded.position_priorities,
            draft_pick_sell_threshold = excluded.draft_pick_sell_threshold,
            draft_pick_bid_aggressiveness = excluded.draft_pick_bid_aggressiveness,
            draft_preference_ability_weight = excluded.draft_preference_ability_weight,
            draft_young_bias = excluded.draft_young_bias,
            updated_at = excluded.updated_at
        "#
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .bind(&personality.to_uppercase())
    .bind(&custom_prompt)
    .bind(risk_tolerance as i64)
    .bind(budget_ratio)
    .bind(&sell_aggressiveness.to_uppercase())
    .bind(preferred_age_min as i64)
    .bind(preferred_age_max as i64)
    .bind(min_ability_threshold as i64)
    .bind(price_premium_max)
    .bind(&position_priorities_json)
    .bind(draft_pick_sell_threshold)
    .bind(draft_pick_bid_aggressiveness)
    .bind(draft_preference_ability_weight)
    .bind(draft_young_bias)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(()))
}

/// 批量更新多个球队的 GM 人格配置
#[tauri::command]
pub async fn batch_update_gm_profiles(
    profiles: Vec<(u64, String, Option<String>, u8, bool)>,
    state: State<'_, AppState>,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut updated = 0u32;

    for (team_id, personality, custom_prompt, risk_tolerance, ai_enabled) in profiles {
        let result = sqlx::query(
            r#"
            INSERT INTO team_gm_profiles (team_id, save_id, personality, custom_prompt, risk_tolerance, ai_enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(team_id, save_id) DO UPDATE SET
                personality = excluded.personality,
                custom_prompt = excluded.custom_prompt,
                risk_tolerance = excluded.risk_tolerance,
                ai_enabled = excluded.ai_enabled,
                updated_at = excluded.updated_at
            "#
        )
        .bind(team_id as i64)
        .bind(&save_id)
        .bind(&personality.to_uppercase())
        .bind(&custom_prompt)
        .bind(risk_tolerance as i64)
        .bind(if ai_enabled { 1i64 } else { 0i64 })
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await;

        if result.is_ok() {
            updated += 1;
        }
    }

    Ok(ApiResponse::success(updated))
}

// ==================== AI 策略命令 ====================

/// 为所有球队生成 AI 策略
#[tauri::command]
pub async fn generate_ai_strategies(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<AIStrategyInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 获取所有球队
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    // 获取所有选手（按球队分组）
    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player);
            }
        }
    }

    // 获取自由球员
    let all_players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let fa_players: Vec<Player> = all_players.into_iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .collect();

    let free_agents: Vec<FreeAgentInfo> = fa_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: crate::models::FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: crate::models::FreeAgentReason::ContractExpire,
                    status: crate::models::FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 获取 GM 配置
    let gm_rows = sqlx::query(
        "SELECT * FROM team_gm_profiles WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut gm_profiles: HashMap<u64, TeamGMProfile> = HashMap::new();
    for row in gm_rows {
        let team_id: i64 = row.get("team_id");
        let personality_str: String = row.get("personality");
        let personality = match personality_str.to_uppercase().as_str() {
            "CHAMPIONSHIP" => GMPersonality::Championship,
            "YOUTH_DEVELOPMENT" | "YOUTHDEVELOPMENT" => GMPersonality::YouthDevelopment,
            "BALANCED" => GMPersonality::Balanced,
            "SPECULATOR" => GMPersonality::Speculator,
            "REBUILDING" => GMPersonality::Rebuilding,
            "CUSTOM" => GMPersonality::Custom,
            _ => GMPersonality::Balanced,
        };

        let sell_agg_str: String = row.get("sell_aggressiveness");
        let sell_aggressiveness = match sell_agg_str.to_uppercase().as_str() {
            "CONSERVATIVE" => crate::models::SellAggressiveness::Conservative,
            "AGGRESSIVE" => crate::models::SellAggressiveness::Aggressive,
            _ => crate::models::SellAggressiveness::Normal,
        };

        let position_priorities_json: String = row.get("position_priorities");
        let position_priorities: HashMap<String, u8> = serde_json::from_str(&position_priorities_json)
            .unwrap_or_else(|_| {
                let mut map = HashMap::new();
                map.insert("TOP".to_string(), 50);
                map.insert("JUG".to_string(), 50);
                map.insert("MID".to_string(), 50);
                map.insert("ADC".to_string(), 50);
                map.insert("SUP".to_string(), 50);
                map
            });

        gm_profiles.insert(team_id as u64, TeamGMProfile {
            id: row.get::<i64, _>("id") as u64,
            team_id: team_id as u64,
            save_id: save_id.clone(),
            personality,
            custom_prompt: row.get("custom_prompt"),
            risk_tolerance: row.get::<i64, _>("risk_tolerance") as u8,
            budget_ratio: row.get::<f64, _>("budget_ratio"),
            sell_aggressiveness,
            preferred_age_min: row.get::<i64, _>("preferred_age_min") as u8,
            preferred_age_max: row.get::<i64, _>("preferred_age_max") as u8,
            min_ability_threshold: row.get::<i64, _>("min_ability_threshold") as u8,
            price_premium_max: row.get::<f64, _>("price_premium_max"),
            position_priorities,
            draft_pick_sell_threshold: row.get::<f64, _>("draft_pick_sell_threshold"),
            draft_pick_bid_aggressiveness: row.get::<f64, _>("draft_pick_bid_aggressiveness"),
            draft_preference_ability_weight: row.get::<f64, _>("draft_preference_ability_weight"),
            draft_young_bias: row.get::<f64, _>("draft_young_bias"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }

    // 生成策略 - 检查是否使用 LLM
    let use_llm = is_llm_configured();
    let mut strategies: Vec<AITransferStrategy> = Vec::new();
    let total_teams = teams.len();

    log::info!("开始生成 AI 策略, 使用 LLM: {}, 球队数量: {}", use_llm, total_teams);

    if use_llm {
        // 使用 LLM 生成策略
        let llm_config = get_llm_config().ok_or("LLM 配置获取失败")?;
        log::info!("LLM 配置: provider={:?}, model={}", llm_config.provider, llm_config.model);
        let llm_service = LLMTransferService::new(llm_config);

        for (idx, team) in teams.iter().enumerate() {
            log::info!("生成策略进度: {}/{} - {}", idx + 1, total_teams, team.name);

            // 发送进度事件 - 开始生成
            let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgress {
                current: idx + 1,
                total: total_teams,
                team_name: team.name.clone(),
                status: "generating".to_string(),
            });

            let roster: Vec<Player> = players_by_team
                .get(&team.id)
                .cloned()
                .unwrap_or_default();

            let gm_profile = gm_profiles
                .get(&team.id)
                .cloned()
                .unwrap_or_else(|| TeamGMProfile::new(team.id, save_id.clone()));

            // 查询球队荣誉信息
            let team_honors = query_team_honors(&pool, &save_id, team.id, season_id as u64, &roster).await;
            let roster_honors = query_roster_honors(&pool, &save_id, &roster).await;
            let roster_performance = query_roster_performance(&pool, &save_id, &roster, season_id as u64).await;

            // 调用 LLM 生成策略
            match llm_service.generate_strategy(
                team,
                &roster,
                &gm_profile,
                &free_agents,
                &players_by_team,
                &save_id,
                season_id as u64,
                team_honors.as_ref(),
                Some(&roster_honors),
                Some(&roster_performance),
            ).await {
                Ok(strategy) => {
                    log::info!("LLM 策略生成成功: {}", team.name);
                    // 发送成功事件
                    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgress {
                        current: idx + 1,
                        total: total_teams,
                        team_name: team.name.clone(),
                        status: "success".to_string(),
                    });
                    strategies.push(strategy);
                },
                Err(e) => {
                    // LLM 失败时回退到规则 AI
                    log::warn!("LLM 生成策略失败 ({}): {}, 使用规则 AI", team.name, e);
                    // 发送失败事件（但仍会使用 fallback）
                    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgress {
                        current: idx + 1,
                        total: total_teams,
                        team_name: team.name.clone(),
                        status: "failed".to_string(),
                    });
                    let fallback = AITransferService::generate_mock_strategy(
                        team,
                        &roster,
                        &gm_profile,
                        &free_agents,
                        &players_by_team,
                        &save_id,
                        season_id as u64,
                    );
                    strategies.push(fallback);
                }
            }
        }
        // 发送完成事件
        let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgress {
            current: total_teams,
            total: total_teams,
            team_name: "".to_string(),
            status: "completed".to_string(),
        });
        log::info!("所有 LLM 策略生成完成");
    } else {
        log::info!("使用规则 AI 生成策略...");
        // 使用规则 AI 生成策略
        strategies = AITransferService::generate_all_strategies(
            &teams,
            &players_by_team,
            &free_agents,
            &gm_profiles,
            &save_id,
            season_id as u64,
        );
        log::info!("规则 AI 策略生成完成");
    }

    log::info!("保存策略到数据库...");

    // 保存策略到数据库
    let now = chrono::Utc::now().to_rfc3339();
    for strategy in &strategies {
        let strategy_json = serde_json::to_string(strategy).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO ai_transfer_strategies (team_id, save_id, season_id, strategy_json, generated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(team_id, save_id, season_id) DO UPDATE SET
                strategy_json = excluded.strategy_json,
                generated_at = excluded.generated_at
            "#
        )
        .bind(strategy.team_id as i64)
        .bind(&save_id)
        .bind(season_id)
        .bind(&strategy_json)
        .bind(&now)
        .execute(&pool)
        .await
        .ok();
    }

    // 构建响应
    let team_map: HashMap<u64, &Team> = teams.iter().map(|t| (t.id, t)).collect();
    let result: Vec<AIStrategyInfo> = strategies
        .iter()
        .map(|s| {
            let mut info = AIStrategyInfo::from(s);
            if let Some(team) = team_map.get(&s.team_id) {
                info.team_name = team.name.clone();
            }
            info
        })
        .collect();

    Ok(ApiResponse::success(result))
}

/// 获取单个球队的 AI 策略
#[tauri::command]
pub async fn get_team_ai_strategy(
    team_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<AITransferStrategy>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 查询策略
    let row = sqlx::query(
        "SELECT * FROM ai_transfer_strategies WHERE save_id = ? AND team_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            let strategy_json: String = r.get("strategy_json");
            let strategy: AITransferStrategy = serde_json::from_str(&strategy_json)
                .map_err(|e| format!("解析策略失败: {}", e))?;
            Ok(ApiResponse::success(strategy))
        }
        None => {
            // 如果没有缓存的策略，返回空策略
            Ok(ApiResponse::success(AITransferStrategy::empty(
                team_id,
                String::new(), // 空策略时暂无球队名
                save_id,
                season_id as u64,
            )))
        }
    }
}

/// 获取所有可用的 GM 人格类型
#[tauri::command]
pub fn get_gm_personality_types() -> ApiResponse<Vec<PersonalityTypeInfo>> {
    let types = vec![
        PersonalityTypeInfo {
            value: "CHAMPIONSHIP".to_string(),
            name: "争冠型".to_string(),
            description: "追求顶级选手，愿意高价签人，以争夺世界冠军为目标".to_string(),
            icon: "trophy".to_string(),
        },
        PersonalityTypeInfo {
            value: "YOUTH_DEVELOPMENT".to_string(),
            name: "青训型".to_string(),
            description: "专注培养年轻选手，低预算运营，挖掘未来之星".to_string(),
            icon: "star".to_string(),
        },
        PersonalityTypeInfo {
            value: "BALANCED".to_string(),
            name: "稳健型".to_string(),
            description: "平衡发展，控制成本，稳定阵容，追求性价比".to_string(),
            icon: "scale".to_string(),
        },
        PersonalityTypeInfo {
            value: "SPECULATOR".to_string(),
            name: "投机型".to_string(),
            description: "擅长买低卖高，通过球员交易赚取差价".to_string(),
            icon: "trending-up".to_string(),
        },
        PersonalityTypeInfo {
            value: "REBUILDING".to_string(),
            name: "重建型".to_string(),
            description: "清洗高薪老将，为年轻选手让路，重新开始".to_string(),
            icon: "refresh".to_string(),
        },
        PersonalityTypeInfo {
            value: "CUSTOM".to_string(),
            name: "自定义".to_string(),
            description: "完全自定义的转会风格，由提示词控制".to_string(),
            icon: "edit".to_string(),
        },
    ];

    ApiResponse::success(types)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalityTypeInfo {
    pub value: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

/// 初始化数据库表（如果不存在）
#[tauri::command]
pub async fn init_ai_transfer_tables(state: State<'_, AppState>) -> Result<ApiResponse<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 检查表结构是否完整，如果缺列则删除重建
    let check_result = sqlx::query("SELECT budget_ratio, sell_aggressiveness, draft_young_bias FROM team_gm_profiles LIMIT 1")
        .fetch_optional(&pool)
        .await;

    if check_result.is_err() {
        // 表结构不完整，删除重建
        let _ = sqlx::query("DROP TABLE IF EXISTS team_gm_profiles").execute(&pool).await;
    }

    // 创建 GM 人格配置表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_gm_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            team_id INTEGER NOT NULL,
            save_id TEXT NOT NULL,
            personality TEXT NOT NULL DEFAULT 'BALANCED',
            custom_prompt TEXT,
            risk_tolerance INTEGER NOT NULL DEFAULT 50,
            budget_ratio REAL NOT NULL DEFAULT 0.6,
            sell_aggressiveness TEXT NOT NULL DEFAULT 'NORMAL',
            preferred_age_min INTEGER NOT NULL DEFAULT 18,
            preferred_age_max INTEGER NOT NULL DEFAULT 30,
            min_ability_threshold INTEGER NOT NULL DEFAULT 70,
            price_premium_max REAL NOT NULL DEFAULT 1.0,
            position_priorities TEXT NOT NULL DEFAULT '{"TOP":50,"JUG":50,"MID":50,"ADC":50,"SUP":50}',
            draft_pick_sell_threshold REAL NOT NULL DEFAULT 0.5,
            draft_pick_bid_aggressiveness REAL NOT NULL DEFAULT 1.0,
            draft_preference_ability_weight REAL NOT NULL DEFAULT 0.4,
            draft_young_bias REAL NOT NULL DEFAULT 0.0,
            created_at TEXT,
            updated_at TEXT,
            UNIQUE(team_id, save_id)
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 创建 AI 策略缓存表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ai_transfer_strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            team_id INTEGER NOT NULL,
            save_id TEXT NOT NULL,
            season_id INTEGER NOT NULL,
            strategy_json TEXT NOT NULL,
            generated_at TEXT NOT NULL,
            UNIQUE(team_id, save_id, season_id)
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(()))
}

// ==================== LLM AI 命令 ====================

use crate::services::llm_service::{LLMConfig, LLMProvider, LLMTransferService, set_llm_config, get_llm_config, is_llm_configured, PlayerHonorInfo, TeamHonorInfo, RosterPlayerHonorSummary, PlayerPerformanceSummary};

/// LLM 配置信息（用于前端展示，不包含 API Key）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LLMConfigInfo {
    pub is_configured: bool,
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
}

/// 检查 LLM 是否已配置
#[tauri::command]
pub fn check_llm_config() -> ApiResponse<LLMConfigInfo> {
    let config = get_llm_config();
    match config {
        Some(c) => ApiResponse::success(LLMConfigInfo {
            is_configured: !c.api_key.is_empty(),
            provider: match c.provider {
                LLMProvider::OpenAI => "openai".to_string(),
                LLMProvider::Claude => "claude".to_string(),
                LLMProvider::DeepSeek => "deepseek".to_string(),
                LLMProvider::Qwen => "qwen".to_string(),
                LLMProvider::Moonshot => "moonshot".to_string(),
                LLMProvider::Zhipu => "zhipu".to_string(),
            },
            model: c.model,
            base_url: c.base_url,
        }),
        None => ApiResponse::success(LLMConfigInfo {
            is_configured: false,
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            base_url: None,
        }),
    }
}

/// 配置 LLM API
#[tauri::command]
pub fn configure_llm(
    provider: String,
    api_key: String,
    model: Option<String>,
    base_url: Option<String>,
) -> ApiResponse<()> {
    let llm_provider = match provider.to_lowercase().as_str() {
        "claude" | "anthropic" => LLMProvider::Claude,
        "deepseek" => LLMProvider::DeepSeek,
        "qwen" | "tongyi" | "dashscope" => LLMProvider::Qwen,
        "moonshot" | "kimi" => LLMProvider::Moonshot,
        "zhipu" | "glm" | "chatglm" => LLMProvider::Zhipu,
        _ => LLMProvider::OpenAI,
    };

    let default_model = match llm_provider {
        LLMProvider::OpenAI => "gpt-4o-mini".to_string(),
        LLMProvider::Claude => "claude-3-5-sonnet-20241022".to_string(),
        LLMProvider::DeepSeek => "deepseek-chat".to_string(),
        LLMProvider::Qwen => "qwen-turbo".to_string(),
        LLMProvider::Moonshot => "moonshot-v1-8k".to_string(),
        LLMProvider::Zhipu => "glm-4-flash".to_string(),
    };

    let config = LLMConfig {
        provider: llm_provider,
        api_key,
        model: model.unwrap_or(default_model),
        base_url,
        max_tokens: 2000,
        temperature: 0.7,
    };

    set_llm_config(config);
    ApiResponse::success(())
}

/// 使用 LLM 为单个球队生成转会策略
#[tauri::command]
pub async fn generate_llm_strategy(
    team_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<AITransferStrategy>, String> {
    // 检查 LLM 是否已配置
    if !is_llm_configured() {
        return Ok(ApiResponse::error("LLM API 未配置，请先配置 API Key"));
    }

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 获取球队信息
    let team = TeamRepository::get_by_id(&pool, team_id).await
        .map_err(|e| format!("球队不存在: {}", e))?;

    // 获取球队阵容
    let all_players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let roster: Vec<Player> = all_players.iter()
        .filter(|p| p.team_id == Some(team_id) && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 获取自由球员
    let fa_players: Vec<Player> = all_players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    let free_agents: Vec<FreeAgentInfo> = fa_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: crate::models::FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: crate::models::FreeAgentReason::ContractExpire,
                    status: crate::models::FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 按球队分组所有球员
    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &all_players {
        if player.status == PlayerStatus::Active {
            if let Some(tid) = player.team_id {
                players_by_team.entry(tid).or_default().push(player.clone());
            }
        }
    }

    // 获取 GM 配置
    let profile_row = sqlx::query(
        "SELECT * FROM team_gm_profiles WHERE save_id = ? AND team_id = ?"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let gm_profile = if let Some(row) = profile_row {
        let personality_str: String = row.get("personality");
        let personality = match personality_str.to_uppercase().as_str() {
            "CHAMPIONSHIP" => GMPersonality::Championship,
            "YOUTH_DEVELOPMENT" | "YOUTHDEVELOPMENT" => GMPersonality::YouthDevelopment,
            "BALANCED" => GMPersonality::Balanced,
            "SPECULATOR" => GMPersonality::Speculator,
            "REBUILDING" => GMPersonality::Rebuilding,
            "CUSTOM" => GMPersonality::Custom,
            _ => GMPersonality::Balanced,
        };

        let sell_agg_str: String = row.get("sell_aggressiveness");
        let sell_aggressiveness = match sell_agg_str.to_uppercase().as_str() {
            "CONSERVATIVE" => crate::models::SellAggressiveness::Conservative,
            "AGGRESSIVE" => crate::models::SellAggressiveness::Aggressive,
            _ => crate::models::SellAggressiveness::Normal,
        };

        let position_priorities_json: String = row.get("position_priorities");
        let position_priorities: HashMap<String, u8> = serde_json::from_str(&position_priorities_json)
            .unwrap_or_else(|_| {
                let mut map = HashMap::new();
                map.insert("TOP".to_string(), 50);
                map.insert("JUG".to_string(), 50);
                map.insert("MID".to_string(), 50);
                map.insert("ADC".to_string(), 50);
                map.insert("SUP".to_string(), 50);
                map
            });

        TeamGMProfile {
            id: row.get::<i64, _>("id") as u64,
            team_id,
            save_id: save_id.clone(),
            personality,
            custom_prompt: row.get("custom_prompt"),
            risk_tolerance: row.get::<i64, _>("risk_tolerance") as u8,
            budget_ratio: row.get::<f64, _>("budget_ratio"),
            sell_aggressiveness,
            preferred_age_min: row.get::<i64, _>("preferred_age_min") as u8,
            preferred_age_max: row.get::<i64, _>("preferred_age_max") as u8,
            min_ability_threshold: row.get::<i64, _>("min_ability_threshold") as u8,
            price_premium_max: row.get::<f64, _>("price_premium_max"),
            position_priorities,
            draft_pick_sell_threshold: row.get::<f64, _>("draft_pick_sell_threshold"),
            draft_pick_bid_aggressiveness: row.get::<f64, _>("draft_pick_bid_aggressiveness"),
            draft_preference_ability_weight: row.get::<f64, _>("draft_preference_ability_weight"),
            draft_young_bias: row.get::<f64, _>("draft_young_bias"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    } else {
        TeamGMProfile::new(team_id, save_id.clone())
    };

    // 查询球队荣誉信息
    let team_honors = query_team_honors(&pool, &save_id, team_id, season_id as u64, &roster).await;
    let roster_honors = query_roster_honors(&pool, &save_id, &roster).await;
    let roster_performance = query_roster_performance(&pool, &save_id, &roster, season_id as u64).await;

    // 创建 LLM 服务并生成策略
    let llm_config = get_llm_config().ok_or("LLM 配置获取失败")?;
    let llm_service = LLMTransferService::new(llm_config);

    let strategy = llm_service.generate_strategy(
        &team,
        &roster,
        &gm_profile,
        &free_agents,
        &players_by_team,
        &save_id,
        season_id as u64,
        team_honors.as_ref(),
        Some(&roster_honors),
        Some(&roster_performance),
    )
    .await
    .map_err(|e| format!("LLM 策略生成失败: {}", e))?;

    // 保存策略到数据库
    let now = chrono::Utc::now().to_rfc3339();
    let strategy_json = serde_json::to_string(&strategy).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO ai_transfer_strategies (team_id, save_id, season_id, strategy_json, generated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(team_id, save_id, season_id) DO UPDATE SET
            strategy_json = excluded.strategy_json,
            generated_at = excluded.generated_at
        "#
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .bind(season_id)
    .bind(&strategy_json)
    .bind(&now)
    .execute(&pool)
    .await
    .ok();

    Ok(ApiResponse::success(strategy))
}

/// 清除 LLM 配置
#[tauri::command]
pub fn clear_llm_config() -> ApiResponse<()> {
    set_llm_config(LLMConfig::default());
    ApiResponse::success(())
}

// ==================== 选手转会策略命令 ====================

use crate::models::{
    PlayerTransferStrategy, PlayerTransferStrategyInfo, PreferredTeamInfo,
};
use crate::services::llm_service::TeamInfo;

/// 为单个选手生成转会策略
#[tauri::command]
pub async fn generate_player_transfer_strategy(
    player_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<PlayerTransferStrategy>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 获取选手信息
    let player = PlayerRepository::get_by_id(&pool, player_id).await
        .map_err(|e| format!("选手不存在: {}", e))?;

    let team_id = match player.team_id {
        Some(id) => id,
        None => return Ok(ApiResponse::error("选手没有所属球队")),
    };

    // 获取选手所在球队
    let team = TeamRepository::get_by_id(&pool, team_id).await
        .map_err(|e| format!("球队不存在: {}", e))?;

    // 获取球队阵容
    let all_players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let roster: Vec<Player> = all_players.iter()
        .filter(|p| p.team_id == Some(team_id) && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 获取所有球队作为可选目标
    let all_teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    // 查询赛区名称映射
    let region_rows = sqlx::query("SELECT id, short_name FROM regions WHERE save_id = ?")
        .bind(&save_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let region_map: HashMap<u64, String> = region_rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let short_name: String = row.get("short_name");
            (id as u64, short_name)
        })
        .collect();

    // 查询年度积分排名
    let rankings = PointsRepository::get_season_rankings(&pool, &save_id, season_id as u64)
        .await
        .unwrap_or_default();

    // 构建排名映射: team_id -> (global_rank, annual_points)
    let ranking_map: HashMap<u64, (u32, u32)> = rankings
        .iter()
        .map(|r| (r.team_id, (r.rank, r.total_points)))
        .collect();

    // 获取当前球队的排名
    let current_team_rank = ranking_map.get(&team_id).copied();

    // 构建可选球队列表（排除当前球队）
    let position_str = player.position
        .map(|p| format!("{:?}", p).to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let available_teams: Vec<TeamInfo> = all_teams.iter()
        .filter(|t| t.id != team_id)
        .map(|t| {
            // 计算该球队对该位置的需求度
            let team_roster: Vec<_> = all_players.iter()
                .filter(|p| p.team_id == Some(t.id) && p.status == PlayerStatus::Active)
                .collect();

            let same_pos_count = team_roster.iter()
                .filter(|p| {
                    p.position
                        .map(|pos| format!("{:?}", pos).to_uppercase())
                        == Some(position_str.clone())
                })
                .count();

            let position_need = match same_pos_count {
                0 => 100,
                1 => 60,
                2 => 30,
                _ => 10,
            };

            let avg_ability = if !team_roster.is_empty() {
                team_roster.iter().map(|p| p.ability as f64).sum::<f64>() / team_roster.len() as f64
            } else {
                0.0
            };

            // 获取该球队的排名数据
            let (global_rank, annual_points) = ranking_map
                .get(&t.id)
                .copied()
                .unwrap_or((0, 0));

            TeamInfo {
                id: t.id,
                name: t.name.clone(),
                region_name: region_map.get(&t.region_id).cloned().unwrap_or_default(),
                avg_ability,
                balance: t.balance,
                position_need,
                annual_points,
                global_rank,
            }
        })
        .collect();

    // 查询选手荣誉记录
    let player_honors = query_player_honors(&pool, &save_id, player_id).await;

    // 查询选手赛季表现
    let player_performance = query_player_performance(&pool, &save_id, player_id, season_id as u64, player.ability).await;

    // 检查是否使用 LLM
    let strategy = if is_llm_configured() {
        // 使用 LLM 生成策略
        let llm_config = get_llm_config().ok_or("LLM 配置获取失败")?;
        let llm_service = LLMTransferService::new(llm_config);

        match llm_service.generate_player_strategy(
            &player,
            &team,
            &roster,
            &available_teams,
            &save_id,
            season_id as u64,
            player_honors.as_ref(),
            player_performance.as_ref(),
            current_team_rank,
        ).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("LLM 生成选手策略失败: {}, 使用规则 AI", e);
                AITransferService::generate_mock_player_strategy(
                    &player,
                    &team,
                    &available_teams,
                    &save_id,
                    season_id as u64,
                )
            }
        }
    } else {
        // 使用规则 AI
        AITransferService::generate_mock_player_strategy(
            &player,
            &team,
            &available_teams,
            &save_id,
            season_id as u64,
        )
    };

    // 确保表存在
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS player_transfer_strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL,
            save_id TEXT NOT NULL,
            season_id INTEGER NOT NULL,
            strategy_json TEXT NOT NULL,
            generated_at TEXT NOT NULL,
            UNIQUE(player_id, save_id, season_id)
        )
        "#
    )
    .execute(&pool)
    .await
    .ok();

    // 保存策略到数据库
    let now = chrono::Utc::now().to_rfc3339();
    let strategy_json = serde_json::to_string(&strategy).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO player_transfer_strategies (player_id, save_id, season_id, strategy_json, generated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(player_id, save_id, season_id) DO UPDATE SET
            strategy_json = excluded.strategy_json,
            generated_at = excluded.generated_at
        "#
    )
    .bind(player_id as i64)
    .bind(&save_id)
    .bind(season_id)
    .bind(&strategy_json)
    .bind(&now)
    .execute(&pool)
    .await
    .ok();

    Ok(ApiResponse::success(strategy))
}

/// 获取选手的转会策略
#[tauri::command]
pub async fn get_player_transfer_strategy(
    player_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Option<PlayerTransferStrategy>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 查询策略
    let row = sqlx::query(
        "SELECT * FROM player_transfer_strategies WHERE save_id = ? AND player_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(player_id as i64)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            let strategy_json: String = r.get("strategy_json");
            let strategy: PlayerTransferStrategy = serde_json::from_str(&strategy_json)
                .map_err(|e| format!("解析策略失败: {}", e))?;
            Ok(ApiResponse::success(Some(strategy)))
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 获取所有想离队选手的策略列表
#[tauri::command]
pub async fn get_all_player_strategies(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PlayerTransferStrategyInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 查询所有策略
    let rows = sqlx::query(
        "SELECT * FROM player_transfer_strategies WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 获取所有选手和球队信息
    let all_players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let player_map: HashMap<u64, &Player> = all_players.iter().map(|p| (p.id, p)).collect();

    let all_teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let team_map: HashMap<u64, &Team> = all_teams.iter().map(|t| (t.id, t)).collect();

    let mut result = Vec::new();
    for row in rows {
        let strategy_json: String = row.get("strategy_json");
        let strategy: PlayerTransferStrategy = match serde_json::from_str(&strategy_json) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let player = match player_map.get(&strategy.player_id) {
            Some(p) => *p,
            None => continue,
        };

        let team_id = match player.team_id {
            Some(id) => id,
            None => continue,
        };

        let team = match team_map.get(&team_id) {
            Some(t) => *t,
            None => continue,
        };

        let departure_reasons: Vec<String> = strategy.departure_reasons.iter()
            .map(|r| r.display_name().to_string())
            .collect();

        let preferred_teams: Vec<PreferredTeamInfo> = strategy.preferred_teams.iter()
            .map(|pt| PreferredTeamInfo {
                team_id: pt.team_id,
                team_name: pt.team_name.clone(),
                priority: pt.priority,
                reason: pt.reason.display_name().to_string(),
                reason_detail: pt.reason_detail.clone(),
                attractiveness_score: pt.attractiveness_score,
            })
            .collect();

        result.push(PlayerTransferStrategyInfo {
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default(),
            ability: player.ability,
            age: player.age,
            team_id,
            team_name: team.name.clone(),
            wants_to_leave: strategy.wants_to_leave,
            decision_confidence: strategy.decision_confidence,
            departure_reasons,
            leave_reasoning: strategy.leave_reasoning.clone(),
            preferred_teams_count: strategy.preferred_teams.len(),
            preferred_teams,
            team_preference_reasoning: strategy.team_preference_reasoning.clone(),
            expected_salary: strategy.expected_salary,
            expected_min_salary: strategy.expected_min_salary,
            expected_years: strategy.expected_years,
            requires_starter: strategy.requires_starter,
            is_mock: strategy.is_mock,
            generated_at: strategy.generated_at.clone(),
        });
    }

    // 只返回想离队的选手
    result.retain(|s| s.wants_to_leave);

    Ok(ApiResponse::success(result))
}

/// 初始化选手策略数据库表
#[tauri::command]
pub async fn init_player_strategy_tables(state: State<'_, AppState>) -> Result<ApiResponse<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 创建选手转会策略表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS player_transfer_strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL,
            save_id TEXT NOT NULL,
            season_id INTEGER NOT NULL,
            strategy_json TEXT NOT NULL,
            generated_at TEXT NOT NULL,
            UNIQUE(player_id, save_id, season_id)
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 创建转会申请表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transfer_applications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL,
            from_team_id INTEGER NOT NULL,
            save_id TEXT NOT NULL,
            season_id INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'PENDING',
            player_strategy_id INTEGER,
            received_offers TEXT DEFAULT '[]',
            accepted_offer TEXT,
            rejected_offers TEXT DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(player_id, save_id, season_id)
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(()))
}

// ==================== 辅助函数 ====================

/// 查询选手的荣誉记录
async fn query_player_honors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    player_id: u64,
) -> Option<PlayerHonorInfo> {
    // 查询选手的所有荣誉
    let rows = sqlx::query(
        r#"
        SELECT honor_type, tournament_type, tournament_name, season_id
        FROM honors
        WHERE save_id = ? AND player_id = ?
        ORDER BY season_id DESC
        "#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .fetch_all(pool)
    .await
    .ok()?;

    if rows.is_empty() {
        return Some(PlayerHonorInfo::default());
    }

    let mut honors = PlayerHonorInfo::default();
    let mut recent_count = 0;

    for row in rows {
        let honor_type: String = row.get("honor_type");
        let tournament_type: Option<String> = row.get("tournament_type");
        let tournament_name: Option<String> = row.get("tournament_name");

        // 根据荣誉类型分类统计
        // 注意：数据库中的荣誉类型格式为 PLAYER_CHAMPION, TEAM_CHAMPION, TOURNAMENT_MVP 等
        let honor_upper = honor_type.to_uppercase();
        if honor_upper.contains("CHAMPION") {
            // 冠军荣誉 (PLAYER_CHAMPION, TEAM_CHAMPION, Champion, CHAMPION)
            if let Some(ref t_type) = tournament_type {
                match t_type.to_uppercase().as_str() {
                    "WORLDS" | "WORLDCHAMPIONSHIP" | "WORLD_CHAMPIONSHIP" => honors.worlds_championships += 1,
                    "MSI" => honors.msi_championships += 1,
                    "SPRINGSPLIT" | "SUMMERSPLIT" | "SPRING" | "SUMMER" | "REGIONAL" | "SPRING_SPLIT" | "SUMMER_SPLIT" => {
                        honors.regional_championships += 1
                    }
                    _ => honors.regional_championships += 1,
                }
            } else {
                honors.regional_championships += 1;
            }

            // 添加到最近荣誉
            if recent_count < 5 {
                if let Some(name) = &tournament_name {
                    honors.recent_honors.push(format!("{} 冠军", name));
                    recent_count += 1;
                }
            }
        } else if honor_upper.contains("TOURNAMENT_MVP") || honor_upper == "MVP" || honor_upper == "TOURNAMENTMVP" {
            // 赛事MVP
            honors.tournament_mvps += 1;
            if recent_count < 5 {
                if let Some(name) = &tournament_name {
                    honors.recent_honors.push(format!("{} MVP", name));
                    recent_count += 1;
                }
            }
        } else if honor_upper.contains("FINALS_MVP") || honor_upper == "FMVP" || honor_upper == "FINALSMVP" {
            // 决赛MVP
            honors.finals_mvps += 1;
            if recent_count < 5 {
                if let Some(name) = &tournament_name {
                    honors.recent_honors.push(format!("{} FMVP", name));
                    recent_count += 1;
                }
            }
        } else if honor_upper.contains("ANNUAL_MVP") || honor_upper.contains("YEARLY_MVP") || honor_upper == "YEARLYMVP" {
            // 年度MVP
            honors.yearly_mvps += 1;
            if recent_count < 5 {
                honors.recent_honors.push("年度MVP".to_string());
                recent_count += 1;
            }
        }
    }

    Some(honors)
}

/// 查询单个选手的赛季表现
async fn query_player_performance(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    player_id: u64,
    season_id: u64,
    player_ability: u8,
) -> Option<crate::services::llm_service::PlayerPerformanceInfo> {
    use crate::services::llm_service::PlayerPerformanceInfo;

    let perf_stats = sqlx::query(
        r#"
        SELECT
            games_played,
            avg_impact,
            avg_performance,
            best_performance,
            consistency_score
        FROM player_season_stats
        WHERE save_id = ? AND player_id = ? AND season_id = ?
        "#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .bind(season_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let (games_played, avg_impact, avg_performance, best_performance, consistency_score) =
        if let Some(row) = perf_stats {
            let games: i64 = row.try_get("games_played").unwrap_or(0);
            let impact: f64 = row.try_get("avg_impact").unwrap_or(0.0);
            let perf: f64 = row.try_get("avg_performance").unwrap_or(0.0);
            let best: f64 = row.try_get("best_performance").unwrap_or(0.0);
            let consistency: f64 = row.try_get("consistency_score").unwrap_or(50.0);
            (games as u32, impact, perf, best, consistency)
        } else {
            return None;
        };

    // 计算与能力值的差异
    let ability_diff = avg_performance - player_ability as f64;

    // 生成表现等级描述
    let performance_tier = if avg_performance >= 90.0 {
        "顶级表现"
    } else if avg_performance >= 80.0 {
        "优秀表现"
    } else if avg_performance >= 70.0 {
        "合格表现"
    } else if avg_performance >= 60.0 {
        "一般表现"
    } else if games_played > 0 {
        "表现欠佳"
    } else {
        "无数据"
    };

    Some(PlayerPerformanceInfo {
        games_played,
        avg_impact,
        avg_performance,
        best_performance,
        consistency_score,
        performance_tier: performance_tier.to_string(),
        ability_diff,
    })
}

/// 查询球队的荣誉记录
async fn query_team_honors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
    current_season: u64,
    roster: &[Player],
) -> Option<TeamHonorInfo> {
    // 查询球队的所有荣誉
    let rows = sqlx::query(
        r#"
        SELECT honor_type, tournament_type, tournament_name, season_id
        FROM honors
        WHERE save_id = ? AND team_id = ?
        ORDER BY season_id DESC
        "#
    )
    .bind(save_id)
    .bind(team_id as i64)
    .fetch_all(pool)
    .await
    .ok()?;

    let mut honors = TeamHonorInfo::default();

    // 统计明星选手
    honors.star_player_count = roster.iter().filter(|p| p.ability >= 90).count() as u32;
    honors.has_star_players = honors.star_player_count > 0;

    for row in &rows {
        let honor_type: String = row.get("honor_type");
        let tournament_type: Option<String> = row.get("tournament_type");
        let tournament_name: Option<String> = row.get("tournament_name");
        let season_id: i64 = row.get("season_id");

        // 根据荣誉类型分类统计
        // 注意：数据库中的荣誉类型格式为 PLAYER_CHAMPION, TEAM_CHAMPION, TOURNAMENT_MVP 等
        let honor_upper = honor_type.to_uppercase();
        if honor_upper.contains("CHAMPION") {
            // 冠军荣誉 (PLAYER_CHAMPION, TEAM_CHAMPION, Champion, CHAMPION)
            if let Some(ref t_type) = tournament_type {
                match t_type.to_uppercase().as_str() {
                    "WORLDS" | "WORLDCHAMPIONSHIP" | "WORLD_CHAMPIONSHIP" => {
                        honors.worlds_championships += 1;
                        // 检查是否为卫冕冠军（上赛季世界赛冠军）
                        if season_id as u64 == current_season.saturating_sub(1) {
                            honors.is_defending_champion = true;
                        }
                    }
                    "MSI" => honors.msi_championships += 1,
                    "SPRINGSPLIT" | "SUMMERSPLIT" | "SPRING" | "SUMMER" | "REGIONAL" | "SPRING_SPLIT" | "SUMMER_SPLIT" => {
                        honors.regional_championships += 1;
                        // 检查是否为赛区卫冕冠军
                        if season_id as u64 == current_season.saturating_sub(1) {
                            honors.is_defending_champion = true;
                        }
                    }
                    _ => honors.regional_championships += 1,
                }
            } else {
                honors.regional_championships += 1;
            }

            // 添加最近成绩
            if honors.recent_results.len() < 3 {
                if let Some(name) = &tournament_name {
                    honors.recent_results.push(format!("{} 冠军", name));
                }
            }
        }
    }

    // 查询最近赛季的排名（如果没有冠军的话）
    if honors.recent_results.is_empty() {
        let recent_rows = sqlx::query(
            r#"
            SELECT t.name as tournament_name, ls.rank
            FROM league_standings ls
            JOIN tournaments t ON ls.tournament_id = t.id
            WHERE ls.save_id = ? AND ls.team_id = ?
            ORDER BY t.season_id DESC
            LIMIT 2
            "#
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .ok();

        if let Some(recent_rows) = recent_rows {
            for row in recent_rows {
                let tournament_name: String = row.get("tournament_name");
                let rank: i64 = row.get("rank");
                honors.recent_results.push(format!("{} 第{}名", tournament_name, rank));
            }
        }
    }

    Some(honors)
}

/// 查询球队阵容选手的荣誉摘要
async fn query_roster_honors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    roster: &[Player],
) -> Vec<RosterPlayerHonorSummary> {
    let mut summaries = Vec::new();

    for player in roster {
        if player.status != PlayerStatus::Active {
            continue;
        }

        // 查询该选手的荣誉统计
        // 注意：数据库中的荣誉类型格式为 PLAYER_CHAMPION, TEAM_CHAMPION, TOURNAMENT_MVP 等
        let honor_stats = sqlx::query(
            r#"
            SELECT
                COUNT(CASE WHEN honor_type LIKE '%CHAMPION%' OR honor_type = 'Champion' THEN 1 END) as championship_count,
                COUNT(CASE WHEN honor_type LIKE '%MVP%' OR honor_type LIKE '%FMVP%' THEN 1 END) as mvp_count
            FROM honors
            WHERE save_id = ? AND player_id = ?
            "#
        )
        .bind(save_id)
        .bind(player.id as i64)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (championship_count, mvp_count) = if let Some(row) = honor_stats {
            (
                row.get::<i64, _>("championship_count") as u32,
                row.get::<i64, _>("mvp_count") as u32,
            )
        } else {
            (0, 0)
        };

        // 生成荣誉描述
        let honor_summary = if championship_count > 0 || mvp_count > 0 {
            let mut parts = Vec::new();
            if championship_count > 0 {
                parts.push(format!("{}冠", championship_count));
            }
            if mvp_count > 0 {
                parts.push(format!("{}MVP", mvp_count));
            }
            parts.join(" ")
        } else {
            "无荣誉".to_string()
        };

        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        summaries.push(RosterPlayerHonorSummary {
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: position_str,
            ability: player.ability,
            is_core: player.is_starter,
            championship_count,
            mvp_count,
            honor_summary,
        });
    }

    // 按能力值排序，优先显示核心选手
    summaries.sort_by(|a, b| {
        b.is_core.cmp(&a.is_core)
            .then(b.ability.cmp(&a.ability))
    });

    summaries
}

/// 查询球队阵容选手的赛季表现
async fn query_roster_performance(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    roster: &[Player],
    season_id: u64,
) -> Vec<PlayerPerformanceSummary> {
    let mut summaries = Vec::new();

    for player in roster {
        if player.status != PlayerStatus::Active {
            continue;
        }

        // 从数据中心的 player_season_stats 表查询选手赛季表现
        let perf_stats = sqlx::query(
            r#"
            SELECT
                games_played,
                avg_impact,
                avg_performance,
                best_performance,
                consistency_score
            FROM player_season_stats
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(player.id as i64)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (games_played, avg_impact, avg_performance, best_performance, consistency_score) =
            if let Some(row) = perf_stats {
                let games: i64 = row.try_get("games_played").unwrap_or(0);
                let impact: f64 = row.try_get("avg_impact").unwrap_or(0.0);
                let perf: f64 = row.try_get("avg_performance").unwrap_or(0.0);
                let best: f64 = row.try_get("best_performance").unwrap_or(0.0);
                let consistency: f64 = row.try_get("consistency_score").unwrap_or(50.0);
                (games as u32, impact, perf, best, consistency)
            } else {
                (0, 0.0, 0.0, 0.0, 50.0)
            };

        // 计算与能力值的差异
        let ability_diff = avg_performance - player.ability as f64;

        // 生成表现等级描述
        let performance_tier = if avg_performance >= 90.0 {
            "顶级表现"
        } else if avg_performance >= 80.0 {
            "优秀表现"
        } else if avg_performance >= 70.0 {
            "合格表现"
        } else if avg_performance >= 60.0 {
            "一般表现"
        } else if games_played > 0 {
            "表现欠佳"
        } else {
            "无数据"
        };

        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        summaries.push(PlayerPerformanceSummary {
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: position_str,
            games_played,
            avg_impact,
            avg_performance,
            best_performance,
            consistency_score,
            performance_tier: performance_tier.to_string(),
            ability_diff,
        });
    }

    // 按平均表现排序
    summaries.sort_by(|a, b| {
        b.avg_performance.partial_cmp(&a.avg_performance).unwrap_or(std::cmp::Ordering::Equal)
    });

    summaries
}
