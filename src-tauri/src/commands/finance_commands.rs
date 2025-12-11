use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::FinancialEngine;
use crate::models::TournamentType;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 队伍财务摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamFinanceSummary {
    pub team_id: u64,
    pub team_name: String,
    pub balance: i64,
    pub financial_status: String,
    pub is_crisis: bool,
    pub transfer_budget: i64,
    pub max_new_salary: u64,
    pub projected_season_profit: i64,
    pub total_salary: u64,
}

/// 财务交易记录
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub id: u64,
    pub team_id: u64,
    pub season_id: u64,
    pub transaction_type: String,
    pub amount: i64,
    pub description: Option<String>,
    pub related_player_id: Option<u64>,
    pub related_tournament_id: Option<u64>,
}

/// 赛季财务报告
#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonFinanceReport {
    pub team_id: u64,
    pub season_id: u64,
    pub opening_balance: i64,
    pub closing_balance: i64,
    pub total_income: u64,
    pub total_expense: u64,
    pub financial_status: String,
    pub salary_expense: u64,
    pub prize_money: u64,
    pub sponsorship: u64,
    pub league_share: u64,
    pub transfer_net: i64,
    pub operating_cost: u64,
}

/// 奖金信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PrizeInfo {
    pub tournament_type: String,
    pub position: String,
    pub amount: u64,
}

/// 获取队伍财务摘要
#[tauri::command]
pub async fn get_team_finance_summary(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<CommandResult<TeamFinanceSummary>, String> {
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

    // 获取队伍信息
    let team_row = sqlx::query(
        "SELECT id, name, balance, power_rating, win_rate FROM teams WHERE id = ? AND save_id = ?"
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let team_row = match team_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Team not found")),
    };

    let balance: i64 = team_row.get("balance");
    let team_name: String = team_row.get("name");
    let power_rating: f64 = team_row.get("power_rating");
    let win_rate: f64 = team_row.get("win_rate");

    // 计算总薪资
    let total_salary: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(salary), 0) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 使用财务引擎计算
    let engine = FinancialEngine::new();

    // 构建临时Team对象用于计算
    let team = crate::models::Team {
        id: team_id,
        region_id: 0,
        name: team_name.clone(),
        short_name: None,
        power_rating,
        total_matches: 0,
        wins: 0,
        win_rate,
        annual_points: 0,
        cross_year_points: 0,
        balance,
    };

    let status = engine.get_financial_status(&team, total_salary as u64);

    // 确定财务状态文本
    let financial_status = if balance > 1000 {
        "Wealthy"
    } else if balance >= 500 {
        "Healthy"
    } else if balance >= 100 {
        "Tight"
    } else if balance >= 0 {
        "Deficit"
    } else {
        "Bankrupt"
    };

    Ok(CommandResult::ok(TeamFinanceSummary {
        team_id,
        team_name,
        balance,
        financial_status: financial_status.to_string(),
        is_crisis: status.is_crisis,
        transfer_budget: status.transfer_budget,
        max_new_salary: status.max_new_salary,
        projected_season_profit: status.projected_season_profit,
        total_salary: total_salary as u64,
    }))
}

/// 获取所有队伍财务状况
#[tauri::command]
pub async fn get_all_teams_finance(
    state: State<'_, AppState>,
    region_id: Option<u64>,
) -> Result<CommandResult<Vec<TeamFinanceSummary>>, String> {
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

    // 获取队伍列表
    let team_rows = if let Some(rid) = region_id {
        sqlx::query(
            "SELECT id, name, balance, power_rating, win_rate FROM teams WHERE save_id = ? AND region_id = ? ORDER BY balance DESC"
        )
        .bind(&save_id)
        .bind(rid as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            "SELECT id, name, balance, power_rating, win_rate FROM teams WHERE save_id = ? ORDER BY balance DESC"
        )
        .bind(&save_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let engine = FinancialEngine::new();
    let mut summaries = Vec::new();

    for row in team_rows {
        let team_id: i64 = row.get("id");
        let balance: i64 = row.get("balance");
        let team_name: String = row.get("name");
        let power_rating: f64 = row.get("power_rating");
        let win_rate: f64 = row.get("win_rate");

        // 计算总薪资
        let total_salary: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(salary), 0) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
        )
        .bind(team_id)
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

        let team = crate::models::Team {
            id: team_id as u64,
            region_id: 0,
            name: team_name.clone(),
            short_name: None,
            power_rating,
            total_matches: 0,
            wins: 0,
            win_rate,
            annual_points: 0,
            cross_year_points: 0,
            balance,
        };

        let status = engine.get_financial_status(&team, total_salary as u64);

        let financial_status = if balance > 1000 {
            "Wealthy"
        } else if balance >= 500 {
            "Healthy"
        } else if balance >= 100 {
            "Tight"
        } else if balance >= 0 {
            "Deficit"
        } else {
            "Bankrupt"
        };

        summaries.push(TeamFinanceSummary {
            team_id: team_id as u64,
            team_name,
            balance,
            financial_status: financial_status.to_string(),
            is_crisis: status.is_crisis,
            transfer_budget: status.transfer_budget,
            max_new_salary: status.max_new_salary,
            projected_season_profit: status.projected_season_profit,
            total_salary: total_salary as u64,
        });
    }

    Ok(CommandResult::ok(summaries))
}

