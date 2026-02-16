//! 选秀权拍卖命令
//!
//! 实现前端调用的拍卖相关命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::{DraftAuctionEngine, DraftRookieInfo, TeamAuctionInfo, AuctionRoundResult};
use crate::models::{
    AuctionStatus, DraftPickListing, DraftPickWanted,
    DraftListingStatus, WantedStatus, get_draft_pick_pricing,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use tauri::State;

/// 拍卖状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionStatusInfo {
    pub id: u64,
    pub status: String,
    pub current_round: u32,
    pub total_rounds: u32,
    pub total_auctions: u32,
    pub successful_auctions: u32,
    pub total_revenue: i64,
    pub total_commission: i64,
    pub listings: Vec<ListingInfo>,
}

/// 挂牌信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingInfo {
    pub id: u64,
    pub seller_team_id: u64,
    pub seller_team_name: String,
    pub draft_position: u32,
    pub position_name: String,
    pub starting_price: i64,
    pub current_price: i64,
    pub min_increment: i64,
    pub status: String,
    pub buyer_team_id: Option<u64>,
    pub buyer_team_name: Option<String>,
    pub final_price: Option<i64>,
    pub current_bid_round: u32,
}

/// 拍卖事件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEventInfo {
    pub id: u64,
    pub event_type: String,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub draft_position: Option<u32>,
    pub position_name: Option<String>,
    pub amount: Option<i64>,
    pub headline: String,
    pub description: String,
    pub importance: String,
    pub round: u32,
    pub created_at: String,
}

/// 签位价格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickPriceInfo {
    pub position: u32,
    pub name: String,
    pub starting_price: i64,
    pub min_increment: i64,
}

/// 获取签位价格配置
#[tauri::command]
pub async fn get_draft_pick_prices() -> Result<CommandResult<Vec<DraftPickPriceInfo>>, String> {
    let prices: Vec<DraftPickPriceInfo> = get_draft_pick_pricing()
        .into_iter()
        .map(|p| DraftPickPriceInfo {
            position: p.position,
            name: p.name,
            starting_price: p.starting_price,
            min_increment: p.min_increment,
        })
        .collect();

    Ok(CommandResult::ok(prices))
}