/// 获取队伍交易记录
#[tauri::command]
pub async fn get_team_transactions(
    state: State<'_, AppState>,
    team_id: u64,
    season_id: Option<u64>,
) -> Result<CommandResult<Vec<TransactionInfo>>, String> {
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

    let rows = if let Some(sid) = season_id {
        sqlx::query(
            r#"
            SELECT id, team_id, season_id, transaction_type, amount, description,
                   related_player_id, related_tournament_id
            FROM financial_transactions
            WHERE save_id = ? AND team_id = ? AND season_id = ?
            ORDER BY id DESC
            "#,
        )
        .bind(&save_id)
        .bind(team_id as i64)
        .bind(sid as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            r#"
            SELECT id, team_id, season_id, transaction_type, amount, description,
                   related_player_id, related_tournament_id
            FROM financial_transactions
            WHERE save_id = ? AND team_id = ?
            ORDER BY id DESC
            "#,
        )
        .bind(&save_id)
        .bind(team_id as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let transactions: Vec<TransactionInfo> = rows
        .iter()
        .map(|row| TransactionInfo {
            id: row.get::<i64, _>("id") as u64,
            team_id: row.get::<i64, _>("team_id") as u64,
            season_id: row.get::<i64, _>("season_id") as u64,
            transaction_type: row.get("transaction_type"),
            amount: row.get("amount"),
            description: row.get("description"),
            related_player_id: row.get::<Option<i64>, _>("related_player_id").map(|v| v as u64),
            related_tournament_id: row.get::<Option<i64>, _>("related_tournament_id").map(|v| v as u64),
        })
        .collect();

    Ok(CommandResult::ok(transactions))
}

/// 记录财务交易
#[tauri::command]
pub async fn record_transaction(
    state: State<'_, AppState>,
    team_id: u64,
    transaction_type: String,
    amount: i64,
    description: Option<String>,
    related_player_id: Option<u64>,
    related_tournament_id: Option<u64>,
) -> Result<CommandResult<TransactionInfo>, String> {
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

    // 插入交易记录
    let result = sqlx::query(
        r#"
        INSERT INTO financial_transactions (
            save_id, team_id, season_id, transaction_type, amount, description,
            related_player_id, related_tournament_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(current_season)
    .bind(&transaction_type)
    .bind(amount)
    .bind(&description)
    .bind(related_player_id.map(|v| v as i64))
    .bind(related_tournament_id.map(|v| v as i64))
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let new_id: i64 = result.get("id");

    // 更新队伍余额
    sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ? AND save_id = ?")
        .bind(amount)
        .bind(team_id as i64)
        .bind(&save_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(TransactionInfo {
        id: new_id as u64,
        team_id,
        season_id: current_season as u64,
        transaction_type,
        amount,
        description,
        related_player_id,
        related_tournament_id,
    }))
}

/// 获取赛季财务报告
#[tauri::command]
pub async fn get_season_finance_report(
    state: State<'_, AppState>,
    team_id: u64,
    season_id: Option<u64>,
) -> Result<CommandResult<SeasonFinanceReport>, String> {
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

    // 获取赛季
    let target_season = if let Some(sid) = season_id {
        sid as i64
    } else {
        let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
            .bind(&save_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
        save_row.get("current_season")
    };

    // 检查是否有现成的财务报告
    let report_row = sqlx::query(
        r#"
        SELECT * FROM team_season_finances
        WHERE team_id = ? AND season_id = ?
        "#,
    )
    .bind(team_id as i64)
    .bind(target_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = report_row {
        return Ok(CommandResult::ok(SeasonFinanceReport {
            team_id,
            season_id: target_season as u64,
            opening_balance: row.get("opening_balance"),
            closing_balance: row.get("closing_balance"),
            total_income: row.get::<i64, _>("total_income") as u64,
            total_expense: row.get::<i64, _>("total_expense") as u64,
            financial_status: row.get("financial_status"),
            salary_expense: row.get::<i64, _>("salary_cap_used") as u64,
            prize_money: 0,
            sponsorship: 0,
            league_share: 0,
            transfer_net: 0,
            operating_cost: 0,
        }));
    }

    // 动态计算财务报告
    let team_row = sqlx::query(
        "SELECT balance, power_rating, win_rate FROM teams WHERE id = ? AND save_id = ?"
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let balance: i64 = team_row.get("balance");
    let power_rating: f64 = team_row.get("power_rating");
    let win_rate: f64 = team_row.get("win_rate");

    let engine = FinancialEngine::new();

    // 获取各类交易汇总
    let salary_expense: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(ABS(amount)), 0) FROM financial_transactions WHERE save_id = ? AND team_id = ? AND season_id = ? AND transaction_type = 'Salary'"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(target_season)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let prize_money: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM financial_transactions WHERE save_id = ? AND team_id = ? AND season_id = ? AND (transaction_type = 'PlayoffBonus' OR transaction_type = 'InternationalBonus')"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(target_season)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let transfer_income: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM financial_transactions WHERE save_id = ? AND team_id = ? AND season_id = ? AND transaction_type = 'TransferIncome'"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(target_season)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let transfer_expense: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(ABS(amount)), 0) FROM financial_transactions WHERE save_id = ? AND team_id = ? AND season_id = ? AND transaction_type = 'TransferExpense'"
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(target_season)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    // 使用引擎计算标准值
    let team = crate::models::Team {
        id: team_id,
        region_id: 0,
        name: String::new(),
        short_name: None,
        power_rating,
        total_matches: 0,
        wins: 0,
        win_rate,
        annual_points: 0,
        cross_year_points: 0,
        balance,
    };

    let sponsorship = engine.calculate_sponsorship(&team);
    let league_share = engine.calculate_league_share();
    let operating_cost = engine.calculate_operating_cost();

    let transfer_net = transfer_income - transfer_expense;
    let total_income = sponsorship + league_share + prize_money as u64
        + if transfer_net > 0 { transfer_net as u64 } else { 0 };
    let total_expense = salary_expense as u64 + operating_cost
        + if transfer_net < 0 { (-transfer_net) as u64 } else { 0 };

    let net_change = total_income as i64 - total_expense as i64;
    let opening_balance = balance - net_change;

    let financial_status = if balance > 1000 {
        "Wealthy"
    } else if balance >= 500 {
        "Healthy"
    } else if balance >= 100 {
        "Tight"
    } else if balance >= 0 {
        "Deficit"
    } else {
        "Bankrupt"
    };

    Ok(CommandResult::ok(SeasonFinanceReport {
        team_id,
        season_id: target_season as u64,
        opening_balance,
        closing_balance: balance,
        total_income,
        total_expense,
        financial_status: financial_status.to_string(),
        salary_expense: salary_expense as u64,
        prize_money: prize_money as u64,
        sponsorship,
        league_share,
        transfer_net,
        operating_cost,
    }))
}

/// 支付球员薪资
#[tauri::command]
pub async fn pay_team_salaries(
    state: State<'_, AppState>,
    team_id: u64,
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 计算总薪资
    let total_salary: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(salary), 0) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if total_salary == 0 {
        return Ok(CommandResult::ok(0));
    }

    // 记录薪资支出
    sqlx::query(
        r#"
        INSERT INTO financial_transactions (
            save_id, team_id, season_id, transaction_type, amount, description
        ) VALUES (?, ?, ?, 'Salary', ?, '赛季薪资支出')
        "#,
    )
    .bind(&save_id)
    .bind(team_id as i64)
    .bind(current_season)
    .bind(-total_salary)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 更新队伍余额
    sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
        .bind(total_salary)
        .bind(team_id as i64)
        .bind(&save_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(total_salary as u64))
}

/// 发放联赛分成
#[tauri::command]
pub async fn distribute_league_share(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<(u64, u64)>>, String> {
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

    // 获取赛区所有队伍
    let team_rows = sqlx::query(
        "SELECT id FROM teams WHERE save_id = ? AND region_id = ?"
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let engine = FinancialEngine::new();
    let league_share = engine.calculate_league_share();

    let mut results = Vec::new();

    for row in team_rows {
        let team_id: i64 = row.get("id");

        // 记录联赛分成
        sqlx::query(
            r#"
            INSERT INTO financial_transactions (
                save_id, team_id, season_id, transaction_type, amount, description
            ) VALUES (?, ?, ?, 'LeagueShare', ?, '联赛分成收入')
            "#,
        )
        .bind(&save_id)
        .bind(team_id)
        .bind(current_season)
        .bind(league_share as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 更新队伍余额
        sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ? AND save_id = ?")
            .bind(league_share as i64)
            .bind(team_id)
            .bind(&save_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        results.push((team_id as u64, league_share));
    }

    Ok(CommandResult::ok(results))
}

/// 获取赛事奖金配置
#[tauri::command]
pub async fn get_prize_pool_info(
    tournament_type: String,
) -> Result<CommandResult<Vec<PrizeInfo>>, String> {
    let engine = FinancialEngine::new();

    let tt = match tournament_type.as_str() {
        "Msi" => TournamentType::Msi,
        "WorldChampionship" => TournamentType::WorldChampionship,
        "MadridMasters" => TournamentType::MadridMasters,
        "ClaudeIntercontinental" => TournamentType::ClaudeIntercontinental,
        "ShanghaiMasters" => TournamentType::ShanghaiMasters,
        "SuperIntercontinental" => TournamentType::SuperIntercontinental,
        "SpringPlayoffs" => TournamentType::SpringPlayoffs,
        "SummerPlayoffs" => TournamentType::SummerPlayoffs,
        _ => return Ok(CommandResult::err("Unknown tournament type")),
    };

    // 预定义的名次列表
    let positions = vec![
        "CHAMPION", "RUNNER_UP", "THIRD", "FOURTH",
        "QUARTER_FINAL", "GROUP_STAGE", "5TH_8TH",
        "SEMI_LOSER", "R1_LOSER", "LOSERS_R2", "LOSERS_R1",
        "PREP_LOSER", "PROMOTION_LOSER", "FIGHTER_OUT",
    ];

    let prizes: Vec<PrizeInfo> = positions
        .into_iter()
        .filter_map(|pos| {
            let amount = engine.calculate_prize_money(tt, pos);
            if amount > 0 {
                Some(PrizeInfo {
                    tournament_type: tournament_type.clone(),
                    position: pos.to_string(),
                    amount,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(CommandResult::ok(prizes))
}

/// 发放赛事奖金
#[tauri::command]
pub async fn distribute_tournament_prizes(
    state: State<'_, AppState>,
    tournament_id: u64,
    results: Vec<(u64, String)>, // (team_id, position)
) -> Result<CommandResult<Vec<(u64, u64)>>, String> {
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

    // 获取赛事类型
    let tournament_row = sqlx::query("SELECT tournament_type FROM tournaments WHERE id = ?")
        .bind(tournament_id as i64)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let tournament_type_str: String = match tournament_row {
        Some(r) => r.get("tournament_type"),
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    let tournament_type = match tournament_type_str.as_str() {
        "Msi" => TournamentType::Msi,
        "WorldChampionship" => TournamentType::WorldChampionship,
        "MadridMasters" => TournamentType::MadridMasters,
        "ClaudeIntercontinental" => TournamentType::ClaudeIntercontinental,
        "ShanghaiMasters" => TournamentType::ShanghaiMasters,
        "SuperIntercontinental" => TournamentType::SuperIntercontinental,
        "SpringPlayoffs" => TournamentType::SpringPlayoffs,
        "SummerPlayoffs" => TournamentType::SummerPlayoffs,
        _ => TournamentType::SpringRegular,
    };

    let engine = FinancialEngine::new();
    let transaction_type = if tournament_type.is_regional() {
        "PlayoffBonus"
    } else {
        "InternationalBonus"
    };

    let mut prize_results = Vec::new();

    for (team_id, position) in results {
        let prize = engine.calculate_prize_money(tournament_type, &position);

        if prize > 0 {
            // 记录奖金
            sqlx::query(
                r#"
                INSERT INTO financial_transactions (
                    save_id, team_id, season_id, transaction_type, amount, description, related_tournament_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&save_id)
            .bind(team_id as i64)
            .bind(current_season)
            .bind(transaction_type)
            .bind(prize as i64)
            .bind(format!("{:?} - {}", tournament_type, position))
            .bind(tournament_id as i64)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

            // 更新队伍余额
            sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ? AND save_id = ?")
                .bind(prize as i64)
                .bind(team_id as i64)
                .bind(&save_id)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

            prize_results.push((team_id, prize));
        }
    }

    Ok(CommandResult::ok(prize_results))
}