/// 开始选秀权拍卖
#[tauri::command]
pub async fn start_draft_auction(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<AuctionStatusInfo>, String> {
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

    // 检查是否已有拍卖
    let existing_auction = sqlx::query(
        "SELECT id FROM draft_pick_auctions WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing_auction.is_some() {
        return Ok(CommandResult::err("Auction already exists for this season and region"));
    }

    // 获取选秀顺位
    let order_rows = sqlx::query(
        r#"
        SELECT do.team_id, do.draft_position, t.name as team_name, t.balance
        FROM draft_orders do
        JOIN teams t ON do.team_id = t.id
        WHERE do.save_id = ? AND do.season_id = ? AND do.region_id = ?
        ORDER BY do.draft_position
        "#
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if order_rows.is_empty() {
        return Ok(CommandResult::err("No draft orders found. Run draft lottery first."));
    }

    // 获取球队信息
    let team_rows = sqlx::query(
        "SELECT id, name, balance, power_rating FROM teams WHERE save_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 获取各队阵容数量
    let mut roster_counts: HashMap<u64, u32> = HashMap::new();
    for row in &team_rows {
        let team_id: i64 = row.get("id");
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM players WHERE team_id = ? AND status = 'Active'"
        )
        .bind(team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
        roster_counts.insert(team_id as u64, count as u32);
    }

    // 查询各队位置需求（每个位置的球员数量 → 转换为需求度）
    let mut position_needs: HashMap<u64, HashMap<String, u8>> = HashMap::new();
    for row in &team_rows {
        let team_id: i64 = row.get("id");
        let pos_rows = sqlx::query(
            "SELECT position, COUNT(*) as cnt FROM players WHERE team_id = ? AND status = 'Active' GROUP BY position"
        )
        .bind(team_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut needs = HashMap::new();
        for pos_name in &["TOP", "JUG", "MID", "ADC", "SUP"] {
            let count = pos_rows.iter()
                .find(|r| {
                    let p: String = r.get("position");
                    p.to_uppercase() == *pos_name
                })
                .map(|r| r.get::<i64, _>("cnt"))
                .unwrap_or(0);
            let need: u8 = match count {
                0 => 100,
                1 => 60,
                2 => 30,
                _ => 10,
            };
            needs.insert(pos_name.to_string(), need);
        }
        position_needs.insert(team_id as u64, needs);
    }

    // 查询本届新秀名单（draft_players），供 AI 卖签/买签决策参考
    let rookie_rows = sqlx::query(
        "SELECT ability, potential, position, draft_rank FROM draft_players WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0 ORDER BY draft_rank"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let draft_rookies: Vec<DraftRookieInfo> = rookie_rows.iter().map(|r| {
        DraftRookieInfo {
            ability: r.get::<i64, _>("ability") as u8,
            potential: r.get::<i64, _>("potential") as u8,
            position: r.get::<Option<String>, _>("position").unwrap_or_default().to_uppercase(),
            draft_rank: r.get::<i64, _>("draft_rank") as u32,
        }
    }).collect();

    // 构建选秀顺位
    let mut draft_orders = Vec::new();
    for row in &order_rows {
        let team_id: i64 = row.get("team_id");
        let draft_position: i64 = row.get("draft_position");
        draft_orders.push(crate::models::DraftOrder {
            id: 0,
            save_id: save_id.clone(),
            season_id: current_season as u64,
            region_id,
            team_id: team_id as u64,
            summer_rank: draft_position as u32,
            draft_position: draft_position as u32,
            lottery_result: None,
        });
    }

    // 构建球队列表
    let teams: Vec<crate::models::Team> = team_rows.iter().map(|row| {
        crate::models::Team {
            id: row.get::<i64, _>("id") as u64,
            region_id,
            name: row.get("name"),
            short_name: None,
            power_rating: row.get("power_rating"),
            total_matches: 0,
            wins: 0,
            win_rate: 0.0,
            annual_points: 0,
            cross_year_points: 0,
            balance: row.get("balance"),
            brand_value: 50.0,
        }
    }).collect();

    // 创建拍卖引擎
    let mut engine = DraftAuctionEngine::new(save_id.clone(), current_season as u64, region_id);
    engine.draft_rookies = draft_rookies;
    engine.initialize(
        &teams,
        &draft_orders,
        &roster_counts,
        &position_needs,
    );

    // 创建拍卖记录
    let auction_id: i64 = sqlx::query(
        r#"
        INSERT INTO draft_pick_auctions (
            save_id, season_id, region_id, status, current_round, total_rounds,
            total_auctions, successful_auctions, total_revenue, total_commission
        ) VALUES (?, ?, ?, 'PREPARING', 0, 3, 0, 0, 0, 0)
        RETURNING id
        "#
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    engine.auction.id = auction_id as u64;

    // 开始拍卖（评估卖签意愿，创建挂牌）
    let _events = engine.start_auction();

    // 保存挂牌到数据库
    for listing in &engine.listings {
        sqlx::query(
            r#"
            INSERT INTO draft_pick_listings (
                save_id, season_id, region_id, auction_id, seller_team_id, seller_team_name,
                draft_position, starting_price, current_price, min_increment, status,
                current_bid_round
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .bind(auction_id)
        .bind(listing.seller_team_id as i64)
        .bind(&listing.seller_team_name)
        .bind(listing.draft_position as i64)
        .bind(listing.starting_price)
        .bind(listing.current_price)
        .bind(listing.min_increment)
        .bind(listing.status.to_string())
        .bind(listing.current_bid_round as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 保存求购请求
    for wanted in &engine.wanted_requests {
        sqlx::query(
            r#"
            INSERT INTO draft_pick_wanted (
                save_id, season_id, region_id, auction_id, buyer_team_id, buyer_team_name,
                target_position, offer_price, reason, status, holder_team_id, holder_team_name,
                response_reason, final_price, resolved_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(wanted.season_id as i64)
        .bind(wanted.region_id as i64)
        .bind(auction_id)
        .bind(wanted.buyer_team_id as i64)
        .bind(&wanted.buyer_team_name)
        .bind(wanted.target_position as i64)
        .bind(wanted.offer_price)
        .bind(&wanted.reason)
        .bind(wanted.status.to_string())
        .bind(wanted.holder_team_id as i64)
        .bind(&wanted.holder_team_name)
        .bind(&wanted.response_reason)
        .bind(wanted.final_price)
        .bind(&wanted.resolved_at)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新拍卖状态
    sqlx::query(
        "UPDATE draft_pick_auctions SET status = ?, total_auctions = ?, started_at = datetime('now') WHERE id = ?"
    )
    .bind(engine.auction.status.to_string())
    .bind(engine.auction.total_auctions as i64)
    .bind(auction_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 返回状态
    let status_info = build_auction_status_info(&engine, auction_id as u64);
    Ok(CommandResult::ok(status_info))
}

/// 执行一轮竞拍
#[tauri::command]
pub async fn execute_auction_round(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<AuctionStatusInfo>, String> {
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

    // 加载拍卖引擎状态
    let mut engine = load_auction_engine(&pool, &save_id, current_season, region_id).await?;

    if engine.auction.status != AuctionStatus::InProgress {
        return Ok(CommandResult::err("Auction is not in progress"));
    }

    // 执行一轮竞拍
    let result = engine.execute_round();

    // 保存结果到数据库
    save_auction_round_result(&pool, &save_id, &engine, &result).await?;

    let status_info = build_auction_status_info(&engine, engine.auction.id);
    Ok(CommandResult::ok(status_info))
}

/// 快进完成所有拍卖轮次
#[tauri::command]
pub async fn fast_forward_auction(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<AuctionStatusInfo>, String> {
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

    // 加载拍卖引擎状态
    let mut engine = load_auction_engine(&pool, &save_id, current_season, region_id).await?;

    if engine.auction.status != AuctionStatus::InProgress {
        return Ok(CommandResult::err("Auction is not in progress"));
    }

    // 快进所有轮次
    let results = engine.fast_forward();

    // 保存所有结果
    for result in &results {
        save_auction_round_result(&pool, &save_id, &engine, result).await?;
    }

    let status_info = build_auction_status_info(&engine, engine.auction.id);
    Ok(CommandResult::ok(status_info))
}

/// 获取拍卖状态
#[tauri::command]
pub async fn get_auction_status(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Option<AuctionStatusInfo>>, String> {
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

    // 查询拍卖
    let auction_row = sqlx::query(
        "SELECT * FROM draft_pick_auctions WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match auction_row {
        Some(row) => {
            let auction_id: i64 = row.get("id");

            // 查询挂牌
            let listing_rows = sqlx::query(
                "SELECT * FROM draft_pick_listings WHERE auction_id = ? ORDER BY draft_position"
            )
            .bind(auction_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

            let listings: Vec<ListingInfo> = listing_rows.iter().map(|r| {
                let position: i64 = r.get("draft_position");
                ListingInfo {
                    id: r.get::<i64, _>("id") as u64,
                    seller_team_id: r.get::<i64, _>("seller_team_id") as u64,
                    seller_team_name: r.get("seller_team_name"),
                    draft_position: position as u32,
                    position_name: crate::models::get_position_name(position as u32),
                    starting_price: r.get("starting_price"),
                    current_price: r.get("current_price"),
                    min_increment: r.get("min_increment"),
                    status: r.get("status"),
                    buyer_team_id: r.get::<Option<i64>, _>("buyer_team_id").map(|id| id as u64),
                    buyer_team_name: r.get("buyer_team_name"),
                    final_price: r.get("final_price"),
                    current_bid_round: r.get::<i64, _>("current_bid_round") as u32,
                }
            }).collect();

            Ok(CommandResult::ok(Some(AuctionStatusInfo {
                id: auction_id as u64,
                status: row.get("status"),
                current_round: row.get::<i64, _>("current_round") as u32,
                total_rounds: row.get::<i64, _>("total_rounds") as u32,
                total_auctions: row.get::<i64, _>("total_auctions") as u32,
                successful_auctions: row.get::<i64, _>("successful_auctions") as u32,
                total_revenue: row.get("total_revenue"),
                total_commission: row.get("total_commission"),
                listings,
            })))
        }
        None => Ok(CommandResult::ok(None)),
    }
}

/// 获取拍卖事件
#[tauri::command]
pub async fn get_auction_events(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<AuctionEventInfo>>, String> {
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

    // 查询拍卖
    let auction_row = sqlx::query(
        "SELECT id FROM draft_pick_auctions WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match auction_row {
        Some(row) => {
            let auction_id: i64 = row.get("id");

            let event_rows = sqlx::query(
                "SELECT * FROM draft_pick_auction_events WHERE auction_id = ? ORDER BY id ASC"
            )
            .bind(auction_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

            let events: Vec<AuctionEventInfo> = event_rows.iter().map(|r| {
                let position: Option<i64> = r.get("draft_position");
                AuctionEventInfo {
                    id: r.get::<i64, _>("id") as u64,
                    event_type: r.get("event_type"),
                    team_id: r.get::<Option<i64>, _>("team_id").map(|id| id as u64),
                    team_name: r.get("team_name"),
                    draft_position: position.map(|p| p as u32),
                    position_name: position.map(|p| crate::models::get_position_name(p as u32)),
                    amount: r.get("amount"),
                    headline: r.get("headline"),
                    description: r.get("description"),
                    importance: r.get("importance"),
                    round: r.get::<i64, _>("round") as u32,
                    created_at: r.get("created_at"),
                }
            }).collect();

            Ok(CommandResult::ok(events))
        }
        None => Ok(CommandResult::ok(Vec::new())),
    }
}

/// 完成拍卖并更新选秀顺位
#[tauri::command]
pub async fn finalize_auction(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<bool>, String> {
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

    // 获取成功售出的挂牌
    let sold_listings = sqlx::query(
        r#"
        SELECT l.*, a.id as auction_id
        FROM draft_pick_listings l
        JOIN draft_pick_auctions a ON l.auction_id = a.id
        WHERE a.save_id = ? AND a.season_id = ? AND a.region_id = ? AND l.status = 'SOLD'
        "#
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for listing in sold_listings {
        let seller_team_id: i64 = listing.get("seller_team_id");
        let buyer_team_id: i64 = listing.get("buyer_team_id");
        let draft_position: i64 = listing.get("draft_position");
        let final_price: i64 = listing.get("final_price");
        let seller_revenue: i64 = listing.get("seller_revenue");

        // 更新选秀顺位
        sqlx::query(
            r#"
            UPDATE draft_orders
            SET team_id = ?, original_team_id = ?, acquired_via = 'AUCTION', acquisition_price = ?
            WHERE save_id = ? AND season_id = ? AND region_id = ? AND draft_position = ?
            "#
        )
        .bind(buyer_team_id)
        .bind(seller_team_id)
        .bind(final_price)
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .bind(draft_position)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 更新球队余额
        // 买家扣款
        sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
            .bind(final_price)
            .bind(buyer_team_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        // 卖家收款（扣除佣金后）
        sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
            .bind(seller_revenue)
            .bind(seller_team_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        // 记录财务交易 - 买家
        sqlx::query(
            r#"
            INSERT INTO financial_transactions (save_id, season_id, team_id, transaction_type, amount, description)
            VALUES (?, ?, ?, 'DRAFT_PICK_PURCHASE', ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(buyer_team_id)
        .bind(-final_price)
        .bind(format!("购买第{}顺位选秀权", draft_position))
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 记录财务交易 - 卖家
        sqlx::query(
            r#"
            INSERT INTO financial_transactions (save_id, season_id, team_id, transaction_type, amount, description)
            VALUES (?, ?, ?, 'DRAFT_PICK_SALE', ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(seller_team_id)
        .bind(seller_revenue)
        .bind(format!("出售第{}顺位选秀权（扣除5%佣金）", draft_position))
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 处理求购成交的选秀权转移
    let fulfilled_wanted = sqlx::query(
        r#"
        SELECT w.*, a.id as auction_id
        FROM draft_pick_wanted w
        JOIN draft_pick_auctions a ON w.auction_id = a.id
        WHERE a.save_id = ? AND a.season_id = ? AND a.region_id = ? AND w.status = 'FULFILLED'
        "#
    )
    .bind(&save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for wanted in fulfilled_wanted {
        let holder_team_id: i64 = wanted.get("holder_team_id");
        let buyer_team_id: i64 = wanted.get("buyer_team_id");
        let target_position: i64 = wanted.get("target_position");
        let final_price: i64 = wanted.get::<Option<i64>, _>("final_price").unwrap_or(0);
        let commission = (final_price as f64 * 0.05) as i64;
        let seller_revenue = final_price - commission;

        sqlx::query(
            r#"
            UPDATE draft_orders
            SET team_id = ?, original_team_id = ?, acquired_via = 'WANTED', acquisition_price = ?
            WHERE save_id = ? AND season_id = ? AND region_id = ? AND draft_position = ?
            "#
        )
        .bind(buyer_team_id)
        .bind(holder_team_id)
        .bind(final_price)
        .bind(&save_id)
        .bind(current_season)
        .bind(region_id as i64)
        .bind(target_position)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
            .bind(final_price)
            .bind(buyer_team_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
            .bind(seller_revenue)
            .bind(holder_team_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO financial_transactions (save_id, season_id, team_id, transaction_type, amount, description)
            VALUES (?, ?, ?, 'DRAFT_PICK_PURCHASE', ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(buyer_team_id)
        .bind(-final_price)
        .bind(format!("求购第{}顺位选秀权", target_position))
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO financial_transactions (save_id, season_id, team_id, transaction_type, amount, description)
            VALUES (?, ?, ?, 'DRAFT_PICK_SALE', ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(current_season)
        .bind(holder_team_id)
        .bind(seller_revenue)
        .bind(format!("出售第{}顺位选秀权（求购成交，扣除5%佣金）", target_position))
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(true))
}

// ============================================
// 辅助函数
// ============================================

/// 加载拍卖引擎状态
async fn load_auction_engine(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    save_id: &str,
    current_season: i64,
    region_id: u64,
) -> Result<DraftAuctionEngine, String> {
    // 查询拍卖
    let auction_row = sqlx::query(
        "SELECT * FROM draft_pick_auctions WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Auction not found: {}", e))?;

    let auction_id: i64 = auction_row.get("id");

    let mut engine = DraftAuctionEngine::new(save_id.to_string(), current_season as u64, region_id);
    engine.auction.id = auction_id as u64;
    engine.auction.status = AuctionStatus::from(auction_row.get::<String, _>("status").as_str());
    engine.auction.current_round = auction_row.get::<i64, _>("current_round") as u32;
    engine.auction.total_rounds = auction_row.get::<i64, _>("total_rounds") as u32;
    engine.auction.total_auctions = auction_row.get::<i64, _>("total_auctions") as u32;
    engine.auction.successful_auctions = auction_row.get::<i64, _>("successful_auctions") as u32;
    engine.auction.total_revenue = auction_row.get("total_revenue");
    engine.auction.total_commission = auction_row.get("total_commission");

    // 加载挂牌
    let listing_rows = sqlx::query(
        "SELECT * FROM draft_pick_listings WHERE auction_id = ?"
    )
    .bind(auction_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in listing_rows {
        engine.listings.push(DraftPickListing {
            id: row.get::<i64, _>("id") as u64,
            save_id: save_id.to_string(),
            season_id: current_season as u64,
            region_id,
            auction_id: auction_id as u64,
            seller_team_id: row.get::<i64, _>("seller_team_id") as u64,
            seller_team_name: row.get("seller_team_name"),
            draft_position: row.get::<i64, _>("draft_position") as u32,
            starting_price: row.get("starting_price"),
            current_price: row.get("current_price"),
            min_increment: row.get("min_increment"),
            status: DraftListingStatus::from(row.get::<String, _>("status").as_str()),
            buyer_team_id: row.get::<Option<i64>, _>("buyer_team_id").map(|id| id as u64),
            buyer_team_name: row.get("buyer_team_name"),
            final_price: row.get("final_price"),
            commission_fee: row.get("commission_fee"),
            seller_revenue: row.get("seller_revenue"),
            current_bid_round: row.get::<i64, _>("current_bid_round") as u32,
            created_at: row.get("created_at"),
            sold_at: row.get("sold_at"),
        });
    }

    // 加载球队信息
    let team_rows = sqlx::query(
        "SELECT id, name, balance, power_rating FROM teams WHERE save_id = ? AND region_id = ?"
    )
    .bind(save_id)
    .bind(region_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in team_rows {
        let team_id: i64 = row.get("id");
        let balance: i64 = row.get("balance");

        let roster_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM players WHERE team_id = ? AND status = 'Active'"
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let pos_rows = sqlx::query(
            "SELECT position, COUNT(*) as cnt FROM players WHERE team_id = ? AND status = 'Active' GROUP BY position"
        )
        .bind(team_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut needs = HashMap::new();
        for pos_name in &["TOP", "JUG", "MID", "ADC", "SUP"] {
            let count = pos_rows.iter()
                .find(|r| {
                    let p: String = r.get("position");
                    p.to_uppercase() == *pos_name
                })
                .map(|r| r.get::<i64, _>("cnt"))
                .unwrap_or(0);
            let need: u8 = match count {
                0 => 100,
                1 => 60,
                2 => 30,
                _ => 10,
            };
            needs.insert(pos_name.to_string(), need);
        }

        engine.teams.insert(team_id as u64, TeamAuctionInfo {
            team_id: team_id as u64,
            team_name: row.get("name"),
            balance,
            financial_status: crate::models::FinancialStatus::from_balance(balance),
            roster_count: roster_count as u32,
            position_needs: needs,
            avg_ability: row.get("power_rating"),
        });
    }

    // 加载选秀顺位
    let order_rows = sqlx::query(
        "SELECT team_id, draft_position FROM draft_orders WHERE save_id = ? AND season_id = ? AND region_id = ?"
    )
    .bind(save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in order_rows {
        let team_id: i64 = row.get("team_id");
        let position: i64 = row.get("draft_position");
        engine.draft_orders.insert(team_id as u64, position as u32);
    }

    // 加载本届新秀名单
    let rookie_rows = sqlx::query(
        "SELECT ability, potential, position, draft_rank FROM draft_players WHERE save_id = ? AND season_id = ? AND region_id = ? AND is_picked = 0 ORDER BY draft_rank"
    )
    .bind(save_id)
    .bind(current_season)
    .bind(region_id as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    engine.draft_rookies = rookie_rows.iter().map(|r| {
        DraftRookieInfo {
            ability: r.get::<i64, _>("ability") as u8,
            potential: r.get::<i64, _>("potential") as u8,
            position: r.get::<Option<String>, _>("position").unwrap_or_default().to_uppercase(),
            draft_rank: r.get::<i64, _>("draft_rank") as u32,
        }
    }).collect();

    // 加载已有求购请求
    let wanted_rows = sqlx::query(
        r#"
        SELECT id, save_id, season_id, region_id, auction_id, buyer_team_id, buyer_team_name,
               target_position, offer_price, reason, status, holder_team_id, holder_team_name,
               response_reason, final_price, created_at, resolved_at
        FROM draft_pick_wanted
        WHERE auction_id = ?
        "#
    )
    .bind(auction_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in wanted_rows {
        engine.wanted_requests.push(DraftPickWanted {
            id: row.get::<i64, _>("id") as u64,
            save_id: save_id.to_string(),
            season_id: current_season as u64,
            region_id,
            auction_id: auction_id as u64,
            buyer_team_id: row.get::<i64, _>("buyer_team_id") as u64,
            buyer_team_name: row.get("buyer_team_name"),
            target_position: row.get::<i64, _>("target_position") as u32,
            offer_price: row.get("offer_price"),
            reason: row.get("reason"),
            status: WantedStatus::from(row.get::<String, _>("status").as_str()),
            holder_team_id: row.get::<i64, _>("holder_team_id") as u64,
            holder_team_name: row.get("holder_team_name"),
            response_reason: row.get("response_reason"),
            final_price: row.get("final_price"),
            created_at: row.get("created_at"),
            resolved_at: row.get("resolved_at"),
        });
    }

    Ok(engine)
}

/// 保存拍卖轮次结果
async fn save_auction_round_result(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    save_id: &str,
    engine: &DraftAuctionEngine,
    result: &AuctionRoundResult,
) -> Result<(), String> {
    // 保存出价
    for bid in &result.bids {
        sqlx::query(
            r#"
            INSERT INTO draft_pick_bids (
                save_id, listing_id, bidder_team_id, bidder_team_name, bid_amount, bid_round, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(save_id)
        .bind(bid.listing_id as i64)
        .bind(bid.bidder_team_id as i64)
        .bind(&bid.bidder_team_name)
        .bind(bid.bid_amount)
        .bind(bid.bid_round as i64)
        .bind(bid.status.to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 保存事件
    for event in &result.events {
        sqlx::query(
            r#"
            INSERT INTO draft_pick_auction_events (
                save_id, auction_id, listing_id, event_type, team_id, team_name,
                draft_position, amount, headline, description, importance, round
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(save_id)
        .bind(engine.auction.id as i64)
        .bind(event.listing_id.map(|id| id as i64))
        .bind(event.event_type.to_string())
        .bind(event.team_id.map(|id| id as i64))
        .bind(&event.team_name)
        .bind(event.draft_position.map(|p| p as i64))
        .bind(event.amount)
        .bind(&event.headline)
        .bind(&event.description)
        .bind(event.importance.to_string())
        .bind(event.round as i64)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新挂牌状态
    for listing in &engine.listings {
        sqlx::query(
            r#"
            UPDATE draft_pick_listings SET
                current_price = ?, status = ?, buyer_team_id = ?, buyer_team_name = ?,
                final_price = ?, commission_fee = ?, seller_revenue = ?,
                current_bid_round = ?, sold_at = ?
            WHERE id = ?
            "#
        )
        .bind(listing.current_price)
        .bind(listing.status.to_string())
        .bind(listing.buyer_team_id.map(|id| id as i64))
        .bind(&listing.buyer_team_name)
        .bind(listing.final_price)
        .bind(listing.commission_fee)
        .bind(listing.seller_revenue)
        .bind(listing.current_bid_round as i64)
        .bind(&listing.sold_at)
        .bind(listing.id as i64)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新求购请求状态
    for wanted in &result.wanted_results {
        sqlx::query(
            r#"
            UPDATE draft_pick_wanted SET
                status = ?, final_price = ?, response_reason = ?, resolved_at = ?, offer_price = ?
            WHERE id = ?
            "#
        )
        .bind(wanted.status.to_string())
        .bind(wanted.final_price)
        .bind(&wanted.response_reason)
        .bind(&wanted.resolved_at)
        .bind(wanted.offer_price)
        .bind(wanted.id as i64)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 插入本轮新生成的求购请求（id == 0 表示尚未写入数据库）
    for wanted in &engine.wanted_requests {
        if wanted.id != 0 {
            continue;
        }
        sqlx::query(
            r#"
            INSERT INTO draft_pick_wanted (
                save_id, season_id, region_id, auction_id, buyer_team_id, buyer_team_name,
                target_position, offer_price, reason, status, holder_team_id, holder_team_name,
                response_reason, final_price, resolved_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(save_id)
        .bind(wanted.season_id as i64)
        .bind(wanted.region_id as i64)
        .bind(wanted.auction_id as i64)
        .bind(wanted.buyer_team_id as i64)
        .bind(&wanted.buyer_team_name)
        .bind(wanted.target_position as i64)
        .bind(wanted.offer_price)
        .bind(&wanted.reason)
        .bind(wanted.status.to_string())
        .bind(wanted.holder_team_id as i64)
        .bind(&wanted.holder_team_name)
        .bind(&wanted.response_reason)
        .bind(wanted.final_price)
        .bind(&wanted.resolved_at)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 更新拍卖状态
    sqlx::query(
        r#"
        UPDATE draft_pick_auctions SET
            status = ?, current_round = ?, successful_auctions = ?,
            total_revenue = ?, total_commission = ?, completed_at = ?
        WHERE id = ?
        "#
    )
    .bind(engine.auction.status.to_string())
    .bind(engine.auction.current_round as i64)
    .bind(engine.auction.successful_auctions as i64)
    .bind(engine.auction.total_revenue)
    .bind(engine.auction.total_commission)
    .bind(&engine.auction.completed_at)
    .bind(engine.auction.id as i64)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 构建拍卖状态信息
fn build_auction_status_info(engine: &DraftAuctionEngine, auction_id: u64) -> AuctionStatusInfo {
    let listings: Vec<ListingInfo> = engine.listings.iter().map(|l| {
        ListingInfo {
            id: l.id,
            seller_team_id: l.seller_team_id,
            seller_team_name: l.seller_team_name.clone(),
            draft_position: l.draft_position,
            position_name: crate::models::get_position_name(l.draft_position),
            starting_price: l.starting_price,
            current_price: l.current_price,
            min_increment: l.min_increment,
            status: l.status.to_string(),
            buyer_team_id: l.buyer_team_id,
            buyer_team_name: l.buyer_team_name.clone(),
            final_price: l.final_price,
            current_bid_round: l.current_bid_round,
        }
    }).collect();

    AuctionStatusInfo {
        id: auction_id,
        status: engine.auction.status.to_string(),
        current_round: engine.auction.current_round,
        total_rounds: engine.auction.total_rounds,
        total_auctions: engine.auction.total_auctions,
        successful_auctions: engine.auction.successful_auctions,
        total_revenue: engine.auction.total_revenue,
        total_commission: engine.auction.total_commission,
        listings,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantedRequestInfo {
    pub id: i64,
    pub buyer_team_id: i64,
    pub buyer_team_name: String,
    pub target_position: i32,
    pub offer_price: i64,
    pub reason: String,
    pub status: String,
    pub holder_team_id: i64,
    pub holder_team_name: String,
    pub response_reason: Option<String>,
    pub final_price: Option<i64>,
}

#[tauri::command]
pub async fn get_auction_wanted_requests(
    state: State<'_, AppState>,
    region_id: u64,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<WantedRequestInfo>>, String> {
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

    let target_season: i64 = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let rows = sqlx::query(
        r#"
        SELECT id, buyer_team_id, buyer_team_name, target_position, offer_price,
               reason, status, holder_team_id, holder_team_name, response_reason, final_price
        FROM draft_pick_wanted
        WHERE save_id = ? AND season_id = ? AND region_id = ?
        ORDER BY target_position ASC
        "#
    )
    .bind(&save_id)
    .bind(target_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let results: Vec<WantedRequestInfo> = rows.iter().map(|row| WantedRequestInfo {
        id: row.get("id"),
        buyer_team_id: row.get("buyer_team_id"),
        buyer_team_name: row.get("buyer_team_name"),
        target_position: row.get("target_position"),
        offer_price: row.get("offer_price"),
        reason: row.get("reason"),
        status: row.get("status"),
        holder_team_id: row.get("holder_team_id"),
        holder_team_name: row.get("holder_team_name"),
        response_reason: row.try_get("response_reason").ok(),
        final_price: row.try_get("final_price").ok(),
    }).collect();

    Ok(CommandResult::ok(results))
}
